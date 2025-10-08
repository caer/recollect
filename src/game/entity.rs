use glam::Vec2;

use crate::{
    engine::tile::{TileMap, TileTexture},
    game::map,
};

// The player sprites.
pub const SPRITE_PLAYER: &[u8] = include_bytes!("../../assets/sprite.png");
pub const SPRITE_PLAYER_BACK: &[u8] = include_bytes!("../../assets/sprite.png");

/// Player movement velocity in grid units per second.
const PLAYER_VELOCITY: f32 = 20.0;

/// Player state.
pub struct Player {
    pub sprite: macroquad::texture::Texture2D,
    pub sprite_back: macroquad::texture::Texture2D,
    pub sprite_flipped: bool,
    pub position: Vec2,
}

impl Player {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            sprite: macroquad::texture::Texture2D::from_file_with_format(SPRITE_PLAYER, None),
            sprite_back: macroquad::texture::Texture2D::from_file_with_format(
                SPRITE_PLAYER_BACK,
                None,
            ),
            sprite_flipped: false,
            position: Vec2::ZERO,
        }
    }

    /// Updates the player position based on cursor and keyboard input.
    ///
    /// Does _not_ render the player sprite.
    pub fn translate(&mut self, frame_time: f32, map: &mut TileMap, wall_tile: &TileTexture) {
        let velocity = PLAYER_VELOCITY * frame_time;
        let last_pos = self.position;
        let mut target_pos = self.position;

        // If the mouse is held, move the sprite towards the cursor.
        if macroquad::prelude::is_mouse_button_down(miniquad::MouseButton::Left) {
            let mouse_pos = Vec2::from(macroquad::prelude::mouse_position());
            target_pos = map.view_to_grid(mouse_pos.x, mouse_pos.y, map::FOREGROUND_LAYER);

        // Otherwise, move the sprite "towards" any
        // held WASD keys, relative to screen-space.
        } else {
            if macroquad::prelude::is_key_down(miniquad::KeyCode::W) {
                target_pos.x -= 1.0;
                target_pos.y -= 1.0;
            } else if macroquad::prelude::is_key_down(miniquad::KeyCode::S) {
                target_pos.x += 1.0;
                target_pos.y += 1.0;
            }

            if macroquad::prelude::is_key_down(miniquad::KeyCode::A) {
                target_pos.x -= 1.0;
                target_pos.y += 1.0;
            } else if macroquad::prelude::is_key_down(miniquad::KeyCode::D) {
                target_pos.x += 1.0;
                target_pos.y -= 1.0;
            }
        }

        // Perform a linear interpolation if the sprite should move.
        let dx = target_pos.x - self.position.x;
        let dy = target_pos.y - self.position.y;
        let distance = (dx * dx + dy * dy).sqrt();

        // Only perform a move if the sprite would move
        // one unit or more. Without this check, the
        // sprite will appear to move erratically when
        // the target location is _very_ close to the sprite.
        if distance.abs() >= 1.0 {
            // Interpolate by a constant velocity so
            // that movement doesn't slow down when the
            // sprite is close to the target.
            let lerp_step = velocity / distance;
            self.position.x = self.position.x + (target_pos.x - self.position.x) * lerp_step;
            self.position.y = self.position.y + (target_pos.y - self.position.y) * lerp_step;

            // Show the back of the sprite during "upwards" motion.
            // TODO: swap sprite front/back

            // Flip the sprite during "rightwards" motion.
            self.sprite_flipped = self.position.x < target_pos.x;

            // Only permit moves which keep the player on the map.
            if self.position.x < 0.0 || self.position.x > (map::WIDTH - 1) as f32 {
                self.position.x = last_pos.x;
            }
            if self.position.y < 0.0 || self.position.y > (map::HEIGHT - 1) as f32 {
                self.position.y = last_pos.y;
            }

            // Check for wall collisions.
            let x = self.position.x as usize;
            let y = self.position.y as usize;
            if let Some(tile_state) = map.get_tile_state(x, y, map::FOREGROUND_LAYER)
                && tile_state.texture.as_ref() == Some(wall_tile)
            {
                // Try reverting X-axis.
                if let Some(tile_state) =
                    map.get_tile_state(last_pos.x as usize, y, map::FOREGROUND_LAYER)
                    && tile_state.texture.as_ref() != Some(wall_tile)
                {
                    self.position.x = last_pos.x;

                // Try reverting Y-axis.
                } else if let Some(tile_state) =
                    map.get_tile_state(x, last_pos.y as usize, map::FOREGROUND_LAYER)
                    && tile_state.texture.as_ref() != Some(wall_tile)
                {
                    self.position.y = last_pos.y;

                // Revert both axes.
                } else {
                    self.position = last_pos;
                }
            }
        }
    }
}
