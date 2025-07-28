use anyhow::Result;
use rmcp::{
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
    ServiceExt,
};
use tokio::process::Command;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,client_example=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting rust-analyzer MCP client example");

    // Start our rust-analyzer MCP server
    let server_path = std::env::current_dir()?
        .join("target")
        .join("debug")
        .join("language-server-mcp");

    let client = ()
        .serve(TokioChildProcess::new(
            Command::new(&server_path).configure(|_cmd| {
                // No additional configuration needed
            }),
        )?)
        .await?;

    // Get server info
    let server_info = client.peer_info();
    tracing::info!("Connected to rust-analyzer MCP server: {server_info:#?}");

    // List available tools
    let tools = client.list_all_tools().await?;
    tracing::info!("Available tools:");
    for tool in &tools {
        tracing::info!("  - {}: {:?}", tool.name, tool.description);
    }

    let test_file = std::env::current_dir()?.join("src/main.rs");
    let test_file_str = test_file.to_str().unwrap();

    // Example 1: Get hover information
    tracing::info!("\n=== Testing hover ===");
    let hover_result = client
        .call_tool(CallToolRequestParam {
            name: "hover".into(),
            arguments: Some(object!({
                "file_path": test_file_str,
                "line": 50,  // Line with struct definition
                "column": 8  // Position in main.rs
            })),
        })
        .await?;
    tracing::info!("Hover result: {hover_result:#?}");

    // Example 2: Get diagnostics
    tracing::info!("\n=== Testing diagnostics ===");
    let diagnostics_result = client
        .call_tool(CallToolRequestParam {
            name: "diagnostics".into(),
            arguments: Some(object!({
                "file_path": test_file_str
            })),
        })
        .await?;
    tracing::info!("Diagnostics result: {diagnostics_result:#?}");

    // Example 3: Get completions
    tracing::info!("\n=== Testing completions ===");
    let completion_result = client
        .call_tool(CallToolRequestParam {
            name: "completion".into(),
            arguments: Some(object!({
                "file_path": test_file_str,
                "line": 60,  // Line in main.rs
                "column": 10  // Position for completion
            })),
        })
        .await?;
    tracing::info!("Completion result: {completion_result:#?}");

    // Example 4: Test with a struct
    tracing::info!("\n=== Testing hover on User struct ===");
    let struct_hover_result = client
        .call_tool(CallToolRequestParam {
            name: "hover".into(),
            arguments: Some(object!({
                "file_path": test_file_str,
                "line": 40,   // Line with struct in main.rs
                "column": 10  // Position for hover
            })),
        })
        .await?;
    tracing::info!("User struct hover result: {struct_hover_result:#?}");

    // Example 5: Test goto_definition
    tracing::info!("\n=== Testing goto_definition ===");
    let goto_def_result = client
        .call_tool(CallToolRequestParam {
            name: "goto_definition".into(),
            arguments: Some(object!({
                "file_path": test_file_str,
                "line": 90,   // Line with a function call or type usage
                "column": 15  // Position for goto definition
            })),
        })
        .await?;
    tracing::info!("Goto definition result: {goto_def_result:#?}");

    // Example 6: Test find_references
    tracing::info!("\n=== Testing find_references ===");
    let find_refs_result = client
        .call_tool(CallToolRequestParam {
            name: "find_references".into(),
            arguments: Some(object!({
                "file_path": test_file_str,
                "line": 40,   // Line with a symbol definition
                "column": 10,  // Position for finding references
                "include_declaration": true
            })),
        })
        .await?;
    tracing::info!("Find references result: {find_refs_result:#?}");

    // Example 7: Test format_document
    tracing::info!("\n=== Testing format_document ===");
    let format_result = client
        .call_tool(CallToolRequestParam {
            name: "format_document".into(),
            arguments: Some(object!({
                "file_path": test_file_str
            })),
        })
        .await?;
    tracing::info!("Format document result: {format_result:#?}");

    // Example 8: Test rename
    tracing::info!("\n=== Testing rename ===");
    let rename_result = client
        .call_tool(CallToolRequestParam {
            name: "rename".into(),
            arguments: Some(object!({
                "file_path": test_file_str,
                "line": 73,   // Line with RustAnalyzerMCP struct
                "column": 12, // Position at the struct name
                "new_name": "RustAnalyzerMCPServer"
            })),
        })
        .await?;
    tracing::info!("Rename result: {rename_result:#?}");

    // Example 9: Test code_actions
    tracing::info!("\n=== Testing code_actions ===");
    let code_actions_result = client
        .call_tool(CallToolRequestParam {
            name: "code_actions".into(),
            arguments: Some(object!({
                "file_path": test_file_str,
                "line": 80,   // Line with struct definition
                "column": 10  // Position for code actions
            })),
        })
        .await?;
    tracing::info!("Code actions result: {code_actions_result:#?}");

    // Example 10: Test workspace_symbols
    tracing::info!("\n=== Testing workspace_symbols ===");
    let workspace_symbols_result = client
        .call_tool(CallToolRequestParam {
            name: "workspace_symbols".into(),
            arguments: Some(object!({
                "query": "Rust"  // Search for symbols containing "Rust"
            })),
        })
        .await?;
    tracing::info!("Workspace symbols result: {workspace_symbols_result:#?}");

    // Example 11: Test inlay_hints
    tracing::info!("\n=== Testing inlay_hints ===");
    let inlay_hints_result = client
        .call_tool(CallToolRequestParam {
            name: "inlay_hints".into(),
            arguments: Some(object!({
                "file_path": test_file_str
            })),
        })
        .await?;
    tracing::info!("Inlay hints result: {inlay_hints_result:#?}");

    // Example 12: Test expand_macro (testing on a position that might have a macro)
    tracing::info!("\n=== Testing expand_macro ===");
    let expand_macro_result = client
        .call_tool(CallToolRequestParam {
            name: "expand_macro".into(),
            arguments: Some(object!({
                "file_path": test_file_str,
                "line": 86,   // Line with #[tool_router] macro
                "column": 5   // Position at the macro
            })),
        })
        .await?;
    tracing::info!("Expand macro result: {expand_macro_result:#?}");

    // Shutdown the client
    client.cancel().await?;

    tracing::info!("Client example completed successfully!");
    Ok(())
}
