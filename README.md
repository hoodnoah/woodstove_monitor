# ESP32 Rust Development

Minimal Nix flake for ESP32 Rust development. Nix provides system dependencies, espup manages the Rust toolchain.

## Setup

```bash
# Enter shell
nix develop

# One-time: install ESP Rust toolchain
cargo install espup
espup install

# Restart shell to pick up toolchain
exit
nix develop

# Create project (select your chip when prompted)
cargo generate esp-rs/esp-template
cd your-project

# Build and flash
cargo build --release
cargo espflash flash --monitor
```

## Resources

- [esp-rs Book](https://docs.esp-rs.org/book/)
- [esp-hal Docs](https://docs.esp-rs.org/esp-hal/)
