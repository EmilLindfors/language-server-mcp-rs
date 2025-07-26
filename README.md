# Language Server MCP

A Model Context Protocol (MCP) server that provides rust-analyzer functionality through MCP tools.

## Setup

1. Install rust-analyzer:
```bash
rustup component add rust-analyzer
```

2. Build the MCP server:
```bash
cargo build --release
```

## Usage with Claude Desktop

Add this to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "rust-analyzer": {
      "command": "/path/to/language-server-mcp/target/release/language-server-mcp"
    }
  }
}
```

## Available Tools

### rust_analyzer.hover
Get type information and documentation at a specific position.

**Parameters:**
- `file_path`: Path to the Rust file
- `line`: Line number (0-indexed)
- `column`: Column number (0-indexed)

**Example:**
```json
{
  "file_path": "/home/user/project/src/main.rs",
  "line": 10,
  "column": 15
}
```

### rust_analyzer.completion
Get code completions at a specific position.

**Parameters:**
- `file_path`: Path to the Rust file
- `line`: Line number (0-indexed)
- `column`: Column number (0-indexed)

### rust_analyzer.diagnostics
Get compile errors and warnings for a file.

**Parameters:**
- `file_path`: Path to the Rust file

### rust_analyzer.goto_definition
Find the definition location of a symbol.

**Parameters:**
- `file_path`: Path to the Rust file
- `line`: Line number (0-indexed)
- `column`: Column number (0-indexed)

### rust_analyzer.format
Format a Rust file using rustfmt.

**Parameters:**
- `file_path`: Path to the Rust file

### rust_analyzer.references
Find all references to a symbol.

**Parameters:**
- `file_path`: Path to the Rust file
- `line`: Line number (0-indexed)
- `column`: Column number (0-indexed)

## Example Workflow

When using with an AI assistant like Claude:

1. **Get diagnostics for a file:**
   "Can you check for errors in src/main.rs?"
   
2. **Get type information:**
   "What's the type of the variable at line 25, column 10 in src/lib.rs?"
   
3. **Get completions:**
   "What methods are available on the object at line 30, column 15?"

## Testing the Server

You can test the server manually by sending JSON-RPC requests:

```bash
# Start the server
cargo run

# In another terminal, send a request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run
```

## Development

The server consists of three main components:

1. **MCP Server** (`src/main.rs`): Handles MCP protocol and tool routing
2. **LSP Client** (`src/lsp_client.rs`): Manages rust-analyzer subprocess
3. **Tools** (`src/tools.rs`): Defines available MCP tools

To add new LSP features, extend the tool definitions and implement the corresponding handler in `call_tool()`.