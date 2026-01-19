# wiz-lights-rs

A **runtime-agnostic** async Rust library for controlling Philips Wiz smart lights over UDP.

## Features

- **Runtime Agnostic**: Works with tokio, async-std, or smol
- **Full Control**: RGB colors, brightness, color temperature, and 36+ preset scenes
- **Fan Control**: Support for fan-equipped fixtures (speed, mode, direction)
- **Discovery**: Automatic bulb discovery via UDP broadcast
- **Batch Operations**: Group lights into rooms for concurrent control
- **Push Notifications**: Real-time state updates from bulbs
- **Type Safe**: Strongly typed API with validation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
wiz-lights-rs = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Runtime Selection

#### tokio (default)

```toml
[dependencies]
wiz-lights-rs = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

#### async-std

```toml
[dependencies]
wiz-lights-rs = { version = "0.1", default-features = false, features = ["runtime-async-std"] }
async-std = { version = "1.12", features = ["attributes"] }
```

#### smol

```toml
[dependencies]
wiz-lights-rs = { version = "0.1", default-features = false, features = ["runtime-smol"] }
smol = "2"
```

## Quick Start

```rust
use std::net::Ipv4Addr;
use std::str::FromStr;
use wiz_lights_rs::{Light, Payload, Color, Brightness};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a light instance
    let light = Light::new(
        Ipv4Addr::from_str("192.168.1.100")?,
        Some("Living Room")
    );

    // Set color and brightness
    let mut payload = Payload::new();
    payload.color(&Color::from_str("255,0,0")?); // Red
    payload.brightness(&Brightness::create(80).unwrap());
    light.set(&payload).await?;

    // Toggle power
    light.toggle().await?;

    Ok(())
}
```

## Examples

### Discovery

Find all bulbs on your network:

```rust
use std::time::Duration;
use wiz_lights_rs::discover_bulbs;

let bulbs = discover_bulbs(Duration::from_secs(5)).await?;
for bulb in bulbs {
    println!("Found: {} at {}", bulb.mac, bulb.ip);
    let light = bulb.into_light(Some("My Light"));
}
```

### Color Control

```rust
use wiz_lights_rs::{Payload, Color, Brightness};

// RGB color
let mut payload = Payload::new();
payload.color(&Color::from_str("0,255,0")?); // Green
payload.brightness(&Brightness::create(100).unwrap());
light.set(&payload).await?;

// Color temperature
let mut payload = Payload::new();
payload.temp(&Kelvin::create(4000).unwrap()); // Warm white
light.set(&payload).await?;
```

### Preset Scenes

```rust
use wiz_lights_rs::{Payload, SceneMode};

let mut payload = Payload::new();
payload.scene(&SceneMode::Sunset);
light.set(&payload).await?;
```

Available scenes: `Ocean`, `Romance`, `Sunset`, `Party`, `Fireplace`, `Cozy`, `Forest`, `WakeUp`, `Bedtime`, `Focus`, `Relax`, `Christmas`, `Halloween`, and more.

### Room Control

Batch operations on multiple lights:

```rust
use wiz_lights_rs::{Room, Light};

let mut room = Room::new("Living Room");

// Add lights
let light1 = Light::new(Ipv4Addr::from_str("192.168.1.100")?, Some("Lamp 1"));
let light2 = Light::new(Ipv4Addr::from_str("192.168.1.101")?, Some("Lamp 2"));
room.new_light(light1)?;
room.new_light(light2)?;

// Get status from all lights concurrently
let statuses = room.get_status().await?;
```

### Fan Control

For fan-equipped fixtures:

```rust
use wiz_lights_rs::{FanSpeed, FanMode, FanDirection};

// Turn fan on at speed 3
light.fan_turn_on(Some(FanMode::Normal), Some(FanSpeed::create(3, None)?)).await?;

// Set fan direction
light.set_fan_direction(FanDirection::Reverse).await?;

// Toggle fan
light.fan_toggle().await?;
```

### Push Notifications

Receive real-time updates:

```rust
use wiz_lights_rs::push::PushManager;

let mut manager = PushManager::new();
manager.register_callback(|response| {
    println!("Light updated: {:?}", response);
});
manager.start().await?;
```

### Query Status

```rust
// Get current state
let status = light.get_status().await?;
println!("Power: {}", status.emitting());
if let Some(color) = status.color() {
    println!("Color: RGB({}, {}, {})", color.red(), color.green(), color.blue());
}
if let Some(brightness) = status.brightness() {
    println!("Brightness: {}%", brightness.value());
}

// Get diagnostics
let diag = light.diagnostics().await;
println!("{}", serde_json::to_string_pretty(&diag)?);
```

## Type System

All parameters use strongly-typed wrappers with validation:

- **Brightness**: 10-100%
- **Kelvin**: 1000-8000K
- **Speed**: 20-200%
- **Color**: RGB (0-255 per channel)
- **FanSpeed**: 1-N (varies by fixture)

Types provide two creation methods:
- `create(value)`: Returns `Option<T>`, `None` if invalid
- `create_or(value)`: Returns valid value or default

```rust
let brightness = Brightness::create(80).unwrap(); // Some(80)
let invalid = Brightness::create(5); // None (below minimum)
let safe = Brightness::create_or(5); // Returns default (100)
```

## Network Requirements

- Bulbs must be on the same local network
- UDP ports 38899 (commands) and 38900 (push notifications)
- Static IP addresses recommended for reliability
- No internet connection required

## Error Handling

All network operations return `Result<T, Error>`:

```rust
match light.set(&payload).await {
    Ok(response) => println!("Success"),
    Err(e) => eprintln!("Error: {}", e),
}
```

Common errors:
- `Socket`: Network communication failures
- `NoAttribute`: Empty payload validation
- `JsonLoad`/`JsonDump`: Serialization errors
- `LightNotFound`: Invalid light reference

## Architecture

The library uses:
- **UDP** for all communication (ports 38899, 38900)
- **JSON** message format matching Wiz protocol
- **Retry logic**: 3 attempts with exponential backoff
- **Timeout**: 1000ms per request
- **Runtime abstraction**: Works with any async runtime

## Contributing

Contributions are welcome! Please ensure code passes:
```bash
cargo test
cargo clippy
cargo fmt --check
```

## License

This project is licensed under the MIT License.

## Acknowledgments

This library is not affiliated with or endorsed by Philips or WiZ. It is a reverse-engineered implementation of the WiZ local UDP protocol.
