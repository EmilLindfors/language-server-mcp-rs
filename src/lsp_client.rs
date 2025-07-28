use lsp_types::*;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

pub struct LspClient {
    process: Child,
    stdin: Mutex<tokio::process::ChildStdin>,
    stdout: Mutex<BufReader<tokio::process::ChildStdout>>,
    request_id: Mutex<i64>,
    workspace_root: PathBuf,
    is_ready: Arc<AtomicBool>,
}

impl LspClient {
    pub async fn new(workspace_root: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Starting rust-analyzer process");

        let mut process = Command::new("rust-analyzer")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = process.stdin.take().unwrap();
        let stdout = BufReader::new(process.stdout.take().unwrap());

        let mut client = Self {
            process,
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(stdout),
            request_id: Mutex::new(0),
            workspace_root: workspace_root.clone(),
            is_ready: Arc::new(AtomicBool::new(false)),
        };

        // Initialize synchronously for now - we'll add async initialization later
        client.initialize().await?;
        client.is_ready.store(true, Ordering::Relaxed);

        Ok(client)
    }

    pub async fn wait_for_ready(&self) {
        while !self.is_ready.load(Ordering::Relaxed) {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    pub fn is_ready(&self) -> bool {
        self.is_ready.load(Ordering::Relaxed)
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let workspace_folder = WorkspaceFolder {
            uri: Url::from_file_path(&self.workspace_root).unwrap(),
            name: self
                .workspace_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("workspace")
                .to_string(),
        };

        let initialize_params = InitializeParams {
            capabilities: ClientCapabilities::default(),
            workspace_folders: Some(vec![workspace_folder]),
            initialization_options: Some(json!({
                "cargo": {
                    "runBuildScripts": true,
                    "features": "all"
                }
            })),
            ..Default::default()
        };

        let response: InitializeResult = self.request("initialize", initialize_params).await?;
        info!(
            "LSP initialized with capabilities: {:?}",
            response.capabilities
        );

        self.notify("initialized", InitializedParams {}).await?;

        Ok(())
    }

    pub async fn open_document(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(file_path).unwrap(),
                language_id: "rust".to_string(),
                version: 1,
                text: content,
            },
        };

        self.notify("textDocument/didOpen", params).await
    }

    pub async fn hover(
        &self,
        file_path: &str,
        line: u32,
        column: u32,
    ) -> Result<Option<Hover>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(file_path).unwrap(),
                },
                position: Position {
                    line,
                    character: column,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        self.request("textDocument/hover", params).await
    }

    pub async fn completion(
        &self,
        file_path: &str,
        line: u32,
        column: u32,
    ) -> Result<Option<CompletionResponse>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(file_path).unwrap(),
                },
                position: Position {
                    line,
                    character: column,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        };

        self.request("textDocument/completion", params).await
    }

    pub async fn diagnostics(
        &self,
        file_path: &str,
    ) -> Result<Vec<Diagnostic>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;
        let params = DocumentDiagnosticParams {
            text_document: TextDocumentIdentifier {
                uri: Url::from_file_path(file_path).unwrap(),
            },
            identifier: None,
            previous_result_id: None,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let response: DocumentDiagnosticReportResult =
            self.request("textDocument/diagnostic", params).await?;

        match response {
            DocumentDiagnosticReportResult::Report(report) => match report {
                DocumentDiagnosticReport::Full(full) => {
                    Ok(full.full_document_diagnostic_report.items)
                }
                DocumentDiagnosticReport::Unchanged(_) => Ok(vec![]),
            },
            DocumentDiagnosticReportResult::Partial(_) => Ok(vec![]),
        }
    }

    pub async fn goto_definition(
        &self,
        file_path: &str,
        line: u32,
        column: u32,
    ) -> Result<Option<GotoDefinitionResponse>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;
        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(file_path).unwrap(),
                },
                position: Position {
                    line,
                    character: column,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        self.request("textDocument/definition", params).await
    }

    pub async fn find_references(
        &self,
        file_path: &str,
        line: u32,
        column: u32,
        include_declaration: bool,
    ) -> Result<Option<Vec<Location>>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;
        let params = ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(file_path).unwrap(),
                },
                position: Position {
                    line,
                    character: column,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: ReferenceContext {
                include_declaration,
            },
        };

        self.request("textDocument/references", params).await
    }

    pub async fn format_document(
        &self,
        file_path: &str,
    ) -> Result<Option<Vec<TextEdit>>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;
        let params = DocumentFormattingParams {
            text_document: TextDocumentIdentifier {
                uri: Url::from_file_path(file_path).unwrap(),
            },
            options: FormattingOptions {
                tab_size: 4,
                insert_spaces: true,
                ..Default::default()
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        self.request("textDocument/formatting", params).await
    }

    pub async fn rename(
        &self,
        file_path: &str,
        line: u32,
        column: u32,
        new_name: &str,
    ) -> Result<Option<WorkspaceEdit>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;
        let params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(file_path).unwrap(),
                },
                position: Position {
                    line,
                    character: column,
                },
            },
            new_name: new_name.to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        self.request("textDocument/rename", params).await
    }

    pub async fn code_actions(
        &self,
        file_path: &str,
        line: u32,
        column: u32,
    ) -> Result<Option<CodeActionResponse>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;
        let params = CodeActionParams {
            text_document: TextDocumentIdentifier {
                uri: Url::from_file_path(file_path).unwrap(),
            },
            range: Range {
                start: Position {
                    line,
                    character: column,
                },
                end: Position {
                    line,
                    character: column,
                },
            },
            context: CodeActionContext {
                diagnostics: vec![], // We could pass current diagnostics here
                only: None,          // Request all types of code actions
                trigger_kind: Some(CodeActionTriggerKind::INVOKED),
                ..Default::default()
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        self.request("textDocument/codeAction", params).await
    }

    pub async fn workspace_symbols(
        &self,
        query: &str,
    ) -> Result<Option<Vec<SymbolInformation>>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        let params = WorkspaceSymbolParams {
            query: query.to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        self.request("workspace/symbol", params).await
    }

    pub async fn inlay_hints(
        &self,
        file_path: &str,
    ) -> Result<Option<Vec<InlayHint>>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;

        // Read the file to get its content and determine the range
        let content = tokio::fs::read_to_string(file_path).await?;
        let lines: Vec<&str> = content.lines().collect();
        let end_line = lines.len().saturating_sub(1) as u32;
        let end_character = lines.last().map(|line| line.len()).unwrap_or(0) as u32;

        let params = InlayHintParams {
            text_document: TextDocumentIdentifier {
                uri: Url::from_file_path(file_path).unwrap(),
            },
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        self.request("textDocument/inlayHint", params).await
    }

    pub async fn expand_macro(
        &self,
        file_path: &str,
        line: u32,
        column: u32,
    ) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;

        // rust-analyzer uses a custom expandMacro request
        let params = json!({
            "textDocument": {
                "uri": Url::from_file_path(file_path).unwrap()
            },
            "position": {
                "line": line,
                "character": column
            }
        });

        // This is a rust-analyzer specific extension, not standard LSP
        self.request("rust-analyzer/expandMacro", params).await
    }

    pub async fn document_symbols(
        &self,
        file_path: &str,
    ) -> Result<Option<DocumentSymbolResponse>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;
        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier {
                uri: Url::from_file_path(file_path).unwrap(),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        self.request("textDocument/documentSymbol", params).await
    }

    pub async fn signature_help(
        &self,
        file_path: &str,
        line: u32,
        column: u32,
    ) -> Result<Option<SignatureHelp>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;
        let params = SignatureHelpParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(file_path).unwrap(),
                },
                position: Position {
                    line,
                    character: column,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            context: None,
        };

        self.request("textDocument/signatureHelp", params).await
    }

    pub async fn document_highlight(
        &self,
        file_path: &str,
        line: u32,
        column: u32,
    ) -> Result<Option<Vec<DocumentHighlight>>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;
        let params = DocumentHighlightParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(file_path).unwrap(),
                },
                position: Position {
                    line,
                    character: column,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        self.request("textDocument/documentHighlight", params).await
    }

    pub async fn selection_range(
        &self,
        file_path: &str,
        positions: Vec<Position>,
    ) -> Result<Option<Vec<SelectionRange>>, Box<dyn std::error::Error>> {
        self.wait_for_ready().await;
        // Ensure document is open
        self.open_document(file_path).await?;
        let params = SelectionRangeParams {
            text_document: TextDocumentIdentifier {
                uri: Url::from_file_path(file_path).unwrap(),
            },
            positions,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        self.request("textDocument/selectionRange", params).await
    }

    async fn request<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R, Box<dyn std::error::Error>> {
        let mut id = self.request_id.lock().await;
        *id += 1;
        let request_id = *id;

        let request = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params
        });

        self.send_message(&request).await?;

        let response = self.read_response(request_id).await?;

        if let Some(error) = response.get("error") {
            return Err(format!("LSP error: {:?}", error).into());
        }

        let result = response.get("result").ok_or("Missing result in response")?;

        Ok(serde_json::from_value(result.clone())?)
    }

    async fn notify<P: serde::Serialize>(
        &self,
        method: &str,
        params: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        self.send_message(&notification).await
    }

    async fn send_message(&self, message: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string(message)?;
        let header = format!("Content-Length: {}\r\n\r\n", content.len());

        let mut stdin = self.stdin.lock().await;
        stdin.write_all(header.as_bytes()).await?;
        stdin.write_all(content.as_bytes()).await?;
        stdin.flush().await?;

        debug!("Sent LSP message: {}", content);

        Ok(())
    }

    async fn read_response(&self, expected_id: i64) -> Result<Value, Box<dyn std::error::Error>> {
        let mut stdout = self.stdout.lock().await;

        loop {
            let mut header = String::new();
            stdout.read_line(&mut header).await?;

            if header.starts_with("Content-Length:") {
                let length: usize = header
                    .trim_start_matches("Content-Length:")
                    .trim()
                    .parse()?;

                stdout.read_line(&mut header).await?;

                let mut content = vec![0; length];
                stdout.read_exact(&mut content).await?;

                let response: Value = serde_json::from_slice(&content)?;
                debug!("Received LSP response: {}", response);

                if let Some(id) = response.get("id") {
                    if id.as_i64() == Some(expected_id) {
                        return Ok(response);
                    }
                }
            }
        }
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}
