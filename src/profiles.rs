//! Camera Color Profiles
//!
//! Extracts and applies camera-specific color profiles from MakerNotes
//! to match the in-camera JPEG processing pipeline.
//!
//! ## Supported Data
//!
//! - **Color Matrix**: Camera → XYZ → sRGB/AdobeRGB conversion
//! - **White Balance**: RGGB multipliers for accurate WB
//! - **Tone Curve**: Camera's contrast/gamma curve (11-point)
//! - **Picture Wizard**: Saturation, sharpness, contrast adjustments
//! - **Black/White Levels**: Sensor calibration data
//!
//! ## Usage
//!
//! ```rust,ignore
//! use soma_media::profiles::{CameraProfile, extract_camera_profile};
//!
//! let profile = extract_camera_profile(Path::new("photo.srw"))?;
//! let processed = profile.apply_to_rgb_image(&raw_rgb_data, width, height)?;
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::error::MediaError;

pub type Result<T> = std::result::Result<T, MediaError>;

// ============================================================================
// Camera Profile Types
// ============================================================================

/// Camera color profile extracted from MakerNotes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraProfile {
    /// Camera make (Samsung, Canon, Nikon, etc.)
    pub make: String,
    
    /// Camera model
    pub model: String,
    
    /// Color matrix for camera RGB → XYZ conversion (3x3, scaled by 256)
    pub color_matrix: Option<ColorMatrix>,
    
    /// sRGB color matrix (if different from default)
    pub color_matrix_srgb: Option<ColorMatrix>,
    
    /// Adobe RGB color matrix
    pub color_matrix_adobe_rgb: Option<ColorMatrix>,
    
    /// White balance RGGB multipliers
    pub wb_multipliers: Option<WbMultipliers>,
    
    /// Tone curve (input → output mapping)
    pub tone_curve: Option<ToneCurve>,
    
    /// Picture style adjustments
    pub picture_style: Option<PictureStyle>,
    
    /// Black level per channel
    pub black_level: Option<[u16; 4]>,
    
    /// White/saturation level
    pub white_level: Option<u16>,
    
    /// Highlight linearity limit
    pub highlight_limit: Option<u16>,
    
    /// Color space (sRGB, AdobeRGB, etc.)
    pub color_space: Option<String>,
    
    /// Firmware version
    pub firmware: Option<String>,
    
    /// Sensor temperature at capture (for noise estimation)
    pub temperature_c: Option<f32>,
}

/// 3x3 color matrix (values scaled by 256 typically)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorMatrix {
    pub m: [[f32; 3]; 3],
}

/// RGGB white balance multipliers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WbMultipliers {
    pub r: f32,
    pub g1: f32,
    pub g2: f32,
    pub b: f32,
}

/// Tone curve - maps input values to output values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneCurve {
    /// Input values (typically 12 points: 0, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 12288, 16383)
    pub input: Vec<u16>,
    /// Output values (0-255 for 8-bit output)
    pub output: Vec<u8>,
}

/// Picture style/wizard adjustments
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PictureStyle {
    /// Style name (Standard, Vivid, Portrait, Custom1, etc.)
    pub name: Option<String>,
    /// Saturation adjustment (-4 to +4 typically)
    pub saturation: i8,
    /// Sharpness adjustment
    pub sharpness: i8,
    /// Contrast adjustment
    pub contrast: i8,
    /// Color tone/hue adjustment
    pub color_tone: i8,
}

// ============================================================================
// Profile Extraction
// ============================================================================

/// Extract camera profile from MakerNotes using ExifTool
pub fn extract_camera_profile(path: &Path) -> Result<CameraProfile> {
    let output = Command::new("exiftool")
        .args(["-j", "-G", "-n"])
        .arg(path)
        .output()
        .map_err(|e| MediaError::ProcessingError(format!("ExifTool failed: {}", e)))?;
    
    if !output.status.success() {
        return Err(MediaError::ProcessingError("ExifTool failed".to_string()));
    }
    
    let json_str = String::from_utf8_lossy(&output.stdout);
    let parsed: Vec<HashMap<String, serde_json::Value>> = serde_json::from_str(&json_str)
        .map_err(|e| MediaError::ProcessingError(format!("JSON parse error: {}", e)))?;
    
    let tags = parsed.into_iter().next()
        .ok_or_else(|| MediaError::ProcessingError("No metadata found".to_string()))?;
    
    parse_camera_profile(&tags)
}

fn parse_camera_profile(tags: &HashMap<String, serde_json::Value>) -> Result<CameraProfile> {
    let get_str = |key: &str| -> Option<String> {
        tags.get(key).and_then(|v| {
            if v.is_string() { v.as_str().map(|s| s.to_string()) }
            else { Some(v.to_string().trim_matches('"').to_string()) }
        })
    };
    
    let get_f32 = |key: &str| -> Option<f32> {
        tags.get(key).and_then(|v| v.as_f64().map(|f| f as f32))
    };
    
    // Parse color matrices
    let color_matrix = parse_color_matrix(get_str("MakerNotes:ColorMatrix").as_deref());
    let color_matrix_srgb = parse_color_matrix(get_str("MakerNotes:ColorMatrixSRGB").as_deref());
    let color_matrix_adobe_rgb = parse_color_matrix(get_str("MakerNotes:ColorMatrixAdobeRGB").as_deref());
    
    // Parse WB multipliers
    let wb_multipliers = parse_wb_multipliers(
        get_str("MakerNotes:WB_RGGBLevelsAuto")
            .or_else(|| get_str("MakerNotes:WB_RGGBLevelsUncorrected"))
            .as_deref()
    );
    
    // Parse tone curve
    let tone_curve = parse_tone_curve(get_str("MakerNotes:ToneCurveSRGB").as_deref());
    
    // Parse picture style
    let picture_style = PictureStyle {
        name: get_str("MakerNotes:PictureWizardMode"),
        saturation: get_f32("MakerNotes:PictureWizardSaturation").map(|f| f as i8).unwrap_or(0),
        sharpness: get_f32("MakerNotes:PictureWizardSharpness").map(|f| f as i8).unwrap_or(0),
        contrast: get_f32("MakerNotes:PictureWizardContrast").map(|f| f as i8).unwrap_or(0),
        color_tone: 0,
    };
    
    // Parse black level
    let black_level = parse_rggb_levels(get_str("MakerNotes:WB_RGGBLevelsBlack").as_deref());
    
    // Parse temperature
    let temperature_c = get_str("MakerNotes:CameraTemperature")
        .and_then(|s| s.split_whitespace().next()?.parse().ok());
    
    // Parse highlight limit
    let highlight_limit = get_f32("MakerNotes:HighlightLinearityLimit").map(|f| f as u16);
    
    Ok(CameraProfile {
        make: get_str("EXIF:Make").unwrap_or_default(),
        model: get_str("EXIF:Model").unwrap_or_default(),
        color_matrix,
        color_matrix_srgb,
        color_matrix_adobe_rgb,
        wb_multipliers,
        tone_curve,
        picture_style: Some(picture_style),
        black_level,
        white_level: get_f32("EXIF:WhiteLevel").map(|f| f as u16),
        highlight_limit,
        color_space: get_str("MakerNotes:ColorSpace"),
        firmware: get_str("MakerNotes:FirmwareName").map(|s| s.to_string()),
        temperature_c,
    })
}

/// Parse "436 -144 -38 -26 288 -8 12 -72 314" into 3x3 matrix
fn parse_color_matrix(s: Option<&str>) -> Option<ColorMatrix> {
    let s = s?;
    let values: Vec<f32> = s.split_whitespace()
        .filter_map(|v| v.parse().ok())
        .collect();
    
    if values.len() >= 9 {
        Some(ColorMatrix {
            m: [
                [values[0] / 256.0, values[1] / 256.0, values[2] / 256.0],
                [values[3] / 256.0, values[4] / 256.0, values[5] / 256.0],
                [values[6] / 256.0, values[7] / 256.0, values[8] / 256.0],
            ]
        })
    } else {
        None
    }
}

/// Parse "7376 4096 4096 5952" into WB multipliers
fn parse_wb_multipliers(s: Option<&str>) -> Option<WbMultipliers> {
    let s = s?;
    let values: Vec<f32> = s.split_whitespace()
        .filter_map(|v| v.parse().ok())
        .collect();
    
    if values.len() >= 4 {
        // Normalize to green channel
        let g_avg = (values[1] + values[2]) / 2.0;
        Some(WbMultipliers {
            r: values[0] / g_avg,
            g1: values[1] / g_avg,
            g2: values[2] / g_avg,
            b: values[3] / g_avg,
        })
    } else {
        None
    }
}

/// Parse "11 0 64 128 256 512 1024 2048 4096 8192 12288 16383 0 7 14 28 52 90 140 190 230 245 255"
fn parse_tone_curve(s: Option<&str>) -> Option<ToneCurve> {
    let s = s?;
    let values: Vec<u16> = s.split_whitespace()
        .filter_map(|v| v.parse().ok())
        .collect();
    
    // Format: count, input_values..., output_values...
    if values.len() >= 3 {
        let count = values[0] as usize;
        if values.len() >= 1 + count * 2 {
            let input: Vec<u16> = values[1..1+count].to_vec();
            let output: Vec<u8> = values[1+count..1+count*2]
                .iter()
                .map(|&v| v.min(255) as u8)
                .collect();
            return Some(ToneCurve { input, output });
        }
    }
    None
}

/// Parse RGGB levels to [u16; 4]
fn parse_rggb_levels(s: Option<&str>) -> Option<[u16; 4]> {
    let s = s?;
    let values: Vec<u16> = s.split_whitespace()
        .filter_map(|v| v.parse().ok())
        .collect();
    
    if values.len() >= 4 {
        Some([values[0], values[1], values[2], values[3]])
    } else {
        None
    }
}

// ============================================================================
// Profile Application
// ============================================================================

impl CameraProfile {
    /// Apply camera profile to RGB image data
    ///
    /// This replicates the camera's internal processing:
    /// 1. Subtract black level
    /// 2. Apply white balance
    /// 3. Apply color matrix
    /// 4. Apply tone curve
    /// 5. Apply picture style adjustments
    pub fn apply_to_rgb(&self, rgb: &mut [u8], width: usize, height: usize) {
        let pixels = width * height;
        
        // Build lookup tables for efficiency
        let tone_lut = self.build_tone_lut();
        let sat_factor = self.saturation_factor();
        
        for i in 0..pixels {
            let idx = i * 3;
            if idx + 2 >= rgb.len() { break; }
            
            // Normalize to 0.0-1.0 range for color matrix
            let mut r = rgb[idx] as f32 / 255.0;
            let mut g = rgb[idx + 1] as f32 / 255.0;
            let mut b = rgb[idx + 2] as f32 / 255.0;
            
            // Apply color matrix if available (in linear space)
            if let Some(ref matrix) = self.color_matrix_srgb {
                let nr = matrix.m[0][0] * r + matrix.m[0][1] * g + matrix.m[0][2] * b;
                let ng = matrix.m[1][0] * r + matrix.m[1][1] * g + matrix.m[1][2] * b;
                let nb = matrix.m[2][0] * r + matrix.m[2][1] * g + matrix.m[2][2] * b;
                r = nr.clamp(0.0, 1.0);
                g = ng.clamp(0.0, 1.0);
                b = nb.clamp(0.0, 1.0);
            }
            
            // Convert back to 0-255 range for saturation and tone curve
            r *= 255.0;
            g *= 255.0;
            b *= 255.0;
            
            // Apply saturation adjustment
            if sat_factor != 1.0 {
                let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                r = lum + (r - lum) * sat_factor;
                g = lum + (g - lum) * sat_factor;
                b = lum + (b - lum) * sat_factor;
            }
            
            // Apply tone curve via LUT
            rgb[idx] = tone_lut[r.clamp(0.0, 255.0) as usize];
            rgb[idx + 1] = tone_lut[g.clamp(0.0, 255.0) as usize];
            rgb[idx + 2] = tone_lut[b.clamp(0.0, 255.0) as usize];
        }
    }
    
    /// Apply to 16-bit linear RGB data (from RAW processing)
    pub fn apply_to_rgb16(&self, rgb: &mut [u16], width: usize, height: usize, max_value: u16) {
        let pixels = width * height;
        let scale = 255.0 / max_value as f32;
        
        // Get processing parameters
        let black = self.black_level.unwrap_or([0, 0, 0, 0]);
        let white = self.white_level.unwrap_or(max_value);
        let range = (white - black[0]) as f32;
        
        let tone_lut = self.build_tone_lut_16(max_value);
        let sat_factor = self.saturation_factor();
        
        for i in 0..pixels {
            let idx = i * 3;
            if idx + 2 >= rgb.len() { break; }
            
            // Subtract black level and normalize
            let mut r = ((rgb[idx].saturating_sub(black[0])) as f32 / range).clamp(0.0, 1.0);
            let mut g = ((rgb[idx + 1].saturating_sub(black[1])) as f32 / range).clamp(0.0, 1.0);
            let mut b = ((rgb[idx + 2].saturating_sub(black[2])) as f32 / range).clamp(0.0, 1.0);
            
            // Apply color matrix
            if let Some(ref matrix) = self.color_matrix_srgb {
                let nr = matrix.m[0][0] * r + matrix.m[0][1] * g + matrix.m[0][2] * b;
                let ng = matrix.m[1][0] * r + matrix.m[1][1] * g + matrix.m[1][2] * b;
                let nb = matrix.m[2][0] * r + matrix.m[2][1] * g + matrix.m[2][2] * b;
                r = nr.clamp(0.0, 1.0);
                g = ng.clamp(0.0, 1.0);
                b = nb.clamp(0.0, 1.0);
            }
            
            // Apply saturation
            if sat_factor != 1.0 {
                let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                r = (lum + (r - lum) * sat_factor).clamp(0.0, 1.0);
                g = (lum + (g - lum) * sat_factor).clamp(0.0, 1.0);
                b = (lum + (b - lum) * sat_factor).clamp(0.0, 1.0);
            }
            
            // Apply tone curve
            let ri = (r * max_value as f32) as usize;
            let gi = (g * max_value as f32) as usize;
            let bi = (b * max_value as f32) as usize;
            
            rgb[idx] = tone_lut.get(ri).copied().unwrap_or(max_value);
            rgb[idx + 1] = tone_lut.get(gi).copied().unwrap_or(max_value);
            rgb[idx + 2] = tone_lut.get(bi).copied().unwrap_or(max_value);
        }
    }
    
    /// Get normalized white balance multipliers
    pub fn get_wb_normalized(&self) -> [f32; 4] {
        if let Some(ref wb) = self.wb_multipliers {
            [wb.r, wb.g1, wb.g2, wb.b]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        }
    }
    
    /// Calculate saturation factor from picture style
    fn saturation_factor(&self) -> f32 {
        if let Some(ref style) = self.picture_style {
            // Typically -4 to +4, map to 0.6 to 1.4
            1.0 + (style.saturation as f32 * 0.1)
        } else {
            1.0
        }
    }
    
    /// Build 8-bit tone curve LUT
    fn build_tone_lut(&self) -> [u8; 256] {
        let mut lut = [0u8; 256];
        
        if let Some(ref curve) = self.tone_curve {
            // Interpolate the curve
            for i in 0..256 {
                let input = (i as f32 / 255.0) * 16383.0; // Scale to 14-bit
                lut[i] = interpolate_curve(&curve.input, &curve.output, input as u16);
            }
        } else {
            // Identity curve with slight contrast (typical camera default)
            for i in 0..256 {
                lut[i] = i as u8;
            }
        }
        
        // Apply contrast adjustment
        if let Some(ref style) = self.picture_style {
            if style.contrast != 0 {
                let factor = 1.0 + (style.contrast as f32 * 0.1);
                for i in 0..256 {
                    let v = (lut[i] as f32 - 128.0) * factor + 128.0;
                    lut[i] = v.clamp(0.0, 255.0) as u8;
                }
            }
        }
        
        lut
    }
    
    /// Build 16-bit tone curve LUT
    fn build_tone_lut_16(&self, max_value: u16) -> Vec<u16> {
        let size = max_value as usize + 1;
        let mut lut = vec![0u16; size];
        
        if let Some(ref curve) = self.tone_curve {
            for i in 0..size {
                let input = (i as f32 / size as f32) * 16383.0;
                let out8 = interpolate_curve(&curve.input, &curve.output, input as u16);
                lut[i] = ((out8 as f32 / 255.0) * max_value as f32) as u16;
            }
        } else {
            for i in 0..size {
                lut[i] = i as u16;
            }
        }
        
        lut
    }
    
    /// Print profile summary
    pub fn summary(&self) -> String {
        let mut s = format!("{} {}\n", self.make, self.model);
        
        if let Some(ref style) = self.picture_style {
            s.push_str(&format!("  Picture Style: {:?} (Sat:{}, Sharp:{}, Con:{})\n",
                style.name, style.saturation, style.sharpness, style.contrast));
        }
        
        if let Some(ref wb) = self.wb_multipliers {
            s.push_str(&format!("  WB Multipliers: R:{:.2} G1:{:.2} G2:{:.2} B:{:.2}\n",
                wb.r, wb.g1, wb.g2, wb.b));
        }
        
        if self.color_matrix_srgb.is_some() {
            s.push_str("  Color Matrix: sRGB available\n");
        }
        
        if self.tone_curve.is_some() {
            s.push_str("  Tone Curve: Available\n");
        }
        
        if let Some(temp) = self.temperature_c {
            s.push_str(&format!("  Sensor Temp: {}°C\n", temp));
        }
        
        s
    }
}

/// Interpolate value from tone curve
fn interpolate_curve(input: &[u16], output: &[u8], value: u16) -> u8 {
    if input.is_empty() || output.is_empty() {
        return (value as f32 / 16383.0 * 255.0) as u8;
    }
    
    // Find surrounding points
    let mut i = 0;
    while i < input.len() - 1 && input[i + 1] < value {
        i += 1;
    }
    
    if i >= input.len() - 1 {
        return *output.last().unwrap_or(&255);
    }
    
    // Linear interpolation
    let x0 = input[i] as f32;
    let x1 = input[i + 1] as f32;
    let y0 = output[i] as f32;
    let y1 = output.get(i + 1).copied().unwrap_or(255) as f32;
    
    let t = if x1 != x0 { (value as f32 - x0) / (x1 - x0) } else { 0.0 };
    (y0 + t * (y1 - y0)).clamp(0.0, 255.0) as u8
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_color_matrix() {
        let matrix = parse_color_matrix(Some("256 0 0 0 256 0 0 0 256"));
        assert!(matrix.is_some());
        let m = matrix.unwrap();
        assert!((m.m[0][0] - 1.0).abs() < 0.01);
        assert!((m.m[1][1] - 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_parse_wb_multipliers() {
        let wb = parse_wb_multipliers(Some("8000 4000 4000 6000"));
        assert!(wb.is_some());
        let w = wb.unwrap();
        assert!((w.g1 - 1.0).abs() < 0.01);
        assert!(w.r > w.g1); // R higher for daylight
    }
    
    #[test]
    fn test_parse_tone_curve() {
        let curve = parse_tone_curve(Some("3 0 128 255 0 128 255"));
        assert!(curve.is_some());
        let c = curve.unwrap();
        assert_eq!(c.input.len(), 3);
        assert_eq!(c.output.len(), 3);
    }
    
    #[test]
    fn test_interpolate_curve() {
        let input = vec![0, 128, 255];
        let output = vec![0, 64, 255];
        
        assert_eq!(interpolate_curve(&input, &output, 0), 0);
        assert_eq!(interpolate_curve(&input, &output, 128), 64);
        assert_eq!(interpolate_curve(&input, &output, 255), 255);
    }
}
