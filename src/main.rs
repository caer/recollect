use macroquad::audio::{PlaySoundParams, load_sound_from_bytes, play_sound};

// Foley samples.
pub const FOLEY_VINYL_TEXTURE: &[u8] =
    include_bytes!("../assets/Clark Audio - Texture Crackle Vinyl.wav");

// Track samples.
pub const SAMPLE_BASELINE: &[u8] = include_bytes!("../assets/bfxr - decay4.wav");
pub const SAMPLE_1_LO: &[u8] = include_bytes!("../assets/Helton Yan - Major Distant Piano Lo.wav");
pub const SAMPLE_1_HI: &[u8] = include_bytes!("../assets/Helton Yan - Major Distant Piano Hi.wav");
pub const SAMPLE_2_LO: &[u8] = include_bytes!("../assets/Helton Yan - Major Lone Beep Lo.wav");
pub const SAMPLE_2_HI: &[u8] = include_bytes!("../assets/Helton Yan - Major Lone Beep Hi.wav");
pub const SAMPLE_PULSE_LO: &[u8] = include_bytes!("../assets/Helton Yan - Pulse Low.wav");
pub const SAMPLE_PULSE_HI: &[u8] = include_bytes!("../assets/Helton Yan - Pulse High.wav");

/// The tempo of the game tracks, in beats per minute.
///
/// Lo-Fi beats tend to be around 60-90 BPM.
const TEMPO_BPM: f32 = 64.0;

fn main() {
    macroquad::Window::from_config(
        macroquad::prelude::Conf {
            window_title: "LDJam58".to_string(),
            fullscreen: false,
            window_width: 800,
            window_height: 600,
            ..Default::default()
        },
        game_loop(),
    );
}

struct SoundTrack {
    /// The sound to play at each enabled step.
    sound: macroquad::audio::Sound,

    /// Each entry indicates if the sound should
    /// play at a given track step, where
    /// `0` means _no_, and `1` means _yes_.
    steps: [u8; 32],

    /// The time interval between each step, in seconds.
    interval_secs: f32,
    interval_accumulator: f32,
    interval_step: usize,

    /// True if the track is currently playing.
    muted: bool,
}

impl SoundTrack {
    async fn new(sound_bytes: &[u8], steps: [u8; 32], tempo_bpm: f32) -> Self {
        let sound = load_sound_from_bytes(sound_bytes).await.unwrap();
        let beat_length = 60.0 / tempo_bpm;

        // There are 4 quarter notes in a whole note (beat) in 4/4 time.
        let interval_secs = beat_length / 4.0;

        Self {
            sound,
            steps,
            interval_secs,
            interval_accumulator: 0.0,
            interval_step: 0,
            muted: true,
        }
    }

    fn update(&mut self, delta_time: f32) {
        self.interval_accumulator += delta_time;
        while self.interval_accumulator >= self.interval_secs {
            if !self.muted && self.steps[self.interval_step] != 0 {
                play_sound(
                    &self.sound,
                    PlaySoundParams {
                        looped: false,
                        volume: 1.0,
                    },
                );
            }

            self.interval_accumulator -= self.interval_secs;
            self.interval_step = (self.interval_step + 1) % self.steps.len();

            // Clamp floating point error.
            if self.interval_accumulator < 0.0 {
                self.interval_accumulator = 0.0;
            }
        }
    }
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
        1,0,0,0, 1,0,0,1, 1,0,0,0, 1,0,0,0, 1,0,0,0, 1,0,0,1, 1,0,0,0, 1,0,0,0
    ], TEMPO_BPM).await;

    #[rustfmt::skip]
    let mut track_2_lo = SoundTrack::new(SAMPLE_1_LO, [
        0,0,0,0, 1,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
    ], TEMPO_BPM).await;
    #[rustfmt::skip]
    let mut track_2_hi = SoundTrack::new(SAMPLE_1_HI, [
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 1,0,0,0, 0,0,0,0, 0,0,0,0
    ], TEMPO_BPM).await;

    #[rustfmt::skip]
    let mut track_3_lo = SoundTrack::new(SAMPLE_2_LO, [
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,1,0, 0,0,0,0
    ], TEMPO_BPM).await;
    #[rustfmt::skip]
    let mut track_3_hi = SoundTrack::new(SAMPLE_2_HI, [
        0,0,0,0, 0,0,0,0, 0,0,1,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
    ], TEMPO_BPM).await;

    // Pulse samples.
    #[rustfmt::skip]
    let mut track_4_lo = SoundTrack::new(SAMPLE_PULSE_LO, [
        1,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
    ], TEMPO_BPM).await;
    #[rustfmt::skip]
    let mut track_5_hi = SoundTrack::new(SAMPLE_PULSE_HI, [
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 1,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
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
            track_4_lo.muted = !track_4_lo.muted;
            track_5_hi.muted = !track_5_hi.muted;
        }
        if macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::A) {
            track_1.muted = !track_1.muted;
        }
        if macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::S) {
            track_2_lo.muted = !track_2_lo.muted;
            track_2_hi.muted = !track_2_hi.muted;
        }
        if macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::D) {
            track_3_lo.muted = !track_3_lo.muted;
            track_3_hi.muted = !track_3_hi.muted;
        }

        // Await next frame.
        macroquad::prelude::clear_background(macroquad::prelude::BLACK);
        macroquad::time::draw_fps();
        macroquad::prelude::next_frame().await
    }
}
