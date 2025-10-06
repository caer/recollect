//! Tile-based, 2.5D dimetric grid system.
use std::collections::BTreeMap;

use glam::{FloatExt, Mat2, Vec2};
use macroquad::{
    color::{GRAY, WHITE},
    texture::{DrawTextureParams, FilterMode, Texture2D},
};
use miniquad::MipmapFilterMode;
use palette::{Srgb, WithAlpha};

pub mod builder;
pub use builder::{ColorMapper, TileLoadResult};

/// Type used for in-memory colors across the crate.
pub type Color = palette::rgb::Rgba<Srgb, u8>;

/// Converts a [`palette`] color to
/// a [`macroquad::color::Color`].
pub const fn as_macroquad_color(color: Color) -> macroquad::color::Color {
    macroquad::color::Color::new(
        color.color.red as f32 / 255.0,
        color.color.green as f32 / 255.0,
        color.color.blue as f32 / 255.0,
        color.alpha as f32 / 255.0,
    )
}

// When tightly packed, tiles in dimetric
// projections are, visually, twice as wide
// and half as tall.
const ISO_X_COEFF: f32 = 0.5;
const ISO_Y_COEFF: f32 = 0.25;

// I/J coefficients for converting between isometric
// and orthographic projections.
const I_HAT: Vec2 = Vec2::new(ISO_X_COEFF, ISO_Y_COEFF);
const J_HAT: Vec2 = Vec2::new(-ISO_X_COEFF, ISO_Y_COEFF);

/// 2D texture assigned to a [`Tile`].
#[derive(PartialEq, Clone)]
pub struct TileTexture {
    texture: Texture2D,
}

impl TileTexture {
    /// Loads a texture from `bytes`.
    ///
    /// The format of the image in `bytes` will
    /// be auto-detected so long as it is one of
    /// [ImageFormat][image::ImageFormat].
    pub fn from_bytes(bytes: &[u8]) -> Self {
        // Load the bytes as an in-memory image.
        let texture_rgba8 = image::load_from_memory(bytes).unwrap().to_rgba8();
        let width = texture_rgba8.width() as u16;
        let height = texture_rgba8.height() as u16;
        let bytes = texture_rgba8.into_raw();

        // Get a texture ID from miniquad.
        let texture_id = unsafe {
            let context = macroquad::window::get_internal_gl();
            let render_context = context.quad_context;

            // Load the texture into the miniquad context.
            let texture_id = render_context.new_texture_from_rgba8(width, height, bytes.as_slice());

            // Configure the texture's filtering.
            render_context.texture_set_filter(
                texture_id,
                FilterMode::Linear,
                MipmapFilterMode::None,
            );

            texture_id
        };

        // Load the miniquad texture into macroquad.
        let texture = Texture2D::from_miniquad_texture(texture_id);

        Self { texture }
    }

    /// Draws the texture.
    pub fn draw(&self, x: f32, y: f32, size: Vec2, blend_color: Color) {
        let draw_params = DrawTextureParams {
            dest_size: Some(size),
            ..Default::default()
        };

        macroquad::prelude::draw_texture_ex(
            &self.texture,
            x,
            y,
            as_macroquad_color(blend_color),
            draw_params,
        );
    }
}

/// A tile in a [`TileMap`].
pub enum Tile {
    /// A filled tile which may be rendered.
    Filled {
        texture: TileTexture,

        /// Vertical offset of the tile relative
        /// to its height.
        height_offset: Option<f32>,

        /// Color to blend the tile's texture
        /// with during drawing.
        blend_color: Option<Color>,
    },

    /// An empty tile which won't be rendered.
    Empty,
}

#[derive(Default)]
pub struct TileState {
    pub texture: Option<TileTexture>,
    pub height_offset: f32,
    pub target_height_offset: f32,

    /// TODO: Rename to more accurately reflect this is the "main"
    /// blend color a tile should revert to.
    pub original_blend_color: Color,

    /// TODO: Rename to more accurately reflect this is the "current"
    /// blend color a tile is at.
    pub blend_color: Color,
    pub target_blend_color: Color,
}

/// 2D grid that renders as an axonometric map of tiles.
pub struct TileMap {
    /// Maximum grid X-value, in units.
    width: usize,

    /// Maximum grid Y-value, in units.
    height: usize,

    /// Total number of tiles per grid layer.
    tiles_per_layer: usize,

    /// Tile layers, indexed by layer number.
    ///
    /// Each layer contains a dense vector of
    /// [`Tile`]s of a length equal to [`Self::tiles_per_layer`]
    layers: BTreeMap<i8, Vec<(Tile, TileState)>>,

    /// Background color for the map.
    color_bg: Color,

    /// Default color for tiles without a blend color.
    color_default: Color,

    /// True if debugging info should be drawn.
    pub draw_debug_info: bool,

    /// Viewport scaling modifier ("camera zoom").
    pub viewport_scale: f32,

    /// Viewport position offset ("camera pan").
    pub viewport_offset: Vec2,
}

impl TileMap {
    /// Returns a new map with `width` x `height` tiles.
    pub fn new(width: usize, height: usize, color_bg: Color, color_default: Color) -> Self {
        let mut map = Self {
            width,
            height,
            tiles_per_layer: width * height,
            draw_debug_info: true,
            viewport_scale: 1.0f32,
            viewport_offset: Vec2::default(),
            layers: Default::default(),
            color_bg,
            color_default,
        };

        let view_size = map.calculate_view_size();
        map.viewport_offset.y -= (view_size.y / height as f32) * 3.0;

        map
    }

    /// Updates all tile states.
    pub fn update(&mut self, frame_time: f32) {
        let interp_speed = 8.0;
        let interp_factor = frame_time * interp_speed;

        for layer in self.layers.values_mut() {
            for (.., state) in layer.iter_mut() {
                // Interpolate height offset.
                state.height_offset = state
                    .height_offset
                    .lerp(state.target_height_offset, interp_factor);

                // Interpolate alpha separately to avoid color shifts.
                let alpha_f = state.blend_color.alpha as f32 / 255.0;
                let target_alpha_f = state.target_blend_color.alpha as f32 / 255.0;
                state.blend_color.alpha =
                    (alpha_f.lerp(target_alpha_f, interp_factor) * 255.0) as u8;

                // Interpolate RGB channels.
                let red_f = state.blend_color.red as f32 / 255.0;
                let target_red_f = state.target_blend_color.red as f32 / 255.0;
                let green_f = state.blend_color.green as f32 / 255.0;
                let target_green_f = state.target_blend_color.green as f32 / 255.0;
                let blue_f = state.blend_color.blue as f32 / 255.0;
                let target_blue_f = state.target_blend_color.blue as f32 / 255.0;
                state.blend_color.red = (red_f.lerp(target_red_f, interp_factor) * 255.0) as u8;
                state.blend_color.green =
                    (green_f.lerp(target_green_f, interp_factor) * 255.0) as u8;
                state.blend_color.blue = (blue_f.lerp(target_blue_f, interp_factor) * 255.0) as u8;
            }
        }
    }

    /// Draws one frame of the map's tiles.
    pub fn draw_tiles(&mut self) {
        // Reset frame.
        macroquad::window::clear_background(as_macroquad_color(self.color_bg));

        // Recalculate current viewport and tile sizes.
        let tile_size = self.calculate_tile_size();

        // Draw tiles.
        for (layer_height, layer) in &self.layers {
            for (i, (tile, tile_state)) in layer.iter().enumerate().take(self.tiles_per_layer) {
                // Convert tile index into logical x/y coordinates.
                let x = i / self.height;
                let y = i % self.height;

                // Draw any filled tiles.
                if let Tile::Filled { texture, .. } = tile {
                    let view_point = self.grid_to_view(x as f32, y as f32, *layer_height);

                    // Apply tile states.
                    let height_offset = tile_state.height_offset;
                    let blend_color = &tile_state.blend_color;

                    // Offset by any manual offsets specified for the tile.
                    let height_offset = -(tile_size.y * height_offset);

                    // Draw the tile.
                    texture.draw(
                        view_point.x,
                        view_point.y + height_offset,
                        tile_size,
                        *blend_color,
                    );
                }
            }
        }

        // Skip drawing debug info if not enabled.
        if !self.draw_debug_info {
            return;
        }

        // Draw viewport debugging info.
        let fps = macroquad::prelude::get_fps();
        macroquad::prelude::draw_text(&format!("{fps:03.0} FPS",), 10., 20., 20., GRAY);
        macroquad::prelude::draw_text(
            &format!(
                "Origin {:.0} @ {:.2} Scale",
                self.viewport_offset, self.viewport_scale
            ),
            10.,
            40.,
            20.,
            GRAY,
        );

        // Identify the highest layer containing a tile underneath the cursor.
        let (mut mouse_x, mut mouse_y) = macroquad::prelude::mouse_position();
        mouse_x = mouse_x.round();
        mouse_y = mouse_y.round();
        let mut max_layer = None;
        let mut cursor_point = None;
        for layer in self.layers.keys() {
            let candidate_point = self.view_to_grid(mouse_x, mouse_y, *layer).round();

            if candidate_point.x < 0.0 || candidate_point.y < 0.0 {
                continue;
            }

            let x = candidate_point.x as usize;
            let y = candidate_point.y as usize;

            if x >= self.width || y >= self.height {
                continue;
            }

            match self.layers.get(layer).map(|l| &l[y + self.height * x]) {
                Some((Tile::Filled { .. }, ..)) => {
                    max_layer = Some(*layer);
                    cursor_point = Some(candidate_point);
                }
                _ => continue,
            }
        }

        // Draw cursor debug info.
        if let Some(cursor_point) = cursor_point {
            let max_layer = max_layer.unwrap();
            let index = cursor_point.y + self.height as f32 * cursor_point.x;

            macroquad::prelude::draw_text(
                &format!(
                    "Tile {cursor_point} (Layer {max_layer}, Index {index:.0}) @ Pixel [{mouse_x:.0}, {mouse_y:.0}]",
                ),
                10.,
                60.,
                20.,
                GRAY,
            );
        } else {
            macroquad::prelude::draw_text(
                &format!("No Tile @ Pixel [{mouse_x:.0}, {mouse_y:.0}]",),
                10.,
                60.,
                20.,
                GRAY,
            );
        }
    }

    /// Draws a sprite onto the map's tile space.
    pub fn draw_sprite(
        &mut self,
        sprite: &Texture2D,
        x: f32,
        y: f32,
        z: f32,
        layer: i8,
        flip_x: bool,
    ) {
        // Convert grid point to isometric space.
        let iso_pixel = self.grid_to_view(x, y, layer);

        let tile_size = self.calculate_tile_size();
        let draw_params = DrawTextureParams {
            dest_size: Some(tile_size),
            source: None,
            flip_x,
            ..Default::default()
        };

        macroquad::prelude::draw_texture_ex(
            sprite,
            iso_pixel.x,
            iso_pixel.y + -(tile_size.y * z),
            WHITE,
            draw_params,
        );
    }

    /// Sets the `tile` at logical coordinate `x, y` in `layer`.
    pub fn set_tile(&mut self, x: usize, y: usize, layer: i8, tile: Tile) {
        let layer = self.layers.entry(layer).or_insert_with(|| {
            // Initialize layers with all-empty tiles.
            let mut tiles = Vec::with_capacity(self.tiles_per_layer);
            for _ in 0..self.tiles_per_layer {
                tiles.push((Tile::Empty, TileState::default()));
            }
            tiles
        });

        // Clone the tile into a new tile state.
        let tile_state = match &tile {
            Tile::Filled {
                texture,
                height_offset,
                blend_color,
            } => TileState {
                texture: Some(texture.clone()),
                height_offset: height_offset.unwrap_or(0.0),
                target_height_offset: height_offset.unwrap_or(0.0),
                original_blend_color: *blend_color.as_ref().unwrap_or(&self.color_default),
                blend_color: *blend_color.as_ref().unwrap_or(&self.color_default),
                target_blend_color: *blend_color.as_ref().unwrap_or(&self.color_default),
            },
            Tile::Empty => TileState::default(),
        };

        // Convert the X/Y coordinate to contiguous vector coordinates.
        let index = y + self.height * x;
        layer[index] = (tile, tile_state);
    }

    /// Gets the state of the `tile` at logical coordinate `x, y` in `layer`.
    pub fn get_tile_state(&mut self, x: usize, y: usize, layer: i8) -> Option<&mut TileState> {
        let layer = self.layers.get_mut(&layer)?;

        // Convert the X/Y coordinate to contiguous vector coordinates.
        let index = y + self.height * x;
        Some(&mut layer[index].1)
    }

    /// Calculates the current active view size.
    pub fn calculate_view_size(&self) -> Vec2 {
        Vec2::new(
            macroquad::prelude::screen_width(),
            macroquad::prelude::screen_height(),
        )
    }

    /// Calculates the actual tile size in view
    /// coordinates for a given view of this grid.
    pub fn calculate_tile_size(&self) -> Vec2 {
        let view_size = self.calculate_view_size();

        let mut tile_size = Vec2::new(
            view_size.x / self.width as f32,
            view_size.y / self.height as f32,
        );

        // Preserve grid aspect ratio.
        if tile_size.y < tile_size.x {
            tile_size.y = tile_size.x;
        } else {
            tile_size.x = tile_size.y;
        }

        tile_size * self.viewport_scale
    }

    /// Returns the transformation matrix for converting
    /// planar grid units into physical pixel view points
    /// within an axonometric projection.
    fn unit_to_pixel_transform(&self) -> Mat2 {
        let tile_size = self.calculate_tile_size();
        let i = tile_size * I_HAT;
        let j = tile_size * J_HAT;
        glam::mat2(i, j)
    }

    /// Converts a planar grid point to a view point
    /// (in physical pixels) within an axonometric projection.
    pub fn grid_to_view(&self, x: f32, y: f32, layer: i8) -> Vec2 {
        let view_size = self.calculate_view_size();
        let tile_size = self.calculate_tile_size();

        // Transform the grid point into a view point.
        let mut point = self.unit_to_pixel_transform().mul_vec2((x, y).into());

        // Shift points left by half a tile, causing the
        // center of tiles at grid point `X == 0`to be
        // visually centered on view point `X == 0`.
        point.x -= tile_size.x * 0.5;

        // Shift points down and to the right by the
        // total view size relative to the axonometric
        // scale factor, causing the center of the
        // projected tile map to be centered in the view.
        point.x += view_size.x * ISO_X_COEFF;
        point.y += view_size.y * ISO_Y_COEFF;

        // Offset points vertical position by the layer index.
        point.y -= tile_size.y * layer as f32;

        point + self.viewport_offset
    }

    /// Converts a view point (in physical pixels) within an
    /// axonometric projection to a planar grid point.
    pub fn view_to_grid(&self, x: f32, y: f32, layer: i8) -> Vec2 {
        let view_size = self.calculate_view_size();
        let tile_size = self.calculate_tile_size();

        // Undo the viewport offset.
        let mut x = x - self.viewport_offset.x;
        let mut y = y - self.viewport_offset.y;

        // Undo the axonometric scaling offset.
        x -= view_size.x * ISO_X_COEFF;
        y -= view_size.y * ISO_Y_COEFF;

        // Offset the point's vertical position by the
        // tile height relative to the axonometric scale
        // factor, causing a view point to be "on" a grid
        // when it is visibly over a tile's center.
        y -= tile_size.y * ISO_Y_COEFF;

        // Offset the point's vertical position by the layer index.
        y += tile_size.y * layer as f32;

        // Transform the adjusted view point into a grid point.
        self.unit_to_pixel_transform()
            .inverse()
            .mul_vec2((x, y).into())
    }

    /// TODO:
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// TODO:
    pub fn tile_has_original_color(&mut self, x: usize, y: usize, layer: i8, color: Color) -> bool {
        if let Some(tile_state) = self.get_tile_state(x, y, layer)
            && tile_state.original_blend_color.without_alpha() == color.without_alpha()
        {
            return true;
        }

        false
    }

    /// TODO: https://en.wikipedia.org/wiki/Flood_fill
    ///
    /// Returns the number of affected tiles.
    pub fn flood_fill_tiles_original_color(
        &mut self,
        x: usize,
        y: usize,
        layer: i8,
        old_blend: Color,
        new_blend: Color,
    ) -> usize {
        let mut affected_tiles = 0;

        if let Some(tile_state) = self.get_tile_state(x, y, layer)
            && tile_state.original_blend_color.without_alpha() == old_blend.without_alpha()
        {
            tile_state.original_blend_color = new_blend;
            affected_tiles += 1;
            affected_tiles +=
                self.flood_fill_tiles_original_color(x + 1, y, layer, old_blend, new_blend);
            affected_tiles +=
                self.flood_fill_tiles_original_color(x - 1, y, layer, old_blend, new_blend);
            affected_tiles +=
                self.flood_fill_tiles_original_color(x, y + 1, layer, old_blend, new_blend);
            affected_tiles +=
                self.flood_fill_tiles_original_color(x, y - 1, layer, old_blend, new_blend);
        }

        affected_tiles
    }

    /// TODO: https://medium.com/geekculture/bresenhams-line-drawing-algorithm-2e0e953901b3.
    pub fn tiles_on_line_between(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> Vec<(usize, usize)> {
        let ax = x1 as isize;
        let ay = y1 as isize;
        let bx = x2 as isize;
        let by = y2 as isize;

        let dx = bx - ax;
        let dy = by - ay;
        let abs_dx = dx.abs();
        let abs_dy = dy.abs();

        let mut x = ax;
        let mut y = ay;

        let mut line_points = vec![(x as usize, y as usize)];

        let inc_x = |x| {
            if dx < 0 { x - 1 } else { x + 1 }
        };

        let inc_y = |y| {
            if dy < 0 { y - 1 } else { y + 1 }
        };

        // Small slope.
        if abs_dx > abs_dy {
            let mut d = 2 * abs_dy - abs_dx;

            for _ in 0..abs_dx {
                x = inc_x(x);

                if d < 0 {
                    d += 2 * abs_dy
                } else {
                    y = inc_y(y);

                    d += (2 * abs_dy) - (2 * abs_dx);
                }

                line_points.push((x as usize, y as usize));
            }

        // Large slope.
        } else {
            let mut d = 2 * abs_dx - abs_dy;

            for _ in 0..abs_dy {
                y = inc_y(y);

                if d < 0 {
                    d += 2 * abs_dx;
                } else {
                    x = inc_x(x);

                    d += (2 * abs_dx) - (2 * abs_dy);
                }

                line_points.push((x as usize, y as usize));
            }
        }

        line_points
    }

    /// TODO: https://en.wikipedia.org/wiki/Midpoint_circle_algorithm
    pub fn tiles_on_radius(
        &self,
        center_x: isize,
        center_y: isize,
        radius: isize,
    ) -> Vec<(usize, usize)> {
        let mut diameter = (5 - radius * 4) / 4;

        let mut radius_points = vec![];
        let mut x = 0;
        let mut y = radius;

        while x <= y {
            radius_points.push((center_x + x, center_y + y));
            radius_points.push((center_x + x, center_y - y));
            radius_points.push((center_x - x, center_y + y));
            radius_points.push((center_x - x, center_y - y));

            radius_points.push((center_x + y, center_y + x));
            radius_points.push((center_x + y, center_y - x));
            radius_points.push((center_x - y, center_y + x));
            radius_points.push((center_x - y, center_y - x));

            if diameter < 0 {
                diameter += 2 * x + 1;
            } else {
                diameter += 2 * (x - y) + 1;
                y -= 1;
            }

            x += 1;
        }

        radius_points
            .into_iter()
            .filter_map(|(x, y)| {
                if x >= 0 && y >= 0 && x < self.width as isize && y < self.height as isize {
                    Some((x as usize, y as usize))
                } else {
                    None
                }
            })
            .collect()
    }

    /// TODO: https://web.archive.org/web/20120422045142/https://banu.com/blog/7/drawing-circles/
    pub fn tiles_in_radius(
        &self,
        center_x: isize,
        center_y: isize,
        radius: isize,
    ) -> Vec<(usize, usize)> {
        let mut tiles = vec![];

        for x in (center_x - radius)..=(center_x + radius) {
            for y in (center_y - radius)..=(center_y + radius) {
                if x >= 0 && y >= 0 && x < self.width as isize && y < self.height as isize {
                    let dx = x - center_x;
                    let dy = y - center_y;

                    if dx * dx + dy * dy <= radius * radius {
                        tiles.push((x as usize, y as usize));
                    }
                }
            }
        }

        tiles
    }
}
