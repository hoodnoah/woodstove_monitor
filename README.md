# ESP32 Rust Development

Minimal Nix flake for ESP32 Rust development. Nix provides system dependencies, espup manages the Rust toolchain.

## Setup

```bash
# Enter shell
nix develop

# One-time: install dev dependencies using just
just setup

# Restart shell to pick up toolchain
exit
nix develop

# Create project (select your chip when prompted)
cargo generate esp-rs/esp-template
cd your-project

# Edit your rust-toolchain.toml so you have the following:
[toolchain]
channel = "esp"           # Custom ESP toolchain installed by espup
components = ["rust-src"]

# Build and flash
just build
just flash # (flash will build project)
```

## Resources

- [esp-rs Book](https://docs.esp-rs.org/book/)
- [esp-hal Docs](https://docs.esp-rs.org/esp-hal/)
