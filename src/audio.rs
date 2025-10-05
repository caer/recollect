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
pub const TEMPO_BPM: f32 = 64.0;

pub struct SoundTrack {
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
    pub async fn new(sound_bytes: &[u8], steps: [u8; 32], tempo_bpm: f32) -> Self {
        let sound = macroquad::audio::load_sound_from_bytes(sound_bytes)
            .await
            .unwrap();
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

    pub fn update(&mut self, delta_time: f32) {
        self.interval_accumulator += delta_time;
        while self.interval_accumulator >= self.interval_secs {
            if !self.muted && self.steps[self.interval_step] != 0 {
                macroquad::audio::play_sound(
                    &self.sound,
                    macroquad::audio::PlaySoundParams {
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

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }
}
