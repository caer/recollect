use std::collections::BTreeSet;

use glam::Vec2;

use crate::engine::tile::{TileMap, TileTexture};

pub const TINY_MAX_PULSE_RADIUS: isize = (super::map::HEIGHT as f32 * 0.05) as isize;
pub const SMALL_MAX_PULSE_RADIUS: isize = (super::map::HEIGHT as f32 * 0.1) as isize;
pub const MEDIUM_MAX_PULSE_RADIUS: isize = (super::map::HEIGHT as f32 * 0.2) as isize;
pub const LARGE_MAX_PULSE_RADIUS: isize = (super::map::HEIGHT as f32 * 0.3) as isize;

pub struct Pulse {
    pub origin: Vec2,
    pub radius: i32,
    pub max_radius: f32,
    pub timestamp: f64,
    affected_tiles: BTreeSet<(usize, usize)>,
}

impl Pulse {
    pub fn new(origin: Vec2, max_radius: f32) -> Self {
        // Round origin to tile coordinates.
        let origin = Vec2::new(origin.x.round(), origin.y.round());

        Self {
            origin,
            radius: 0,
            max_radius,
            timestamp: 0.0,
            affected_tiles: BTreeSet::new(),
        }
    }

    /// Returns true iff the pulse will update again.
    pub fn update(
        &mut self,
        current_time: f64,
        map: &mut TileMap,
        wall_texture: &TileTexture,
    ) -> bool {
        let interval = 2.0 / self.max_radius as f64;
        if current_time > self.timestamp + interval {
            self.timestamp = current_time;
            self.radius += 1;
            if self.radius as f32 > self.max_radius {
                return false;
            }
        }

        // Find all tiles within the wavefront.
        self.affected_tiles.clear();
        map.tiles_on_radius_with_thickness(
            &mut self.affected_tiles,
            self.origin.x as isize,
            self.origin.y as isize,
            self.radius as isize,
            5,
        );

        // Subtract occluded tiles from the wavefront.
        self.affected_tiles.retain(|(x, y)| {
            // TODO: Cheesy ray-tracing for occlusion on the wavefront.
            let mut occluded = false;
            let line_points =
                map.tiles_on_line_between(self.origin.x, self.origin.y, *x as f32, *y as f32);
            for (x, y) in line_points.iter().take(line_points.len() - 1).skip(1) {
                if let Some(tile) = map.get_tile_state(*x, *y, super::map::FOREGROUND_LAYER)
                    && tile.texture.as_ref() == Some(wall_texture)
                {
                    occluded = true;
                    break;
                }
            }

            !occluded
        });

        true
    }

    /// Returns true if this pulse affects the specified tile.
    pub fn affects_tile(&self, x: usize, y: usize) -> bool {
        self.affected_tiles.contains(&(x, y))
    }
}
