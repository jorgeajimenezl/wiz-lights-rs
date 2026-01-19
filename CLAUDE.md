# CLAUDE.md - AI Assistant Guide for wiz-lights-rs

## Project Overview

**wiz-lights-rs** is a runtime-agnostic async Rust library for controlling Philips Wiz smart lights over UDP. Supports colors, brightness, temperature, scenes, power states, and fan control.

- **Language**: Rust (Edition 2024)
- **Runtime**: Async (tokio/async-std/smol)
- **Protocol**: UDP on ports 38899 (commands) and 38900 (push notifications)
- **Message Format**: JSON

## Quick Commands

```bash
cargo build                    # Build
cargo test                     # Run tests
cargo clippy                   # Lints
cargo fmt                      # Format
```

## Project Structure

```
src/
├── lib.rs              # Public API entry point
├── light.rs            # Individual light control
├── room.rs             # Batch operations
├── payload.rs          # Command builder
├── discovery.rs        # UDP broadcast discovery
├── response.rs         # Response types
├── status.rs           # Light status tracking
├── history.rs          # Message history
├── config.rs           # Bulb configuration
├── errors.rs           # Error types (thiserror)
├── push.rs             # Push notification manager
├── runtime/            # Runtime abstraction layer
└── types/              # Type definitions
    ├── brightness.rs   # 10-100%
    ├── color.rs        # RGB, RGBW, RGBWW
    ├── fan.rs          # Fan control types
    ├── kelvin.rs       # 1000-8000K
    ├── scene.rs        # Preset scenes
    └── ...
```

## Key Components

### Light Control (`light.rs`)
- UDP port 38899, 1000ms timeout
- 3 retries with exponential backoff (750ms, 1500ms, 3000ms)
- Methods: `get_status()`, `set()`, `set_power()`, `toggle()`, `reset()`
- Fan methods: `fan_set_state()`, `fan_turn_on()`, `fan_turn_off()`

### Payload Builder (`payload.rs`)
```rust
let mut payload = Payload::new();
payload.color(&color);
payload.brightness(&brightness);
light.set(&payload).await?;
```

### Discovery (`discovery.rs`)
```rust
let bulbs = discover_bulbs(Duration::from_secs(5)).await?;
```

### Push Notifications (`push.rs`)
- Listens on port 38900
- Thread-based with callback system

## Code Conventions

### Error Handling
- Use `thiserror` derive macro
- Type alias: `type Result<T> = std::result::Result<T, Error>;`
- Factory methods: `Error::socket("action", err)`

### Serialization
- `#[serde(rename = "camelCase")]` for Wiz API compatibility
- `#[serde_with::skip_serializing_none]` for optional fields

### Type Safety
- Validation methods: `create()` returns `Option`, `create_or()` uses defaults
- Brightness: 10-100, Kelvin: 1000-8000, Speed: 20-200

### Memory Management
- `Arc<Mutex<T>>` for async shared state
- Runtime-agnostic async traits

## Runtime Abstraction

The library is runtime-agnostic. Feature flags:
- `runtime-tokio` (default)
- `runtime-async-std`
- `runtime-smol`

All async code uses traits from `runtime` module.

## Common Tasks

### Adding a Type
1. Create file in `src/types/`
2. Add validation in `create()` returning `Option`
3. Add serde derives
4. Re-export in `src/types/mod.rs` and `src/lib.rs`

### Adding a Command
1. Add field to `Payload` struct
2. Add builder method
3. Update `is_valid()` if needed
4. Update `LightStatus` for tracking

### Adding an Error
1. Add variant to `Error` enum with `#[error("...")]`
2. Add factory method if needed

## Wiz Protocol

### JSON Methods
- `getPilot` - Query state
- `setPilot` - Apply settings
- `setState` - Power on/off
- `getSystemConfig` - System config
- `getUserConfig` / `getModelConfig` - User/model config
- `registration` - Discovery
- `syncPilot` - Push updates

### Discovery Protocol
Broadcast `registration` message to 255.255.255.255:38899

## Things to Avoid

1. Don't use raw numeric values - use type wrappers
2. Don't skip payload validation
3. Don't ignore socket errors
4. Don't block async operations
5. Don't assume all bulbs have all features
