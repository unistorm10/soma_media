//! Example demonstrating soma_media UMA Organ interface
//!
//! Run with: cargo run --example uma_demo

use soma_media::{Organ, MediaOrgan, Stimulus};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== soma_media UMA Interface Demo ===\n");
    
    let organ = MediaOrgan::new();
    
    // 1. Query capabilities
    println!("1. Querying organ capabilities...");
    let stimulus = Stimulus {
        op: "media.capabilities".to_string(),
        input: json!({}),
        context: HashMap::new(),
    };
    
    let response = organ.stimulate(stimulus).await?;
    println!("   Status: {}", if response.ok { "✓ SUCCESS" } else { "✗ FAILED" });
    println!("   Organ: {}", response.output["name"]);
    println!("   Version: {}", response.output["version"]);
    println!("   Division: {}", response.output["division"]);
    println!("   Functions: {}", response.output["functions"].as_array().unwrap().len());
    println!("   Latency: {}ms\n", response.latency_ms);
    
    // 2. Test audio preprocessing (structure)
    println!("2. Testing audio.preprocess operation structure...");
    let stimulus = Stimulus {
        op: "audio.preprocess".to_string(),
        input: json!({
            "input_path": "/tmp/test.mp3",
            "output_path": "/tmp/test.wav",
            "sample_rate": 48000,
            "channels": 1
        }),
        context: HashMap::new(),
    };
    
    // Note: This will fail if files don't exist, but shows the interface
    match organ.stimulate(stimulus).await {
        Ok(resp) => {
            println!("   Status: {}", if resp.ok { "✓ SUCCESS" } else { "✗ FAILED" });
            println!("   Output: {}", resp.output);
        }
        Err(e) => {
            println!("   Expected error (files don't exist): {}", e);
        }
    }
    println!();
    
    // 3. Test video frame extraction (structure)
    println!("3. Testing video.extract_frames operation structure...");
    let stimulus = Stimulus {
        op: "video.extract_frames".to_string(),
        input: json!({
            "video_path": "/tmp/test.mp4",
            "output_dir": "/tmp/frames",
            "fps": 1,
            "width": 336,
            "height": 336
        }),
        context: HashMap::new(),
    };
    
    match organ.stimulate(stimulus).await {
        Ok(resp) => {
            println!("   Status: {}", if resp.ok { "✓ SUCCESS" } else { "✗ FAILED" });
            println!("   Output: {}", resp.output);
        }
        Err(e) => {
            println!("   Expected error (files don't exist): {}", e);
        }
    }
    println!();
    
    // 4. Test unsupported operation
    println!("4. Testing unsupported operation handling...");
    let stimulus = Stimulus {
        op: "invalid.operation".to_string(),
        input: json!({}),
        context: HashMap::new(),
    };
    
    let response = organ.stimulate(stimulus).await?;
    println!("   Status: {}", if response.ok { "✓ SUCCESS" } else { "✗ FAILED (as expected)" });
    println!("   Error: {}", response.output["error"]);
    println!("   Available ops: {:?}\n", response.output["available_operations"]);
    
    // 5. Display organ card
    println!("5. Organ Card Details:");
    let card = organ.describe();
    println!("   Name: {}", card.name);
    println!("   Description: {}", card.description);
    println!("   Tags: {:?}", card.tags);
    println!("   Functions:");
    for func in &card.functions {
        println!("     - {}: {}", func.name, func.description);
        println!("       Tags: {:?}", func.tags);
        println!("       Idempotent: {}", func.idempotent);
        println!("       Examples: {}", func.examples.len());
    }
    
    println!("\n=== UMA Interface Demo Complete ===");
    println!("\nKey Takeaways:");
    println!("  • Stimulus/Response pattern for all operations");
    println!("  • Self-describing via OrganCard");
    println!("  • MCP-compliant capability cards in cards/organ.toml");
    println!("  • Async operations with latency tracking");
    println!("  • Error handling via Response.ok flag");
    
    Ok(())
}
