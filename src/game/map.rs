//! Level loading utilities for the game.

use image::Rgba;

use crate::engine::tile::{Color, ColorMapper, Tile, TileLoadResult, TileTexture};

// Map size in grid units.
pub const WIDTH: usize = 48;
pub const HEIGHT: usize = 48;

// Map draw layers.
pub const FOREGROUND_LAYER: i8 = 0;
pub const BACKGROUND_LAYER: i8 = -1;

/// "Default" color, typically only meaningful for blend masks.
pub const DEFAULT: Color = Color::new(255, 255, 255, 255);

/// Primary accent.
pub const ACCENT_1: Color = Color::new(143, 182, 87, 255);

/// Secondary accent.
pub const ACCENT_2: Color = Color::new(151, 171, 212, 255);

/// Tertiary accent.
pub const ACCENT_3: Color = Color::new(239, 146, 117, 255);

/// Background.
pub const BACKGROUND: Color = Color::new(19, 21, 16, 255);

/// Tile map images.
pub const TILEMAPS: &[&[u8]] = &[include_bytes!("../../assets/map-1.png")];

// Tile assets
pub const TILE_BACKGROUND: &[u8] = include_bytes!("../../assets/tile-background.png");
pub const TILE_FLOOR: &[u8] = include_bytes!("../../assets/tile-floor.png");
pub const TILE_WALL: &[u8] = include_bytes!("../../assets/tile-wall.png");

/// Game-specific color mapper for loading levels from bitmaps.
///
/// This defines the color semantics for the Layered game:
/// - BACKGROUND color = walls
/// - ACCENT_1 = objectives (orange)
/// - ACCENT_2 = dangers/threats (teal)
/// - ACCENT_3 = avatar spawn point (pink)
/// - Any other color = floor
pub struct LayeredColorMapper {
    pub wall_texture: TileTexture,
    pub floor_texture: TileTexture,
    pub floor_opacity: f32,
}

impl ColorMapper for LayeredColorMapper {
    fn map_pixel(&mut self, x: u32, y: u32, color: Rgba<u8>) -> TileLoadResult {
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
