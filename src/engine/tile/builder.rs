//! TileMap builder utilities for loading tilemaps from bitmaps.

use image::{DynamicImage, GenericImageView, Rgba};

use super::{Tile, TileMap};

/// Result of processing a pixel from a bitmap during tilemap loading.
pub enum TileLoadResult {
    /// Create a tile at this position.
    Tile(Tile),
    /// Create a tile and mark this position as a spawn point.
    TileWithSpawn(Tile, f32, f32),
    /// Skip this pixel (no tile created).
    Skip,
}

/// Trait for mapping bitmap pixels to tiles.
///
/// This allows games to define their own color semantics without
/// hardcoding them into the engine.
pub trait ColorMapper {
    /// Maps a pixel color at position (x, y) to a tile load result.
    fn map_pixel(&mut self, x: u32, y: u32, color: Rgba<u8>) -> TileLoadResult;
}

impl TileMap {
    /// Loads tiles from a bitmap using a custom color mapper.
    ///
    /// This is the generic version that delegates color interpretation
    /// to the provided mapper, making the engine agnostic about color meanings.
    ///
    /// Returns the first spawn point found, if any.
    pub fn load_from_bitmap<M: ColorMapper>(
        &mut self,
        bitmap: &DynamicImage,
        layer: i8,
        mut color_mapper: M,
    ) -> Option<(f32, f32)> {
        let mut spawn_point = None;

        for (x, y, color) in bitmap.pixels() {
            match color_mapper.map_pixel(x, y, color) {
                TileLoadResult::Tile(tile) => {
                    self.set_tile(x as usize, y as usize, layer, tile);
                }
                TileLoadResult::TileWithSpawn(tile, spawn_x, spawn_y) => {
                    self.set_tile(x as usize, y as usize, layer, tile);
                    spawn_point = Some((spawn_x, spawn_y));
                }
                TileLoadResult::Skip => {
                    // Don't create a tile for this pixel
                }
            }
        }

        spawn_point
    }
}
