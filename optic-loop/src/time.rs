use std::collections::VecDeque;
use std::time::Instant;

use optic_core::log_warn;

// ---------------------------------------------------------------------------
// FixedStepper
// ---------------------------------------------------------------------------

/// A reusable fixed-timestep accumulator.
///
/// Used internally by [`Time`] for both physics and update scheduling.
/// Users never interact with this type directly.
///
/// When `target_hz` is `None`, the stepper runs once per frame using the
/// raw frame delta (no accumulation). When `Some(hz)`, it accumulates
/// frame time and fires callbacks at exactly `hz` Hz using the classic
/// *Fix Your Timestep!* pattern.
pub(crate) struct FixedStepper {
    target_hz: Option<f64>,
    accumulator: f64,
    max_steps_per_frame: u32,
    last_fixed_dt: f64,
    last_alpha: f32,
}

impl FixedStepper {
    /// Creates a new stepper.
    ///
    /// `target_hz == None` means "run once per frame". `Some(hz)` means
    /// fixed timestep at the given rate. Panics in debug builds if `hz`
    /// is zero, negative, NaN, or infinite.
    pub(crate) fn new(target_hz: Option<f64>) -> Self {
        if let Some(hz) = target_hz {
            assert!(hz > 0.0 && hz.is_finite(), "invalid target_hz: {hz}");
        }
        Self {
            target_hz,
            accumulator: 0.0,
            max_steps_per_frame: 240,
            last_fixed_dt: target_hz.map_or(0.0, |hz| 1.0 / hz),
            last_alpha: if target_hz.is_some() { 0.0 } else { 1.0 },
        }
    }

    /// Sets the target rate. `None` means once per frame. `Some(hz)` means
    /// fixed timestep. Panics in debug builds for invalid values.
    pub(crate) fn set_target_hz(&mut self, hz: Option<f64>) {
        if let Some(h) = hz {
            assert!(h > 0.0 && h.is_finite(), "invalid target_hz: {h}");
        }
        self.target_hz = hz;
    }

    /// Current target rate.
    pub(crate) fn target_hz(&self) -> Option<f64> {
        self.target_hz
    }

    /// Sets the spiral-of-death guard. Default is 240.
    pub(crate) fn set_max_steps_per_frame(&mut self, max: u32) {
        self.max_steps_per_frame = max;
    }

    /// Current maximum steps per frame.
    pub(crate) fn max_steps_per_frame(&self) -> u32 {
        self.max_steps_per_frame
    }

    /// Advance by `frame_delta` seconds. Returns `(step_count, fixed_dt)`.
    ///
    /// When `target_hz` is `None`, always returns `(1, frame_delta)` with
    /// no accumulation.
    ///
    /// When `Some(hz)`, accumulates and drains at `1/hz` fixed steps, capped
    /// at `max_steps_per_frame`. Excess backlog is discarded with phase
    /// preservation (`accumulator %= fixed_dt`).
    pub(crate) fn step(&mut self, frame_delta: f64) -> (u32, f64) {
        let hz = match self.target_hz {
            Some(hz) => hz,
            None => {
                self.last_fixed_dt = frame_delta;
                self.last_alpha = 1.0;
                return (1, frame_delta);
            }
        };

        let fixed_dt = 1.0 / hz;
        self.last_fixed_dt = fixed_dt;
        self.accumulator += frame_delta;

        let mut steps: u32 = 0;
        while self.accumulator >= fixed_dt {
            self.accumulator -= fixed_dt;
            steps += 1;
            if steps >= self.max_steps_per_frame {
                let discarded = self.accumulator;
                self.accumulator %= fixed_dt;
                log_warn!(
                    "fixed-stepper: truncated {discarded:.4}s of backlog (>{steps} steps)"
                );
                break;
            }
        }

        self.last_alpha = if fixed_dt > 0.0 {
            (self.accumulator / fixed_dt).clamp(0.0, 0.999999) as f32
        } else {
            0.0
        };

        (steps, fixed_dt)
    }

    /// Interpolation alpha from the most recent [`step`](Self::step) call.
    ///
    /// Returns a value in `[0, 1)` for fixed-timestep mode, or `1.0` when
    /// operating without a fixed timestep.
    pub(crate) fn interpolation_alpha(&self) -> f32 {
        self.last_alpha
    }
}

// ---------------------------------------------------------------------------
// FpsLimit
// ---------------------------------------------------------------------------

/// Rendering frame-rate policy controlling how the engine paces presented frames.
///
/// Select Uncapped for maximum throughput, VSync for tear-free display at monitor refresh, or
/// Limited(fps) to target a specific frame rate via sleep-based pacing. Set this via
/// Time::set_fps_limit() to control performance and power usage.
pub enum FpsLimit {
    /// No frame-rate cap. Render as fast as the platform allows.
    Uncapped,

    /// Swap interval determines pacing. No additional sleeping.
    VSync,

    /// Sleep to approximately the given frame rate.
    Limited(f64),
}

impl FpsLimit {
    /// Returns the target frame time in seconds, or `None` for uncapped / VSync.
    pub(crate) fn target_frame_time(&self) -> Option<f64> {
        match self {
            FpsLimit::Uncapped | FpsLimit::VSync => None,
            FpsLimit::Limited(fps) => {
                if *fps > 0.0 && fps.is_finite() {
                    Some(1.0 / fps)
                } else {
                    None
                }
            }
        }
    }
}

impl Default for FpsLimit {
    fn default() -> Self {
        FpsLimit::VSync
    }
}

// ---------------------------------------------------------------------------
// Time
// ---------------------------------------------------------------------------

/// Frame timing data — delta time, smoothed FPS, elapsed wall-clock time,
/// physics/update steppers, and FPS limit.
///
/// `Time` is updated once per frame by the engine (via [`Time::update`])
/// before user code runs. It provides three fundamental measurements:
///
/// | Field | Meaning | Use case |
/// |---|---|---|
/// | [`delta`](Time::delta) | Seconds since the last frame | Frame-rate independent motion |
/// | [`fps`](Time::fps) | Smoothed frames-per-second | HUD display, monitoring |
/// | [`elapsed`](Time::elapsed) | Total seconds since start | Timers, replays |
///
/// # Fixed-rate scheduling
///
/// Physics and update each run at independently configurable rates via
/// [`set_target_physics_rate`](Time::set_target_physics_rate) and
/// [`set_target_tps`](Time::set_target_tps). The engine calls
/// [`advance_physics`](Time::advance_physics) and
/// [`advance_update`](Time::advance_update) internally; users never
/// call these directly.
///
/// # FPS limiting
///
/// Rendering pacing is controlled by [`set_fps_limit`](Time::set_fps_limit).
///
/// # Example
///
/// ```ignore
/// use optic_loop::{Game, Runtime, FpsLimit};
///
/// struct App;
/// impl Runtime for App {
///     fn start(&mut self, game: &mut Game) {
///         game.time.set_target_physics_rate(120.0);
///         game.time.set_target_tps(Some(20.0));
///         game.time.set_fps_limit(FpsLimit::Limited(60.0));
///     }
///     fn update(&mut self, _game: &mut Game) {}
/// }
/// ```
pub struct Time {
    pub fps: f64,
    pub delta: f64,
    pub tick_count: u64,
    pub elapsed: f64,
    pub start_time: Instant,
    pub prev_time: Instant,
    pub prev_sec: Instant,
    pub local_tick: u32,
    prev_deltas: VecDeque<f64>,

    physics_stepper: FixedStepper,
    update_stepper: FixedStepper,
    fps_limit: FpsLimit,

    physics_delta: f64,
    physics_alpha: f32,
    frame_start: Instant,
}

impl Time {
    /// Creates a new timer with all counters at zero.
    ///
    /// # Defaults
    ///
    /// - Physics: 60 Hz fixed timestep
    /// - Update: once per frame (no fixed timestep)
    /// - FPS limit: VSync
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut time = Time::new();
    /// assert_eq!(time.target_physics_rate(), 60.0);
    /// assert_eq!(time.target_tps(), None);
    /// ```
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            fps: 0.0,
            delta: 0.0,
            tick_count: 0,
            elapsed: 0.0,
            start_time: now,
            prev_time: now,
            prev_sec: now,
            local_tick: 0,
            prev_deltas: VecDeque::with_capacity(32),

            physics_stepper: FixedStepper::new(Some(60.0)),
            update_stepper: FixedStepper::new(None),
            fps_limit: FpsLimit::VSync,

            physics_delta: 1.0 / 60.0,
            physics_alpha: 0.0,
            frame_start: now,
        }
    }

    // ----- frame lifecycle (called by engine) -----

    /// Record the start of a new frame. Called once at the top of each frame
    /// before any scheduling occurs.
    pub(crate) fn begin_frame(&mut self) {
        self.frame_start = Instant::now();
    }

    /// Advances the timer by one frame.
    ///
    /// Called automatically by the engine each frame. Computes `delta`,
    /// `elapsed`, and smoothed FPS from wall-clock time.
    pub fn update(&mut self) {
        self.tick_count += 1;
        self.local_tick += 1;
        let now = Instant::now();

        self.elapsed = now.duration_since(self.start_time).as_secs_f64();
        self.delta = now.duration_since(self.prev_time).as_secs_f64();
        self.prev_time = now;

        self.prev_deltas.push_back(self.delta);
        if self.prev_deltas.len() > 32 {
            self.prev_deltas.pop_front();
        }

        let avg = self.prev_deltas.iter().sum::<f64>() / self.prev_deltas.len() as f64;
        self.fps = if avg > 0.0 { 1.0 / avg } else { 0.0 };

        if now.duration_since(self.prev_sec).as_secs_f64() >= 1.0 {
            self.local_tick = 0;
            self.prev_sec = now;
        }
    }

    /// Advance the physics scheduler. Returns the number of physics steps
    /// to execute this frame.
    ///
    /// Called internally by the engine. Each step should invoke
    /// [`Runtime::physics`](crate::Runtime::physics).
    pub(crate) fn advance_physics(&mut self, frame_delta: f64) -> u32 {
        let (steps, fixed_dt) = self.physics_stepper.step(frame_delta);
        self.physics_delta = fixed_dt;
        self.physics_alpha = self.physics_stepper.interpolation_alpha();
        steps
    }

    /// Advance the update scheduler. Returns the number of update steps
    /// to execute this frame.
    ///
    /// Called internally by the engine. Each step should invoke
    /// [`Runtime::update`](crate::Runtime::update).
    pub(crate) fn advance_update(&mut self, frame_delta: f64) -> u32 {
        let (steps, _) = self.update_stepper.step(frame_delta);
        steps
    }

    // ----- physics rate -----

    /// Sets the target physics rate in Hz.
    ///
    /// Must be positive and finite. Panics in debug builds for invalid values.
    ///
    /// Takes effect on the next call to [`advance_physics`](Time::advance_physics).
    /// Existing accumulator contents are preserved and interpreted using the
    /// new timestep.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Run physics at 120 Hz
    /// game.time.set_target_physics_rate(120.0);
    ///
    /// // Run physics at 30 Hz
    /// game.time.set_target_physics_rate(30.0);
    /// ```
    ///
    /// # Panics (debug only)
    ///
    /// Panics if `hz` is zero, negative, NaN, or infinite.
    pub fn set_target_physics_rate(&mut self, hz: f64) {
        self.physics_stepper.set_target_hz(Some(hz));
    }

    /// Current target physics rate in Hz.
    pub fn target_physics_rate(&self) -> f64 {
        self.physics_stepper.target_hz().unwrap_or(60.0)
    }

    /// Fixed physics delta (`1.0 / target_physics_rate`).
    ///
    /// This value is constant across all physics callbacks executed during
    /// a single frame. If a frame requires three physics steps, all three
    /// observe the same `physics_delta()`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn physics(&mut self, game: &mut Game) {
    ///     let dt = game.time.physics_delta(); // e.g. 1/60 ≈ 0.01667
    ///     entity.velocity += acceleration * dt;
    ///     entity.position += entity.velocity * dt;
    /// }
    /// ```
    pub fn physics_delta(&self) -> f64 {
        self.physics_delta
    }

    /// Interpolation alpha for rendering, in `[0, 1)`.
    ///
    /// Use this to lerp between previous and current simulation state for
    /// smooth presentation when physics and render rates differ.
    ///
    /// When `target_hz` is `None` (no fixed timestep), returns `1.0`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn render(&mut self, game: &mut Game) {
    ///     let alpha = game.time.physics_alpha();
    ///     let display_pos = previous_pos.lerp(current_pos, alpha);
    ///     // draw at display_pos
    /// }
    /// ```
    ///
    /// # Bounds
    ///
    /// Always satisfies `0.0 <= alpha < 1.0` for fixed-timestep mode,
    /// or `alpha == 1.0` when operating without a fixed timestep.
    pub fn physics_alpha(&self) -> f32 {
        self.physics_alpha
    }

    // ----- update rate -----

    /// Sets the target updates-per-second.
    ///
    /// - `None`: once per rendered frame (the default, preserves legacy behaviour)
    /// - `Some(hz)`: fixed update timestep at the given rate
    ///
    /// Takes effect on the next call to [`advance_update`](Time::advance_update).
    /// Existing accumulator contents are preserved.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Run update at 20 Hz (independent of render rate)
    /// game.time.set_target_tps(Some(20.0));
    ///
    /// // Revert to once-per-frame (default)
    /// game.time.set_target_tps(None);
    /// ```
    pub fn set_target_tps(&mut self, hz: Option<f64>) {
        self.update_stepper.set_target_hz(hz);
    }

    /// Current target TPS, or `None` if updating once per frame.
    pub fn target_tps(&self) -> Option<f64> {
        self.update_stepper.target_hz()
    }

    // ----- FPS limit -----

    /// Sets the rendering frame-rate limit.
    pub fn set_fps_limit(&mut self, limit: FpsLimit) {
        self.fps_limit = limit;
    }

    /// Current FPS limit.
    pub fn fps_limit(&self) -> &FpsLimit {
        &self.fps_limit
    }

    /// Elapsed wall-clock seconds since the frame began. Used by the engine
    /// to determine how much to sleep for FPS limiting.
    pub(crate) fn frame_elapsed(&self) -> f64 {
        self.frame_start.elapsed().as_secs_f64()
    }

    // ----- original public API (unchanged) -----

    /// Smoothed frames-per-second (averaged over the last 32 frames).
    ///
    /// Returns `0.0` until at least one frame has been timed.
    pub fn fps(&self) -> f64 { self.fps }

    /// Delta time in seconds since the last frame.
    ///
    /// Multiply speeds by this value to get frame-rate-independent motion:
    ///
    /// ```ignore
    /// let speed = 10.0; // units per second
    /// entity.position.x += speed * time.delta();
    /// ```
    pub fn delta(&self) -> f64 { self.delta }

    /// Total wall-clock seconds since [`Time::new`] was called.
    ///
    /// Useful for timers and deadlines:
    ///
    /// ```ignore
    /// let deadline = game.time.elapsed() + 5.0;
    /// // later:
    /// if game.time.elapsed() > deadline { /* ... */ }
    /// ```
    pub fn elapsed(&self) -> f64 { self.elapsed }

    /// Current elapsed time in seconds (re-queries `Instant::now`).
    pub fn now(&self) -> f64 {
        Instant::now().duration_since(self.start_time).as_secs_f64()
    }

    /// Current elapsed time in milliseconds.
    pub fn now_ms(&self) -> u64 {
        Instant::now().duration_since(self.start_time).as_millis() as u64
    }

    /// Alias for [`now_ms`](Time::now_ms).
    pub fn now_as_ms(&self) -> u64 {
        self.now_ms()
    }

    /// Current elapsed time in nanoseconds.
    pub fn now_as_ns(&self) -> u64 {
        Instant::now().duration_since(self.start_time).as_nanos() as u64
    }

    /// Blocks the current thread for the given fractional seconds.
    pub fn sleep(&self, secs: f64) {
        std::thread::sleep(std::time::Duration::from_secs_f64(secs));
    }

    /// Blocks the current thread for the given milliseconds.
    pub fn sleep_ms(&self, millis: u64) {
        std::thread::sleep(std::time::Duration::from_millis(millis));
    }

    /// Blocks the current thread for the given nanoseconds.
    pub fn sleep_ns(&self, nanos: u64) {
        std::thread::sleep(std::time::Duration::from_nanos(nanos));
    }
}
