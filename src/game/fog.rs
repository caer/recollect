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
}

impl Pulse {
    pub fn new(origin: Vec2, max_radius: f32) -> Self {
        Self {
            origin,
            radius: 0,
            max_radius,
            timestamp: 0.0,
        }
    }

    /// Returns true iff the pulse will update again.
    pub fn update(&mut self, current_time: f64) -> bool {
        let interval = 2.0 / self.max_radius as f64;
        if current_time > self.timestamp + interval {
            self.timestamp = current_time;
            self.radius += 1;
            if self.radius as f32 > self.max_radius {
                return false;
            }
        }

        true
    }

    /// Returns the set of tiles affected by this pulse at its current radius.
    pub fn affected_tiles(
        &self,
        map: &mut TileMap,
        wall_texture: &TileTexture,
    ) -> Vec<(usize, usize)> {
        let mut pulse_tiles = Vec::new();

        for i in 1..5 {
            let mut radius_tiles = map.tiles_on_radius(
                self.origin.x as isize,
                self.origin.y as isize,
                self.radius as isize - i,
            );

            radius_tiles.retain(|(x, y)| {
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

            pulse_tiles.extend(radius_tiles);
        }

        pulse_tiles
    }
}
