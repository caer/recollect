use glam::Vec2;
use macroquad::texture::{DrawTextureParams, Texture2D};

use crate::engine::tile::Color;

pub const DEFAULT_TRANSITION_DURATION: f32 = 0.75;

pub struct TransitionOverlay {
    color: Color,
    image: Option<macroquad::prelude::Texture2D>,
    out_duration: f32,
    hold_duration: f32,
    in_duration: f32,
    elapsed_time: f32,
}

impl TransitionOverlay {
    pub fn new(out_duration: f32, hold_duration: f32, in_duration: f32) -> Self {
        Self {
            color: super::map::BACKGROUND,
            image: None,
            out_duration,
            hold_duration,
            in_duration,
            elapsed_time: 0.0,
        }
    }

    pub fn with_image(mut self, image: &[u8]) -> Self {
        self.image = Some(Texture2D::from_file_with_format(image, None));
        self
    }

    pub fn update(&mut self, delta_time: f32) -> TransitionState {
        self.elapsed_time += delta_time;

        let remaining_time =
            self.out_duration + self.hold_duration + self.in_duration - self.elapsed_time;

        // Transition complete.
        let (opacity, state) = if remaining_time <= 0.0 {
            return TransitionState::Complete;

        // Fading out.
        } else if self.elapsed_time <= self.out_duration {
            let remaining_time = self.out_duration - self.elapsed_time;
            let opacity = 1.0 - (remaining_time / self.out_duration);
            (opacity, TransitionState::FadeIn)

        // Holding full opacity.
        } else if self.elapsed_time <= self.out_duration + self.hold_duration {
            (1.0, TransitionState::Hold)

        // Fading in.
        } else {
            let opacity = remaining_time / self.in_duration;
            (opacity, TransitionState::FadeOut)
        };

        let screen_width = macroquad::prelude::screen_width();
        let screen_height = macroquad::prelude::screen_height();
        let mut color = self.color;
        color.alpha = (opacity * 255.) as u8;
        macroquad::prelude::draw_rectangle(
            0.0,
            0.0,
            screen_width,
            screen_height,
            super::as_macroquad_color(color),
        );

        if let Some(image) = &self.image {
            // Resize texture, preserving aspect ratio.
            let screen_width = macroquad::prelude::screen_width();
            let screen_height = macroquad::prelude::screen_height();
            let ratio_width = image.width() / screen_width;
            let ratio_height = image.height() / screen_height;

            let (width, height) = if ratio_width > ratio_height {
                (screen_width, (image.height() / ratio_width).round())
            } else {
                ((image.width() / ratio_height).round(), screen_height)
            };

            let draw_params = DrawTextureParams {
                dest_size: Some(Vec2::new(width, height)),
                ..Default::default()
            };

            // Center texture on screen.
            let x = (screen_width - width) / 2.0;
            let y = (screen_height - height) / 2.0;

            // Draw the splash screen.
            let mut blend_color = macroquad::prelude::WHITE;
            blend_color.a = opacity;
            macroquad::prelude::draw_texture_ex(image, x, y, blend_color, draw_params);
        }

        state
    }
}

#[derive(PartialEq, Debug)]
pub enum TransitionState {
    FadeOut,
    Hold,
    FadeIn,
    Complete,
}
