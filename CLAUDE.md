# CLAUDE.md - AI Assistant Guide for wiz-lights-rs

## Project Overview

**wiz-lights-rs** is a Rust library for controlling Philips Wiz smart lights over UDP. It provides a comprehensive API to communicate with Wiz smart bulbs on local networks, supporting colors, brightness, color temperature, scenes, power states, and fan control for fan-equipped fixtures.

- **Language**: Rust (Edition 2024)
- **Version**: 0.1.0
- **Protocol**: UDP on ports 38899 (commands) and 38900 (push notifications)
- **Message Format**: JSON

## Quick Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Check for compilation errors without building
cargo check

# Format code
cargo fmt

# Run clippy lints
cargo clippy
```

## Project Structure

```
wiz-lights-rs/
├── Cargo.toml              # Project manifest and dependencies
├── Cargo.lock              # Dependency lock file
├── .gitignore              # Git exclusions
└── src/
    ├── lib.rs              # Library entry point with public API re-exports
    ├── light.rs            # Individual light control (main interface)
    ├── room.rs             # Room grouping for batch operations
    ├── payload.rs          # Command payload builder (builder pattern)
    ├── discovery.rs        # UDP broadcast device discovery
    ├── response.rs         # Response envelope types
    ├── status.rs           # Light status tracking
    ├── history.rs          # Message history for debugging
    ├── config.rs           # Bulb configuration and type detection
    ├── errors.rs           # Error types using thiserror
    ├── push.rs             # Push notification manager
    └── types/              # Type definitions
        ├── mod.rs          # Type module re-exports
        ├── brightness.rs   # 10-100% brightness
        ├── color.rs        # RGB, RGBW, RGBWW colors
        ├── fan.rs          # Fan state, mode, direction, speed
        ├── hue_saturation.rs # HSV color representation
        ├── kelvin.rs       # Color temperature (1000-8000K)
        ├── power.rs        # Power mode (On/Off/Reboot)
        ├── ratio.rs        # Dual-head fixture balance
        ├── scene.rs        # 36 preset scenes
        ├── speed.rs        # Animation speed (20-200%)
        └── white.rs        # Cool/warm white intensity
```

## Key Modules and Their Responsibilities

### `light.rs` - Core Light Control
The main interface for controlling individual bulbs:
- UDP communication on port 38899
- Retry logic: 3 retries with exponential backoff (750ms, 1500ms, 3000ms)
- Socket timeout: 1000ms
- Methods: `get_status()`, `set()`, `set_power()`, `toggle()`, `reset()`
- Fan methods: `fan_set_state()`, `fan_turn_on()`, `fan_turn_off()`, `fan_toggle()`

### `payload.rs` - Command Builder
Uses builder pattern for composing lighting commands:
```rust
let mut payload = Payload::new();
payload.color(&color);
payload.brightness(&brightness);
light.set(&payload)?;
```
Validation: Payloads must have at least one attribute set.

### `room.rs` - Batch Operations
Groups lights for batch operations (independent of Wiz app rooms):
- HashMap-based storage with UUID keys
- IP address validation to prevent duplicates

### `discovery.rs` - Network Discovery
UDP broadcast discovery to find bulbs on local network:
```rust
let bulbs = discover_bulbs(Duration::from_secs(5))?;
```

### `push.rs` - Real-time Updates
Push notification manager listening on port 38900:
- Thread-based listener with start/stop lifecycle
- Callback-based event system for state changes

### `errors.rs` - Error Handling
Comprehensive error enum using `thiserror`:
- `JsonDump` / `JsonLoad` - Serialization errors
- `Socket` - Network errors with action context
- `NoAttribute` - Empty payload validation
- `LightNotFound` / `RoomNotFound` - Entity lookup errors
- `InvalidIP` - IP validation errors

## Code Conventions

### Error Handling
- Use `thiserror` derive macro for error types
- `Result<T>` type alias: `type Result<T> = std::result::Result<T, Error>;`
- Factory methods for common error construction (e.g., `Error::socket()`)

### Serialization
- `serde` with `#[serde(rename = "camelCase")]` for Wiz API compatibility
- `#[serde_with::skip_serializing_none]` for optional fields
- Custom serialization for enum types (e.g., SceneMode uses numeric IDs)

### Memory Management
- `RefCell<MessageHistory>` for interior mutability in `Light`
- `Arc<Mutex<T>>` for thread-safe shared state in `PushManager`
- Careful handling of UDP socket lifetimes

### Type Safety
- Enum-based types instead of raw values
- Validation methods: `create()` returns `Option`, `create_or()` uses defaults
- Value ranges enforced at creation time

### Documentation
- Use `///` doc comments on all public items
- Include examples using `no_run` for network operations
- Module-level documentation with `//!`

## Dependencies

| Crate | Purpose |
|-------|---------|
| `log` | Logging facade |
| `serde` | Serialization framework |
| `serde_json` | JSON support |
| `serde_with` | Serde extensions |
| `uuid` | UUID generation (v4) |
| `strum` / `strum_macros` | Enum utilities |
| `thiserror` | Error derive macro |

## Testing

Tests are located alongside the code using `#[cfg(test)]` modules:
- `push.rs`: Push manager tests
- `history.rs`: Message history tests

```bash
cargo test                    # Run all tests
cargo test <test_name>        # Run specific test
cargo test -- --nocapture     # Show println output
```

## Networking Details

### Communication Protocol
- **Protocol**: UDP (connectionless)
- **Command Port**: 38899
- **Push Port**: 38900
- **Discovery**: Broadcast to 255.255.255.255:38899

### JSON Message Methods
| Method | Description |
|--------|-------------|
| `getPilot` | Query current state |
| `setPilot` | Apply settings |
| `setState` | Power on/off |
| `getSystemConfig` | Fetch system configuration |
| `getUserConfig` | Get user settings |
| `getModelConfig` | Get model config (FW >= 1.22) |
| `getPower` | Get power consumption |
| `reset` | Factory reset |
| `reboot` | Reboot bulb |
| `registration` | Discovery/push registration |
| `syncPilot` | Push notification state update |

## Common Tasks for AI Assistants

### Adding a New Type
1. Create file in `src/types/`
2. Implement validation in `create()` method returning `Option`
3. Add serde derive macros
4. Re-export in `src/types/mod.rs`
5. Re-export in `src/lib.rs`

### Adding a New Light Command
1. Add parameters to `Payload` struct in `payload.rs`
2. Add builder method in `Payload` impl
3. Update `is_valid()` if needed
4. Update `LightStatus` if state needs tracking

### Adding a New Error Variant
1. Add variant to `Error` enum in `errors.rs`
2. Add `#[error("...")]` message
3. Add factory method if complex construction needed

### Modifying UDP Communication
Key constants in `Light` struct:
- `PORT: u16 = 38899`
- `TIMEOUT_MS: u64 = 1000`
- `MAX_RETRIES: u32 = 3`
- `RETRY_DELAYS_MS: [u64; 3] = [750, 1500, 3000]`

## Important Patterns

### Builder Pattern (Payload)
```rust
let mut payload = Payload::new();
payload.color(&color);
payload.brightness(&brightness);
payload.scene(&SceneMode::Ocean);
// Validation happens in light.set()
```

### Response Processing
```rust
let response = light.set(&payload)?;
light.process_reply(&response);  // Updates cached status
```

### Type Validation
```rust
// Returns None if invalid
let brightness = Brightness::create(50);  // Some(Brightness)
let invalid = Brightness::create(5);      // None (below 10)

// Or use default for invalid values
let brightness = Brightness::create_or(5, 10);  // Brightness(10)
```

## Things to Avoid

1. **Don't use raw numeric values** - Always use type wrappers (Brightness, Kelvin, etc.)
2. **Don't skip payload validation** - Ensure `is_valid()` returns true before sending
3. **Don't ignore socket errors** - Handle all network failures gracefully
4. **Don't block on push listener** - It runs in a separate thread
5. **Don't assume all bulbs have all features** - Check `BulbType.features` first

## File Size Reference

| File | Lines | Purpose |
|------|-------|---------|
| light.rs | ~520 | Core light control |
| payload.rs | ~400 | Command builder |
| status.rs | ~270 | Status tracking |
| room.rs | ~240 | Room grouping |
| push.rs | ~230 | Push notifications |
| history.rs | ~160 | Message history |
| config.rs | ~150 | Bulb configuration |
| errors.rs | ~100 | Error types |
| types/* | ~560 | Type definitions |
