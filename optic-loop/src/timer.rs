/// A countdown timer with optional repeat, polled explicitly each frame.
///
/// Created via [`Timer::new`] (one-shot) or [`Timer::new_repeating`].
/// Call [`tick`](Timer::tick) with per-frame delta time; the return value
/// is `true` on the frame the timer finishes.
///
/// # Example
///
/// ```ignore
/// use optic_loop::Timer;
///
/// let mut timer = Timer::new(2.0);
///
/// // Each frame:
/// if timer.tick(game.time.dt()) {
///     // 2 seconds have elapsed
/// }
/// ```
pub struct Timer {
    wait_time: f32,
    time_left: f32,
    repeating: bool,
    active: bool,
}

impl Timer {
    /// Creates a one-shot timer that fires after `wait_time` seconds.
    pub fn new(wait_time: f32) -> Self {
        let wt = wait_time.max(0.0);
        Self {
            wait_time: wt,
            time_left: wt,
            repeating: false,
            active: true,
        }
    }

    /// Creates a repeating timer that fires every `wait_time` seconds.
    ///
    /// On completion the timer auto-resets `time_left` to `wait_time`,
    /// matching Godot-style behavior.
    pub fn new_repeating(wait_time: f32) -> Self {
        let wt = wait_time.max(0.0);
        Self {
            wait_time: wt,
            time_left: wt,
            repeating: true,
            active: true,
        }
    }

    /// Advances the timer by `dt` seconds.
    ///
    /// Returns `true` if the timer finished this frame. For repeating timers,
    /// the timer auto-resets; call repeatedly in the same frame if you need
    /// to detect multi-fire.
    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.active || dt <= 0.0 {
            return false;
        }
        self.time_left -= dt;
        if self.time_left <= 0.0 {
            if self.repeating {
                self.time_left = self.wait_time;
            } else {
                self.active = false;
            }
            return true;
        }
        false
    }

    /// Reduces the remaining time by `amount` seconds.
    ///
    /// Crossing zero triggers the same completion logic as [`tick`](Timer::tick).
    /// Returns `true` if the timer finished as a result of this reduction.
    pub fn reduce(&mut self, amount: f32) -> bool {
        if !self.active || amount <= 0.0 {
            return false;
        }
        self.time_left -= amount;
        if self.time_left <= 0.0 {
            if self.repeating {
                self.time_left = self.wait_time;
            } else {
                self.active = false;
            }
            return true;
        }
        false
    }

    /// Resets the timer to its initial state.
    pub fn reset(&mut self) {
        self.time_left = self.wait_time;
        self.active = true;
    }

    /// Like [`tick`](Timer::tick) but also emits a named signal via
    /// [`Events::emit`](optic_window::Events::emit) when the timer finishes.
    ///
    /// Returns `true` if the timer finished this frame.
    pub fn tick_and_emit(&mut self, dt: f32, name: &str, events: &mut optic_window::Events) -> bool {
        let finished = self.tick(dt);
        if finished {
            events.emit(name);
        }
        finished
    }
}
