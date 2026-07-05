//! Audio playback for the Optic engine.
//!
//! Provides a render-like pipeline for audio: [`SoundFile`] (disk asset) →
//! [`AudioEngine::upload_sound2d`]/[`upload_sound3d`](AudioEngine::upload_sound3d) →
//! [`Sound2D`]/[`Sound3D`] (playable handles).
//!
//! # Architecture
//!
//! | Layer | Type | Role |
//! |-------|------|------|
//! | Assets | [`SoundFile`] | Decoded PCM with sample rate and channel metadata |
//! | Engine | [`AudioEngine`] | Kira-backed audio manager, listener, sound factory |
//! | 2D handles | [`Sound2D`] | Simple playback with volume/pitch/pan |
//! | 3D handles | [`Sound3D`] | Positional playback with distance-based attenuation |
//!
//! # Listener convention
//!
//! 3D sound requires a listener reference point. Use
//! [`AudioEngine::set_listener_from_camera`] each frame to tie the listener to
//! the [`Camera`](optic_render::Camera) — this keeps audio and visuals in sync
//! without a separate `Listener` type.

mod engine;
mod file;
mod sound2d;
mod sound3d;

pub use engine::*;
pub use file::*;
pub use sound2d::*;
pub use sound3d::*;
