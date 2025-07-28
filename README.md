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

### Configuration

Claude Desktop uses a configuration file to register MCP servers. The location varies by platform:

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

Add this configuration to your Claude Desktop config file:

```json
{
  "mcpServers": {
    "rust-analyzer": {
      "command": "/path/to/language-server-mcp/target/release/language-server-mcp",
      "args": ["/path/to/your/rust/workspace"]
    }
  }
}
```

**Important Notes:**
- Replace `/path/to/language-server-mcp` with the actual path to this repository
- Replace `/path/to/your/rust/workspace` with the path to your Rust project
- Ensure rust-analyzer is installed and available in your PATH
- After editing the config, completely quit and restart Claude Desktop
- Look for the MCP server indicator (hammer/tools icon) in the bottom-right corner

### Using Desktop Extensions (2025)

For easier installation, you can create a Desktop Extension (.dxt file) that bundles this MCP server. This eliminates the need for manual configuration files.

## Usage with Claude Code CLI

### Installation

First, ensure Claude Code CLI is installed:
```bash
npm install -g @anthropic-ai/claude-code
```

### Configuration Method 1: CLI Wizard

Use the built-in MCP configuration wizard:
```bash
claude mcp add rust-analyzer -s user -- /path/to/language-server-mcp/target/release/language-server-mcp /path/to/your/rust/workspace
```

### Configuration Method 2: Direct Config File

For more control, directly edit the Claude Code configuration file. Create or edit the config file at the appropriate location for your platform and add:

```json
{
  "mcpServers": {
    "rust-analyzer": {
      "type": "stdio",
      "command": "/path/to/language-server-mcp/target/release/language-server-mcp",
      "args": ["/path/to/your/rust/workspace"]
    }
  }
}
```

### Verification

Check that the MCP server is connected:
```bash
# Inside Claude Code, run:
/mcp
```

You should see:
```
⎿ MCP Server Status ⎿
⎿ • rust-analyzer: connected ⎿
```

### Usage Examples

Once configured, you can use natural language commands with Claude Code:

```bash
# Get diagnostics
"Check for errors in src/main.rs"

# Get type information  
"What's the type of the variable at line 25, column 10?"

# Get completions
"What methods are available on this struct?"

# Find definitions
"Where is this function defined?"

# Search workspace
"Find all functions named 'parse' in the codebase"
```

## Available Tools

### hover
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

### completion
Get code completions at a specific position.

**Parameters:**
- `file_path`: Path to the Rust file
- `line`: Line number (0-indexed)
- `column`: Column number (0-indexed)

### diagnostics
Get compile errors and warnings for a file.

**Parameters:**
- `file_path`: Path to the Rust file

### goto_definition
Find the definition location of a symbol.

**Parameters:**
- `file_path`: Path to the Rust file
- `line`: Line number (0-indexed)
- `column`: Column number (0-indexed)

### find_references
Find all references to a symbol.

**Parameters:**
- `file_path`: Path to the Rust file
- `line`: Line number (0-indexed)
- `column`: Column number (0-indexed)
- `include_declaration`: Include the declaration in results (optional, default: true)

### format_document
Format a Rust file using rustfmt.

**Parameters:**
- `file_path`: Path to the Rust file

### rename
Rename symbols across the entire workspace safely.

**Parameters:**
- `file_path`: Path to the Rust file
- `line`: Line number (0-indexed)
- `column`: Column number (0-indexed)
- `new_name`: The new name for the symbol

### code_actions
Get available quick fixes and refactorings at a specific position.

**Parameters:**
- `file_path`: Path to the Rust file
- `line`: Line number (0-indexed)
- `column`: Column number (0-indexed)

### workspace_symbols
Search for symbols across the entire workspace.

**Parameters:**
- `query`: Search query string (symbol name pattern)

**Example:**
```json
{
  "query": "Result"
}
```

### inlay_hints
Get type and parameter hints for a file.

**Parameters:**
- `file_path`: Path to the Rust file

### expand_macro
Expand Rust macros to see the generated code.

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

4. **Search for symbols across the workspace:**
   "Find all functions named 'parse' in the codebase"
   
5. **Get inlay hints for better code understanding:**
   "Show me type hints for src/main.rs"
   
6. **Expand macros to understand generated code:**
   "What does this derive macro expand to at line 15?"
   
7. **Get quick fixes and refactoring suggestions:**
   "What code actions are available for this error?"
   
8. **Safely rename symbols across the workspace:**
   "Rename this function to 'process_data' everywhere it's used"

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