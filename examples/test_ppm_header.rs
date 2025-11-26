use std::fs;

fn main() {
    // Read one of the working direct PPM files
    let ppm_path = "sample/output/libraw_direct_201811174456.ppm";
    
    if let Ok(data) = fs::read(ppm_path) {
        println!("File size: {} bytes", data.len());
        println!("First 200 bytes:");
        for (i, &b) in data.iter().take(200).enumerate() {
            if b == b'\n' {
                print!("\\n");
            } else if b.is_ascii_graphic() || b == b' ' {
                print!("{}", b as char);
            } else {
                print!("[{}]", b);
            }
            if i % 50 == 49 {
                println!();
            }
        }
        println!("\n\nFinding header end...");
        
        let mut newlines = 0;
        let mut header_end = 0;
        for (i, &b) in data.iter().enumerate() {
            if i < 100 {
                if b == b'\n' {
                    print!("\\n({})", newlines);
                    newlines += 1;
                    if newlines == 3 {
                        header_end = i + 1;
                        println!(" <- HEADER END at byte {}", header_end);
                        break;
                    }
                } else if b.is_ascii_graphic() || b == b' ' {
                    print!("{}", b as char);
                } else {
                    print!("[{}]", b);
                }
            }
        }
        
        println!("\nRGB data starts at byte: {}", header_end);
        println!("RGB data length: {}", data.len() - header_end);
        println!("First 30 RGB bytes: {:?}", &data[header_end..header_end.min(data.len()).min(header_end + 30)]);
    }
}
