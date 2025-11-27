use crate::error::Result;
use std::path::Path;
use rsraw::{RawImage, BIT_DEPTH_8, BIT_DEPTH_16};
use rsraw_sys as sys;

/// RAW image processing using libraw via FFI
pub struct RawProcessor;

/// White balance mode for RAW processing
#[derive(Debug, Clone)]
pub enum WhiteBalance {
    /// Use camera's white balance
    Camera,
    /// Auto white balance (average whole image)
    Auto,
    /// No white balance adjustment (raw sensor values)
    None,
    /// Custom RGB multipliers [r, g, b, g2]
    Custom([f32; 4]),
}

/// Output color space
#[derive(Debug, Clone, Copy)]
pub enum ColorSpace {
    /// Raw color space
    Raw = 0,
    /// sRGB (D65)
    SRGB = 1,
    /// Adobe RGB (1998) (D65)
    AdobeRGB = 2,
    /// Wide-gamut RGB (D50)
    WideGamutRGB = 3,
    /// Kodak ProPhoto RGB (D50)
    ProPhotoRGB = 4,
    /// XYZ
    XYZ = 5,
}

/// RAW processing options
#[derive(Debug, Clone)]
pub struct RawOptions {
    /// White balance mode
    pub white_balance: WhiteBalance,
    
    /// Output bit depth (8 or 16)
    pub bit_depth: u8,
    
    /// Output color space
    pub color_space: ColorSpace,
    
    /// Brightness adjustment (0.25 to 8.0, default 1.0)
    pub brightness: f32,
    
    /// Auto-brightness (ignore brightness value, auto-adjust)
    pub auto_brightness: bool,
    
    /// Gamma curve (power, slope) - typical: (2.222, 4.5) for sRGB
    pub gamma: Option<(f32, f32)>,
    
    /// Highlight recovery mode
    /// 0 = clip (default), 1 = unclip, 2 = blend, 3+ = rebuild
    pub highlight_mode: u8,
    
    /// Chromatic aberration correction [red_scale, blue_scale]
    pub chromatic_aberration: Option<(f64, f64)>,
    
    /// Noise reduction threshold (0.0 = off, higher = more NR)
    pub noise_threshold: f32,
    
    /// Median filter passes for noise reduction (0-10, default 0)
    pub median_filter_passes: u8,
    
    /// FBDD noise reduction (0 = off, 1 = light, 2 = full)
    pub fbdd_noise_reduction: u8,
    
    /// Half-size color image (faster, 2x2 downsampling)
    pub half_size: bool,
    
    /// Four-color RGB interpolation (better quality, slower)
    pub four_color_rgb: bool,
    
    /// Demosaicing algorithm quality (0-12, default auto)
    /// 0 = linear, 3 = AHD, 11 = DHT, 12 = AAHD
    pub demosaic_algorithm: Option<u8>,
}

impl Default for RawOptions {
    fn default() -> Self {
        Self {
            white_balance: WhiteBalance::Camera,
            bit_depth: 8,
            color_space: ColorSpace::SRGB,
            brightness: 1.0,
            auto_brightness: false,  // Respect photographer's intent by default
            gamma: None, // Use libraw's default
            highlight_mode: 0,
            chromatic_aberration: None,
            noise_threshold: 0.0,
            median_filter_passes: 0,
            fbdd_noise_reduction: 0,
            half_size: true,  // Enable by default for speed
            four_color_rgb: false,
            demosaic_algorithm: Some(12),  // AAHD for best quality
        }
    }
}

impl RawOptions {
    /// Fast preview preset - optimized for speed
    /// Use for: Quick culling, thumbnail generation, preview rendering
    /// No color/brightness adjustments - shows raw sensor data
    pub fn fast_preview() -> Self {
        Self {
            white_balance: WhiteBalance::None,
            bit_depth: 8,
            color_space: ColorSpace::SRGB,
            brightness: 1.0,
            auto_brightness: false,
            gamma: None,
            highlight_mode: 0, // Clip (fastest)
            chromatic_aberration: None,
            noise_threshold: 0.0,
            median_filter_passes: 0,
            fbdd_noise_reduction: 0,
            half_size: true, // 2x2 downsampling for speed
            four_color_rgb: false,
            demosaic_algorithm: Some(12), // AAHD (best quality)
        }
    }

    /// Maximum quality preset - for final processing and archival
    /// Use for: Client deliverables, prints, archival, reprocessing
    /// Preserves all sensor data without adjustments but applies noise reduction
    pub fn maximum() -> Self {
        Self {
            white_balance: WhiteBalance::None, // Preserve sensor data
            bit_depth: 16,
            color_space: ColorSpace::ProPhotoRGB,
            brightness: 1.0,
            auto_brightness: false,
            gamma: None, // Linear for post-processing
            highlight_mode: 3, // Rebuild (best highlight recovery)
            chromatic_aberration: Some((1.0, 1.0)),
            noise_threshold: 100.0, // Light noise reduction
            median_filter_passes: 1,
            fbdd_noise_reduction: 2, // Full FBDD noise reduction
            half_size: true, // Use half-size like fast_preview
            four_color_rgb: true, // Best color accuracy
            demosaic_algorithm: Some(12), // AAHD (best quality)
        }
    }
    
    /// Recovery preset - for severely under/overexposed images
    /// Use for: Backlit subjects, silhouettes, blown highlights, deep shadows
    /// Aggressive brightness adjustment and highlight/shadow recovery
    pub fn recovery() -> Self {
        Self {
            white_balance: WhiteBalance::Auto,
            bit_depth: 8,
            color_space: ColorSpace::SRGB,
            brightness: 4.0, // Moderate boost
            auto_brightness: false,
            gamma: Some((1.2, 2.0)), // Very low power = aggressive shadow lift, low slope = gentle highlights
            highlight_mode: 2, // Blend mode - better for already-blown highlights than rebuild
            chromatic_aberration: None,
            noise_threshold: 300.0,
            median_filter_passes: 2,
            fbdd_noise_reduction: 2,
            half_size: false,
            four_color_rgb: true,
            demosaic_algorithm: Some(11),
        }
    }
}

impl RawProcessor {
    /// Create a new RAW processor (FFI-based, no external binary needed)
    pub fn new() -> Result<Self> {
        // No external dependency check needed - we link directly to libraw
        Ok(Self)
    }

    /// Check if file is a RAW format using tree_magic_mini for MIME detection
    pub fn is_raw_format(path: &Path) -> bool {
        let mime_type = tree_magic_mini::from_filepath(path);
        
        // Check for known RAW MIME types
        matches!(
            mime_type,
            Some("image/x-canon-cr2") |
            Some("image/x-canon-cr3") |
            Some("image/x-canon-crw") |
            Some("image/x-nikon-nef") |
            Some("image/x-sony-arw") |
            Some("image/x-sony-srf") |
            Some("image/x-sony-sr2") |
            Some("image/x-pentax-pef") |
            Some("image/x-samsung-srw") |
            Some("image/x-olympus-orf") |
            Some("image/x-panasonic-raw") |
            Some("image/x-fuji-raf") |
            Some("image/x-sigma-x3f") |
            Some("image/x-adobe-dng") |
            Some("image/x-dcraw") |
            Some("image/tiff")  // Many RAW formats report as TIFF
        )
    }
    
    /// Check if data is a RAW format by trying to open with libraw
    pub fn is_raw_data(&self, data: &[u8]) -> bool {
        RawImage::open(data).is_ok()
    }
    
    /// Process RAW data from memory and return (RGB data, width, height)
    pub fn process_raw_from_memory(
        &self,
        file_data: &[u8],
        options: &RawOptions,
    ) -> Result<(Vec<u8>, u32, u32)> {
        // Open RAW file from buffer
        let mut raw = RawImage::open(file_data)
            .map_err(|e| crate::error::MediaError::ProcessingError(
                format!("Failed to open RAW file: {:?}", e)
            ))?;
        
        // Configure processing parameters (same as process_raw)
        let raw_ptr: *mut sys::libraw_data_t = unsafe {
            std::mem::transmute_copy(&raw)
        };
        
        unsafe {
            let params = &mut (*raw_ptr).params;
            
            // White balance
            match &options.white_balance {
                WhiteBalance::Camera => params.use_camera_wb = 1,
                WhiteBalance::Auto => params.use_auto_wb = 1,
                WhiteBalance::None => {
                    params.use_camera_wb = 0;
                    params.use_auto_wb = 0;
                }
                WhiteBalance::Custom(mults) => {
                    params.use_camera_wb = 0;
                    params.use_auto_wb = 0;
                    params.user_mul = *mults;
                }
            }
            
            // Auto brightness
            params.no_auto_bright = if options.auto_brightness { 0 } else { 1 };
            if !options.auto_brightness {
                params.bright = options.brightness;
            }
            
            // Gamma curve
            if let Some((power, slope)) = options.gamma {
                params.gamm[0] = 1.0 / power as f64;
                params.gamm[1] = slope as f64;
            }
            
            // Highlight recovery
            params.highlight = options.highlight_mode as i32;
            
            // Noise reduction
            params.threshold = options.noise_threshold;
            params.med_passes = options.median_filter_passes as i32;
            params.fbdd_noiserd = options.fbdd_noise_reduction as i32;
            
            // Half-size
            params.half_size = if options.half_size { 1 } else { 0 };
            
            // Demosaic algorithm
            if let Some(algo) = options.demosaic_algorithm {
                params.user_qual = algo as i32;
            }
        }
        
        raw.unpack()
            .map_err(|e| crate::error::MediaError::ProcessingError(
                format!("Failed to unpack RAW: {:?}", e)
            ))?;
        
        // Call dcraw_process to actually demosaic the image
        unsafe {
            let ret = sys::libraw_dcraw_process(raw_ptr);
            if ret != 0 {
                return Err(crate::error::MediaError::ProcessingError(
                    format!("libraw_dcraw_process failed with code {}", ret)
                ));
            }
        }
        
        // Write to temp PPM file (dcraw_make_mem_image has wrong format)
        let temp_ppm = std::env::temp_dir().join(format!("libraw_mem_{}.ppm", std::process::id()));
        let c_path = std::ffi::CString::new(temp_ppm.to_str().unwrap()).unwrap();
        
        unsafe {
            let write_ret = sys::libraw_dcraw_ppm_tiff_writer(raw_ptr, c_path.as_ptr());
            if write_ret != 0 {
                return Err(crate::error::MediaError::ProcessingError(
                    format!("libraw_dcraw_ppm_tiff_writer failed with code {}", write_ret)
                ));
            }
        }
        
        // Read the PPM file
        let ppm_data = std::fs::read(&temp_ppm)
            .map_err(|e| crate::error::MediaError::Io(e))?;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_ppm);
        
        // Parse PPM header to get actual dimensions and find data start
        // Format: P6\nWIDTH HEIGHT\n255\n
        let header_str = String::from_utf8_lossy(&ppm_data[..100]);
        let lines: Vec<&str> = header_str.lines().collect();
        
        if lines.len() < 3 || lines[0] != "P6" {
            return Err(crate::error::MediaError::ProcessingError(
                "Invalid PPM format".to_string()
            ));
        }
        
        // Parse width and height from second line
        let dims: Vec<&str> = lines[1].split_whitespace().collect();
        if dims.len() != 2 {
            return Err(crate::error::MediaError::ProcessingError(
                "Invalid PPM dimensions".to_string()
            ));
        }
        
        let width: u32 = dims[0].parse().map_err(|_| crate::error::MediaError::ProcessingError("Invalid width".to_string()))?;
        let height: u32 = dims[1].parse().map_err(|_| crate::error::MediaError::ProcessingError("Invalid height".to_string()))?;
        
        // Calculate header end
        let mut header_end = 0;
        let mut newlines = 0;
        for (i, &b) in ppm_data.iter().enumerate() {
            if b == b'\n' {
                newlines += 1;
                if newlines == 3 {
                    header_end = i + 1;
                    break;
                }
            }
        }
        
        let data = ppm_data[header_end..].to_vec();
        
        Ok((data, width, height))
    }

    /// Process a RAW file and return RGB image data
    pub fn process_raw(
        &self,
        input_path: &Path,
        options: &RawOptions,
    ) -> Result<Vec<u8>> {
        // Read file into memory
        let file_data = std::fs::read(input_path)
            .map_err(|e| crate::error::MediaError::Io(e))?;
        
        // Open RAW file from buffer
        let mut raw = RawImage::open(&file_data)
            .map_err(|e| crate::error::MediaError::ProcessingError(
                format!("Failed to open RAW file: {:?}", e)
            ))?;
        
        // Configure processing parameters via unsafe access to libraw_data_t
        // We need to get the raw pointer - rsraw doesn't expose this, so we use transmute
        let raw_ptr: *mut sys::libraw_data_t = unsafe {
            std::mem::transmute_copy(&raw)
        };
        
        unsafe {
            let params = &mut (*raw_ptr).params;
            
            // White balance
            match &options.white_balance {
                WhiteBalance::Camera => params.use_camera_wb = 1,
                WhiteBalance::Auto => params.use_auto_wb = 1,
                WhiteBalance::None => {
                    params.use_camera_wb = 0;
                    params.use_auto_wb = 0;
                }
                WhiteBalance::Custom(mults) => {
                    params.use_camera_wb = 0;
                    params.use_auto_wb = 0;
                    params.user_mul = *mults;
                }
            }
            
            // Auto brightness
            params.no_auto_bright = if options.auto_brightness { 0 } else { 1 };
            if !options.auto_brightness {
                params.bright = options.brightness;
            }
            
            // Gamma curve
            if let Some((power, slope)) = options.gamma {
                params.gamm[0] = 1.0 / power as f64;
                params.gamm[1] = slope as f64;
            }
            
            // Highlight recovery
            params.highlight = options.highlight_mode as i32;
            
            // Noise reduction
            params.threshold = options.noise_threshold;
            params.med_passes = options.median_filter_passes as i32;
            params.fbdd_noiserd = options.fbdd_noise_reduction as i32;
            
            // Half-size
            params.half_size = if options.half_size { 1 } else { 0 };
            
            // Demosaic algorithm
            if let Some(algo) = options.demosaic_algorithm {
                params.user_qual = algo as i32;
            }
        }
        
        // Unpack RAW data
        raw.unpack()
            .map_err(|e| crate::error::MediaError::ProcessingError(
                format!("Failed to unpack RAW: {:?}", e)
            ))?;
        
        // Call dcraw_process to actually demosaic the image
        unsafe {
            let ret = sys::libraw_dcraw_process(raw_ptr);
            if ret != 0 {
                return Err(crate::error::MediaError::ProcessingError(
                    format!("libraw_dcraw_process failed with code {}", ret)
                ));
            }
        }
        
        // Write to temp PPM file (dcraw_make_mem_image has wrong format)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_ppm = std::env::temp_dir().join(format!("libraw_file_{}_{}.ppm", std::process::id(), timestamp));
        let c_path = std::ffi::CString::new(temp_ppm.to_str().unwrap()).unwrap();
        
        unsafe {
            let write_ret = sys::libraw_dcraw_ppm_tiff_writer(raw_ptr, c_path.as_ptr());
            if write_ret != 0 {
                return Err(crate::error::MediaError::ProcessingError(
                    format!("libraw_dcraw_ppm_tiff_writer failed with code {}", write_ret)
                ));
            }
        }
        
        // Read the PPM file
        let ppm_data = std::fs::read(&temp_ppm)
            .map_err(|e| crate::error::MediaError::Io(e))?;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_ppm);
        
        // Skip PPM header (P6\nWIDTH HEIGHT\n255\n)
        let mut header_end = 0;
        let mut newlines = 0;
        for (i, &b) in ppm_data.iter().enumerate() {
            if b == b'\n' {
                newlines += 1;
                if newlines == 3 {
                    header_end = i + 1;
                    break;
                }
            }
        }
        
        let data = ppm_data[header_end..].to_vec();
        
        Ok(data)
    }

    /// Get image dimensions from RAW file (accounting for half_size mode)
    pub fn get_dimensions(&self, path: &Path, options: &RawOptions) -> Result<(u32, u32)> {
        let file_data = std::fs::read(path)
            .map_err(|e| crate::error::MediaError::Io(e))?;
            
        let raw = RawImage::open(&file_data)
            .map_err(|e| crate::error::MediaError::ProcessingError(
                format!("Failed to open RAW file: {:?}", e)
            ))?;
        
        let mut width = raw.width();
        let mut height = raw.height();
        
        // Adjust for half_size mode
        if options.half_size {
            width /= 2;
            height /= 2;
        }
        
        Ok((width, height))
    }
}
