//! Countdown timers with optional repeat, polled explicitly each frame.
//!
//! This crate provides [`Timer`] (single timer) and [`Timers`] (a managed
//! collection). Both are designed for frame-driven games where you call
//! [`tick`](Timer::tick) with the per-frame delta time.

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

    /// Returns `true` if the timer is currently running.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Activates the timer.
    pub fn start(&mut self) {
        self.active = true;
    }

    /// Deactivates the timer.
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Returns the total wait time.
    pub fn wait_time(&self) -> f32 {
        self.wait_time
    }

    /// Sets a new wait time and resets the remaining time to the new value.
    pub fn set_wait_time(&mut self, wait_time: f32) {
        let wt = wait_time.max(0.0);
        self.wait_time = wt;
        self.time_left = wt;
    }
}

/// A managed collection of timers that can be ticked as a group.
///
/// Provides batch operations for ticking all timers at once, emitting signals, and adding/removing
/// individual timers by index. Use this when you need to track many timed events in your game.
pub struct Timers {
    timers: Vec<Timer>,
}

impl Timers {
    /// Creates an empty timer collection.
    pub fn new() -> Self {
        Self { timers: Vec::new() }
    }

    /// Adds a timer to the collection.
    pub fn add(&mut self, timer: Timer) {
        self.timers.push(timer);
    }

    /// Removes the timer at the given index.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) {
        self.timers.remove(index);
    }

    /// Removes all timers.
    pub fn clear(&mut self) {
        self.timers.clear();
    }

    /// Returns the number of timers.
    pub fn len(&self) -> usize {
        self.timers.len()
    }

    /// Returns `true` if there are no timers.
    pub fn is_empty(&self) -> bool {
        self.timers.is_empty()
    }

    /// Returns a reference to the timer at the given index, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&Timer> {
        self.timers.get(index)
    }

    /// Returns a mutable reference to the timer at the given index, or `None` if out of bounds.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Timer> {
        self.timers.get_mut(index)
    }

    /// Ticks all active timers by `dt` seconds.
    ///
    /// Returns a vector of indices whose timers elapsed this frame.
    pub fn tick_all(&mut self, dt: f32) -> Vec<usize> {
        let mut elapsed = Vec::new();
        for (i, timer) in self.timers.iter_mut().enumerate() {
            if timer.tick(dt) {
                elapsed.push(i);
            }
        }
        elapsed
    }

    /// Ticks all active timers by `dt` seconds and emits a named signal for
    /// each timer that elapsed.
    pub fn tick_and_emit_all(&mut self, dt: f32, name: &str, events: &mut optic_window::Events) {
        for timer in self.timers.iter_mut() {
            timer.tick_and_emit(dt, name, events);
        }
    }

    /// Returns an iterator over all timers.
    pub fn iter(&self) -> impl Iterator<Item = &Timer> {
        self.timers.iter()
    }

    /// Returns a mutable iterator over all timers.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Timer> {
        self.timers.iter_mut()
    }
}
