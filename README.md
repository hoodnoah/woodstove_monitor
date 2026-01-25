# Woodstove Monitor

A Rust-based temperature monitoring system for woodstoves using an Arduino Nano ESP32 and a MAX31855 thermocouple sensor. The device reads temperature data and publishes it to MQTT for remote monitoring.

## Features

- **Temperature Monitoring**: Reads thermocouple data via MAX31855 SPI sensor
- **WiFi Connectivity**: Connects to WiFi networks for remote access
- **MQTT Publishing**: Publishes temperature readings and stove state to an MQTT broker
- **State Machine**: Tracks woodstove burn state based on temperature readings
- **Low Power**: Optimized for minimal resource usage on embedded systems

## Hardware

- Arduino Nano ESP32 (ESP32-S3)
- MAX31855 Thermocouple Amplifier (SPI interface)
- Status LED (GPIO8)
- WiFi antenna

## Project Structure

- `monitor/`: Main ESP32 application (firmware)
  - `main.rs`: Entry point, hardware setup, main loop
  - `mqtt.rs`: MQTT client and publishing logic
  - `wifi.rs`: WiFi connectivity
- `woodstove_lib/`: Shared library with core logic
  - `state_machine.rs`: Stove state tracking
  - `sensor.rs`: Sensor abstractions
  - `temperature.rs`: Temperature value types

## Setup

### Prerequisites

```bash
# Enter development environment
nix develop

# One-time: install dev dependencies
just setup

# Restart shell to pick up toolchain
exit
nix develop
```

### Configuration

Set environment variables for your WiFi and MQTT credentials:

```bash
export WIFI_SSID="your_ssid"
export WIFI_PASSWORD="your_password"
export MQTT_ENDPOINT="mqtt://broker.example.com"
export MQTT_USER="username"
export MQTT_PASS="password"
```

Alternatively, create a `.env` file in the `monitor/` directory.

## Building and Flashing

```bash
# Build release binary
just build

# Flash to device and monitor serial output
just flash

# Monitor only (if device is already flashed)
just monitor

# Clean build artifacts
just clean
```

## Development

```bash
# Test the shared library
just test-lib
```

## Resources

- [esp-rs Book](https://docs.espressif.com/projects/rust/book/)
- [esp-idf-svc Documentation](https://docs.rs/esp-idf-svc/)
- [MAX31855 Datasheet](https://datasheets.maximintegrated.com/en/ds/MAX31855.pdf)
