# Examples

This directory contains example applications demonstrating how to use the `wiz-lights-rs` library.

## discover_and_set_red

Discovers all Wiz lights on your network and sets them to red color.

**Usage:**
```bash
cargo run --example discover_and_set_red
```

**What it does:**
- Broadcasts a discovery message on your local network
- Waits 5 seconds for bulb responses
- Displays all discovered lights with their IP and MAC addresses
- Sets each discovered light to red (RGB: 255, 0, 0)

## wiz_cli

A full-featured command-line interface for controlling Wiz lights.

**Usage:**
```bash
# Get help
cargo run --example wiz_cli -- --help

# Get status of a light
cargo run --example wiz_cli -- --ip 192.168.1.100 status

# Turn light on/off
cargo run --example wiz_cli -- --ip 192.168.1.100 on
cargo run --example wiz_cli -- --ip 192.168.1.100 off

# Toggle light
cargo run --example wiz_cli -- --ip 192.168.1.100 toggle

# Set RGB color
cargo run --example wiz_cli -- --ip 192.168.1.100 color 255 0 0    # Red
cargo run --example wiz_cli -- --ip 192.168.1.100 color 0 255 0    # Green
cargo run --example wiz_cli -- --ip 192.168.1.100 color 0 0 255    # Blue

# Set brightness (10-100)
cargo run --example wiz_cli -- --ip 192.168.1.100 brightness 50

# Set color temperature in Kelvin (1000-8000)
cargo run --example wiz_cli -- --ip 192.168.1.100 temperature 2700  # Warm white
cargo run --example wiz_cli -- --ip 192.168.1.100 temperature 6500  # Cool white

# Set a preset scene
cargo run --example wiz_cli -- --ip 192.168.1.100 scene Ocean
cargo run --example wiz_cli -- --ip 192.168.1.100 scene Sunset
cargo run --example wiz_cli -- --ip 192.168.1.100 scene Party

# Reset the light
cargo run --example wiz_cli -- --ip 192.168.1.100 reset

# Get detailed diagnostics
cargo run --example wiz_cli -- --ip 192.168.1.100 diagnostics
```

**Available scenes:**
Ocean, Romance, Sunset, Party, Fireplace, Cozy, Forest, PastelColors, WakeUp, Bedtime, WarmWhite, Daylight, CoolWhite, NightLight, Focus, Relax, TrueColors, TvTime, Plantgrowth, Spring, Summer, Fall, Deepdive, Jungle, Mojito, Club, Christmas, Halloween, Candlelight, GoldenWhite, Pulse, Steampunk, Diwali, Alarm, WarmFeeling, Rhythm

## Notes

- Make sure your Wiz lights are on the same network as your computer
- For best results, assign static IP addresses to your lights in your router settings
- The examples use the default tokio runtime
