use glam::Vec2;

use crate::engine::tile::TileMap;

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
    pub fn affected_tiles(&self, map: &TileMap) -> Vec<(usize, usize)> {
        let mut pulse_tiles = map.tiles_on_radius(
            self.origin.x as isize,
            self.origin.y as isize,
            3 + self.radius as isize,
        );
        pulse_tiles.extend(map.tiles_on_radius(
            self.origin.x as isize,
            self.origin.y as isize,
            2 + self.radius as isize,
        ));
        pulse_tiles.extend(map.tiles_on_radius(
            self.origin.x as isize,
            self.origin.y as isize,
            1 + self.radius as isize,
        ));

        pulse_tiles
    }
}
