use kira::{
    sound::static_sound::StaticSoundHandle,
    track::SpatialTrackHandle,
    Decibels, PlaybackRate, Tween,
};

use optic_render::Transform3D;

/// A handle to a playing 3D sound with spatial audio.
///
/// Positioned in 3D space relative to the listener. Created by
/// [`AudioEngine::upload_sound3d`](crate::AudioEngine::upload_sound3d).
///
/// Call [`update`](Sound3D::update) each frame to keep the emitter position
/// in sync with the game object it's attached to.
///
/// # Example
///
/// ```ignore
    /// let mut sound = audio.upload_sound3d(&sfx)?;
/// sound.transform.set_position(10.0, 0.0, 5.0);
/// sound.play();
/// // each frame:
/// sound.update();
/// ```
pub struct Sound3D {
    handle: Option<StaticSoundHandle>,
    spatial_track: Option<SpatialTrackHandle>,
    volume: f32,
    pitch: f32,
    looping: bool,
    transform: Transform3D,
    min_distance: f32,
    max_distance: f32,
    duration_secs: f32,
}

impl Sound3D {
    pub(crate) fn new(
        handle: StaticSoundHandle,
        spatial_track: SpatialTrackHandle,
        duration_secs: f32,
    ) -> Self {
        Self {
            handle: Some(handle),
            spatial_track: Some(spatial_track),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            transform: Transform3D::default(),
            min_distance: 1.0,
            max_distance: f32::MAX,
            duration_secs,
        }
    }

    /// Returns the volume (0.0..1.0).
    pub fn volume(&self) -> f32 { self.volume }
    /// Returns the pitch multiplier.
    pub fn pitch(&self) -> f32 { self.pitch }
    /// Returns whether the sound loops.
    pub fn looping(&self) -> bool { self.looping }
    /// Returns a reference to the world-space transform.
    pub fn transform(&self) -> &Transform3D { &self.transform }
    /// Returns a mutable reference to the world-space transform.
    pub fn transform_mut(&mut self) -> &mut Transform3D { &mut self.transform }
    /// Returns the minimum distance for 3D attenuation.
    pub fn min_distance(&self) -> f32 { self.min_distance }
    /// Returns the maximum distance for 3D attenuation.
    pub fn max_distance(&self) -> f32 { self.max_distance }

    /// Starts or restarts playback.
    pub fn play(&mut self) {
        if let Some(ref mut h) = self.handle {
            let _ = h.resume(Tween::default());
        }
    }

    /// Pauses playback (position is preserved).
    pub fn pause(&mut self) {
        if let Some(ref mut h) = self.handle {
            let _ = h.pause(Tween::default());
        }
    }

    /// Resumes playback if paused.
    pub fn resume(&mut self) {
        if let Some(ref mut h) = self.handle {
            let _ = h.resume(Tween::default());
        }
    }

    /// Stops playback and resets position to the start.
    pub fn stop(&mut self) {
        if let Some(ref mut h) = self.handle {
            let _ = h.stop(Tween::default());
        }
    }

    /// Returns `true` if the sound is currently playing.
    pub fn is_playing(&self) -> bool {
        self.handle.as_ref().map_or(false, |h| h.state() == kira::sound::PlaybackState::Playing)
    }

    /// Returns `true` if the sound is paused.
    pub fn is_paused(&self) -> bool {
        self.handle.as_ref().map_or(false, |h| h.state() == kira::sound::PlaybackState::Paused)
    }

    /// Sets the volume (0.0..1.0).
    pub fn set_volume(&mut self, v: f32) {
        self.volume = v.clamp(0.0, 1.0);
        let db = if self.volume <= 0.0 {
            Decibels::SILENCE
        } else {
            Decibels(20.0 * self.volume.log10())
        };
        if let Some(ref mut h) = self.handle {
            let _ = h.set_volume(db, Tween::default());
        }
    }

    /// Sets the pitch multiplier (1.0 = normal).
    pub fn set_pitch(&mut self, p: f32) {
        self.pitch = p.max(0.01);
        if let Some(ref mut h) = self.handle {
            let _ = h.set_playback_rate(PlaybackRate(self.pitch as f64), Tween::default());
        }
    }

    /// Sets whether the sound loops.
    pub fn set_looping(&mut self, l: bool) {
        self.looping = l;
        if let Some(ref mut h) = self.handle {
            if l {
                let _ = h.set_loop_region(0.0f64..);
            } else {
                let _ = h.set_loop_region(None::<kira::sound::Region>);
            }
        }
    }

    /// Seeks to a position in seconds.
    ///
    /// # Errors
    ///
    /// Currently always succeeds, but returns [`OpticResult`] for forward
    /// compatibility with backend implementations that may fail.
    pub fn seek(&mut self, secs: f32) -> optic_core::OpticResult<()> {
        if let Some(ref mut h) = self.handle {
            h.seek_to(secs as f64);
        }
        Ok(())
    }

    /// Returns the current playback position in seconds.
    pub fn position_secs(&self) -> f32 {
        self.handle.as_ref().map_or(0.0, |h| h.position() as f32)
    }

    /// Returns the total duration in seconds.
    pub fn duration_secs(&self) -> f32 {
        self.duration_secs
    }

    /// Sets the min/max distance for 3D attenuation.
    pub fn set_min_max_distance(&mut self, min: f32, max: f32) {
        self.min_distance = min.max(0.0);
        self.max_distance = max.max(self.min_distance);
    }

    /// Pushes the current `transform.pos` into the audio backend's spatial
    /// emitter. Call once per frame for any `Sound3D` that moves.
    ///
    /// The backend (not this engine) computes actual panning and attenuation
    /// from emitter position vs. listener position.
    pub fn update(&mut self) {
        if let Some(ref mut st) = self.spatial_track {
            let pos = self.transform.pos();
            let _ = st.set_position(pos, Tween::default());
        }
    }

    /// Stops and destroys the sound, releasing backend resources.
    pub fn delete(mut self) {
        if let Some(mut h) = self.handle.take() {
            let _ = h.stop(Tween::default());
        }
        self.spatial_track.take();
    }
}
