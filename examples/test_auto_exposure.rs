use soma_media::{RawProcessor, RawOptions};
use webp::Encoder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = RawProcessor::new()?;
    let file = "sample/03240163.dng";
    
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║              Auto-Exposure Demonstration                      ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");
    println!("File: {}\n", file);
    
    // Helper function to encode WebP
    let encode_webp = |rgb: Vec<u8>, width: u32, height: u32, quality: u8| -> Vec<u8> {
        let encoder = Encoder::from_rgb(&rgb, width, height);
        encoder.encode(quality as f32).to_vec()
    };
    
    // Test 1: Original (no adjustment)
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 1: Original Exposure");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let opts1 = RawOptions {
        auto_exposure: false,
        exposure_compensation: None,
        brightness: 1.0,
        ..RawOptions::fast_preview()
    };
    let start = std::time::Instant::now();
    let (rgb1, w1, h1) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts1)?;
    let webp1 = encode_webp(rgb1, w1, h1, 92);
    std::fs::write("test_output/auto_exp_1_original.webp", &webp1)?;
    println!("✓ Output: {} KB ({}ms)", webp1.len() / 1024, start.elapsed().as_millis());
    println!("  Mode: Manual | Exposure: 0 EV\n");
    
    // Test 2: Auto-Exposure
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 2: Auto-Exposure (Histogram-based)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let opts2 = RawOptions {
        auto_exposure: true,
        exposure_compensation: None,
        brightness: 1.0,
        ..RawOptions::fast_preview()
    };
    let start = std::time::Instant::now();
    let (rgb2, w2, h2) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts2)?;
    let webp2 = encode_webp(rgb2, w2, h2, 92);
    std::fs::write("test_output/auto_exp_2_auto.webp", &webp2)?;
    println!("✓ Output: {} KB ({}ms)", webp2.len() / 1024, start.elapsed().as_millis());
    println!("  Mode: Auto | Exposure: Optimized from RAW histogram\n");
    
    // Test 3: Different image to show auto-exposure adaptation
    let file2 = "sample/202309101781.SRW";
    if std::path::Path::new(file2).exists() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Test 3: Auto-Exposure on Different File");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("File: {}\n", file2);
        
        let (rgb3, w3, h3) = processor.process_raw_from_memory(&std::fs::read(file2)?, &opts2)?;
        let webp3 = encode_webp(rgb3, w3, h3, 92);
        std::fs::write("test_output/auto_exp_3_auto_srw.webp", &webp3)?;
        println!("✓ Output: {} KB", webp3.len() / 1024);
        println!("  Mode: Auto | Adapts to each image\n");
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    println!("📁 Generated files in test_output/:");
    println!("  • auto_exp_1_original.webp  - Original exposure");
    println!("  • auto_exp_2_auto.webp      - Auto-exposure ⭐");
    println!("  • auto_exp_3_auto_srw.webp  - Auto-exposure on SRW");
    
    println!("\n🎯 Auto-Exposure Features:");
    println!("  • Analyzes RAW histogram before demosaic");
    println!("  • Targets middle gray (optimal tonal distribution)");
    println!("  • Adjusts -2 to +3 EV based on image content");
    println!("  • Operates on RAW sensor data (not post-processing)");
    println!("  • Works on underexposed AND overexposed images");
    
    println!("\n💡 Use RawOptions::auto_exposure = true for automatic optimization!");
    
    Ok(())
}
