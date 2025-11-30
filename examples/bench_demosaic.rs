use soma_media::{RawProcessor, RawOptions};
use webp::Encoder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = RawProcessor::new()?;
    let file = "sample/03240163.dng";
    let data = std::fs::read(file)?;
    
    println!("\n=== Demosaic Algorithm Benchmark ===\n");
    println!("Testing all 13 LibRaw demosaic algorithms");
    println!("With production defaults: Auto-Exposure + Camera WB");
    println!("Output: WebP @ 92% quality");
    println!("File: {}\n", file);
    
    let save_webp = |rgb: &[u8], width: u32, height: u32, path: &str| -> Result<(), Box<dyn std::error::Error>> {
        let encoder = Encoder::from_rgb(rgb, width, height);
        let webp = encoder.encode(92.0);
        std::fs::write(path, &*webp)?;
        Ok(())
    };
    
    let algorithms = vec![
        (0, "LINEAR", "Fast bilinear"),
        (1, "VNG", "Variable Number of Gradients"),
        (2, "PPG", "Patterned Pixel Grouping"),
        (3, "AHD", "Adaptive Homogeneity-Directed"),
        (4, "DCB", "DCB (Color-Based)"),
        (5, "MODIFIED_AHD", "Modified AHD"),
        (6, "AFD", "Adaptive Filtered Demosaicing"),
        (7, "VCD", "Variable Color Differences"),
        (8, "VCD_MODIFIED_AHD", "VCD + Modified AHD"),
        (9, "LMMSE", "Linear Minimum Mean Square Error"),
        (10, "AMAZE", "AMaZE (requires GPL3)"),
        (11, "DHT", "DHT (Discrete Hartley Transform)"),
        (12, "AAHD", "Adaptive AHD (best quality)"),
    ];
    
    let mut results = Vec::new();
    let mut total_time = std::time::Duration::from_millis(0);
    
    for (algo_id, name, description) in &algorithms {
        print!("{:2}. {:17} - {:<35} ", algo_id, name, description);
        std::io::Write::flush(&mut std::io::stdout())?;
        
        let start = std::time::Instant::now();
        
        // Use RawOptions::default() which has auto_exposure=true + camera WB
        let opts = RawOptions {
            demosaic_algorithm: Some(*algo_id),
            dcb_iterations: if *algo_id == 4 { 5 } else { 0 },
            dcb_enhance: *algo_id == 4,
            ..RawOptions::default()  // Auto-exposure + Camera WB enabled!
        };
        
        let result = processor.process_raw_from_memory(&data, &opts);
        let elapsed = start.elapsed();
        
        match result {
            Ok((rgb, w, h)) => {
                let avg = rgb.iter().map(|&v| v as u32).sum::<u32>() / rgb.len() as u32;
                let output_path = format!("test_output/demosaic_{:02}_{}.webp", algo_id, name.to_lowercase());
                save_webp(&rgb, w, h, &output_path)?;
                
                println!("✅ {}ms (avg: {})", elapsed.as_millis(), avg);
                results.push((*algo_id, name.to_string(), elapsed, true, avg));
                total_time += elapsed;
            }
            Err(e) => {
                println!("❌ FAILED: {}", e);
                results.push((*algo_id, name.to_string(), elapsed, false, 0));
            }
        }
    }
    
    println!("\n=== Results Summary ===\n");
    
    // Sort by speed (successful ones only)
    let mut successful: Vec<_> = results.iter()
        .filter(|(_, _, _, success, _)| *success)
        .collect();
    successful.sort_by_key(|(_, _, time, _, _)| *time);
    
    println!("📊 Performance Ranking (fastest to slowest):");
    for (rank, (_id, name, time, _, avg)) in successful.iter().enumerate() {
        let marker = if rank == 0 { "🥇" } else if rank == 1 { "🥈" } else if rank == 2 { "🥉" } else { "  " };
        println!("  {} {:2}. {:17} - {:4}ms (avg brightness: {})", 
                 marker, rank + 1, name, time.as_millis(), avg);
    }
    
    let failed: Vec<_> = results.iter()
        .filter(|(_, _, _, success, _)| !*success)
        .collect();
    
    if !failed.is_empty() {
        println!("\n❌ Failed Algorithms:");
        for (id, name, _, _, _) in failed {
            println!("  {:2}. {} (requires GPL3 demosaic pack or newer LibRaw)", id, name);
        }
    }
    
    println!("\n⏱️  Total Processing Time: {}ms", total_time.as_millis());
    println!("📁 Output: test_output/demosaic_*.webp (WebP @ 92%)");
    println!("\n⚙️  Settings: Auto-Exposure ✅ | Camera WB ✅ | Half-size ✅");
    
    // Find AAHD result (our actual default)
    let aahd_result = results.iter().find(|(id, _, _, _, _)| *id == 12);
    
    // Quality vs Speed recommendations
    println!("\n🎯 Recommendations:");
    if let Some((_, name, time, _, _)) = successful.first() {
        println!("  🚀 Fastest: {} ({}ms) - Use for real-time preview", name, time.as_millis());
    }
    if let Some((_, name, time, _, _)) = successful.get(successful.len() / 2) {
        println!("  ⚖️  Balanced: {} ({}ms) - Good speed/quality trade-off", name, time.as_millis());
    }
    if let Some((_, name, time, success, _)) = aahd_result {
        if *success {
            println!("  ✨ Default (AAHD): {} ({}ms) - Best quality, our default", name, time.as_millis());
        }
    }
    
    Ok(())
}
