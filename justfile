# ESP32 Rust

default:
    @just --list

# One-time setup
setup:
    cargo install espup
    cargo install cargo-espflash espflash
    espup install --targets "esp32c2"
    @echo "Restart shell: exit, then nix develop"

new:
    cargo generate esp-rs/esp-idf-template

build:
    cargo build --release

flash:
    cargo espflash flash --release --monitor

monitor:
    cargo espflash monitor

clean:
    cargo clean

test-lib:
    cargo test -p woodstove-logic --target aarch64-apple-darwin