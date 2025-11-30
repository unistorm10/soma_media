use soma_media::{RawProcessor, RawOptions};
use image::codecs::jpeg::JpegEncoder;
use webp::Encoder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = RawProcessor::new()?;
    let file = "sample/03240163.dng";
    
    println!("\n=== RAW-Level Exposure Adjustment Test ===\n");
    println!("Using exp_shift for true RAW exposure (like rawpy/Darktable/ACR)\n");
    
    let save_jpeg = |rgb: &[u8], width: u32, height: u32, path: &str| -> Result<(), Box<dyn std::error::Error>> {
        let mut output = std::fs::File::create(path)?;
        let mut encoder = JpegEncoder::new_with_quality(&mut output, 85);
        encoder.encode(rgb, width, height, image::ExtendedColorType::Rgb8)?;
        Ok(())
    };
    
    let save_webp = |rgb: &[u8], width: u32, height: u32, path: &str| -> Result<(), Box<dyn std::error::Error>> {
        let encoder = Encoder::from_rgb(rgb, width, height);
        let webp = encoder.encode(92.0);
        std::fs::write(path, &*webp)?;
        Ok(())
    };
    
    // Test 1: Baseline
    println!("1. Baseline (0 EV) - RAW level");
    let start = std::time::Instant::now();
    let opts1 = RawOptions {
        exposure_compensation: None,
        auto_exposure: false,
        white_balance: soma_media::WhiteBalance::None,
        ..RawOptions::fast_preview()
    };
    let (rgb1, w1, h1) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts1)?;
    let process_time1 = start.elapsed();
    let avg1 = rgb1.iter().map(|&v| v as u32).sum::<u32>() / rgb1.len() as u32;
    save_jpeg(&rgb1, w1, h1, "test_output/raw_exp_0ev.jpg")?;
    save_webp(&rgb1, w1, h1, "test_output/raw_exp_0ev.webp")?;
    println!("   Avg brightness: {} | Processing: {}ms\n", avg1, process_time1.as_millis());
    
    // Test 2: +1 EV via exp_shift
    println!("2. +1 EV - RAW level (exp_shift)");
    let start = std::time::Instant::now();
    let opts2 = RawOptions {
        exposure_compensation: Some(1.0),
        auto_exposure: false,
        white_balance: soma_media::WhiteBalance::None,
        ..RawOptions::fast_preview()
    };
    let (rgb2, w2, h2) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts2)?;
    let process_time2 = start.elapsed();
    let avg2 = rgb2.iter().map(|&v| v as u32).sum::<u32>() / rgb2.len() as u32;
    save_jpeg(&rgb2, w2, h2, "test_output/raw_exp_1ev.jpg")?;
    save_webp(&rgb2, w2, h2, "test_output/raw_exp_1ev.webp")?;
    println!("   Avg brightness: {} | Change: {:+} ({:.1}% brighter)", 
             avg2, avg2 as i32 - avg1 as i32, (avg2 as f32 / avg1 as f32 - 1.0) * 100.0);
    println!("   Processing: {}ms | Expected: ~100% brighter (2x)\n", process_time2.as_millis());
    
    // Test 3: +3 EV via exp_shift
    println!("3. +3 EV - RAW level (exp_shift)");
    let start = std::time::Instant::now();
    let opts3 = RawOptions {
        exposure_compensation: Some(3.0),
        auto_exposure: false,
        white_balance: soma_media::WhiteBalance::None,
        ..RawOptions::fast_preview()
    };
    let (rgb3, w3, h3) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts3)?;
    let process_time3 = start.elapsed();
    let avg3 = rgb3.iter().map(|&v| v as u32).sum::<u32>() / rgb3.len() as u32;
    save_jpeg(&rgb3, w3, h3, "test_output/raw_exp_3ev.jpg")?;
    save_webp(&rgb3, w3, h3, "test_output/raw_exp_3ev.webp")?;
    println!("   Avg brightness: {} | Change: {:+} ({:.1}% brighter)",
             avg3, avg3 as i32 - avg1 as i32, (avg3 as f32 / avg1 as f32 - 1.0) * 100.0);
    println!("   Processing: {}ms | Expected: ~700% brighter (8x)\n", process_time3.as_millis());
    
    // Test 4: Auto-exposure
    println!("4. Auto-Exposure - RAW level");
    let start = std::time::Instant::now();
    let opts4 = RawOptions {
        exposure_compensation: None,
        auto_exposure: true,
        white_balance: soma_media::WhiteBalance::None,
        ..RawOptions::fast_preview()
    };
    let (rgb4, w4, h4) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts4)?;
    let process_time4 = start.elapsed();
    let avg4 = rgb4.iter().map(|&v| v as u32).sum::<u32>() / rgb4.len() as u32;
    save_jpeg(&rgb4, w4, h4, "test_output/raw_exp_auto.jpg")?;
    save_webp(&rgb4, w4, h4, "test_output/raw_exp_auto.webp")?;
    println!("   Avg brightness: {} | Change: {:+} ({:.1}% brighter)",
             avg4, avg4 as i32 - avg1 as i32, (avg4 as f32 / avg1 as f32 - 1.0) * 100.0);
    println!("   Processing: {}ms | Auto-calculated optimal exposure\n", process_time4.as_millis());
    
    // Test 5: Camera White Balance
    println!("5. Camera White Balance (as-shot)");
    let start = std::time::Instant::now();
    let opts5 = RawOptions {
        exposure_compensation: None,
        auto_exposure: false,
        white_balance: soma_media::WhiteBalance::Camera,
        ..RawOptions::fast_preview()
    };
    let (rgb5, w5, h5) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts5)?;
    let process_time5 = start.elapsed();
    let avg5 = rgb5.iter().map(|&v| v as u32).sum::<u32>() / rgb5.len() as u32;
    save_jpeg(&rgb5, w5, h5, "test_output/raw_wb_camera.jpg")?;
    save_webp(&rgb5, w5, h5, "test_output/raw_wb_camera.webp")?;
    println!("   Avg brightness: {} | Processing: {}ms\n", avg5, process_time5.as_millis());
    
    // Test 6: Auto White Balance
    println!("6. Auto White Balance");
    let start = std::time::Instant::now();
    let opts6 = RawOptions {
        exposure_compensation: None,
        auto_exposure: false,
        white_balance: soma_media::WhiteBalance::Auto,
        ..RawOptions::fast_preview()
    };
    let (rgb6, w6, h6) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts6)?;
    let process_time6 = start.elapsed();
    let avg6 = rgb6.iter().map(|&v| v as u32).sum::<u32>() / rgb6.len() as u32;
    save_jpeg(&rgb6, w6, h6, "test_output/raw_wb_auto.jpg")?;
    save_webp(&rgb6, w6, h6, "test_output/raw_wb_auto.webp")?;
    println!("   Avg brightness: {} | Processing: {}ms\n", avg6, process_time6.as_millis());
    
    // Test 7: Auto-Exposure + Camera WB (full processing)
    println!("7. Auto-Exposure + Camera WB (complete)");
    let start = std::time::Instant::now();
    let opts7 = RawOptions {
        exposure_compensation: None,
        auto_exposure: true,
        white_balance: soma_media::WhiteBalance::Camera,
        ..RawOptions::fast_preview()
    };
    let (rgb7, w7, h7) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts7)?;
    let process_time7 = start.elapsed();
    let avg7 = rgb7.iter().map(|&v| v as u32).sum::<u32>() / rgb7.len() as u32;
    save_jpeg(&rgb7, w7, h7, "test_output/raw_complete.jpg")?;
    save_webp(&rgb7, w7, h7, "test_output/raw_complete.webp")?;
    println!("   Avg brightness: {} | Processing: {}ms\n", avg7, process_time7.as_millis());
    
    // Test 8: Auto-Exposure + Auto WB (full auto)
    println!("8. Auto-Exposure + Auto WB (full auto)");
    let start = std::time::Instant::now();
    let opts8 = RawOptions {
        exposure_compensation: None,
        auto_exposure: true,
        white_balance: soma_media::WhiteBalance::Auto,
        ..RawOptions::fast_preview()
    };
    let (rgb8, w8, h8) = processor.process_raw_from_memory(&std::fs::read(file)?, &opts8)?;
    let process_time8 = start.elapsed();
    let avg8 = rgb8.iter().map(|&v| v as u32).sum::<u32>() / rgb8.len() as u32;
    save_jpeg(&rgb8, w8, h8, "test_output/raw_auto_complete.jpg")?;
    save_webp(&rgb8, w8, h8, "test_output/raw_auto_complete.webp")?;
    println!("   Avg brightness: {} | Processing: {}ms\n", avg8, process_time8.as_millis());
    
    let total_time = process_time1 + process_time2 + process_time3 + process_time4 + process_time5 + process_time6 + process_time7 + process_time8;
    
    println!("=== Results ===");
    println!("📁 Output files (JPEG @ 85% + WebP @ 92%):");
    println!("  • raw_exp_0ev.jpg/.webp        - Baseline (no WB)");
    println!("  • raw_exp_1ev.jpg/.webp        - +1 EV");
    println!("  • raw_exp_3ev.jpg/.webp        - +3 EV");
    println!("  • raw_exp_auto.jpg/.webp       - Auto-exposure");
    println!("  • raw_wb_camera.jpg/.webp      - Camera WB");
    println!("  • raw_wb_auto.jpg/.webp        - Auto WB");
    println!("  • raw_complete.jpg/.webp       - Auto-Exp + Camera WB ⭐");
    println!("  • raw_auto_complete.jpg/.webp  - Auto-Exp + Auto WB 🤖");
    
    println!("\n⏱️  Performance:");
    println!("  Baseline:          {}ms", process_time1.as_millis());
    println!("  +1 EV:             {}ms", process_time2.as_millis());
    println!("  +3 EV:             {}ms", process_time3.as_millis());
    println!("  Auto-exposure:     {}ms", process_time4.as_millis());
    println!("  Camera WB:         {}ms", process_time5.as_millis());
    println!("  Auto WB:           {}ms", process_time6.as_millis());
    println!("  Auto-Exp+CameraWB: {}ms ⭐", process_time7.as_millis());
    println!("  Auto-Exp+AutoWB:   {}ms 🤖", process_time8.as_millis());
    println!("  ─────────────────────");
    println!("  Total:             {}ms ({:.1} images/sec)", total_time.as_millis(), 8000.0 / total_time.as_millis() as f32);
    
    let brightness_gain_1ev = avg2 as f32 / avg1 as f32;
    let brightness_gain_3ev = avg3 as f32 / avg1 as f32;
    
    println!("\n🎯 Verification:");
    println!("  +1 EV gain: {:.2}x (expected: 2.0x)", brightness_gain_1ev);
    println!("  +3 EV gain: {:.2}x (expected: 8.0x)", brightness_gain_3ev);
    
    if brightness_gain_1ev > 1.5 && brightness_gain_1ev < 2.5 {
        println!("  ✅ RAW-level exposure IS working!");
    } else {
        println!("  ❌ RAW-level exposure NOT working properly");
    }
    
    Ok(())
}
