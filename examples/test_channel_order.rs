use soma_media::{RawProcessor, RawOptions};

fn main() {
    let processor = RawProcessor::new().unwrap();
    
    let mut options = RawOptions::default();
    options.half_size = true;
    
    let file = "sample/03240053.SRW";
    
    match processor.process_raw(file.as_ref(), &options) {
        Ok(data) => {
            let (width, height) = processor.get_dimensions(file.as_ref(), &options).unwrap();
            let pixel_count = (width * height) as usize;
            
            println!("Width: {}, Height: {}, Pixels: {}", width, height, pixel_count);
            println!("Data length: {}", data.len());
            println!("Expected for RGB: {}", pixel_count * 3);
            println!("Expected for RGBA: {}", pixel_count * 4);
            
            // Check bytes per pixel
            let bpp = data.len() / pixel_count;
            println!("Actual bytes per pixel: {}", bpp);
            
            if bpp == 3 {
                // Try different channel orderings
                let mut rgb_interleaved = Vec::with_capacity(pixel_count * 3);
                let mut bgr_interleaved = Vec::with_capacity(pixel_count * 3);
                
                for i in 0..pixel_count {
                    // RGB order
                    rgb_interleaved.push(data[i]);
                    rgb_interleaved.push(data[pixel_count + i]);
                    rgb_interleaved.push(data[pixel_count * 2 + i]);
                    
                    // BGR order
                    bgr_interleaved.push(data[pixel_count * 2 + i]);
                    bgr_interleaved.push(data[pixel_count + i]);
                    bgr_interleaved.push(data[i]);
                }
                
                // Save both versions as PPM
                let ppm_rgb = format!("P6\n{} {}\n255\n", width, height);
                let mut output_rgb = ppm_rgb.clone().into_bytes();
                output_rgb.extend_from_slice(&rgb_interleaved);
                std::fs::write("sample/output/test_rgb_order.ppm", output_rgb).unwrap();
                
                let mut output_bgr = ppm_rgb.into_bytes();
                output_bgr.extend_from_slice(&bgr_interleaved);
                std::fs::write("sample/output/test_bgr_order.ppm", output_bgr).unwrap();
                
                println!("Saved RGB order to: sample/output/test_rgb_order.ppm");
                println!("Saved BGR order to: sample/output/test_bgr_order.ppm");
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
