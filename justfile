# ESP32 Rust

default:
    @just --list

# One-time setup
setup:
    cargo install espup
    espup install
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
