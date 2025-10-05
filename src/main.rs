pub mod engine;
pub mod game;

fn main() {
    macroquad::Window::from_config(
        macroquad::prelude::Conf {
            window_title: "LDJam58".to_string(),
            high_dpi: true,
            sample_count: 2,
            fullscreen: false,
            ..Default::default()
        },
        game::game_loop(),
    );
}
