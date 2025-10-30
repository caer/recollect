// Foley samples.
pub const FOLEY_VINYL_TEXTURE: &[u8] =
    include_bytes!("../../assets/Clark Audio - Texture Crackle Vinyl.wav");

// Track samples.
pub const SAMPLE_BASELINE: &[u8] = include_bytes!("../../assets/bfxr - decay4.wav");
pub const SAMPLE_1_LO: &[u8] =
    include_bytes!("../../assets/Helton Yan - Major Distant Piano Lo.wav");
pub const SAMPLE_1_HI: &[u8] =
    include_bytes!("../../assets/Helton Yan - Major Distant Piano Hi.wav");
pub const SAMPLE_2_LO: &[u8] = include_bytes!("../../assets/Helton Yan - Major Lone Beep Lo.wav");
pub const SAMPLE_2_HI: &[u8] = include_bytes!("../../assets/Helton Yan - Major Lone Beep Hi.wav");
pub const SAMPLE_3_LO: &[u8] = include_bytes!("../../assets/Helton Yan - Pulse Low.wav");
pub const SAMPLE_3_HI: &[u8] = include_bytes!("../../assets/Helton Yan - Pulse High.wav");

/// The tempo of the game tracks, in beats per minute.
///
/// Lo-Fi beats tend to be around 60-90 BPM.
pub const TEMPO_BPM: f32 = 80.0;

/// A musical piece comprised of one or more [Track]s.
pub struct Piece {
    /// The tracks comprising the piece.
    ///
    /// The first track is considered the "baseline" track,
    /// and is used to determine the interval timing (with
    /// respect to the piece's tempo) between steps for all tracks.
    tracks: Vec<Track>,

    /// The playback status of each track in the piece,
    /// where `true` means the track played during the
    /// most recent update, and `false` means it did not.
    track_states: Vec<bool>,

    /// The time interval between each beat-step, in seconds.
    interval_secs: f32,
    interval_accumulator: f32,

    /// The index of the next beat-step to play.
    interval_step: usize,
}

impl Piece {
    /// Creates a new piece with a baseline track and tempo.
    pub fn new(baseline_track: Track, tempo_bpm: f32) -> Self {
        let mut piece = Self {
            tracks: vec![baseline_track],
            track_states: vec![false],
            interval_secs: 0.0,
            interval_accumulator: 0.0,
            interval_step: 0,
        };

        piece.set_tempo(tempo_bpm);

        piece
    }

    /// Adds a track to the piece.
    pub fn with(mut self, track: Track) -> Self {
        assert_eq!(track.steps.len(), self.tracks[0].steps.len());
        self.tracks.push(track);
        self.track_states.push(false);
        self
    }

    /// Changes the tempo of the piece.
    pub fn set_tempo(&mut self, tempo_bpm: f32) {
        // Calculate the time interval between each beat-step (quarter note) such that
        // we'll play `tempo_bpm` beats per minute. There are four quarter notes
        // per measure, and 8 measures per phrase, for a total of 32 steps per phrase.
        let beats_per_phrase = self.tracks[0].steps.iter().filter(|&&s| s != 0).count() as f32;
        let phrases_per_minute = tempo_bpm / beats_per_phrase;
        let seconds_per_phrase = 60.0 / phrases_per_minute;
        let interval_secs = seconds_per_phrase / 32.0;

        eprintln!(
            "Piece: {} BPM, {} steps, {} phrases/min, {} secs/phrase, {} secs/step",
            tempo_bpm, beats_per_phrase, phrases_per_minute, seconds_per_phrase, interval_secs,
        );

        self.interval_secs = interval_secs;
    }

    /// Changes the volume of a given track in the piece.
    pub fn set_track_volume(&mut self, track_index: usize, volume: f32) {
        if let Some(track) = self.tracks.get_mut(track_index) {
            track.volume = volume;
        }
    }

    /// Returns the number of tracks in the piece.
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Updates the piece, playing sounds as needed.
    ///
    /// Returns a slice of booleans indicating which tracks
    /// played during this update, where each index corresponds
    /// to the track at the same index in [Self::tracks].
    pub fn update(&mut self, delta_time: f32) -> &[bool] {
        self.interval_accumulator += delta_time;

        // Clear previous track states.
        self.track_states.fill(false);

        // Play any tracks that have a sound at the current step.
        while self.interval_accumulator >= self.interval_secs {
            for (i, track) in self.tracks.iter().enumerate() {
                if track.volume > 0.0 && track.steps[self.interval_step] != 0 {
                    macroquad::audio::play_sound(
                        &track.sound,
                        macroquad::audio::PlaySoundParams {
                            looped: false,
                            volume: track.volume,
                        },
                    );

                    self.track_states[i] = true;
                }
            }

            self.interval_accumulator -= self.interval_secs;

            // @caer: todo: Track steps are pinned to the baseline track.
            //        What happens when tracks have different lengths?
            self.interval_step = (self.interval_step + 1) % self.tracks[0].steps.len();

            // Clamp floating point error.
            if self.interval_accumulator < 0.0 {
                self.interval_accumulator = 0.0;
            }
        }

        &self.track_states
    }
}

/// A single track within a [Piece], representing a single
/// instrument or sound source.
///
/// @caer: todo: Currently, tracks define a single "phrase"
///        of 32 beat-steps (8 measures of 4 beat-steps).
pub struct Track {
    /// The sound to play at each step.
    sound: macroquad::audio::Sound,

    /// List of steps (beat subdivions) in the track.
    ///
    /// Each step containing a `0` means no sound
    /// should be played at that step, while a `1`
    /// means the sound should be played.
    steps: [u8; 32],

    /// The track's relative volume in a piece, ranging
    /// from `0.0` (silent) to `1.0` (full volume).
    volume: f32,
}

impl Track {
    pub async fn new(sound_bytes: &[u8], steps: [u8; 32]) -> Self {
        let sound = macroquad::audio::load_sound_from_bytes(sound_bytes)
            .await
            .unwrap();

        Self {
            sound,
            steps,
            volume: 0.0,
        }
    }
}
