# ESP32 Rust

LIB_NAME := "woodstove_lib"
DEV_TARGET := "aarch64-apple-darwin"

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
    cd monitor && \
    cargo build --release

flash:
    cd monitor && \
    cargo espflash flash --release --monitor

monitor:
    cd monitor && \
    cargo espflash monitor

clean:
    cargo clean

test-lib:
    cargo test -p "{{ LIB_NAME }}" --target "{{ DEV_TARGET }}"
