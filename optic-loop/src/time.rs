use std::time::Instant;

/// Frame timing data — delta time, smoothed FPS, and elapsed wall-clock time.
///
/// `Time` is updated once per frame by the engine (via [`Time::update`])
/// before user code runs. It provides three fundamental measurements:
///
/// | Field | Meaning | Use case |
/// |---|---|---|
/// | [`delta`](Time::delta) | Seconds since the last frame | Physics, movement (frame-rate independent) |
/// | [`fps`](Time::fps) | Smoothed frames-per-second | Display in HUD, performance monitoring |
/// | [`elapsed`](Time::elapsed) | Total seconds since start | Timers, countdowns, replays |
///
/// # FPS smoothing
///
/// The FPS value is a running average of the last **32** delta samples.
/// This avoids jarring frame-to-frame fluctuations while still responding
/// to sustained changes in frame rate. The smoothing window size is fixed.
///
/// # Which timing method should I use?
///
/// | You want… | Use |
/// |---|---|
/// | Frame-independent movement speed | [`delta`](Time::delta) |
/// | "Time since game started" | [`elapsed`](Time::elapsed) |
/// | Timestamp for logging / profiling | [`now_ms`](Time::now_ms) |
/// | "5 seconds from now" | `let deadline = time.elapsed() + 5.0;` |
///
/// # Example
///
/// ```ignore
/// use optic_loop::Time;
///
/// let mut time = Time::new();
///
/// // Simulate a 16 ms frame
/// std::thread::sleep(std::time::Duration::from_millis(16));
/// time.update();
///
/// println!("Delta: {:.4}s  FPS: {:.1}", time.delta(), time.fps());
/// // → "Delta: 0.0160s  FPS: 62.5"
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
    prev_deltas: Vec<f64>,
    prev_deltas_size: usize,
}

impl Time {
    /// Creates a new timer with all counters at zero.
    ///
    /// The internal `start_time` is set to the current wall-clock instant.
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
            prev_deltas: Vec::with_capacity(32),
            prev_deltas_size: 32,
        }
    }

    /// Advances the timer by one frame.
    ///
    /// Called automatically by the engine each frame. Increments
    /// `tick_count`, computes `delta` and `elapsed` from wall-clock time,
    /// and updates the smoothed FPS.
    ///
    /// You should not normally call this manually — [`Game`](crate::Game)
    /// and [`GameLoop`](crate::GameLoop) call it before invoking user code.
    pub fn update(&mut self) {
        self.tick_count += 1;
        self.local_tick += 1;
        let now = Instant::now();

        self.elapsed = now.duration_since(self.start_time).as_secs_f64();
        self.delta = now.duration_since(self.prev_time).as_secs_f64();
        self.prev_time = now;

        self.prev_deltas.push(self.delta);
        if self.prev_deltas.len() > self.prev_deltas_size {
            self.prev_deltas.remove(0);
        }

        let avg = self.prev_deltas.iter().sum::<f64>() / self.prev_deltas.len() as f64;
        self.fps = if avg > 0.0 { 1.0 / avg } else { 0.0 };

        if now.duration_since(self.prev_sec).as_secs_f64() >= 1.0 {
            self.local_tick = 0;
            self.prev_sec = now;
        }
    }

    /// Smoothed frames-per-second (averaged over the last 32 frames).
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
    pub fn elapsed(&self) -> f64 { self.elapsed }

    /// Current elapsed time in seconds (re-queries `Instant::now`).
    ///
    /// Unlike [`elapsed`](Time::elapsed), this always returns the very latest
    /// wall-clock time, even if `update` has not been called yet for this
    /// frame.
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
    ///
    /// Useful for frame-rate limiting in non-interactive contexts.
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
