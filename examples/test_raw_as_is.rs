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
            
            println!("Checking if data is already interleaved...");
            println!("Width: {}, Height: {}", width, height);
            
            // Save data AS-IS without any conversion
            let ppm = format!("P6\n{} {}\n255\n", width, height);
            let mut output = ppm.into_bytes();
            output.extend_from_slice(&data);
            std::fs::write("sample/output/test_raw_as_is.ppm", output).unwrap();
            println!("Saved raw data as-is to: sample/output/test_raw_as_is.ppm");
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
