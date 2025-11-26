use soma_media::{RawProcessor, RawOptions};

fn main() {
    let processor = RawProcessor::new().unwrap();
    
    let mut options = RawOptions::default();
    options.half_size = true;
    
    let file = "sample/03240053.SRW";
    
    match processor.process_raw(file.as_ref(), &options) {
        Ok(data) => {
            let (width, height) = processor.get_dimensions(file.as_ref(), &options).unwrap();
            
            // Check if data might be planar instead of interleaved
            println!("Checking data layout...");
            println!("Total pixels: {}", width * height);
            println!("Data length: {}", data.len());
            println!("Bytes per pixel: {}", data.len() as f64 / (width * height) as f64);
            
            // Sample first 100 bytes
            println!("\nFirst 100 bytes:");
            for (i, &byte) in data.iter().take(100).enumerate() {
                if i % 20 == 0 {
                    println!();
                }
                print!("{:3} ", byte);
            }
            println!("\n");
            
            // Check if first 1/3 is similar (would indicate planar R channel)
            let third = data.len() / 3;
            println!("First third avg: {}", data[..third].iter().map(|&b| b as u32).sum::<u32>() / third as u32);
            println!("Second third avg: {}", data[third..third*2].iter().map(|&b| b as u32).sum::<u32>() / third as u32);
            println!("Last third avg: {}", data[third*2..].iter().map(|&b| b as u32).sum::<u32>() / (data.len() - third*2) as u32);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
