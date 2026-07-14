use optic_core::{OpticError, OpticResult};
use std::time::Duration;

use cgmath::{InnerSpace, Rotation};
use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend,
    listener::ListenerHandle,
    track::SpatialTrackBuilder,
    Tween, Decibels,
};

use crate::file::SoundFile;
use crate::sound2d::Sound2D;
use crate::sound3d::Sound3D;

/// The audio subsystem wrapping Kira, managing sound uploads, volume, and 3D listener position.
///
/// Owns the Kira audio manager and a spatial listener. Provides factory methods for creating
/// playable Sound2D and Sound3D handles from SoundFile assets. Every Game has one — use it to
/// upload sounds, set master volume, and position the 3D audio listener each frame.
pub struct AudioEngine {
    manager: AudioManager<DefaultBackend>,
    listener: ListenerHandle,
    /// Master volume multiplier (0.0..1.0). Applied to all sounds.
    master_volume: f32,
}

fn amplitude_to_decibels(amplitude: f32) -> Decibels {
    if amplitude <= 0.0 {
        return Decibels::SILENCE;
    }
    Decibels(20.0 * amplitude.log10())
}

impl AudioEngine {
    /// Creates a new audio engine and initialises the kira backend.
    ///
    /// Spawns the audio render thread. The engine is ready for `ship_sound*`
    /// calls immediately after construction.
    ///
    /// # Errors
    ///
    /// Returns [`OpticErrorKind::Asset`] if the audio backend fails to
    /// initialise or the listener cannot be created.
    pub fn new() -> OpticResult<Self> {
        let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .map_err(|e| {
                OpticError::new(
                    optic_core::OpticErrorKind::Asset,
                    &format!("failed to initialise audio manager: {e}"),
                )
            })?;

        manager.main_track().set_volume(
            Decibels::IDENTITY,
            Tween {
                duration: Duration::from_secs(0),
                ..Default::default()
            },
        );

        let listener = manager.add_listener(
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
        ).map_err(|e| {
            OpticError::new(
                optic_core::OpticErrorKind::Asset,
                &format!("failed to create audio listener: {e}"),
            )
        })?;

        Ok(Self {
            manager,
            listener,
            master_volume: 1.0,
        })
    }

    /// Upload a `SoundFile` into a playable [`Sound2D`] handle.
    ///
    /// The sound is loaded into the audio backend but does not start playing
    /// until [`Sound2D::play`] is called.
    ///
    /// # Errors
    ///
    /// Returns [`OpticErrorKind::Asset`] if the sound cannot be played
    /// through the audio backend.
    pub fn upload_sound2d(&mut self, file: &SoundFile) -> OpticResult<Sound2D> {
        let handle = self.manager.play(file.to_static_sound_data()).map_err(|e| {
            OpticError::new(
                optic_core::OpticErrorKind::Asset,
                &format!("failed to play sound: {e}"),
            )
        })?;
        Ok(Sound2D::new(handle, file.duration_secs))
    }

    /// Upload a `SoundFile` into a playable [`Sound3D`] handle with spatial audio.
    ///
    /// The sound plays through a spatial sub-track linked to the engine's
    /// listener. Call [`Sound3D::update`] each frame to keep the emitter
    /// position in sync.
    ///
    /// # Errors
    ///
    /// Returns [`OpticErrorKind::Asset`] if the spatial track cannot be
    /// created or the sound cannot be played through the audio backend.
    pub fn upload_sound3d(&mut self, file: &SoundFile) -> OpticResult<Sound3D> {
        let mut spatial_track = self.manager.add_spatial_sub_track(
            &self.listener,
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            SpatialTrackBuilder::new()
                .distances((1.0, f32::MAX)),
        ).map_err(|e| {
            OpticError::new(
                optic_core::OpticErrorKind::Asset,
                &format!("failed to create spatial track: {e}"),
            )
        })?;

        let handle = spatial_track.play(file.to_static_sound_data()).map_err(|e| {
            OpticError::new(
                optic_core::OpticErrorKind::Asset,
                &format!("failed to play spatial sound: {e}"),
            )
        })?;
        Ok(Sound3D::new(handle, spatial_track, file.duration_secs))
    }

    /// Returns the master volume (0.0..1.0).
    pub fn master_volume(&self) -> f32 { self.master_volume }

    /// Sets the master volume (0.0..1.0).
    ///
    /// Uses kira's built-in volume control on the main mixer track.
    /// A linear 0..1 value is converted to decibels internally.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        self.manager.main_track().set_volume(
            amplitude_to_decibels(self.master_volume),
            Tween {
                duration: Duration::from_secs(0),
                ..Default::default()
            },
        );
    }

    /// Sets the listener position and orientation directly.
    ///
    /// Call this once per frame if the listener moves. `pos` is the world-space
    /// position, `forward` is the look direction, `up` is the world up vector.
    pub fn set_listener(
        &mut self,
        pos: cgmath::Vector3<f32>,
        forward: cgmath::Vector3<f32>,
        up: cgmath::Vector3<f32>,
    ) {
        let forward = forward.normalize();
        let up = up.normalize();
        let right = forward.cross(up).normalize();
        let real_up = right.cross(forward);
        let quat = cgmath::Quaternion::look_at(forward, real_up);
        let _ = self.listener.set_position(pos, Tween::default());
        let _ = self.listener.set_orientation(quat, Tween::default());
    }

    /// Derives listener position/orientation from a Camera's transform.
    ///
    /// Call this once per frame, right after
    /// [`camera.pre_update()`](optic_render::Camera::pre_update).
    pub fn set_listener_from_camera(&mut self, camera: &optic_render::Camera) {
        let pos = camera.pos();
        let front = camera.front();
        let up = cgmath::Vector3::unit_y();
        self.set_listener(pos, front, up);
    }
}
