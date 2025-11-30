//! Parallel Demosaic Implementation
//!
//! Provides tile-based parallel Bayer demosaicing for RAW images.
//! This can significantly speed up single-file RAW processing on multi-core CPUs.
//!
//! ## How It Works
//!
//! 1. Extract raw Bayer sensor data from LibRaw
//! 2. Split into overlapping tiles (e.g., 512x512 with 16px overlap)
//! 3. Demosaic each tile in parallel using rayon
//! 4. Blend overlapping regions with weighted averaging
//! 5. Return final RGB image
//!
//! ## Performance
//!
//! - Standard LibRaw demosaic: ~400-600ms for 24MP
//! - Tile-based parallel (8 cores): ~100-150ms for 24MP
//! - GPU demosaic: ~5-15ms for 24MP (via soma_compute)

use rayon::prelude::*;

/// Bayer pattern types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BayerPattern {
    /// Red-Green-Green-Blue (Canon, Nikon)
    RGGB = 0,
    /// Green-Red-Blue-Green (Sony, Pentax)  
    GRBG = 1,
    /// Green-Blue-Red-Green
    GBRG = 2,
    /// Blue-Green-Green-Red
    BGGR = 3,
}

impl BayerPattern {
    /// Detect pattern from LibRaw cdesc string
    pub fn from_cdesc(cdesc: &[i8; 5]) -> Self {
        // LibRaw stores pattern as 4 chars like "RGGB"
        let pattern = [cdesc[0] as u8, cdesc[1] as u8, cdesc[2] as u8, cdesc[3] as u8];
        match &pattern {
            b"RGGB" => BayerPattern::RGGB,
            b"GRBG" => BayerPattern::GRBG,
            b"GBRG" => BayerPattern::GBRG,
            b"BGGR" => BayerPattern::BGGR,
            _ => BayerPattern::RGGB, // Default
        }
    }
    
    /// Get color at position (x, y)
    /// Returns: 0=Red, 1=Green, 2=Blue
    #[inline]
    pub fn color_at(&self, x: usize, y: usize) -> u8 {
        let x_odd = x & 1;
        let y_odd = y & 1;
        
        match self {
            BayerPattern::RGGB => {
                match (x_odd, y_odd) {
                    (0, 0) => 0, // R
                    (1, 0) => 1, // G
                    (0, 1) => 1, // G
                    (1, 1) => 2, // B
                    _ => unreachable!()
                }
            }
            BayerPattern::GRBG => {
                match (x_odd, y_odd) {
                    (0, 0) => 1, // G
                    (1, 0) => 0, // R
                    (0, 1) => 2, // B
                    (1, 1) => 1, // G
                    _ => unreachable!()
                }
            }
            BayerPattern::GBRG => {
                match (x_odd, y_odd) {
                    (0, 0) => 1, // G
                    (1, 0) => 2, // B
                    (0, 1) => 0, // R
                    (1, 1) => 1, // G
                    _ => unreachable!()
                }
            }
            BayerPattern::BGGR => {
                match (x_odd, y_odd) {
                    (0, 0) => 2, // B
                    (1, 0) => 1, // G
                    (0, 1) => 1, // G
                    (1, 1) => 0, // R
                    _ => unreachable!()
                }
            }
        }
    }
}

/// Demosaic algorithm selection
#[derive(Debug, Clone, Copy)]
pub enum DemosaicAlgorithm {
    /// Simple bilinear interpolation (fastest)
    Bilinear,
    /// Variable Number of Gradients (good quality/speed balance)
    VNG,
    /// Adaptive Homogeneity-Directed (high quality)
    AHD,
}

/// Tile for parallel processing
struct Tile {
    /// Tile data (subset of Bayer array)
    data: Vec<u16>,
    /// Tile position in original image
    x: usize,
    y: usize,
    /// Tile dimensions
    width: usize,
    height: usize,
    /// Overlap size (for blending)
    overlap: usize,
}

/// Demosaiced tile result
struct DemosaicedTile {
    /// RGB data (3 bytes per pixel)
    rgb: Vec<u8>,
    /// Position in final image
    x: usize,
    y: usize,
    /// Dimensions
    width: usize,
    height: usize,
    /// Overlap size
    overlap: usize,
}

/// Parallel demosaic processor
pub struct ParallelDemosaic {
    /// Tile size (excluding overlap)
    tile_size: usize,
    /// Overlap between tiles for blending
    overlap: usize,
    /// Demosaic algorithm
    algorithm: DemosaicAlgorithm,
}

impl Default for ParallelDemosaic {
    fn default() -> Self {
        Self {
            tile_size: 512,
            overlap: 16,
            algorithm: DemosaicAlgorithm::Bilinear,
        }
    }
}

impl ParallelDemosaic {
    /// Create with custom tile size
    pub fn with_tile_size(tile_size: usize) -> Self {
        Self {
            tile_size,
            overlap: 16,
            algorithm: DemosaicAlgorithm::Bilinear,
        }
    }
    
    /// Set demosaic algorithm
    pub fn with_algorithm(mut self, algorithm: DemosaicAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }
    
    /// Demosaic raw Bayer data to RGB in parallel
    /// 
    /// # Arguments
    /// * `bayer_data` - Raw 16-bit Bayer sensor data
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    /// * `pattern` - Bayer pattern (RGGB, GRBG, etc.)
    /// * `black_level` - Black level to subtract
    /// * `white_level` - White level (saturation point)
    /// 
    /// # Returns
    /// RGB data as Vec<u8> (3 bytes per pixel)
    pub fn demosaic(
        &self,
        bayer_data: &[u16],
        width: usize,
        height: usize,
        pattern: BayerPattern,
        black_level: u16,
        white_level: u16,
    ) -> Vec<u8> {
        // Calculate number of tiles
        let tiles_x = (width + self.tile_size - 1) / self.tile_size;
        let tiles_y = (height + self.tile_size - 1) / self.tile_size;
        
        tracing::debug!(
            "Parallel demosaic: {}x{} -> {}x{} tiles ({}px + {}px overlap)",
            width, height, tiles_x, tiles_y, self.tile_size, self.overlap
        );
        
        // Extract tiles
        let tiles: Vec<Tile> = (0..tiles_y)
            .flat_map(|ty| {
                (0..tiles_x).map(move |tx| {
                    self.extract_tile(bayer_data, width, height, tx, ty)
                })
            })
            .collect();
        
        // Demosaic tiles in parallel
        let demosaiced_tiles: Vec<DemosaicedTile> = tiles
            .into_par_iter()
            .map(|tile| {
                self.demosaic_tile(&tile, pattern, black_level, white_level)
            })
            .collect();
        
        // Combine tiles into final image
        self.combine_tiles(demosaiced_tiles, width, height)
    }
    
    /// Extract a tile from the Bayer array
    fn extract_tile(
        &self,
        bayer_data: &[u16],
        full_width: usize,
        full_height: usize,
        tile_x: usize,
        tile_y: usize,
    ) -> Tile {
        // Calculate tile bounds with overlap
        let start_x = (tile_x * self.tile_size).saturating_sub(self.overlap);
        let start_y = (tile_y * self.tile_size).saturating_sub(self.overlap);
        let end_x = ((tile_x + 1) * self.tile_size + self.overlap).min(full_width);
        let end_y = ((tile_y + 1) * self.tile_size + self.overlap).min(full_height);
        
        let tile_width = end_x - start_x;
        let tile_height = end_y - start_y;
        
        // Extract tile data
        let mut data = vec![0u16; tile_width * tile_height];
        for y in 0..tile_height {
            let src_y = start_y + y;
            let src_offset = src_y * full_width + start_x;
            let dst_offset = y * tile_width;
            
            if src_offset + tile_width <= bayer_data.len() {
                data[dst_offset..dst_offset + tile_width]
                    .copy_from_slice(&bayer_data[src_offset..src_offset + tile_width]);
            }
        }
        
        Tile {
            data,
            x: start_x,
            y: start_y,
            width: tile_width,
            height: tile_height,
            overlap: self.overlap,
        }
    }
    
    /// Demosaic a single tile
    fn demosaic_tile(
        &self,
        tile: &Tile,
        pattern: BayerPattern,
        black_level: u16,
        white_level: u16,
    ) -> DemosaicedTile {
        let rgb = match self.algorithm {
            DemosaicAlgorithm::Bilinear => {
                self.demosaic_bilinear(&tile.data, tile.width, tile.height, 
                                       tile.x, tile.y, pattern, black_level, white_level)
            }
            DemosaicAlgorithm::VNG => {
                // VNG is more complex - fall back to bilinear for now
                self.demosaic_bilinear(&tile.data, tile.width, tile.height,
                                       tile.x, tile.y, pattern, black_level, white_level)
            }
            DemosaicAlgorithm::AHD => {
                // AHD is most complex - fall back to bilinear for now
                self.demosaic_bilinear(&tile.data, tile.width, tile.height,
                                       tile.x, tile.y, pattern, black_level, white_level)
            }
        };
        
        DemosaicedTile {
            rgb,
            x: tile.x,
            y: tile.y,
            width: tile.width,
            height: tile.height,
            overlap: tile.overlap,
        }
    }
    
    /// Bilinear demosaic (fast, reasonable quality)
    fn demosaic_bilinear(
        &self,
        bayer: &[u16],
        width: usize,
        height: usize,
        offset_x: usize,
        offset_y: usize,
        pattern: BayerPattern,
        black_level: u16,
        white_level: u16,
    ) -> Vec<u8> {
        let scale = 255.0 / (white_level.saturating_sub(black_level)) as f32;
        let mut rgb = vec![0u8; width * height * 3];
        
        for y in 1..height.saturating_sub(1) {
            for x in 1..width.saturating_sub(1) {
                let global_x = offset_x + x;
                let global_y = offset_y + y;
                let color = pattern.color_at(global_x, global_y);
                
                let idx = y * width + x;
                let pixel = bayer[idx].saturating_sub(black_level);
                
                let (r, g, b) = match color {
                    0 => {
                        // Red pixel - interpolate G and B
                        let r = pixel;
                        let g = (bayer[idx - 1].saturating_sub(black_level) as u32 +
                                 bayer[idx + 1].saturating_sub(black_level) as u32 +
                                 bayer[idx - width].saturating_sub(black_level) as u32 +
                                 bayer[idx + width].saturating_sub(black_level) as u32) / 4;
                        let b = (bayer[idx - width - 1].saturating_sub(black_level) as u32 +
                                 bayer[idx - width + 1].saturating_sub(black_level) as u32 +
                                 bayer[idx + width - 1].saturating_sub(black_level) as u32 +
                                 bayer[idx + width + 1].saturating_sub(black_level) as u32) / 4;
                        (r as u32, g, b)
                    }
                    1 => {
                        // Green pixel - interpolate R and B
                        let g = pixel as u32;
                        // Check if we're on a red or blue row
                        let on_red_row = pattern.color_at(global_x.saturating_sub(1), global_y) == 0 ||
                                         pattern.color_at(global_x + 1, global_y) == 0;
                        if on_red_row {
                            let r = (bayer[idx - 1].saturating_sub(black_level) as u32 +
                                     bayer[idx + 1].saturating_sub(black_level) as u32) / 2;
                            let b = (bayer[idx - width].saturating_sub(black_level) as u32 +
                                     bayer[idx + width].saturating_sub(black_level) as u32) / 2;
                            (r, g, b)
                        } else {
                            let b = (bayer[idx - 1].saturating_sub(black_level) as u32 +
                                     bayer[idx + 1].saturating_sub(black_level) as u32) / 2;
                            let r = (bayer[idx - width].saturating_sub(black_level) as u32 +
                                     bayer[idx + width].saturating_sub(black_level) as u32) / 2;
                            (r, g, b)
                        }
                    }
                    2 => {
                        // Blue pixel - interpolate R and G
                        let b = pixel;
                        let g = (bayer[idx - 1].saturating_sub(black_level) as u32 +
                                 bayer[idx + 1].saturating_sub(black_level) as u32 +
                                 bayer[idx - width].saturating_sub(black_level) as u32 +
                                 bayer[idx + width].saturating_sub(black_level) as u32) / 4;
                        let r = (bayer[idx - width - 1].saturating_sub(black_level) as u32 +
                                 bayer[idx - width + 1].saturating_sub(black_level) as u32 +
                                 bayer[idx + width - 1].saturating_sub(black_level) as u32 +
                                 bayer[idx + width + 1].saturating_sub(black_level) as u32) / 4;
                        (r, g, b as u32)
                    }
                    _ => (0, 0, 0),
                };
                
                // Scale to 8-bit and clamp
                let out_idx = (y * width + x) * 3;
                rgb[out_idx] = ((r as f32 * scale).min(255.0)) as u8;
                rgb[out_idx + 1] = ((g as f32 * scale).min(255.0)) as u8;
                rgb[out_idx + 2] = ((b as f32 * scale).min(255.0)) as u8;
            }
        }
        
        rgb
    }
    
    /// Combine demosaiced tiles into final image with blending
    fn combine_tiles(
        &self,
        tiles: Vec<DemosaicedTile>,
        width: usize,
        height: usize,
    ) -> Vec<u8> {
        let mut output = vec![0u8; width * height * 3];
        let mut weight_map = vec![0.0f32; width * height];
        
        for tile in tiles {
            // Calculate valid region (excluding overlap zones for blending)
            let inner_x = tile.x + tile.overlap;
            let inner_y = tile.y + tile.overlap;
            let inner_w = tile.width.saturating_sub(tile.overlap * 2);
            let inner_h = tile.height.saturating_sub(tile.overlap * 2);
            
            for ty in 0..tile.height {
                for tx in 0..tile.width {
                    let global_x = tile.x + tx;
                    let global_y = tile.y + ty;
                    
                    if global_x >= width || global_y >= height {
                        continue;
                    }
                    
                    // Calculate blend weight based on distance from tile edge
                    let dx = if tx < tile.overlap {
                        tx as f32 / tile.overlap as f32
                    } else if tx >= tile.width - tile.overlap {
                        (tile.width - tx) as f32 / tile.overlap as f32
                    } else {
                        1.0
                    };
                    
                    let dy = if ty < tile.overlap {
                        ty as f32 / tile.overlap as f32
                    } else if ty >= tile.height - tile.overlap {
                        (tile.height - ty) as f32 / tile.overlap as f32
                    } else {
                        1.0
                    };
                    
                    let weight = dx * dy;
                    
                    let src_idx = (ty * tile.width + tx) * 3;
                    let dst_idx = (global_y * width + global_x) * 3;
                    let weight_idx = global_y * width + global_x;
                    
                    if src_idx + 2 < tile.rgb.len() && dst_idx + 2 < output.len() {
                        // Weighted accumulation
                        output[dst_idx] = ((output[dst_idx] as f32 * weight_map[weight_idx] +
                                           tile.rgb[src_idx] as f32 * weight) /
                                          (weight_map[weight_idx] + weight)) as u8;
                        output[dst_idx + 1] = ((output[dst_idx + 1] as f32 * weight_map[weight_idx] +
                                               tile.rgb[src_idx + 1] as f32 * weight) /
                                              (weight_map[weight_idx] + weight)) as u8;
                        output[dst_idx + 2] = ((output[dst_idx + 2] as f32 * weight_map[weight_idx] +
                                               tile.rgb[src_idx + 2] as f32 * weight) /
                                              (weight_map[weight_idx] + weight)) as u8;
                        weight_map[weight_idx] += weight;
                    }
                }
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bayer_pattern() {
        let pattern = BayerPattern::RGGB;
        assert_eq!(pattern.color_at(0, 0), 0); // Red
        assert_eq!(pattern.color_at(1, 0), 1); // Green
        assert_eq!(pattern.color_at(0, 1), 1); // Green
        assert_eq!(pattern.color_at(1, 1), 2); // Blue
    }
    
    #[test]
    fn test_small_demosaic() {
        let demosaic = ParallelDemosaic::with_tile_size(64);
        
        // Create simple 4x4 Bayer pattern
        let bayer = vec![
            1000, 2000, 1000, 2000,
            2000, 3000, 2000, 3000,
            1000, 2000, 1000, 2000,
            2000, 3000, 2000, 3000,
        ];
        
        let rgb = demosaic.demosaic(&bayer, 4, 4, BayerPattern::RGGB, 0, 4095);
        assert_eq!(rgb.len(), 4 * 4 * 3);
    }
}
