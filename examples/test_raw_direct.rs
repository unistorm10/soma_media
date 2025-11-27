use soma_media::{RawProcessor, RawOptions, WhiteBalance};

fn main() {
    let processor = RawProcessor::new().unwrap();
    
    let mut options = RawOptions::default();
    options.half_size = true;  // Only set this, nothing else
    
    let file = "sample/03240053.SRW";
    
    println!("Processing RAW without resize...");
    
    match processor.process_raw(file.as_ref(), &options) {
        Ok(data) => {
            let (width, height) = processor.get_dimensions(file.as_ref(), &options).unwrap();
            println!("Dimensions: {}x{}", width, height);
            println!("Data length: {} bytes", data.len());
            println!("Expected (WxHx3): {} bytes", width * height * 3);
            println!("Ratio: {:.2}", data.len() as f64 / (width * height * 3) as f64);
            
            // Save directly as PPM (no encoding, just raw RGB)
            let ppm = format!("P6\n{} {}\n255\n", width, height);
            let mut output = ppm.into_bytes();
            output.extend_from_slice(&data);
            std::fs::write("sample/output/test_raw_direct.ppm", output).unwrap();
            println!("Saved to sample/output/test_raw_direct.ppm");
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
