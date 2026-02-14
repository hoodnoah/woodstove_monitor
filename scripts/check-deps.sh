#!/usr/bin/env bash
set -e

echo "Checking development dependencies..."

missing=()

# Check required commands
for cmd in cmake ninja pkg-config python3 rustup cargo; do
    if ! command -v $cmd &> /dev/null; then
        missing+=($cmd)
    fi
done

# Check LLVM/clang
if ! command -v clang &> /dev/null; then
    missing+=(clang)
fi

# Check environment variables
if [ -z "$LIBCLANG_PATH" ]; then
    echo "⚠️   Warning: LIBCLANG_PATH not set. This may cause esp-idf-sys build failures."
    echo "   See SETUP.md for platform-specific export commands."
fi

if [ ! -f "$HOME/export-esp.sh" ]; then
    echo "⚠️   Warning: ~/export-esp.sh not found. ESP toolchain may not be installed."
    echo "   Run: espup install --targets esp32s3"
fi

if [ ${#missing[@]} -gt 0 ]; then
    echo "❌ Missing required dependencies: ${missing[*]}"
    echo "   See SETUP.md for installation instructions."
    exit 1
fi

echo "✅ All required dependencies found!"
