use soma_media::{RawProcessor, RawOptions};
use webp::Encoder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = RawProcessor::new()?;
    let file = "sample/03240163.dng";
    
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║     Exposure Compensation vs Brightness Test                 ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");
    println!("File: {}\n", file);
    
    // Helper function to encode WebP
    let encode_webp = |rgb: Vec<u8>, width: u32, height: u32, quality: u8| -> Vec<u8> {
        let encoder = Encoder::from_rgb(&rgb, width, height);
        encoder.encode(quality as f32).to_vec()
    };
    
    // Test 1: Baseline (no adjustment)
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 1: Baseline (no adjustment)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let opts1 = RawOptions {
        exposure_compensation: None,
        brightness: 1.0,
        ..RawOptions::fast_preview()
    };
    let start = std::time::Instant::now();
    let (rgb1, w1, h1) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts1)?;
    let webp1 = encode_webp(rgb1, w1, h1, 92);
    std::fs::write("test_output/exp_1_baseline.webp", &webp1)?;
    println!("✓ Output: {} KB ({}ms)", webp1.len() / 1024, start.elapsed().as_millis());
    println!("  Exposure: 0 EV | Brightness: 1.0x\n");
    
    // Test 2: +1 EV exposure compensation (RAW data)
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 2: +1.0 EV Exposure Compensation (on RAW data)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let opts2 = RawOptions {
        exposure_compensation: Some(1.0),  // +1 EV = 2x exposure on RAW
        brightness: 1.0,
        ..RawOptions::fast_preview()
    };
    let start = std::time::Instant::now();
    let (rgb2, w2, h2) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts2)?;
    let webp2 = encode_webp(rgb2, w2, h2, 92);
    std::fs::write("test_output/exp_2_plus1ev_raw.webp", &webp2)?;
    println!("✓ Output: {} KB ({}ms)", webp2.len() / 1024, start.elapsed().as_millis());
    println!("  Exposure: +1 EV | Brightness: 1.0x");
    println!("  Applied to RAW sensor data (better quality)\n");
    
    // Test 3: 2x brightness (post-processing)
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 3: 2x Brightness (post-processing)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let opts3 = RawOptions {
        exposure_compensation: None,
        brightness: 2.0,  // 2x post-demosaic (lower quality)
        ..RawOptions::fast_preview()
    };
    let start = std::time::Instant::now();
    let (rgb3, w3, h3) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts3)?;
    let webp3 = encode_webp(rgb3, w3, h3, 92);
    std::fs::write("test_output/exp_3_2x_brightness.webp", &webp3)?;
    println!("✓ Output: {} KB ({}ms)", webp3.len() / 1024, start.elapsed().as_millis());
    println!("  Exposure: 0 EV | Brightness: 2.0x");
    println!("  Applied AFTER demosaic (may clip highlights)\n");
    
    // Test 4: +2 EV exposure compensation
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 4: +2.0 EV Exposure Compensation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let opts4 = RawOptions {
        exposure_compensation: Some(2.0),  // +2 EV = 4x exposure
        brightness: 1.0,
        ..RawOptions::fast_preview()
    };
    let start = std::time::Instant::now();
    let (rgb4, w4, h4) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts4)?;
    let webp4 = encode_webp(rgb4, w4, h4, 92);
    std::fs::write("test_output/exp_4_plus2ev_raw.webp", &webp4)?;
    println!("✓ Output: {} KB ({}ms)", webp4.len() / 1024, start.elapsed().as_millis());
    println!("  Exposure: +2 EV (4x) | Brightness: 1.0x");
    println!("  Extreme boost on RAW data\n");
    
    // Test 5: -1 EV exposure compensation
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 5: -1.0 EV Exposure Compensation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let opts5 = RawOptions {
        exposure_compensation: Some(-1.0),  // -1 EV = 0.5x exposure
        brightness: 1.0,
        ..RawOptions::fast_preview()
    };
    let start = std::time::Instant::now();
    let (rgb5, w5, h5) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts5)?;
    let webp5 = encode_webp(rgb5, w5, h5, 92);
    std::fs::write("test_output/exp_5_minus1ev_raw.webp", &webp5)?;
    println!("✓ Output: {} KB ({}ms)", webp5.len() / 1024, start.elapsed().as_millis());
    println!("  Exposure: -1 EV (0.5x) | Brightness: 1.0x");
    println!("  Darker, preserves highlights\n");
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    println!("📁 Generated files in test_output/:");
    println!("  1. exp_1_baseline.webp         - No adjustment");
    println!("  2. exp_2_plus1ev_raw.webp      - +1 EV on RAW data ⭐");
    println!("  3. exp_3_2x_brightness.webp    - 2x post-processing");
    println!("  4. exp_4_plus2ev_raw.webp      - +2 EV on RAW data");
    println!("  5. exp_5_minus1ev_raw.webp     - -1 EV on RAW data");
    
    println!("\n🔍 Compare #2 vs #3 to see the difference:");
    println!("  • #2 (exposure): Better highlight preservation, natural look");
    println!("  • #3 (brightness): May clip highlights, artificial look");
    
    println!("\n💡 Exposure compensation operates on RAW sensor data,");
    println!("   preserving more dynamic range than brightness adjustment!");
    
    Ok(())
}
