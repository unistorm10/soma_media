use soma_media::{RawProcessor, RawOptions};

fn main() {
    let processor = RawProcessor::new().unwrap();
    
    let mut options = RawOptions::default();
    options.half_size = true;
    
    let file = "sample/03240053.SRW";
    
    println!("Testing libraw output...");
    
    match processor.process_raw(file.as_ref(), &options) {
        Ok(data) => {
            let (width, height) = processor.get_dimensions(file.as_ref(), &options).unwrap();
            
            println!("Data length: {}", data.len());
            println!("Width: {}, Height: {}", width, height);
            println!("Pixels: {}", width * height);
            println!("Bytes per pixel: {}", data.len() as f64 / (width * height) as f64);
            
            // Check first 50 values
            println!("\nFirst 50 bytes:");
            for (i, &b) in data.iter().take(50).enumerate() {
                if i % 10 == 0 {
                    print!("\n");
                }
                print!("{:3} ", b);
            }
            println!("\n");
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
