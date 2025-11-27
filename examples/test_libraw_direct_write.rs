use rsraw::RawImage;
use rsraw_sys as sys;
use std::time::Instant;

fn main() {
    let files = vec![
        "sample/03240053.SRW",
        "sample/202309101781.SRW",
        "sample/202310042332.SRW",
        "sample/201811174456.dng",
    ];
    
    for file in files {
        let start = Instant::now();
        println!("\nProcessing: {}", file);
        
        let data = std::fs::read(file).unwrap();
        let mut raw = RawImage::open(&data).unwrap();
        
        let raw_ptr: *mut sys::libraw_data_t = unsafe {
            std::mem::transmute_copy(&raw)
        };
        
        unsafe {
            let params = &mut (*raw_ptr).params;
            params.half_size = 1;
        }
        
        raw.unpack().unwrap();
        
        // Use libraw to write PPM directly
        unsafe {
            let ret = sys::libraw_dcraw_process(raw_ptr);
            println!("  dcraw_process returned: {}", ret);
            
            let output = format!("sample/output/libraw_direct_{}.ppm", 
                std::path::Path::new(file).file_stem().unwrap().to_string_lossy());
            let c_path = std::ffi::CString::new(output.as_str()).unwrap();
            
            let write_ret = sys::libraw_dcraw_ppm_tiff_writer(raw_ptr, c_path.as_ptr());
            println!("  ppm_writer returned: {}", write_ret);
            println!("  Saved to: {}", output);
        }
        
        let elapsed = start.elapsed();
        println!("  Time: {:.3}s", elapsed.as_secs_f64());
    }
}
