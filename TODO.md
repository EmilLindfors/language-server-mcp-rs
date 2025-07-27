# Language Server MCP - Project Status & TODO

## 🎉 Project Status: WORKING PROTOTYPE

We have successfully implemented a working rust-analyzer MCP bridge that allows AI assistants to access Rust language server functionality through the Model Context Protocol.

## ✅ Completed Features

### Core Implementation
- [x] **MCP Server with rmcp**: Full server implementation using official rmcp SDK
- [x] **LSP Client**: Rust-analyzer subprocess management and LSP communication
- [x] **Tool Router**: Clean tool definitions using `#[tool]` and `#[tool_router]` macros
- [x] **Document Synchronization**: Proper `textDocument/didOpen` handling
- [x] **Error Handling**: Graceful error propagation from LSP to MCP

### Available Tools
- [x] **`hover`**: Get type information and documentation at cursor position
- [x] **`completion`**: Get code completions at cursor position  
- [x] **`diagnostics`**: Get compile errors and warnings for files
- [x] **`goto_definition`**: Find definition of symbol at position
- [x] **`find_references`**: Find all references to symbol at position
- [x] **`format_document`**: Format Rust code using rustfmt

### Infrastructure
- [x] **Build System**: Working Cargo.toml with all dependencies
- [x] **Client Example**: Full working example demonstrating all features
- [x] **Documentation**: README with setup and usage instructions
- [x] **Project Structure**: Clean separation of concerns (main, lsp_client, tools)

## 🚧 Known Issues & Limitations

### LSP Integration
- [ ] **Limited Position Accuracy**: Hover/completion may not work on all positions
- [x] **Workspace Folders**: Now using `workspace_folders` instead of deprecated `root_uri`
- [ ] **Single Document Focus**: No multi-file project analysis optimization
- [ ] **No Incremental Sync**: Documents are re-opened on each request

### Tool Coverage  
- [ ] **Limited Diagnostics**: Only basic diagnostic information returned
- [ ] **No Semantic Tokens**: Missing syntax highlighting information
- [ ] **No Code Actions**: Missing quick fixes and refactoring suggestions

### Error Handling
- [ ] **LSP Startup Failures**: Better handling when rust-analyzer fails to start
- [ ] **File Not Found**: Improve error messages for missing files
- [ ] **Timeout Handling**: No timeout on LSP requests

## 🎯 Next Steps - Priority Order

### High Priority (Essential Features)

#### 1. High-Value Tools for AI-Assisted Rust Development

##### Critical Tools (Highest Impact)
- [x] **`rename`**: Rename symbols across the entire workspace safely
  - Essential for refactoring support
  - Parameters: file_path, line, column, new_name
  
- [x] **`code_actions`**: Get available quick fixes and refactorings
  - Provides automatic fixes for common issues
  - Add missing imports, fix visibility, implement traits, etc.
  - Parameters: file_path, line, column
  
- [ ] **`workspace_symbols`**: Search for symbols across entire workspace
  - Navigate large codebases efficiently
  - Find any struct, function, trait by name pattern
  - Parameters: query (string pattern)

##### Important Tools (High Value)
- [ ] **`inlay_hints`**: Get type and parameter hints
  - Shows inferred types, parameter names in calls
  - Helps understand complex code
  - Parameters: file_path
  
- [ ] **`expand_macro`**: Expand Rust macros to see generated code
  - Rust-analyzer specific, crucial for debugging macros
  - Parameters: file_path, line, column
  
- [ ] **`runnables`**: Find runnable items (tests, main functions)
  - Identify tests, benchmarks, executables with cargo commands
  - Parameters: file_path

##### Useful Tools (Good to Have)
- [ ] **`implementations`**: Find all implementations of a trait
  - Understand trait usage across codebase
  - Parameters: file_path, line, column
  
- [ ] **`type_definition`**: Go to type definition (not value definition)
  - Find actual type declaration
  - Parameters: file_path, line, column
  
- [ ] **`call_hierarchy`**: Show incoming/outgoing calls
  - Trace function call flow
  - Parameters: file_path, line, column
  
- [ ] **`semantic_tokens`**: Get syntax highlighting information
  - Understand code structure and token types
  - Parameters: file_path

#### 2. Improve LSP Integration
- [x] **Workspace Folders**: ✅ Already implemented - using proper workspace folder support
- [ ] **Better Document Tracking**: Cache opened documents to avoid re-opening
- [ ] **Request Timeouts**: Add configurable timeouts for LSP requests

#### 3. Enhanced Error Handling
- **Startup Validation**: Check rust-analyzer availability before starting
- **Graceful Degradation**: Fall back to basic functionality if LSP fails
- **Better Error Messages**: More helpful error descriptions for users

### Medium Priority (Quality of Life)

#### 4. Configuration Support
```rust
// Add configuration structure
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ServerConfig {
    pub rust_analyzer_path: Option<String>,
    pub workspace_root: Option<String>,
    pub timeout_ms: Option<u64>,
}
```

#### 5. Performance Optimizations
- **Connection Pooling**: Reuse LSP connections across requests
- **Async Request Batching**: Group multiple requests efficiently
- **Caching**: Cache frequently requested information

#### 6. Extended Tool Features
```rust
// Enhanced tool implementations
#[tool(description = "Get code actions (quick fixes) for position")]
async fn code_actions(&self, ...) -> Result<CallToolResult, McpError>

#[tool(description = "Get semantic tokens for syntax highlighting")]
async fn semantic_tokens(&self, ...) -> Result<CallToolResult, McpError>

#[tool(description = "Rename symbol across workspace")]
async fn rename(&self, ...) -> Result<CallToolResult, McpError>
```

### Low Priority (Nice to Have)

#### 7. Multi-Language Support
- **Generic LSP Bridge**: Support any LSP server, not just rust-analyzer
- **Language Detection**: Auto-detect language from file extension
- **Server Management**: Start/stop different language servers as needed

#### 8. Advanced Features
- **Incremental Updates**: Support `textDocument/didChange` for live editing
- **Workspace Symbols**: Search symbols across entire workspace
- **Call Hierarchy**: Show call graphs and relationships
- **Type Hierarchy**: Show type inheritance relationships

#### 9. Deployment & Distribution
- **Docker Container**: Containerized deployment option
- **Binary Releases**: Pre-compiled binaries for different platforms
- **Integration Examples**: Examples for Claude Desktop, other MCP clients

## 📂 File Structure Status

```
language-server-mcp/
├── src/
│   ├── main.rs           ✅ Complete - MCP server with tools
│   └── lsp_client.rs     ✅ Complete - LSP communication
├── examples/
│   ├── client.rs         ✅ Complete - Working client example
│   ├── test_file.rs      ✅ Complete - Test Rust file
│   └── Cargo.toml        ✅ Complete - Example project config
├── Cargo.toml            ✅ Complete - Dependencies configured
├── README.md             ✅ Complete - Usage documentation
├── TODO.md               ✅ Complete - This file
└── run_example.sh        ✅ Complete - Convenience script
```

## 🛠 Development Workflow

### To Add a New Tool:
1. Define request struct with `#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]`
2. Add method to `RustAnalyzerMCP` impl block with `#[tool]` attribute
3. Implement LSP request in `lsp_client.rs` if needed
4. Update client example to test new tool
5. Update README documentation

### To Test Changes:
```bash
# Build and test
cargo build
cargo run --bin client-example

# Or use convenience script
./run_example.sh
```

## 🎯 Success Metrics

### Current State ✅
- [x] MCP server starts without errors
- [x] rust-analyzer connects successfully  
- [x] All three basic tools respond (even if with "no information")
- [x] Client completes example run without crashes
- [x] Clean error handling and logging

### Next Milestone Goals 🎯
- [ ] **Functional hover**: Returns actual type information for Rust code
- [ ] **Working completions**: Returns relevant code completion suggestions
- [ ] **Useful diagnostics**: Shows real compile errors and warnings
- [x] **8 tools available**: ✅ hover, completion, diagnostics, goto_definition, find_references, format_document, rename, code_actions
- [ ] **10+ tools available**: Add workspace_symbols, inlay_hints, expand_macro, and more
- [ ] **Production ready**: Proper error handling, timeouts, configuration

This project successfully demonstrates the potential of bridging LSP servers with MCP for AI assistant integration! 🚀