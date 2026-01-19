//! Discover all Wiz lights on the network and set them to red.
//!
//! This example demonstrates:
//! - Discovery of Wiz bulbs on the local network
//! - Setting all discovered lights to red color
//!
//! Run with: cargo run --example discover_and_set_red

use std::time::Duration;
use wiz_lights_rs::{Color, Payload, discover_bulbs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Discovering Wiz lights on the network...");

    // Discover bulbs with a 5-second timeout
    let bulbs = discover_bulbs(Duration::from_secs(5)).await?;

    if bulbs.is_empty() {
        println!("No lights found on the network.");
        return Ok(());
    }

    println!("Found {} light(s):", bulbs.len());
    for bulb in &bulbs {
        println!("  - IP: {}, MAC: {}", bulb.ip, bulb.mac);
    }

    // Create red color (RGB: 255, 0, 0)
    let red = Color::rgb(255, 0, 0);

    // Create a payload with the red color
    let mut payload = Payload::new();
    payload.color(&red);

    println!("\nSetting all lights to red...");

    // Set each discovered light to red
    for bulb in bulbs {
        let light = bulb.into_light(None);
        match light.set(&payload).await {
            Ok(_) => println!("  ✓ Successfully set {} to red", light.ip()),
            Err(e) => eprintln!("  ✗ Failed to set {} to red: {}", light.ip(), e),
        }
    }

    println!("\nDone!");
    Ok(())
}
