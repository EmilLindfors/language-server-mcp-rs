# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Model Context Protocol (MCP) server that bridges rust-analyzer functionality to MCP tools. It allows MCP clients (like Claude Desktop) to interact with Rust code through language server features.

## Development Commands

### Build
```bash
cargo build --release      # Build optimized binary
cargo build               # Build debug binary
```

### Lint and Format
```bash
cargo fmt                 # Format code
cargo clippy             # Run linter
```

### Test
```bash
cargo test               # Run tests (note: no tests currently exist)
./run_example.sh         # Run the example client to test functionality
```

### Check Code
```bash
cargo check              # Quick compilation check without producing binary
```

## Architecture

The codebase has three main components:

1. **MCP Server** (`src/main.rs`): 
   - Handles MCP protocol communication
   - Routes tool calls to appropriate handlers
   - Uses the `rmcp` crate with `tool_router` macro for tool registration

2. **LSP Client** (`src/lsp_client.rs`):
   - Manages rust-analyzer subprocess lifecycle
   - Handles LSP protocol communication
   - Converts between MCP tool calls and LSP requests

3. **Tool Definitions**:
   - `hover`: Get type info and docs at position
   - `completion`: Get code completions
   - `diagnostics`: Get compile errors/warnings
   - `goto_definition`: Find symbol definitions (TODO)
   - `format`: Format code with rustfmt (TODO)
   - `references`: Find all references to a symbol (TODO)

The server starts rust-analyzer as a subprocess and communicates via JSON-RPC over stdin/stdout. Tool handlers in `main.rs` translate MCP requests to LSP requests and back.

## Key Dependencies

- `rmcp`: MCP protocol implementation from official Rust SDK
  - Source: `https://github.com/modelcontextprotocol/rust-sdk` (main branch)
  - The rmcp crate provides server/client implementations, transport layers, and macro support
  - Key features used: `tool_router` macro for automatic tool registration
- `lsp-types`: LSP protocol types
- `tokio`: Async runtime
- `tracing`: Logging framework

## Important Notes

- The server expects rust-analyzer to be available in PATH
- Workspace root is determined from current directory or first argument
- Example client available in `examples/client.rs`
- The rmcp dependency is pulled directly from GitHub for latest MCP protocol support
- When adding new tools, use the `#[tool]` attribute macro provided by rmcp
- See `TODO.md` for detailed project status and planned improvements

## Local rmcp Reference

The rmcp SDK has been cloned locally in the `rmcp/` directory for easy reference. Key examples include:
- `rmcp/examples/servers/`: Various MCP server implementations
- `rmcp/examples/clients/`: Client implementation examples
- `rmcp/examples/simple-chat-client/`: Full chat client example
- `rmcp/examples/transport/`: Different transport layer examples
The rmcp directory is gitignored to avoid committing the external repository