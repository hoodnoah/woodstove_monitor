# ESP32 Rust

LIB_NAME := "woodstove_lib"
DEV_TARGET := "aarch64-apple-darwin"

default:
    @just --list

# Check if all required dependencies are installed
check-deps:
    @./scripts/check-deps.sh

# One-time setup
setup:
    @echo "Installing ESP Rust tooling..."
    cargo install espup
    cargo install cargo-espflash espflash
    espup install --targets "esp32s3"
    @echo ""
    @echo "✅ Setup complete!"
    @echo ""
    @echo "Next steps:"
    @echo "  1. Source ESP environment: source ~/export-esp.sh"
    @echo "  2. Verify dependencies: just check-deps"
    @echo "  3. Set environment variables (see SETUP.md)"
    @echo "  4. Build: just build"

new:
    cargo generate esp-rs/esp-idf-template

build:
    cd monitor && \
    cargo build --release

flash:
    cd monitor && \
    cargo espflash flash --release --monitor --partition-table partitions.csv

monitor:
    cd monitor && \
    cargo espflash monitor

clean:
    cargo clean

test-lib:
    cargo test -p "{{ LIB_NAME }}" --target "{{ DEV_TARGET }}"
