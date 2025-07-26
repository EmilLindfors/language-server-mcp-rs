use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
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
        .serve(TokioChildProcess::new(Command::new(&server_path).configure(
            |_cmd| {
                // No additional configuration needed
            },
        ))?)
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

    // Shutdown the client
    client.cancel().await?;

    tracing::info!("Client example completed successfully!");
    Ok(())
}