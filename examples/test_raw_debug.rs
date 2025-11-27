use soma_media::raw::{RawProcessor, RawOptions};

fn main() {
    let processor = RawProcessor::new().unwrap();
    
    let file = "sample/03240053.SRW";
    
    let mut options = RawOptions::default();
    options.half_size = true;
    
    println!("Processing RAW file: {}", file);
    
    // Get dimensions
    match processor.get_dimensions(file.as_ref(), &options) {
        Ok((w, h)) => {
            println!("RAW dimensions: {}x{}", w, h);
        }
        Err(e) => {
            println!("Failed to get dimensions: {:?}", e);
            return;
        }
    }
    
    // Process RAW
    match processor.process_raw(file.as_ref(), &options) {
        Ok(data) => {
            println!("RAW processing successful");
            println!("Data length: {} bytes", data.len());
            println!("First 32 bytes: {:?}", &data[..32.min(data.len())]);
            
            // Check if data looks valid (not all zeros or all same value)
            let first = data[0];
            let all_same = data.iter().all(|&b| b == first);
            if all_same {
                println!("WARNING: All bytes are the same value ({})", first);
            } else {
                println!("Data appears valid (varying values)");
            }
        }
        Err(e) => {
            println!("Failed to process RAW: {:?}", e);
        }
    }
}
