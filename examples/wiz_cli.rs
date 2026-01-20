//! CLI application for controlling Wiz lights.
//!
//! This example demonstrates a full-featured command-line interface for
//! controlling Wiz lights using various commands.
//!
//! Run with: cargo run --example wiz_cli -- --help

use clap::{Parser, Subcommand};
use std::net::Ipv4Addr;
use std::time::Duration;
use wiz_lights_rs::{
    Brightness, Color, Kelvin, Light, Payload, PowerMode, SceneMode, discover_bulbs,
    push::PushManager,
};

#[derive(Parser)]
#[command(name = "wiz-cli")]
#[command(about = "Control Wiz smart lights from the command line", long_about = None)]
struct Cli {
    /// IP address of the Wiz light (not required for discover command)
    #[arg(short, long, global = true)]
    ip: Option<Ipv4Addr>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Discover all Wiz lights on the network
    Discover {
        /// Discovery timeout in seconds (default: 5)
        #[arg(short, long, default_value = "5")]
        timeout: u64,
    },

    /// Get the current status of the light
    Status,

    /// Turn the light on
    On,

    /// Turn the light off
    Off,

    /// Toggle the light on/off
    Toggle,

    /// Set RGB color (0-255 for each component)
    Color {
        /// Red component (0-255)
        red: u8,
        /// Green component (0-255)
        green: u8,
        /// Blue component (0-255)
        blue: u8,
    },

    /// Set brightness (10-100)
    Brightness {
        /// Brightness level (10-100)
        #[arg(value_parser = clap::value_parser!(u8).range(10..=100))]
        level: u8,
    },

    /// Set color temperature in Kelvin (1000-8000)
    Temperature {
        /// Temperature in Kelvin (1000-8000)
        #[arg(value_parser = clap::value_parser!(u16).range(1000..=8000))]
        kelvin: u16,
    },

    /// Set a preset scene
    Scene {
        /// Scene name (e.g., Ocean, Romance, Sunset, Party, etc.)
        /// Available scenes: Ocean, Romance, Sunset, Party, Fireplace, Cozy,
        /// Forest, PastelColors, WakeUp, Bedtime, WarmWhite, Daylight, CoolWhite,
        /// NightLight, Focus, Relax, TrueColors, TvTime, Plantgrowth, Spring,
        /// Summer, Fall, Deepdive, Jungle, Mojito, Club, Christmas, Halloween,
        /// Candlelight, GoldenWhite, Pulse, Steampunk, Diwali, Alarm, WarmFeeling, Rhythm
        scene: String,
    },

    /// Reset the light
    Reset,

    /// Get detailed diagnostics
    Diagnostics,

    /// Listen for push notifications from a light
    Listen {
        /// Local IP address for registration (IP of this machine on the network)
        #[arg(short, long)]
        local_ip: Ipv4Addr,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Discover { timeout } => {
            println!(
                "Discovering Wiz lights on the network (timeout: {}s)...",
                timeout
            );

            match discover_bulbs(Duration::from_secs(timeout)).await {
                Ok(bulbs) => {
                    if bulbs.is_empty() {
                        println!("No lights found on the network.");
                    } else {
                        println!("\nFound {} light(s):", bulbs.len());
                        for bulb in bulbs {
                            println!("  IP: {:15}  MAC: {}", bulb.ip.to_string(), bulb.mac);
                        }
                    }
                }
                Err(e) => eprintln!("Error during discovery: {}", e),
            }
        }

        _ => {
            // All other commands require an IP address
            let ip = cli.ip.ok_or("IP address is required for this command. Use --ip <IP>")?;
            let light = Light::new(ip, None);

            match cli.command {
                Commands::Discover { .. } => unreachable!(),

                Commands::Status => {
                    println!("Getting status for light at {}...", ip);
                    match light.get_status().await {
                        Ok(status) => {
                            println!("\nLight Status:");
                            println!("  Power: {}", if status.emitting() { "ON" } else { "OFF" });

                            if let Some(color) = status.color() {
                                println!(
                                    "  Color: RGB({}, {}, {})",
                                    color.red(),
                                    color.green(),
                                    color.blue()
                                );
                            }

                            if let Some(brightness) = status.brightness() {
                                println!("  Brightness: {}%", brightness.value());
                            }

                            if let Some(temp) = status.temp() {
                                println!("  Temperature: {}K", temp.kelvin());
                            }

                            if let Some(scene) = status.scene() {
                                println!("  Scene: {:?}", scene);
                            }
                        }
                        Err(e) => eprintln!("Error getting status: {}", e),
                    }
                }

                Commands::On => {
                    println!("Turning light ON at {}...", ip);
                    match light.set_power(&PowerMode::On).await {
                        Ok(_) => println!("Light turned ON"),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }

                Commands::Off => {
                    println!("Turning light OFF at {}...", ip);
                    match light.set_power(&PowerMode::Off).await {
                        Ok(_) => println!("Light turned OFF"),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }

                Commands::Toggle => {
                    println!("Toggling light at {}...", ip);
                    match light.toggle().await {
                        Ok(_) => println!("Light toggled"),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }

                Commands::Color { red, green, blue } => {
                    println!(
                        "Setting color to RGB({}, {}, {}) at {}...",
                        red, green, blue, ip
                    );
                    let color = Color::rgb(red, green, blue);
                    let mut payload = Payload::new();
                    payload.color(&color);

                    match light.set(&payload).await {
                        Ok(_) => println!("Color set successfully"),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }

                Commands::Brightness { level } => {
                    println!("Setting brightness to {}% at {}...", level, ip);
                    if let Some(brightness) = Brightness::create(level) {
                        let mut payload = Payload::new();
                        payload.brightness(&brightness);

                        match light.set(&payload).await {
                            Ok(_) => println!("Brightness set successfully"),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    } else {
                        eprintln!("Invalid brightness value. Must be between 10 and 100.");
                    }
                }

                Commands::Temperature { kelvin } => {
                    println!("Setting temperature to {}K at {}...", kelvin, ip);
                    if let Some(temp) = Kelvin::create(kelvin) {
                        let mut payload = Payload::new();
                        payload.temp(&temp);

                        match light.set(&payload).await {
                            Ok(_) => println!("Temperature set successfully"),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    } else {
                        eprintln!("Invalid temperature value. Must be between 1000 and 8000K.");
                    }
                }

                Commands::Scene { scene } => {
                    println!("Setting scene to '{}' at {}...", scene, ip);
                    let scene_mode = match scene.to_lowercase().as_str() {
                        "ocean" => Some(SceneMode::Ocean),
                        "romance" => Some(SceneMode::Romance),
                        "sunset" => Some(SceneMode::Sunset),
                        "party" => Some(SceneMode::Party),
                        "fireplace" => Some(SceneMode::Fireplace),
                        "cozy" => Some(SceneMode::Cozy),
                        "forest" => Some(SceneMode::Forest),
                        "pastelcolors" => Some(SceneMode::PastelColors),
                        "wakeup" => Some(SceneMode::WakeUp),
                        "bedtime" => Some(SceneMode::Bedtime),
                        "warmwhite" => Some(SceneMode::WarmWhite),
                        "daylight" => Some(SceneMode::Daylight),
                        "coolwhite" => Some(SceneMode::CoolWhite),
                        "nightlight" => Some(SceneMode::NightLight),
                        "focus" => Some(SceneMode::Focus),
                        "relax" => Some(SceneMode::Relax),
                        "truecolors" => Some(SceneMode::TrueColors),
                        "tvtime" => Some(SceneMode::TvTime),
                        "plantgrowth" => Some(SceneMode::Plantgrowth),
                        "spring" => Some(SceneMode::Spring),
                        "summer" => Some(SceneMode::Summer),
                        "fall" => Some(SceneMode::Fall),
                        "deepdive" => Some(SceneMode::Deepdive),
                        "jungle" => Some(SceneMode::Jungle),
                        "mojito" => Some(SceneMode::Mojito),
                        "club" => Some(SceneMode::Club),
                        "christmas" => Some(SceneMode::Christmas),
                        "halloween" => Some(SceneMode::Halloween),
                        "candlelight" => Some(SceneMode::Candlelight),
                        "goldenwhite" => Some(SceneMode::GoldenWhite),
                        "pulse" => Some(SceneMode::Pulse),
                        "steampunk" => Some(SceneMode::Steampunk),
                        "diwali" => Some(SceneMode::Diwali),
                        "alarm" => Some(SceneMode::Alarm),
                        "warmfeeling" => Some(SceneMode::WarmFeeling),
                        "rhythm" => Some(SceneMode::Rhythm),
                        _ => None,
                    };

                    if let Some(scene) = scene_mode {
                        let mut payload = Payload::new();
                        payload.scene(&scene);

                        match light.set(&payload).await {
                            Ok(_) => println!("Scene set successfully"),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    } else {
                        eprintln!("Unknown scene name. Use --help to see available scenes.");
                    }
                }

                Commands::Reset => {
                    println!("Resetting light at {}...", ip);
                    match light.reset().await {
                        Ok(_) => println!("Light reset successfully"),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }

                Commands::Diagnostics => {
                    println!("Getting diagnostics for light at {}...", ip);
                    let diag = light.diagnostics().await;
                    println!("\nDiagnostics:\n{}", serde_json::to_string_pretty(&diag)?);
                }

                Commands::Listen { local_ip } => {
                    println!("Setting up push notification listener for light at {}...", ip);
                    println!("Local IP: {}", local_ip);

                    // Get the light's MAC address first
                    let config = light.get_system_config().await?;
                    let mac = config.mac.clone();
                    println!("Light MAC: {}\n", mac);

                    // Create and start push manager
                    let push_manager = PushManager::new();

                    // Subscribe to notifications from this light
                    let display_mac = mac.to_string();
                    push_manager.subscribe(&mac, move |_mac, params| {
                        println!("[{}] State update received:", display_mac);
                        println!("{}\n", serde_json::to_string_pretty(params).unwrap_or_else(|_| format!("{:?}", params)));
                    }).await;

                    // Start listening for push notifications
                    push_manager.start(local_ip).await?;
                    println!("Push manager started on port 38900");

                    // Register with the bulb
                    push_manager.register_bulb(ip).await?;
                    println!("Registered with light at {}", ip);
                    println!("\nListening for push notifications... (Press Ctrl+C to stop)\n");

                    // Keep the program running
                    loop {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }

    Ok(())
}
