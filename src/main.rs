use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
};
use serde_json::json;
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
            instructions: Some("This server provides rust-analyzer functionality through MCP tools. Use 'hover' to get type information, 'completion' for code completions, and 'diagnostics' for compile errors.".to_string()),
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