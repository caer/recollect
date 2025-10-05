use macroquad::audio::{PlaySoundParams, load_sound_from_bytes, play_sound};

pub mod engine;
pub mod game;

use game::audio::*;

fn main() {
    macroquad::Window::from_config(
        macroquad::prelude::Conf {
            window_title: "LDJam58".to_string(),
            high_dpi: true,
            sample_count: 2,
            fullscreen: false,
            ..Default::default()
        },
        game_loop(),
    );
}

async fn game_loop() {
    // Start background texture.
    let bg_track = load_sound_from_bytes(FOLEY_VINYL_TEXTURE).await.unwrap();
    play_sound(
        &bg_track,
        PlaySoundParams {
            looped: true,
            volume: 1.0,
        },
    );

    // Drum samples.
    #[rustfmt::skip]
    let mut track_1 = SoundTrack::new(SAMPLE_BASELINE, [
        1,0,0,0, 1,0,0,1, 1,0,0,0, 1,0,0,0, 
        1,0,0,0, 1,0,0,1, 1,0,0,0, 1,0,0,0
    ], TEMPO_BPM).await;

    #[rustfmt::skip]
    let mut track_2_lo = SoundTrack::new(SAMPLE_1_LO, [
        0,0,0,0, 1,0,0,0, 0,0,0,0, 0,0,0,0, 
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
    ], TEMPO_BPM).await;
    #[rustfmt::skip]
    let mut track_2_hi = SoundTrack::new(SAMPLE_1_HI, [
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 
        0,0,0,0, 1,0,0,0, 0,0,0,0, 0,0,0,0
    ], TEMPO_BPM).await;

    #[rustfmt::skip]
    let mut track_3_lo = SoundTrack::new(SAMPLE_2_LO, [
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 
        0,0,0,0, 0,0,0,0, 0,0,1,0, 0,0,0,0
    ], TEMPO_BPM).await;
    #[rustfmt::skip]
    let mut track_3_hi = SoundTrack::new(SAMPLE_2_HI, [
        0,0,0,0, 0,0,0,0, 0,0,1,0, 0,0,0,0, 
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
    ], TEMPO_BPM).await;

    // Pulse samples.
    #[rustfmt::skip]
    let mut track_4_lo = SoundTrack::new(SAMPLE_3_LO, [
        1,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
    ], TEMPO_BPM).await;
    #[rustfmt::skip]
    let mut track_5_hi = SoundTrack::new(SAMPLE_3_HI, [
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
        1,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
    ], TEMPO_BPM).await;

    loop {
        let frame_time = macroquad::prelude::get_frame_time();

        // Update sample tracks.
        track_1.update(frame_time);
        track_2_lo.update(frame_time);
        track_2_hi.update(frame_time);
        track_3_lo.update(frame_time);
        track_3_hi.update(frame_time);
        track_4_lo.update(frame_time);
        track_5_hi.update(frame_time);

        // Map ASD keys to track muting.
        if macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::W) {
            track_4_lo.toggle_mute();
            track_5_hi.toggle_mute();
        }
        if macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::A) {
            track_1.toggle_mute();
        }
        if macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::S) {
            track_2_lo.toggle_mute();
            track_2_hi.toggle_mute();
        }
        if macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::D) {
            track_3_lo.toggle_mute();
            track_3_hi.toggle_mute();
        }

        // Await next frame.
        macroquad::prelude::clear_background(macroquad::prelude::BLACK);
        macroquad::time::draw_fps();
        macroquad::prelude::next_frame().await
    }
}
