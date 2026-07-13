use kira::{
    sound::static_sound::StaticSoundHandle,
    Decibels, PlaybackRate, Tween,
};

/// A handle to a playing 2D sound.
///
/// Controls playback of a single sound instance. Created by
/// [`AudioEngine::upload_sound2d`](crate::AudioEngine::upload_sound2d).
///
/// # Example
///
/// ```ignore
    /// let mut sound = audio.upload_sound2d(&sfx)?;
/// sound.set_looping(true);
/// sound.play();
/// ```
pub struct Sound2D {
    handle: Option<StaticSoundHandle>,
    volume: f32,
    pitch: f32,
    looping: bool,
    pan: f32,
    duration_secs: f32,
}

impl Sound2D {
    pub(crate) fn new(handle: StaticSoundHandle, duration_secs: f32) -> Self {
        Self {
            handle: Some(handle),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            pan: 0.0,
            duration_secs,
        }
    }

    /// Returns the volume (0.0..1.0).
    pub fn volume(&self) -> f32 { self.volume }
    /// Returns the pitch multiplier.
    pub fn pitch(&self) -> f32 { self.pitch }
    /// Returns whether the sound loops.
    pub fn looping(&self) -> bool { self.looping }
    /// Returns the stereo pan position (-1.0 left, 0.0 centre, 1.0 right).
    pub fn pan(&self) -> f32 { self.pan }

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

    /// Sets the stereo pan position (-1.0 = full left, 1.0 = full right).
    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
        if let Some(ref mut h) = self.handle {
            let _ = h.set_panning(self.pan, Tween::default());
        }
    }

    /// Sets the volume (0.0..1.0).
    pub fn set_volume(&mut self, v: f32) {
        self.volume = v.clamp(0.0, 1.0);
        // Convert linear 0..1 to Decibels: silence at 0, identity at 1
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

    /// Stops and destroys the sound, releasing backend resources.
    pub fn delete(mut self) {
        if let Some(mut h) = self.handle.take() {
            let _ = h.stop(Tween::default());
        }
    }
}
