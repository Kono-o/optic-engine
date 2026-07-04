use optic_core::consts::{OPTIC_CACHE_VERSION, OPTIC_MAGIC};
use optic_core::{OpticError, OpticErrorKind, OpticResult};
use std::sync::Arc;

use kira::sound::static_sound::StaticSoundData;
use kira::Frame;

/// A decoded sound file loaded from disk or cache.
///
/// Stores raw interleaved PCM samples in memory, analogous to how
/// [`TextureFile`](optic_render::asset::TextureFile) stores decoded pixel data.
///
/// # Loading
///
/// ```ignore
/// use optic_sound::SoundFile;
///
/// let sfx = SoundFile::from_disk("sounds/explosion.wav")?;
/// // sfx.duration_secs, sfx.sample_rate, sfx.channels, sfx.samples
/// ```
///
/// # Caching
///
/// In debug builds, `from_disk` decodes the source file and writes a binary
/// cache (`.omusic`). In release builds, it reads the cache directly for
/// faster startup.
pub struct SoundFile {
    /// Interleaved PCM samples (one frame = `channels` consecutive samples).
    pub samples: Vec<f32>,
    /// Sample rate in Hz (e.g. 44100).
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: u8,
    /// Total duration in seconds.
    pub duration_secs: f32,
}

impl SoundFile {
    /// Loads a sound file from disk, with caching.
    ///
    /// Debug builds decode the source and overwrite the cache. Release builds
    /// load the cache directly.
    #[cfg(debug_assertions)]
    pub fn from_disk(path: &str) -> OpticResult<Self> {
        let sound = StaticSoundData::from_file(path).map_err(|e| {
            OpticError::new(
                OpticErrorKind::File,
                &format!("failed to decode audio {path}: {e}"),
            )
        })?;

        let sample_rate = sound.sample_rate;
        let frame_count = sound.frames.len();
        let mut samples = Vec::with_capacity(frame_count * 2);
        for frame in sound.frames.iter() {
            samples.push(frame.left);
            samples.push(frame.right);
        }

        // Try to detect mono by checking if all frames have identical channels
        let is_mono = frames_are_mono(&sound.frames);
        let actual_channels = if is_mono { 1u8 } else { 2u8 };
        let duration_secs = frame_count as f32 / sample_rate as f32;

        let sf = Self {
            samples,
            sample_rate,
            channels: actual_channels,
            duration_secs,
        };

        let cache = optic_file::cached_path(path, "omusic");
        let _ = sf.save_cached(&cache);
        Ok(sf)
    }

    /// Loads a sound file from the binary cache (release only).
    #[cfg(not(debug_assertions))]
    pub fn from_disk(path: &str) -> OpticResult<Self> {
        let cache = optic_file::cached_path(path, "omusic");
        Self::from_cached(&cache)
    }

    /// Loads from a `.omusic` binary cache file.
    pub fn from_cached(path: &str) -> OpticResult<Self> {
        let data = optic_file::read_bytes(path)?;
        if data.len() < 19 {
            return Err(OpticError::new(
                OpticErrorKind::Asset,
                &format!("cached sound file too short: {path}"),
            ));
        }
        if data[0..8] != OPTIC_MAGIC {
            return Err(OpticError::new(
                OpticErrorKind::Asset,
                &format!("not a valid Optic cache file (bad magic): {path}"),
            ));
        }
        let version = u16::from_le_bytes([data[8], data[9]]);
        if version != OPTIC_CACHE_VERSION {
            return Err(OpticError::new(
                OpticErrorKind::Asset,
                &format!(
                    "cache file version {version} is not supported (expected {OPTIC_CACHE_VERSION}): {path}"
                ),
            ));
        }
        let sample_rate = u32::from_le_bytes([data[10], data[11], data[12], data[13]]);
        let channels = data[14];
        let sample_count = u64::from_le_bytes([
            data[15], data[16], data[17], data[18], data[19], data[20], data[21], data[22],
        ]);
        let expected_bytes = sample_count as usize * 4;
        if data.len() < 23 + expected_bytes {
            return Err(OpticError::new(
                OpticErrorKind::Asset,
                &format!(
                    "cached sound file size mismatch: expected {} bytes, got {} for {path}",
                    23 + expected_bytes,
                    data.len()
                ),
            ));
        }
        let samples_raw = &data[23..23 + expected_bytes];
        let samples: Vec<f32> = samples_raw
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let frame_count = sample_count / channels as u64;
        let duration_secs = frame_count as f32 / sample_rate as f32;

        Ok(Self {
            samples,
            sample_rate,
            channels,
            duration_secs,
        })
    }

    /// Saves this sound to a `.omusic` binary cache file.
    pub fn save_cached(&self, path: &str) -> OpticResult<()> {
        let sample_count = self.samples.len() as u64;
        let mut data = Vec::with_capacity(23 + self.samples.len() * 4);
        data.extend_from_slice(&OPTIC_MAGIC);
        data.extend_from_slice(&OPTIC_CACHE_VERSION.to_le_bytes());
        data.extend_from_slice(&self.sample_rate.to_le_bytes());
        data.push(self.channels);
        data.extend_from_slice(&sample_count.to_le_bytes());
        for &sample in &self.samples {
            data.extend_from_slice(&sample.to_le_bytes());
        }
        optic_file::write_bytes(path, &data)
    }

    /// Construct a `StaticSoundData` from this `SoundFile` for playback via kira.
    pub(crate) fn to_static_sound_data(&self) -> StaticSoundData {
        let frame_count = self.samples.len() / self.channels as usize;
        let mut frames = Vec::with_capacity(frame_count);
        for i in 0..frame_count {
            let base = i * self.channels as usize;
            if self.channels == 1 {
                frames.push(Frame::from_mono(self.samples[base]));
            } else {
                let l = self.samples[base];
                let r = self.samples.get(base + 1).copied().unwrap_or(l);
                frames.push(Frame::new(l, r));
            }
        }
        StaticSoundData {
            sample_rate: self.sample_rate,
            frames: frames.into(),
            settings: kira::sound::static_sound::StaticSoundSettings::default(),
            slice: None,
        }
    }
}

/// Heuristic: check if all kira frames are essentially mono (left ≈ right).
fn frames_are_mono(frames: &Arc<[Frame]>) -> bool {
    let threshold = 1.0 / 65536.0;
    frames.iter().all(|f| (f.left - f.right).abs() < threshold)
}
