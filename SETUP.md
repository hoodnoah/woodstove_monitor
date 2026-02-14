# Development Environment Setup

This guide explains how to set up your development environment for the ESP32 woodstove monitor project without Nix.

## Prerequisites

This project requires:
- **System build tools**: cmake, ninja, pkg-config, LLVM/clang
- **Rust toolchain**: rustup and cargo
- **Python 3.12+**: For ESP-IDF build system
- **ESP-specific tools**: espup, cargo-espflash

Follow the instructions for your platform below.

---

## macOS (Apple Silicon)

### 1. Install Homebrew (if not already installed)

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### 2. Install System Dependencies

```bash
brew install cmake ninja pkg-config openssl llvm python@3.12 just
```

### 3. Install Rust

If you don't have Rust installed:

```bash
brew install rustup
rustup-init -y
```

**For bash/zsh:**
```bash
source $HOME/.cargo/env
```

**For fish:**
```fish
set -gx PATH $HOME/.cargo/bin $PATH
```

### 4. Set Environment Variables

**For bash/zsh**, add these to your shell profile (~/.zshrc or ~/.bashrc):

```bash
export LIBCLANG_PATH="$(brew --prefix llvm)/lib"
export PKG_CONFIG_PATH="$(brew --prefix openssl)/lib/pkgconfig"
```

**For fish**, add these to ~/.config/fish/config.fish:

```fish
set -gx LIBCLANG_PATH (brew --prefix llvm)/lib
set -gx PKG_CONFIG_PATH (brew --prefix openssl)/lib/pkgconfig
```

Then reload your shell:

```bash
source ~/.zshrc  # or source ~/.bashrc
```

```fish
source ~/.config/fish/config.fish
```

### 5. Install ESP Tooling

```bash
cargo install espup cargo-espflash espflash
espup install --targets "esp32s3"
```

### 6. Load ESP Environment

**For bash/zsh:**
```bash
source ~/export-esp.sh
```

Add this to your shell profile for automatic loading:

```bash
echo 'source ~/export-esp.sh' >> ~/.zshrc
```

**For fish:**
```fish
source ~/export-esp.sh
```

Add this to ~/.config/fish/config.fish for automatic loading:

```fish
source ~/export-esp.sh
```

### 7. Verify Installation

```bash
cd /path/to/woodstove_monitor
just check-deps
```

---

## Linux (x86_64 - Debian/Ubuntu)

### 1. Install System Dependencies

```bash
sudo apt update
sudo apt install -y \
    cmake ninja-build pkg-config libssl-dev \
    llvm libclang-dev clang \
    python3 python3-pip python3-venv \
    git curl build-essential \
    udev libusb-1.0-0 libudev-dev
```

### 2. Install just (command runner)

Ubuntu/Debian repositories may have an outdated version. Install from cargo:

```bash
# Skip this if apt has a recent version
cargo install just
```

Or use the prebuilt binary:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/bin
```

### 3. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

**For bash/zsh:**
```bash
source $HOME/.cargo/env
```

**For fish:**
```fish
set -gx PATH $HOME/.cargo/bin $PATH
```

### 4. Set Environment Variables

Find your LLVM version:

```bash
ls /usr/lib/ | grep llvm
```

**For bash/zsh**, add to your shell profile (~/.bashrc or ~/.zshrc):

```bash
# Replace '14' with your LLVM version
export LIBCLANG_PATH=/usr/lib/llvm-14/lib
```

**For fish**, add to ~/.config/fish/config.fish:

```fish
# Replace '14' with your LLVM version
set -gx LIBCLANG_PATH /usr/lib/llvm-14/lib
```

Reload your shell:

```bash
source ~/.bashrc
```

```fish
source ~/.config/fish/config.fish
```

### 5. Configure USB Permissions for ESP32

```bash
# Add yourself to dialout group
sudo usermod -a -G dialout $USER

# Create udev rule for ESP32
sudo tee /etc/udev/rules.d/99-esp32.rules > /dev/null <<EOF
SUBSYSTEMS=="usb", ATTRS{idVendor}=="303a", ATTRS{idProduct}=="1001", MODE="0666"
EOF

# Reload udev rules
sudo udevadm control --reload-rules
```

**Important**: Log out and log back in for group membership to take effect.

### 6. Install ESP Tooling

```bash
cargo install espup cargo-espflash espflash
espup install --targets "esp32s3"
```

### 7. Load ESP Environment

**For bash/zsh:**
```bash
source ~/export-esp.sh
```

Add to your shell profile for automatic loading:

```bash
echo 'source ~/export-esp.sh' >> ~/.bashrc
```

**For fish:**
```fish
source ~/export-esp.sh
```

Add to ~/.config/fish/config.fish for automatic loading:

```fish
source ~/export-esp.sh
```

### 8. Verify Installation

```bash
cd /path/to/woodstove_monitor
just check-deps
```

---

## Using the Project

### Environment Setup (Each Session)

Instead of nix develop, source the environment helper:

**For bash/zsh:**
```bash
cd woodstove_monitor
source env.sh
```

**For fish:**
```fish
cd woodstove_monitor
source env.sh
```

This automatically:
- Sets LIBCLANG_PATH for your platform
- Sources ESP toolchain (~/export-esp.sh)
- Loads WiFi/MQTT credentials from monitor/.env if present

### Building and Flashing

```bash
# Check dependencies
just check-deps

# Build release binary
just build

# Flash to device and monitor
just flash

# Monitor only
just monitor
```

### Configuration

Create monitor/.env with your WiFi and MQTT credentials:

```bash
WIFI_SSID=your_network
WIFI_PASSWORD=your_password
MQTT_ENDPOINT=mqtt://broker.example.com
MQTT_USER=mqtt_user
MQTT_PASS=mqtt_password
```

---

## Troubleshooting

### "libclang not found" or "bindgen failed"

**Symptom**: Build fails with errors about libclang or bindgen.

**Solution**:
- Ensure LIBCLANG_PATH is set correctly
- macOS: `export LIBCLANG_PATH="$(brew --prefix llvm)/lib"`
- Linux: `export LIBCLANG_PATH=/usr/lib/llvm-<version>/lib`

### "espflash: command not found"

**Symptom**: Can't flash the ESP32 device.

**Solution**:
- Ensure ~/.cargo/bin is in your PATH
- Run: `cargo install espflash cargo-espflash`

### "Permission denied" when flashing (Linux)

**Symptom**: Can't access /dev/ttyUSB0 or /dev/ttyACM0.

**Solution**:
- Add yourself to dialout group: `sudo usermod -a -G dialout $USER`
- Log out and log back in
- Verify: `groups` (should include "dialout")

### ESP environment not loading

**Symptom**: ~/export-esp.sh not found.

**Solution**:
- Run `espup install --targets "esp32s3"`
- This creates ~/export-esp.sh automatically

### First build is very slow

**Expected behavior**: The first build downloads and compiles ESP-IDF (10-15 minutes). Subsequent builds are fast (<1 minute for incremental changes).

---

## Differences from Nix Setup

| Aspect | Nix (Old) | Direct Install (New) |
|--------|-----------|-------------------|
| Environment activation | nix develop (automatic with direnv) | source env.sh |
| Dependency installation | Automatic | Manual per platform |
| Reproducibility | Exact versions locked | System package versions |
| Setup time | ~5 minutes (first time) | ~10-15 minutes |
| Updates | nix flake update | brew upgrade / apt upgrade |
