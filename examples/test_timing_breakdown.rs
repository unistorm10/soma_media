use soma_media::{RawProcessor, RawOptions};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_file = Path::new("/run/user/1000/gvfs/smb-share:server=main.local,share=test_data/sample_ext/07270143.SRW");
    
    if !raw_file.exists() {
        eprintln!("RAW file not found: {:?}", raw_file);
        return Ok(());
    }

    let processor = RawProcessor::new()?;
    let options = RawOptions::fast_preview();
    
    println!("Timing breakdown:");
    println!();
    
    // Step 1: RAW processing
    let start = Instant::now();
    let rgb_data = processor.process_raw(raw_file, &options)?;
    let raw_time = start.elapsed();
    println!("1. RAW processing: {:.3}s ({} bytes)", raw_time.as_secs_f64(), rgb_data.len());
    
    // Step 2: Get dimensions
    let (width, height) = processor.get_dimensions(raw_file, &options)?;
    println!("2. Dimensions: {}x{}", width, height);
    
    // Step 3: Fast resize using fast_image_resize
    let start = Instant::now();
    use fast_image_resize as fr;
    use fr::images::Image as FrImage;
    
    let src_image = FrImage::from_vec_u8(
        width,
        height,
        rgb_data,
        fr::PixelType::U8x3,
    ).unwrap();
    
    let mut dst_image = FrImage::new(800, 600, src_image.pixel_type());
    let mut resizer = fr::Resizer::new();
    resizer.resize(&src_image, &mut dst_image, None).unwrap();
    
    let rgb_bytes = dst_image.buffer().to_vec();
    let resize_time = start.elapsed();
    println!("3. Fast resize ({}x{} → 800x600): {:.3}s ({} bytes)", 
             width, height, resize_time.as_secs_f64(), rgb_bytes.len());
    
    // Step 4: WEBP encoding
    let start = Instant::now();
    let encoder = webp::Encoder::from_rgb(&rgb_bytes, 800, 600);
    let webp_data = encoder.encode(85.0);
    let encode_time = start.elapsed();
    println!("4. WEBP encoding: {:.3}s ({} bytes)", encode_time.as_secs_f64(), webp_data.len());
    
    // Step 5: Write to disk
    let start = Instant::now();
    std::fs::write("/tmp/test_timing.webp", &*webp_data)?;
    let write_time = start.elapsed();
    println!("5. Write to disk: {:.3}s", write_time.as_secs_f64());
    
    println!();
    let total = raw_time + resize_time + encode_time + write_time;
    println!("Total: {:.3}s", total.as_secs_f64());

    Ok(())
}
