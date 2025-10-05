//! Level loading utilities for the game.

use image::Rgba;

use crate::engine::tile::{Color, Tile, TileLoadResult, TileTexture};

// Map size in grid units.
pub const WIDTH: usize = 48;
pub const HEIGHT: usize = 48;

// Map draw layers.
pub const FOREGROUND_LAYER: i8 = 0;
pub const BACKGROUND_LAYER: i8 = -1;

/// "Default" color, typically only meaningful for blend masks.
pub const DEFAULT: Color = Color::new(255, 255, 255, 255);

/// Primary accent.
pub const ACCENT_1: Color = Color::new(228, 140, 53, 255);

/// Secondary accent.
pub const ACCENT_2: Color = Color::new(81, 156, 160, 255);

/// Tertiary accent.
pub const ACCENT_3: Color = Color::new(204, 116, 167, 255);

/// Background.
pub const BACKGROUND: Color = Color::new(38, 38, 34, 255);

/// Game-specific color mapper for loading levels from bitmaps.
///
/// This defines the color semantics for the Layered game:
/// - BACKGROUND color = walls
/// - ACCENT_1 = objectives (orange)
/// - ACCENT_2 = dangers/threats (teal)
/// - ACCENT_3 = avatar spawn point (pink)
/// - Any other color = floor
pub struct LayeredColorMapper {
    wall_texture: TileTexture,
    floor_texture: TileTexture,
    floor_opacity: f32,
}

impl LayeredColorMapper {
    /// Creates a new color mapper for the Layered game.
    pub fn new(wall_texture: TileTexture, floor_texture: TileTexture, floor_opacity: f32) -> Self {
        Self {
            wall_texture,
            floor_texture,
            floor_opacity,
        }
    }

    /// Maps a bitmap pixel to a tile based on Layered's color semantics.
    pub fn map_pixel(&mut self, x: u32, y: u32, color: Rgba<u8>) -> TileLoadResult {
        let wall_color: [u8; 4] = BACKGROUND.into();
        let objective_color: [u8; 4] = ACCENT_1.into();
        let danger_color: [u8; 4] = ACCENT_2.into();
        let avatar_color: [u8; 4] = ACCENT_3.into();

        // Check if this is a wall tile
        if color.0 == wall_color {
            return TileLoadResult::Tile(Tile::Filled {
                texture: self.wall_texture.clone(),
                height_offset: None,
                blend_color: None,
            });
        }

        // Determine the blend color for floor tiles
        let blend_color = if color.0 == objective_color {
            Some(ACCENT_1)
        } else if color.0 == danger_color {
            Some(ACCENT_2)
        } else if color.0 == avatar_color {
            Some(ACCENT_3)
        } else {
            let mut color = DEFAULT;
            color.alpha = (self.floor_opacity * 255.) as u8;
            Some(color)
        };

        let tile = Tile::Filled {
            texture: self.floor_texture.clone(),
            height_offset: None,
            blend_color,
        };

        // Check if this is an avatar spawn point
        if color.0 == avatar_color {
            TileLoadResult::TileWithSpawn(tile, x as f32, y as f32)
        } else {
            TileLoadResult::Tile(tile)
        }
    }
}
