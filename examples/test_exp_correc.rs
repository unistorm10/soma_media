use soma_media::{RawProcessor, RawOptions};
use webp::Encoder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = RawProcessor::new()?;
    let file = "sample/03240163.dng";
    
    println!("\n=== Testing if exp_correc actually works ===\n");
    
    let encode_webp = |rgb: Vec<u8>, width: u32, height: u32| -> Vec<u8> {
        let encoder = Encoder::from_rgb(&rgb, width, height);
        encoder.encode(92.0).to_vec()
    };
    
    // Test 1: Baseline
    println!("1. Baseline (0 EV)");
    let opts1 = RawOptions {
        exposure_compensation: None,
        auto_exposure: false,
        ..RawOptions::fast_preview()
    };
    let (rgb1, w1, h1) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts1)?;
    
    // Calculate average brightness
    let avg1 = rgb1.iter().map(|&v| v as u32).sum::<u32>() / rgb1.len() as u32;
    
    let webp1 = encode_webp(rgb1, w1, h1);
    std::fs::write("test_output/exp_test_0ev.webp", &webp1)?;
    println!("   Avg brightness: {} | Size: {} KB\n", avg1, webp1.len() / 1024);
    
    // Test 2: Manual +1 EV
    println!("2. Manual +1 EV");
    let opts2 = RawOptions {
        exposure_compensation: Some(1.0),
        auto_exposure: false,
        ..RawOptions::fast_preview()
    };
    let (rgb2, w2, h2) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts2)?;
    let avg2 = rgb2.iter().map(|&v| v as u32).sum::<u32>() / rgb2.len() as u32;
    let webp2 = encode_webp(rgb2, w2, h2);
    std::fs::write("test_output/exp_test_1ev.webp", &webp2)?;
    println!("   Avg brightness: {} | Size: {} KB", avg2, webp2.len() / 1024);
    println!("   Change: {:+} ({:.1}% brighter)\n", avg2 as i32 - avg1 as i32, 
             (avg2 as f32 / avg1 as f32 - 1.0) * 100.0);
    
    // Test 3: Manual +3 EV
    println!("3. Manual +3 EV");
    let opts3 = RawOptions {
        exposure_compensation: Some(3.0),
        auto_exposure: false,
        ..RawOptions::fast_preview()
    };
    let (rgb3, w3, h3) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts3)?;
    let avg3 = rgb3.iter().map(|&v| v as u32).sum::<u32>() / rgb3.len() as u32;
    let webp3 = encode_webp(rgb3, w3, h3);
    std::fs::write("test_output/exp_test_3ev.webp", &webp3)?;
    println!("   Avg brightness: {} | Size: {} KB", avg3, webp3.len() / 1024);
    println!("   Change: {:+} ({:.1}% brighter)\n", avg3 as i32 - avg1 as i32,
             (avg3 as f32 / avg1 as f32 - 1.0) * 100.0);
    
    // Test 4: Auto exposure
    println!("4. Auto Exposure");
    let opts4 = RawOptions {
        exposure_compensation: None,
        auto_exposure: true,
        ..RawOptions::fast_preview()
    };
    let (rgb4, w4, h4) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts4)?;
    let avg4 = rgb4.iter().map(|&v| v as u32).sum::<u32>() / rgb4.len() as u32;
    let webp4 = encode_webp(rgb4, w4, h4);
    std::fs::write("test_output/exp_test_auto.webp", &webp4)?;
    println!("   Avg brightness: {} | Size: {} KB", avg4, webp4.len() / 1024);
    println!("   Change: {:+} ({:.1}% brighter)\n", avg4 as i32 - avg1 as i32,
             (avg4 as f32 / avg1 as f32 - 1.0) * 100.0);
    
    println!("=== Results ===");
    if avg2 > avg1 {
        println!("✅ exp_correc IS working! (+1 EV made image brighter)");
    } else {
        println!("❌ exp_correc NOT working! (no brightness change)");
    }
    
    if avg4 > avg1 {
        println!("✅ Auto-exposure IS working!");
    } else {
        println!("❌ Auto-exposure NOT working!");
    }
    
    Ok(())
}
