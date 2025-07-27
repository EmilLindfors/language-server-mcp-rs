use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};
use tracing_subscriber::{self, EnvFilter};
use std::future::Future;

mod lsp_client;
use lsp_client::LspClient;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HoverRequest {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CompletionRequest {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DiagnosticsRequest {
    pub file_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GotoDefinitionRequest {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindReferencesRequest {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    #[serde(default = "default_include_declaration")]
    pub include_declaration: bool,
}

fn default_include_declaration() -> bool {
    true
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FormatRequest {
    pub file_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RenameRequest {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub new_name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CodeActionsRequest {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone)]
pub struct RustAnalyzerMCP {
    lsp_client: Arc<Mutex<LspClient>>,
    workspace_root: PathBuf,
    tool_router: ToolRouter<RustAnalyzerMCP>,
}

#[tool_router]
impl RustAnalyzerMCP {
    pub async fn new(workspace_root: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let lsp_client = LspClient::new(&workspace_root).await?;
        Ok(Self {
            lsp_client: Arc::new(Mutex::new(lsp_client)),
            workspace_root,
            tool_router: Self::tool_router(),
        })
    }

    #[tool(description = "Get type information and documentation at a specific position")]
    async fn hover(&self, Parameters(request): Parameters<HoverRequest>) -> Result<CallToolResult, McpError> {
        let lsp_client = self.lsp_client.lock().await;
        
        match lsp_client.hover(&request.file_path, request.line, request.column).await {
            Ok(Some(hover)) => {
                let content = match hover.contents {
                    lsp_types::HoverContents::Markup(markup) => markup.value,
                    lsp_types::HoverContents::Array(markups) => {
                        markups.into_iter()
                            .map(|m| match m {
                                lsp_types::MarkedString::String(s) => s,
                                lsp_types::MarkedString::LanguageString(ls) => ls.value,
                            })
                            .collect::<Vec<_>>()
                            .join("\n\n")
                    },
                    lsp_types::HoverContents::Scalar(ms) => match ms {
                        lsp_types::MarkedString::String(s) => s,
                        lsp_types::MarkedString::LanguageString(ls) => ls.value,
                    },
                };
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            Ok(None) => Ok(CallToolResult::success(vec![Content::text("No hover information available")])),
            Err(e) => Err(McpError::internal_error(format!("LSP error: {}", e), None)),
        }
    }

    #[tool(description = "Get code completions at a specific position")]
    async fn completion(&self, Parameters(request): Parameters<CompletionRequest>) -> Result<CallToolResult, McpError> {
        let lsp_client = self.lsp_client.lock().await;
        
        match lsp_client.completion(&request.file_path, request.line, request.column).await {
            Ok(Some(result)) => {
                let completions = match result {
                    lsp_types::CompletionResponse::Array(items) => items,
                    lsp_types::CompletionResponse::List(list) => list.items,
                };
                
                let completion_text = completions.into_iter()
                    .take(10) // Limit to first 10 for readability
                    .map(|item| {
                        let detail = item.detail.unwrap_or_default();
                        let doc = item.documentation.map(|d| match d {
                            lsp_types::Documentation::String(s) => s,
                            lsp_types::Documentation::MarkupContent(mc) => mc.value,
                        }).unwrap_or_default();
                        
                        if doc.is_empty() {
                            format!("- {}: {}", item.label, detail)
                        } else {
                            format!("- {}: {} - {}", item.label, detail, doc)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                    
                Ok(CallToolResult::success(vec![Content::text(format!("Completions:\n{}", completion_text))]))
            }
            Ok(None) => Ok(CallToolResult::success(vec![Content::text("No completions available")])),
            Err(e) => Err(McpError::internal_error(format!("LSP error: {}", e), None)),
        }
    }

    #[tool(description = "Get compile errors and warnings for a file")]
    async fn diagnostics(&self, Parameters(request): Parameters<DiagnosticsRequest>) -> Result<CallToolResult, McpError> {
        let lsp_client = self.lsp_client.lock().await;
        
        match lsp_client.diagnostics(&request.file_path).await {
            Ok(diagnostics) => {
                if diagnostics.is_empty() {
                    Ok(CallToolResult::success(vec![Content::text("No diagnostics found")]))
                } else {
                    let diagnostic_text = diagnostics.into_iter()
                        .map(|diag| {
                            let severity = diag.severity.map(|s| format!("{:?}", s)).unwrap_or("Info".to_string());
                            let range = format!("{}:{}-{}:{}", 
                                diag.range.start.line, diag.range.start.character,
                                diag.range.end.line, diag.range.end.character);
                            format!("[{}] {}: {} ({})", severity, range, diag.message, 
                                diag.source.unwrap_or_default())
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                        
                    Ok(CallToolResult::success(vec![Content::text(format!("Diagnostics:\n{}", diagnostic_text))]))
                }
            }
            Err(e) => Err(McpError::internal_error(format!("LSP error: {}", e), None)),
        }
    }

    #[tool(description = "Find definition of symbol at position")]
    async fn goto_definition(&self, Parameters(request): Parameters<GotoDefinitionRequest>) -> Result<CallToolResult, McpError> {
        let lsp_client = self.lsp_client.lock().await;
        
        match lsp_client.goto_definition(&request.file_path, request.line, request.column).await {
            Ok(Some(response)) => {
                use lsp_types::GotoDefinitionResponse;
                let locations = match response {
                    GotoDefinitionResponse::Scalar(location) => vec![location],
                    GotoDefinitionResponse::Array(locations) => locations,
                    GotoDefinitionResponse::Link(links) => {
                        links.into_iter()
                            .map(|link| lsp_types::Location {
                                uri: link.target_uri,
                                range: link.target_selection_range,
                            })
                            .collect()
                    }
                };
                
                if locations.is_empty() {
                    Ok(CallToolResult::success(vec![Content::text("No definition found")]))
                } else {
                    let definition_text = locations.into_iter()
                        .map(|loc| {
                            let path = loc.uri.to_file_path().ok()
                                .and_then(|p| p.to_str().map(|s| s.to_string()))
                                .unwrap_or_else(|| loc.uri.to_string());
                            format!("Definition at: {}:{}:{}", 
                                path,
                                loc.range.start.line,
                                loc.range.start.character)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                        
                    Ok(CallToolResult::success(vec![Content::text(format!("Found definitions:\n{}", definition_text))]))
                }
            }
            Ok(None) => Ok(CallToolResult::success(vec![Content::text("No definition found")])),
            Err(e) => Err(McpError::internal_error(format!("LSP error: {}", e), None)),
        }
    }

    #[tool(description = "Find all references to symbol at position")]
    async fn find_references(&self, Parameters(request): Parameters<FindReferencesRequest>) -> Result<CallToolResult, McpError> {
        let lsp_client = self.lsp_client.lock().await;
        
        match lsp_client.find_references(&request.file_path, request.line, request.column, request.include_declaration).await {
            Ok(Some(locations)) => {
                if locations.is_empty() {
                    Ok(CallToolResult::success(vec![Content::text("No references found")]))
                } else {
                    let references_text = locations.into_iter()
                        .map(|loc| {
                            let path = loc.uri.to_file_path().ok()
                                .and_then(|p| p.to_str().map(|s| s.to_string()))
                                .unwrap_or_else(|| loc.uri.to_string());
                            format!("Reference at: {}:{}:{}", 
                                path,
                                loc.range.start.line,
                                loc.range.start.character)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                        
                    Ok(CallToolResult::success(vec![Content::text(format!("Found references:\n{}", references_text))]))
                }
            }
            Ok(None) => Ok(CallToolResult::success(vec![Content::text("No references found")])),
            Err(e) => Err(McpError::internal_error(format!("LSP error: {}", e), None)),
        }
    }

    #[tool(description = "Format Rust code")]
    async fn format_document(&self, Parameters(request): Parameters<FormatRequest>) -> Result<CallToolResult, McpError> {
        let lsp_client = self.lsp_client.lock().await;
        
        let result = lsp_client.format_document(&request.file_path).await;
        drop(lsp_client);  // Release the lock before doing async I/O
        
        match result {
            Ok(Some(edits)) => {
                if edits.is_empty() {
                    Ok(CallToolResult::success(vec![Content::text("No formatting changes needed")]))
                } else {
                    // For simplicity, we'll just return a message about the number of edits
                    // In a real implementation, you'd apply the TextEdits to the content
                    let edit_count = edits.len();
                    Ok(CallToolResult::success(vec![Content::text(format!("Formatting would apply {} edits to the file", edit_count))]))
                }
            }
            Ok(None) => Ok(CallToolResult::success(vec![Content::text("No formatting changes needed")])),
            Err(e) => Err(McpError::internal_error(format!("LSP error: {}", e), None)),
        }
    }

    #[tool(description = "Rename symbols across the entire workspace safely")]
    async fn rename(&self, Parameters(request): Parameters<RenameRequest>) -> Result<CallToolResult, McpError> {
        let lsp_client = self.lsp_client.lock().await;
        
        match lsp_client.rename(&request.file_path, request.line, request.column, &request.new_name).await {
            Ok(Some(workspace_edit)) => {
                let mut changes_description = Vec::new();
                
                if let Some(changes) = workspace_edit.changes {
                    for (uri, edits) in changes {
                        let file_path = uri.path();
                        changes_description.push(format!("File: {}", file_path));
                        
                        for edit in &edits {
                            changes_description.push(format!(
                                "  - Line {}-{}: Replace '{}' with '{}'",
                                edit.range.start.line + 1,
                                edit.range.end.line + 1,
                                edit.new_text.trim_end_matches('\n').replace('\n', "\\n"),
                                request.new_name
                            ));
                        }
                    }
                }
                
                if let Some(document_changes) = workspace_edit.document_changes {
                    use lsp_types::DocumentChangeOperation;
                    use lsp_types::DocumentChanges;
                    
                    let changes: Vec<DocumentChangeOperation> = match document_changes {
                        DocumentChanges::Edits(edits) => edits.into_iter()
                            .map(|edit| DocumentChangeOperation::Edit(edit))
                            .collect(),
                        DocumentChanges::Operations(ops) => ops,
                    };
                    
                    for change in changes {
                        match change {
                            DocumentChangeOperation::Edit(text_doc_edit) => {
                                let file_path = text_doc_edit.text_document.uri.path();
                                changes_description.push(format!("File: {}", file_path));
                                
                                for edit in &text_doc_edit.edits {
                                    use lsp_types::OneOf;
                                    let text_edit = match edit {
                                        OneOf::Left(edit) => edit,
                                        OneOf::Right(annotated) => &annotated.text_edit,
                                    };
                                    changes_description.push(format!(
                                        "  - Line {}-{}: Replace with '{}'",
                                        text_edit.range.start.line + 1,
                                        text_edit.range.end.line + 1,
                                        text_edit.new_text.trim_end_matches('\n').replace('\n', "\\n")
                                    ));
                                }
                            }
                            _ => {
                                changes_description.push("  - Other document changes (create/rename/delete)".to_string());
                            }
                        }
                    }
                }
                
                if changes_description.is_empty() {
                    Ok(CallToolResult::success(vec![Content::text("No changes needed for rename")]))
                } else {
                    let summary = format!(
                        "Rename operation would make the following changes:\n\n{}",
                        changes_description.join("\n")
                    );
                    Ok(CallToolResult::success(vec![Content::text(summary)]))
                }
            }
            Ok(None) => Ok(CallToolResult::success(vec![Content::text("Cannot rename at this position")])),
            Err(e) => Err(McpError::internal_error(format!("LSP error: {}", e), None)),
        }
    }

    #[tool(description = "Get available quick fixes and refactorings")]
    async fn code_actions(&self, Parameters(request): Parameters<CodeActionsRequest>) -> Result<CallToolResult, McpError> {
        let lsp_client = self.lsp_client.lock().await;
        
        match lsp_client.code_actions(&request.file_path, request.line, request.column).await {
            Ok(Some(actions)) => {
                let mut action_descriptions = Vec::new();
                
                for action in actions {
                    use lsp_types::CodeActionOrCommand;
                    match action {
                        CodeActionOrCommand::CodeAction(code_action) => {
                            let title = &code_action.title;
                            let kind = code_action.kind
                                .as_ref()
                                .map(|k| format!(" ({})", k.as_str()))
                                .unwrap_or_default();
                            
                            let diagnostics_info = if code_action.diagnostics.is_some() {
                                let diag_count = code_action.diagnostics.as_ref().unwrap().len();
                                if diag_count > 0 {
                                    format!(" [Fixes {} diagnostic(s)]", diag_count)
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };
                            
                            action_descriptions.push(format!("• {}{}{}", title, kind, diagnostics_info));
                            
                            // If there's a workspace edit, show what it would change
                            if let Some(edit) = &code_action.edit {
                                if let Some(changes) = &edit.changes {
                                    for (uri, edits) in changes {
                                        if !edits.is_empty() {
                                            action_descriptions.push(format!("  → Modifies: {}", uri.path()));
                                        }
                                    }
                                }
                                
                                if let Some(document_changes) = &edit.document_changes {
                                    use lsp_types::DocumentChanges;
                                    match document_changes {
                                        DocumentChanges::Edits(edits) => {
                                            for edit in edits {
                                                action_descriptions.push(format!("  → Modifies: {}", edit.text_document.uri.path()));
                                            }
                                        }
                                        DocumentChanges::Operations(ops) => {
                                            action_descriptions.push(format!("  → {} workspace operations", ops.len()));
                                        }
                                    }
                                }
                            }
                        }
                        CodeActionOrCommand::Command(command) => {
                            action_descriptions.push(format!("• {} (command: {})", command.title, command.command));
                        }
                    }
                }
                
                if action_descriptions.is_empty() {
                    Ok(CallToolResult::success(vec![Content::text("No code actions available at this position")]))
                } else {
                    let summary = format!(
                        "Available code actions:\n\n{}",
                        action_descriptions.join("\n")
                    );
                    Ok(CallToolResult::success(vec![Content::text(summary)]))
                }
            }
            Ok(None) => Ok(CallToolResult::success(vec![Content::text("No code actions available at this position")])),
            Err(e) => Err(McpError::internal_error(format!("LSP error: {}", e), None)),
        }
    }
}

#[tool_handler]
impl ServerHandler for RustAnalyzerMCP {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides rust-analyzer functionality through MCP tools. Available tools: 'hover' for type information, 'completion' for code completions, 'diagnostics' for compile errors, 'goto_definition' to find definitions, 'find_references' to find all references, 'format_document' to format code, 'rename' to rename symbols across the workspace, and 'code_actions' to get quick fixes and refactorings.".to_string()),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("Starting rust-analyzer MCP server");

    let workspace_root = std::env::current_dir()?;
    let service = RustAnalyzerMCP::new(workspace_root).await?
        .serve(stdio()).await
        .inspect_err(|e| {
            error!("serving error: {:?}", e);
        })?;

    info!("MCP server is running");
    service.waiting().await?;

    Ok(())
}