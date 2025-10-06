//! Level loading utilities for the game.

use glam::Vec2;
use image::{DynamicImage, Rgba};

use crate::engine::tile::{Color, ColorMapper, Tile, TileLoadResult, TileTexture};

// Map size in grid units.
pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 128;

// Map draw layers.
pub const FOREGROUND_LAYER: i8 = 0;
pub const BACKGROUND_LAYER: i8 = -1;

/// "Default" color, typically only meaningful for blend masks.
pub const DEFAULT: Color = Color::new(255, 255, 255, 255);

/// Primary accent, used for unreached objectives.
pub const ACCENT_1: Color = Color::new(143, 182, 87, 255);

/// Secondary accent, used for reached objectives.
pub const ACCENT_2: Color = Color::new(151, 171, 212, 255);

/// Tertiary accent, used for player spawn points.
pub const ACCENT_3: Color = Color::new(239, 146, 117, 255);

/// Fog of war color.
pub const FOG_OF_WAR: Color = Color::new(0, 0, 0, 156);

/// Background.
pub const BACKGROUND: Color = Color::new(19, 21, 16, 255);

/// Tile map images.
pub const TILEMAPS: &[&[u8]] = &[
    include_bytes!("../../assets/map-0.png"),
    include_bytes!("../../assets/map-1.png"),
    include_bytes!("../../assets/map-2.png"),
    include_bytes!("../../assets/map-3.png"),
    include_bytes!("../../assets/map-4.png"),
];

// Tile assets
pub const TILE_BACKGROUND: &[u8] = include_bytes!("../../assets/tile-background.png");
pub const TILE_FLOOR: &[u8] = include_bytes!("../../assets/tile-floor.png");
pub const TILE_WALL: &[u8] = include_bytes!("../../assets/tile-wall.png");

/// Game map state.
pub struct GameMap {
    pub wall_texture: TileTexture,
    pub floor_texture: TileTexture,
    pub map: crate::engine::tile::TileMap,
    pub objectives_remaining: usize,
}

impl GameMap {
    /// Create a new, empty game map.
    pub fn new(wall_texture: TileTexture, floor_texture: TileTexture) -> Self {
        let mut map = crate::engine::tile::TileMap::new(WIDTH, HEIGHT, BACKGROUND, DEFAULT);
        map.draw_debug_info = false;
        map.viewport_scale = 6.0;

        Self {
            wall_texture,
            floor_texture,
            map,
            objectives_remaining: 0,
        }
    }
    /// Load the game map from the specified tilemap index.
    ///
    /// Returns the player spawn position.
    pub fn load_map(&mut self, bitmap: &DynamicImage) -> Vec2 {
        // FIXME: This is a bit hacky, but it works for now.
        // We recreate the tile map from scratch to clear out any old state.
        self.map = crate::engine::tile::TileMap::new(WIDTH, HEIGHT, BACKGROUND, DEFAULT);
        self.map.draw_debug_info = false;
        self.map.viewport_scale = 6.0;

        let spawn_point = self
            .map
            .load_from_bitmap(
                bitmap,
                FOREGROUND_LAYER,
                LayeredColorMapper {
                    wall_texture: self.wall_texture.clone(),
                    floor_texture: self.floor_texture.clone(),
                    floor_opacity: 0.75,
                },
            )
            .unwrap();

        // Set all tiles' heights to be very low so that they rise up on game load.
        // Also count the total number of objective (ACCENT_1) tiles that are present.
        self.objectives_remaining = 0;
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if let Some(tile_state) = self.map.get_tile_state(x, y, FOREGROUND_LAYER) {
                    tile_state.height_offset = -100.0;
                    tile_state.target_height_offset = 0.0;

                    if tile_state.original_blend_color == ACCENT_1 {
                        self.objectives_remaining += 1;
                    }
                }
            }
        }

        spawn_point.into()
    }

    pub fn update(&mut self, delta_time: f32) {
        self.map.update(delta_time);
    }
}

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
