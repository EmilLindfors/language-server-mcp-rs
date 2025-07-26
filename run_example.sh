#!/bin/bash

echo "Building the language server MCP..."
cargo build

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo -e "\nRunning the client example..."
echo "This will start the MCP server and demonstrate various rust-analyzer features."
echo "=========================================="

cargo run --bin client-example