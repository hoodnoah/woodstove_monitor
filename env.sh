#!/usr/bin/env bash
# Source this file to set up the development environment
# Usage: source env.sh

# Detect platform and set LIBCLANG_PATH
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    if command -v brew &> /dev/null; then
        export LIBCLANG_PATH="$(brew --prefix llvm)/lib"
        export PKG_CONFIG_PATH="$(brew --prefix openssl)/lib/pkgconfig"
    else
        echo "Warning: Homebrew not found. Please install from https://brew.sh"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    # Try to find LLVM (check common versions)
    for version in 18 17 16 15 14; do
        if [ -d "/usr/lib/llvm-$version" ]; then
            export LIBCLANG_PATH="/usr/lib/llvm-$version/lib"
            break
        fi
    done
    if [ -z "$LIBCLANG_PATH" ]; then
        echo "Warning: Could not locate LLVM. Please set LIBCLANG_PATH manually."
    fi
fi

# Source ESP toolchain if available
if [ -f "$HOME/export-esp.sh" ]; then
    source "$HOME/export-esp.sh"
else
    echo "Warning: ~/export-esp.sh not found. Run 'just setup' if you haven't already."
fi

# Load .env file if it exists
if [ -f "monitor/.env" ]; then
    set -a
    source monitor/.env
    set +a
fi

echo "Development environment loaded!"
echo "Run 'just check-deps' to verify your setup."
