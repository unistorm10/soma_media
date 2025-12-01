//! Universal Media Metadata Extraction
//!
//! Provides comprehensive metadata extraction for all media types:
//! - Images (JPEG, PNG, TIFF, WebP, HEIC, AVIF)
//! - RAW files (CR2, CR3, NEF, ARW, DNG, RAF, ORF, RW2, etc.)
//! - Video (MP4, MOV, AVI, MKV, WebM)
//! - Audio (MP3, FLAC, WAV, AAC, OGG)
//!
//! ## Backend Priority
//!
//! 1. **ExifTool** (primary) - Most comprehensive, supports 400+ formats
//! 2. **kamadak-exif** (Rust native) - Pure Rust EXIF parsing for images
//! 3. **LibRaw** (RAW fallback) - Specialized RAW metadata via rsraw
//! 4. **FFprobe** (video/audio) - FFmpeg's metadata tool
//! 5. **file magic** (MIME only) - Basic type detection
//!
//! ## Example
//!
//! ```rust,ignore
//! use soma_media::metadata::{extract_metadata, MediaMetadata};
//!
//! let meta = extract_metadata(Path::new("photo.jpg"))?;
//! println!("Type: {}", meta.mime_type);
//! println!("Camera: {} {}", meta.make.unwrap_or_default(), meta.model.unwrap_or_default());
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use tracing::debug;

use crate::error::MediaError;

pub type Result<T> = std::result::Result<T, MediaError>;

// ============================================================================
// Core Metadata Types
// ============================================================================

/// Universal media metadata extracted from any file type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaMetadata {
    /// Source file path
    pub source_file: String,
    
    /// Detected MIME type (e.g., "image/jpeg", "video/mp4")
    pub mime_type: String,
    
    /// File extension
    pub file_type: String,
    
    /// File size in bytes
    pub file_size: u64,
    
    /// Creation/capture timestamp (ISO 8601)
    pub date_created: Option<String>,
    
    /// Last modified timestamp
    pub date_modified: Option<String>,
    
    // ---- Image/Camera Metadata ----
    
    /// Camera/device manufacturer
    pub make: Option<String>,
    
    /// Camera/device model
    pub model: Option<String>,
    
    /// Software used to create/edit
    pub software: Option<String>,
    
    /// Lens information
    pub lens: Option<LensInfo>,
    
    /// Image dimensions
    pub dimensions: Option<Dimensions>,
    
    /// Exposure settings
    pub exposure: Option<ExposureInfo>,
    
    /// GPS location
    pub gps: Option<GpsCoordinates>,
    
    /// Color/white balance info
    pub color: Option<ColorInfo>,
    
    /// Image orientation (1-8, EXIF standard)
    pub orientation: Option<u8>,
    
    // ---- Video/Audio Metadata ----
    
    /// Duration in seconds
    pub duration: Option<f64>,
    
    /// Video/audio codec
    pub codec: Option<String>,
    
    /// Bitrate in bits/second
    pub bitrate: Option<u64>,
    
    /// Frame rate (video)
    pub frame_rate: Option<f64>,
    
    /// Sample rate in Hz (audio)
    pub sample_rate: Option<u32>,
    
    /// Number of audio channels
    pub channels: Option<u8>,
    
    // ---- Document Metadata ----
    
    /// Title
    pub title: Option<String>,
    
    /// Author/artist
    pub artist: Option<String>,
    
    /// Description/caption
    pub description: Option<String>,
    
    /// Copyright notice
    pub copyright: Option<String>,
    
    /// Keywords/tags
    pub keywords: Vec<String>,
    
    // ---- Raw Extended Data ----
    
    /// Backend used to extract metadata
    pub backend: MetadataBackend,
    
    /// All raw key-value pairs from the backend
    pub raw_tags: HashMap<String, String>,
}

/// Image/video dimensions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
    pub bit_depth: Option<u8>,
}

/// Lens information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LensInfo {
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub focal_length: Option<f32>,
    pub focal_length_35mm: Option<u16>,
    pub min_focal: Option<f32>,
    pub max_focal: Option<f32>,
    pub max_aperture: Option<f32>,
}

/// Exposure/shooting settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExposureInfo {
    pub iso: Option<f32>,
    pub aperture: Option<f32>,
    pub shutter_speed: Option<f64>,
    pub exposure_compensation: Option<f32>,
    pub exposure_mode: Option<String>,
    pub metering_mode: Option<String>,
    pub flash: Option<String>,
    pub white_balance: Option<String>,
}

/// GPS coordinates
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpsCoordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
}

/// Color/white balance information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColorInfo {
    pub color_space: Option<String>,
    pub color_profile: Option<String>,
    pub white_balance: Option<String>,
    pub color_temperature: Option<u32>,
}

/// Metadata extraction backend
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum MetadataBackend {
    /// ExifTool (most comprehensive)
    ExifTool,
    /// kamadak-exif (pure Rust)
    KamadakExif,
    /// LibRaw (RAW files)
    LibRaw,
    /// FFprobe (video/audio)
    FFprobe,
    /// Basic file detection only
    #[default]
    FileMagic,
    /// No metadata available
    None,
}

// ============================================================================
// Sensor Info (for RAW files specifically)
// ============================================================================

/// Image sensor information (RAW files only)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SensorInfo {
    /// Sensor width in pixels (raw)
    pub raw_width: u32,
    /// Sensor height in pixels (raw)
    pub raw_height: u32,
    /// Output width (after crop)
    pub width: u32,
    /// Output height (after crop)
    pub height: u32,
    /// Top margin (crop)
    pub top_margin: u32,
    /// Left margin (crop)
    pub left_margin: u32,
    /// Bits per pixel
    pub bits_per_pixel: u8,
    /// Number of colors (3 for RGB, 4 for RGBG)
    pub colors: u8,
    /// Bayer pattern (RGGB, GRBG, etc.)
    pub bayer_pattern: Option<String>,
    /// Black level per channel
    pub black_level: Option<[u16; 4]>,
    /// White level (saturation)
    pub white_level: Option<u16>,
}

/// Shooting/capture information (RAW files only)  
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShootingInfo {
    /// Exposure mode (Auto, Manual, Aperture Priority, etc.)
    pub exposure_mode: Option<String>,
    /// Metering mode (Matrix, Spot, Center-weighted, etc.)
    pub metering_mode: Option<String>,
    /// Focus mode (AF-S, AF-C, Manual, etc.)
    pub focus_mode: Option<String>,
    /// Flash status
    pub flash: Option<String>,
    /// Image stabilization status
    pub image_stabilization: Option<String>,
    /// Drive mode (Single, Continuous, etc.)
    pub drive_mode: Option<String>,
    /// Shot order in sequence
    pub shot_order: Option<u32>,
}

// ============================================================================
// Metadata Extraction - Main API
// ============================================================================

/// Extract metadata from any media file
///
/// Automatically selects the best backend based on file type and availability:
/// 1. ExifTool (if installed) - handles everything
/// 2. Specialized fallbacks for each media type
///
/// # Example
/// ```rust,ignore
/// let meta = extract_metadata(Path::new("photo.jpg"))?;
/// println!("Camera: {:?}", meta.make);
/// ```
pub fn extract_metadata(path: &Path) -> Result<MediaMetadata> {
    // Try ExifTool first (most comprehensive)
    if let Ok(meta) = extract_with_exiftool(path) {
        return Ok(meta);
    }
    
    debug!("ExifTool not available, trying fallbacks");
    
    // Detect file type for fallback selection
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    
    // Try specialized fallbacks based on file type
    match extension.as_str() {
        // RAW files -> LibRaw
        "cr2" | "cr3" | "nef" | "arw" | "dng" | "raf" | "orf" | "rw2" | "pef" | "srw" => {
            if let Ok(meta) = extract_with_libraw(path) {
                return Ok(meta);
            }
        }
        // Standard images -> kamadak-exif
        "jpg" | "jpeg" | "tiff" | "tif" | "heic" | "heif" => {
            if let Ok(meta) = extract_with_kamadak_exif(path) {
                return Ok(meta);
            }
        }
        // Video/audio -> FFprobe
        "mp4" | "mov" | "avi" | "mkv" | "webm" | "mp3" | "flac" | "wav" | "aac" | "ogg" | "m4a" => {
            if let Ok(meta) = extract_with_ffprobe(path) {
                return Ok(meta);
            }
        }
        _ => {}
    }
    
    // Last resort: basic file info
    extract_basic_info(path)
}

/// Check if ExifTool is available on the system
pub fn exiftool_available() -> bool {
    Command::new("exiftool")
        .arg("-ver")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if FFprobe is available on the system
pub fn ffprobe_available() -> bool {
    Command::new("ffprobe")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ============================================================================
// ExifTool Backend (Primary)
// ============================================================================

/// Extract metadata using ExifTool (most comprehensive)
fn extract_with_exiftool(path: &Path) -> Result<MediaMetadata> {
    let output = Command::new("exiftool")
        .arg("-j")           // JSON output
        .arg("-G")           // Group names
        .arg("-n")           // Numeric values (not formatted)
        .arg("-a")           // Allow duplicates
        .arg("-u")           // Unknown tags
        .arg(path)
        .output()
        .map_err(|e| MediaError::ProcessingError(format!("ExifTool failed: {}", e)))?;
    
    if !output.status.success() {
        return Err(MediaError::ProcessingError(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }
    
    let json_str = String::from_utf8_lossy(&output.stdout);
    let parsed: Vec<HashMap<String, serde_json::Value>> = serde_json::from_str(&json_str)
        .map_err(|e| MediaError::ProcessingError(format!("JSON parse error: {}", e)))?;
    
    let tags = parsed.into_iter().next()
        .ok_or_else(|| MediaError::ProcessingError("No metadata found".to_string()))?;
    
    Ok(parse_exiftool_output(path, tags))
}

/// Parse ExifTool JSON output into MediaMetadata
fn parse_exiftool_output(path: &Path, tags: HashMap<String, serde_json::Value>) -> MediaMetadata {
    let get_str = |key: &str| -> Option<String> {
        tags.get(key).and_then(|v| {
            if v.is_string() {
                v.as_str().map(|s| s.to_string())
            } else {
                Some(v.to_string().trim_matches('"').to_string())
            }
        })
    };
    
    let get_f64 = |key: &str| -> Option<f64> {
        tags.get(key).and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    };
    
    let get_u32 = |key: &str| -> Option<u32> {
        tags.get(key).and_then(|v| v.as_u64().map(|n| n as u32).or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    };
    
    // Parse dimensions
    let dimensions = match (get_u32("EXIF:ImageWidth").or(get_u32("File:ImageWidth")),
                           get_u32("EXIF:ImageHeight").or(get_u32("File:ImageHeight"))) {
        (Some(w), Some(h)) => Some(Dimensions {
            width: w,
            height: h,
            bit_depth: get_u32("EXIF:BitsPerSample").map(|b| b as u8),
        }),
        _ => None,
    };
    
    // Parse exposure info
    let exposure = ExposureInfo {
        iso: get_f64("EXIF:ISO").map(|v| v as f32),
        aperture: get_f64("EXIF:FNumber").or(get_f64("EXIF:ApertureValue")).map(|v| v as f32),
        shutter_speed: parse_shutter_speed(get_str("EXIF:ExposureTime").as_deref()),
        exposure_compensation: get_f64("EXIF:ExposureCompensation").map(|v| v as f32),
        exposure_mode: get_str("EXIF:ExposureMode"),
        metering_mode: get_str("EXIF:MeteringMode"),
        flash: get_str("EXIF:Flash"),
        white_balance: get_str("EXIF:WhiteBalance"),
    };
    
    // Parse lens info
    let lens = LensInfo {
        make: get_str("EXIF:LensMake"),
        model: get_str("EXIF:LensModel").or(get_str("MakerNotes:LensType")),
        serial: get_str("EXIF:LensSerialNumber"),
        focal_length: get_f64("EXIF:FocalLength").map(|v| v as f32),
        focal_length_35mm: get_u32("EXIF:FocalLengthIn35mmFormat").map(|v| v as u16),
        min_focal: None,
        max_focal: None,
        max_aperture: get_f64("EXIF:MaxApertureValue").map(|v| v as f32),
    };
    
    // Parse GPS
    let gps = match (get_f64("Composite:GPSLatitude"), get_f64("Composite:GPSLongitude")) {
        (Some(lat), Some(lon)) => Some(GpsCoordinates {
            latitude: lat,
            longitude: lon,
            altitude: get_f64("Composite:GPSAltitude"),
        }),
        _ => None,
    };
    
    // Parse color info
    let color = ColorInfo {
        color_space: get_str("EXIF:ColorSpace").or(get_str("ICC_Profile:ColorSpaceData")),
        color_profile: get_str("ICC_Profile:ProfileDescription"),
        white_balance: get_str("EXIF:WhiteBalance"),
        color_temperature: get_u32("MakerNotes:ColorTemperature"),
    };
    
    // Build raw_tags from all exiftool output
    let raw_tags: HashMap<String, String> = tags.iter()
        .map(|(k, v)| (k.clone(), v.to_string().trim_matches('"').to_string()))
        .collect();
    
    MediaMetadata {
        source_file: path.to_string_lossy().to_string(),
        mime_type: get_str("File:MIMEType").unwrap_or_default(),
        file_type: get_str("File:FileType").unwrap_or_default(),
        file_size: get_f64("File:FileSize").map(|v| v as u64).unwrap_or(0),
        date_created: get_str("EXIF:DateTimeOriginal").or(get_str("QuickTime:CreateDate")),
        date_modified: get_str("File:FileModifyDate"),
        make: get_str("EXIF:Make"),
        model: get_str("EXIF:Model"),
        software: get_str("EXIF:Software"),
        lens: if lens.model.is_some() || lens.focal_length.is_some() { Some(lens) } else { None },
        dimensions,
        exposure: Some(exposure),
        gps,
        color: Some(color),
        orientation: get_u32("EXIF:Orientation").map(|v| v as u8),
        duration: get_f64("QuickTime:Duration").or(get_f64("Composite:Duration")),
        codec: get_str("QuickTime:CompressorID").or(get_str("File:FileType")),
        bitrate: get_f64("Composite:AvgBitrate").map(|v| v as u64),
        frame_rate: get_f64("QuickTime:VideoFrameRate"),
        sample_rate: get_u32("QuickTime:AudioSampleRate"),
        channels: get_u32("QuickTime:AudioChannels").map(|v| v as u8),
        title: get_str("XMP:Title").or(get_str("IPTC:ObjectName")),
        artist: get_str("EXIF:Artist").or(get_str("XMP:Creator")),
        description: get_str("EXIF:ImageDescription").or(get_str("XMP:Description")),
        copyright: get_str("EXIF:Copyright").or(get_str("XMP:Rights")),
        keywords: get_str("IPTC:Keywords")
            .or(get_str("XMP:Subject"))
            .map(|s| s.split(',').map(|k| k.trim().to_string()).collect())
            .unwrap_or_default(),
        backend: MetadataBackend::ExifTool,
        raw_tags,
    }
}

/// Parse shutter speed string to seconds
fn parse_shutter_speed(s: Option<&str>) -> Option<f64> {
    let s = s?;
    if s.contains('/') {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 2 {
            let num: f64 = parts[0].parse().ok()?;
            let den: f64 = parts[1].parse().ok()?;
            return Some(num / den);
        }
    }
    s.parse().ok()
}

// ============================================================================
// kamadak-exif Backend (Pure Rust, Images)
// ============================================================================

/// Extract metadata using kamadak-exif (pure Rust)
fn extract_with_kamadak_exif(path: &Path) -> Result<MediaMetadata> {
    use std::fs::File;
    use std::io::BufReader;
    
    let file = File::open(path).map_err(MediaError::Io)?;
    let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut reader = BufReader::new(file);
    
    let exif_data = exif::Reader::new()
        .read_from_container(&mut reader)
        .map_err(|e| MediaError::ProcessingError(format!("EXIF parse error: {}", e)))?;
    
    let get_str = |tag: exif::Tag| -> Option<String> {
        exif_data.get_field(tag, exif::In::PRIMARY)
            .map(|f| f.display_value().to_string())
    };
    
    let get_rational = |tag: exif::Tag| -> Option<f64> {
        exif_data.get_field(tag, exif::In::PRIMARY).and_then(|f| {
            match f.value {
                exif::Value::Rational(ref v) if !v.is_empty() => Some(v[0].to_f64()),
                _ => None,
            }
        })
    };
    
    let get_u32 = |tag: exif::Tag| -> Option<u32> {
        exif_data.get_field(tag, exif::In::PRIMARY).and_then(|f| {
            match f.value {
                exif::Value::Short(ref v) if !v.is_empty() => Some(v[0] as u32),
                exif::Value::Long(ref v) if !v.is_empty() => Some(v[0]),
                _ => None,
            }
        })
    };
    
    // Dimensions
    let dimensions = match (get_u32(exif::Tag::ImageWidth).or(get_u32(exif::Tag::PixelXDimension)),
                           get_u32(exif::Tag::ImageLength).or(get_u32(exif::Tag::PixelYDimension))) {
        (Some(w), Some(h)) => Some(Dimensions { width: w, height: h, bit_depth: None }),
        _ => None,
    };
    
    // Exposure
    let exposure = ExposureInfo {
        iso: get_u32(exif::Tag::PhotographicSensitivity).map(|v| v as f32),
        aperture: get_rational(exif::Tag::FNumber).map(|v| v as f32),
        shutter_speed: get_rational(exif::Tag::ExposureTime),
        exposure_compensation: get_rational(exif::Tag::ExposureBiasValue).map(|v| v as f32),
        exposure_mode: get_str(exif::Tag::ExposureMode),
        metering_mode: get_str(exif::Tag::MeteringMode),
        flash: get_str(exif::Tag::Flash),
        white_balance: get_str(exif::Tag::WhiteBalance),
    };
    
    // Lens
    let lens = LensInfo {
        make: get_str(exif::Tag::LensMake),
        model: get_str(exif::Tag::LensModel),
        serial: get_str(exif::Tag::LensSerialNumber),
        focal_length: get_rational(exif::Tag::FocalLength).map(|v| v as f32),
        focal_length_35mm: get_u32(exif::Tag::FocalLengthIn35mmFilm).map(|v| v as u16),
        min_focal: None,
        max_focal: None,
        max_aperture: get_rational(exif::Tag::MaxApertureValue).map(|v| v as f32),
    };
    
    // GPS
    let gps = parse_exif_gps(&exif_data);
    
    Ok(MediaMetadata {
        source_file: path.to_string_lossy().to_string(),
        mime_type: detect_mime_from_extension(path),
        file_type: path.extension().map(|e| e.to_string_lossy().to_uppercase()).unwrap_or_default(),
        file_size,
        date_created: get_str(exif::Tag::DateTimeOriginal),
        date_modified: get_str(exif::Tag::DateTime),
        make: get_str(exif::Tag::Make),
        model: get_str(exif::Tag::Model),
        software: get_str(exif::Tag::Software),
        lens: if lens.model.is_some() || lens.focal_length.is_some() { Some(lens) } else { None },
        dimensions,
        exposure: Some(exposure),
        gps,
        color: Some(ColorInfo {
            color_space: get_str(exif::Tag::ColorSpace),
            ..Default::default()
        }),
        orientation: get_u32(exif::Tag::Orientation).map(|v| v as u8),
        backend: MetadataBackend::KamadakExif,
        ..Default::default()
    })
}

/// Parse GPS from EXIF fields
fn parse_exif_gps(exif_data: &exif::Exif) -> Option<GpsCoordinates> {
    let lat = exif_data.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)?;
    let lon = exif_data.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY)?;
    let lat_ref = exif_data.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY);
    let lon_ref = exif_data.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY);
    
    let parse_dms = |field: &exif::Field| -> Option<f64> {
        match &field.value {
            exif::Value::Rational(v) if v.len() >= 3 => {
                Some(v[0].to_f64() + v[1].to_f64() / 60.0 + v[2].to_f64() / 3600.0)
            }
            _ => None,
        }
    };
    
    let mut latitude = parse_dms(lat)?;
    let mut longitude = parse_dms(lon)?;
    
    // Apply hemisphere
    if let Some(r) = lat_ref {
        if r.display_value().to_string().contains('S') {
            latitude = -latitude;
        }
    }
    if let Some(r) = lon_ref {
        if r.display_value().to_string().contains('W') {
            longitude = -longitude;
        }
    }
    
    let altitude = exif_data.get_field(exif::Tag::GPSAltitude, exif::In::PRIMARY)
        .and_then(|f| match &f.value {
            exif::Value::Rational(v) if !v.is_empty() => Some(v[0].to_f64()),
            _ => None,
        });
    
    Some(GpsCoordinates { latitude, longitude, altitude })
}

// ============================================================================
// LibRaw Backend (RAW Files)
// ============================================================================

/// Extract metadata using LibRaw (for RAW files)
fn extract_with_libraw(path: &Path) -> Result<MediaMetadata> {
    use rsraw::RawImage;
    use rsraw_sys as sys;
    
    let file_data = std::fs::read(path).map_err(MediaError::Io)?;
    let file_size = file_data.len() as u64;
    
    let raw = RawImage::open(&file_data)
        .map_err(|e| MediaError::ProcessingError(format!("LibRaw error: {:?}", e)))?;
    
    let raw_ptr: *mut sys::libraw_data_t = unsafe { std::mem::transmute_copy(&raw) };
    
    unsafe {
        let idata = &(*raw_ptr).idata;
        let other = &(*raw_ptr).other;
        let sizes = &(*raw_ptr).sizes;
        let lens_data = &(*raw_ptr).lens;
        
        let cstr = |ptr: *const i8| -> Option<String> {
            let s = std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string();
            if s.is_empty() { None } else { Some(s) }
        };
        
        let make = cstr(idata.make.as_ptr());
        let model = cstr(idata.model.as_ptr());
        
        let lens = LensInfo {
            make: cstr(lens_data.LensMake.as_ptr()),
            model: cstr(lens_data.Lens.as_ptr()),
            serial: cstr(lens_data.LensSerial.as_ptr()),
            focal_length: if other.focal_len > 0.0 { Some(other.focal_len) } else { None },
            focal_length_35mm: if lens_data.FocalLengthIn35mmFormat > 0 { 
                Some(lens_data.FocalLengthIn35mmFormat) 
            } else { 
                None 
            },
            min_focal: if lens_data.MinFocal > 0.0 { Some(lens_data.MinFocal) } else { None },
            max_focal: if lens_data.MaxFocal > 0.0 { Some(lens_data.MaxFocal) } else { None },
            max_aperture: if lens_data.MaxAp4MinFocal > 0.0 { Some(lens_data.MaxAp4MinFocal) } else { None },
        };
        
        let exposure = ExposureInfo {
            iso: if other.iso_speed > 0.0 { Some(other.iso_speed) } else { None },
            aperture: if other.aperture > 0.0 { Some(other.aperture) } else { None },
            shutter_speed: if other.shutter > 0.0 { Some(other.shutter as f64) } else { None },
            ..Default::default()
        };
        
        // Parse GPS
        let parsed_gps = &other.parsed_gps;
        let gps = if parsed_gps.gpsparsed != 0 {
            let lat = parsed_gps.latitude[0] as f64 
                + parsed_gps.latitude[1] as f64 / 60.0 
                + parsed_gps.latitude[2] as f64 / 3600.0;
            let lon = parsed_gps.longitude[0] as f64 
                + parsed_gps.longitude[1] as f64 / 60.0 
                + parsed_gps.longitude[2] as f64 / 3600.0;
            
            let lat = if parsed_gps.latref as u8 == b'S' { -lat } else { lat };
            let lon = if parsed_gps.longref as u8 == b'W' { -lon } else { lon };
            
            Some(GpsCoordinates {
                latitude: lat,
                longitude: lon,
                altitude: if parsed_gps.altitude != 0.0 { Some(parsed_gps.altitude as f64) } else { None },
            })
        } else {
            None
        };
        
        Ok(MediaMetadata {
            source_file: path.to_string_lossy().to_string(),
            mime_type: detect_mime_from_extension(path),
            file_type: path.extension().map(|e| e.to_string_lossy().to_uppercase()).unwrap_or_default(),
            file_size,
            date_created: if other.timestamp > 0 {
                Some(format_timestamp(other.timestamp as i64))
            } else {
                None
            },
            make,
            model,
            lens: if lens.model.is_some() || lens.focal_length.is_some() { Some(lens) } else { None },
            dimensions: Some(Dimensions {
                width: sizes.width as u32,
                height: sizes.height as u32,
                bit_depth: None,
            }),
            exposure: Some(exposure),
            gps,
            artist: cstr(other.artist.as_ptr()),
            description: cstr(other.desc.as_ptr()),
            backend: MetadataBackend::LibRaw,
            ..Default::default()
        })
    }
}

// ============================================================================
// FFprobe Backend (Video/Audio)
// ============================================================================

/// Extract metadata using FFprobe (for video/audio)
fn extract_with_ffprobe(path: &Path) -> Result<MediaMetadata> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .output()
        .map_err(|e| MediaError::ProcessingError(format!("FFprobe failed: {}", e)))?;
    
    if !output.status.success() {
        return Err(MediaError::ProcessingError("FFprobe failed".to_string()));
    }
    
    let json_str = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| MediaError::ProcessingError(format!("JSON parse error: {}", e)))?;
    
    let format = parsed.get("format").and_then(|f| f.as_object());
    let streams = parsed.get("streams").and_then(|s| s.as_array());
    
    // Find video and audio streams
    let video_stream = streams.and_then(|s| s.iter().find(|st| {
        st.get("codec_type").and_then(|t| t.as_str()) == Some("video")
    }));
    let audio_stream = streams.and_then(|s| s.iter().find(|st| {
        st.get("codec_type").and_then(|t| t.as_str()) == Some("audio")
    }));
    
    let get_format_str = |key: &str| -> Option<String> {
        format.and_then(|f| f.get(key)).and_then(|v| v.as_str()).map(|s| s.to_string())
    };
    
    let get_format_tags = |key: &str| -> Option<String> {
        format.and_then(|f| f.get("tags"))
            .and_then(|t| t.get(key))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };
    
    let duration = format.and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str())
        .and_then(|s| s.parse::<f64>().ok());
    
    let bitrate = format.and_then(|f| f.get("bit_rate"))
        .and_then(|b| b.as_str())
        .and_then(|s| s.parse::<u64>().ok());
    
    // Video info
    let dimensions = video_stream.and_then(|v| {
        let w = v.get("width").and_then(|x| x.as_u64()).map(|x| x as u32)?;
        let h = v.get("height").and_then(|x| x.as_u64()).map(|x| x as u32)?;
        Some(Dimensions { width: w, height: h, bit_depth: None })
    });
    
    let frame_rate = video_stream.and_then(|v| {
        v.get("r_frame_rate").and_then(|r| r.as_str()).and_then(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                let num: f64 = parts[0].parse().ok()?;
                let den: f64 = parts[1].parse().ok()?;
                Some(num / den)
            } else {
                s.parse().ok()
            }
        })
    });
    
    let codec = video_stream
        .or(audio_stream)
        .and_then(|s| s.get("codec_name"))
        .and_then(|c| c.as_str())
        .map(|s| s.to_string());
    
    // Audio info
    let sample_rate = audio_stream
        .and_then(|a| a.get("sample_rate"))
        .and_then(|r| r.as_str())
        .and_then(|s| s.parse::<u32>().ok());
    
    let channels = audio_stream
        .and_then(|a| a.get("channels"))
        .and_then(|c| c.as_u64())
        .map(|c| c as u8);
    
    let file_size = format
        .and_then(|f| f.get("size"))
        .and_then(|s| s.as_str())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    
    Ok(MediaMetadata {
        source_file: path.to_string_lossy().to_string(),
        mime_type: detect_mime_from_extension(path),
        file_type: get_format_str("format_name").unwrap_or_default(),
        file_size,
        date_created: get_format_tags("creation_time"),
        title: get_format_tags("title"),
        artist: get_format_tags("artist").or(get_format_tags("album_artist")),
        description: get_format_tags("description").or(get_format_tags("comment")),
        copyright: get_format_tags("copyright"),
        dimensions,
        duration,
        codec,
        bitrate,
        frame_rate,
        sample_rate,
        channels,
        backend: MetadataBackend::FFprobe,
        ..Default::default()
    })
}

// ============================================================================
// Basic File Info (Last Resort)
// ============================================================================

/// Extract basic file info when no other backend is available
fn extract_basic_info(path: &Path) -> Result<MediaMetadata> {
    let metadata = std::fs::metadata(path).map_err(MediaError::Io)?;
    
    Ok(MediaMetadata {
        source_file: path.to_string_lossy().to_string(),
        mime_type: detect_mime_from_extension(path),
        file_type: path.extension()
            .map(|e| e.to_string_lossy().to_uppercase())
            .unwrap_or_default(),
        file_size: metadata.len(),
        date_modified: metadata.modified().ok().map(|t| {
            chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()
        }),
        backend: MetadataBackend::FileMagic,
        ..Default::default()
    })
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Detect MIME type from file extension
pub fn detect_mime_from_extension(path: &Path) -> String {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    
    match ext.as_str() {
        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "tiff" | "tif" => "image/tiff",
        "bmp" => "image/bmp",
        "heic" | "heif" => "image/heic",
        "avif" => "image/avif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        // RAW
        "cr2" => "image/x-canon-cr2",
        "cr3" => "image/x-canon-cr3",
        "nef" => "image/x-nikon-nef",
        "arw" => "image/x-sony-arw",
        "dng" => "image/x-adobe-dng",
        "raf" => "image/x-fuji-raf",
        "orf" => "image/x-olympus-orf",
        "rw2" => "image/x-panasonic-rw2",
        "pef" => "image/x-pentax-pef",
        "srw" => "image/x-samsung-srw",
        // Video
        "mp4" | "m4v" => "video/mp4",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        "webm" => "video/webm",
        "wmv" => "video/x-ms-wmv",
        "flv" => "video/x-flv",
        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "wma" => "audio/x-ms-wma",
        // Documents
        "pdf" => "application/pdf",
        "json" => "application/json",
        "xml" => "application/xml",
        // Default
        _ => "application/octet-stream",
    }.to_string()
}

/// Format Unix timestamp to ISO 8601
fn format_timestamp(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_default()
}

// ============================================================================
// Helper Methods on MediaMetadata
// ============================================================================

impl MediaMetadata {
    /// Get human-readable exposure summary
    pub fn exposure_summary(&self) -> Option<String> {
        let exp = self.exposure.as_ref()?;
        
        let shutter = exp.shutter_speed.map(|s| {
            if s >= 1.0 {
                format!("{:.1}s", s)
            } else if s > 0.0 {
                format!("1/{:.0}s", 1.0 / s)
            } else {
                "?".to_string()
            }
        }).unwrap_or_else(|| "?".to_string());
        
        Some(format!("f/{:.1} {} ISO{:.0} {}mm",
            exp.aperture.unwrap_or(0.0),
            shutter,
            exp.iso.unwrap_or(0.0),
            self.lens.as_ref().and_then(|l| l.focal_length).unwrap_or(0.0)
        ))
    }
    
    /// Get megapixel count
    pub fn megapixels(&self) -> Option<f32> {
        self.dimensions.as_ref().map(|d| {
            (d.width * d.height) as f32 / 1_000_000.0
        })
    }
    
    /// Get aspect ratio as string (e.g., "16:9", "3:2")
    pub fn aspect_ratio(&self) -> Option<String> {
        let d = self.dimensions.as_ref()?;
        let g = gcd(d.width, d.height);
        Some(format!("{}:{}", d.width / g, d.height / g))
    }
    
    /// Check if this is a video file
    pub fn is_video(&self) -> bool {
        self.mime_type.starts_with("video/") || (self.duration.is_some() && self.frame_rate.is_some())
    }
    
    /// Check if this is an audio file
    pub fn is_audio(&self) -> bool {
        self.mime_type.starts_with("audio/") || 
        (self.duration.is_some() && self.sample_rate.is_some() && !self.is_video())
    }
    
    /// Check if this is an image file
    pub fn is_image(&self) -> bool {
        self.mime_type.starts_with("image/")
    }
    
    /// Check if this is a RAW image file
    pub fn is_raw(&self) -> bool {
        self.mime_type.contains("-x-") || 
        matches!(self.file_type.to_lowercase().as_str(), 
            "cr2" | "cr3" | "nef" | "arw" | "dng" | "raf" | "orf" | "rw2" | "pef" | "srw")
    }
}

/// GCD for aspect ratio calculation
fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

// ============================================================================
// Re-exports for backward compatibility
// ============================================================================

/// Legacy type alias for RawMetadata
pub type RawMetadata = MediaMetadata;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exiftool_available() {
        // Just check it doesn't panic
        let _ = exiftool_available();
    }
    
    #[test]
    fn test_mime_detection() {
        assert_eq!(detect_mime_from_extension(Path::new("test.jpg")), "image/jpeg");
        assert_eq!(detect_mime_from_extension(Path::new("test.MP4")), "video/mp4");
        assert_eq!(detect_mime_from_extension(Path::new("test.dng")), "image/x-adobe-dng");
        assert_eq!(detect_mime_from_extension(Path::new("test.cr2")), "image/x-canon-cr2");
        assert_eq!(detect_mime_from_extension(Path::new("test.mp3")), "audio/mpeg");
    }
    
    #[test]
    fn test_shutter_speed_parsing() {
        assert_eq!(parse_shutter_speed(Some("1/250")), Some(0.004));
        assert_eq!(parse_shutter_speed(Some("1/1000")), Some(0.001));
        assert_eq!(parse_shutter_speed(Some("2")), Some(2.0));
        assert_eq!(parse_shutter_speed(None), None);
    }
    
    #[test]
    fn test_aspect_ratio() {
        let meta = MediaMetadata {
            dimensions: Some(Dimensions { width: 1920, height: 1080, bit_depth: None }),
            ..Default::default()
        };
        assert_eq!(meta.aspect_ratio(), Some("16:9".to_string()));
        
        let meta = MediaMetadata {
            dimensions: Some(Dimensions { width: 6000, height: 4000, bit_depth: None }),
            ..Default::default()
        };
        assert_eq!(meta.aspect_ratio(), Some("3:2".to_string()));
    }
    
    #[test]
    fn test_media_type_detection() {
        let video = MediaMetadata {
            mime_type: "video/mp4".to_string(),
            ..Default::default()
        };
        assert!(video.is_video());
        assert!(!video.is_audio());
        assert!(!video.is_image());
        
        let image = MediaMetadata {
            mime_type: "image/jpeg".to_string(),
            ..Default::default()
        };
        assert!(image.is_image());
        assert!(!image.is_video());
        
        let raw = MediaMetadata {
            file_type: "DNG".to_string(),
            mime_type: "image/x-adobe-dng".to_string(),
            ..Default::default()
        };
        assert!(raw.is_raw());
        assert!(raw.is_image());
    }
}
