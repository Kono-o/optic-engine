//! Countdown timers with optional repeat, polled explicitly each frame.
//!
//! This crate provides [`Timer`] (single timer) and [`Timers`] (a managed
//! collection). Both are designed for frame-driven games where you call
//! [`tick`](Timer::tick) with the per-frame delta time.
//!
//! All time values use `f64` to match [`Time::delta()`](optic_loop::Time::delta).

use optic_core::{OpticError, OpticErrorKind, OpticResult};

/// A countdown timer with optional repeat, paused state, and query API.
///
/// Created via [`Timer::new`] (one-shot) or [`Timer::new_repeating`].
/// Call [`tick`](Timer::tick) with per-frame delta time; the return value
/// is `true` on the frame the timer finishes.
///
/// # States
///
/// A timer is in one of three states at any time:
///
/// | State | `is_running()` | `is_paused()` | `tick()` advances? |
/// |-------|----------------|---------------|---------------------|
/// | Running | `true` | `false` | Yes |
/// | Paused | `false` | `true` | No |
/// | Finished | `false` | `false` | No |
///
/// # Example
///
/// ```ignore
/// use optic_loop::Timer;
///
/// let mut timer = Timer::new(2.0);
///
/// // Each frame:
/// if timer.tick(game.time.delta()) {
///     // 2 seconds have elapsed
/// }
/// ```
pub struct Timer {
    wait_time: f64,
    time_left: f64,
    repeating: bool,
    active: bool,
    paused: bool,
}

impl Timer {
    /// Creates a one-shot timer that fires after `wait_time` seconds.
    pub fn new(wait_time: f64) -> Self {
        let wt = wait_time.max(0.0);
        Self {
            wait_time: wt,
            time_left: wt,
            repeating: false,
            active: true,
            paused: false,
        }
    }

    /// Creates a repeating timer that fires every `wait_time` seconds.
    ///
    /// On completion the timer auto-resets `time_left` to `wait_time`,
    /// matching Godot-style behavior.
    pub fn new_repeating(wait_time: f64) -> Self {
        let wt = wait_time.max(0.0);
        Self {
            wait_time: wt,
            time_left: wt,
            repeating: true,
            active: true,
            paused: false,
        }
    }

    /// Advances the timer by `dt` seconds.
    ///
    /// Returns `true` if the timer finished this frame. For repeating timers,
    /// the timer auto-resets; call repeatedly in the same frame if you need
    /// to detect multi-fire.
    ///
    /// Does nothing (returns `false`) if the timer is paused, finished, or
    /// `dt <= 0`.
    pub fn tick(&mut self, dt: f64) -> bool {
        if !self.active || self.paused || dt <= 0.0 {
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
    pub fn reduce(&mut self, amount: f64) -> bool {
        if !self.active || self.paused || amount <= 0.0 {
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

    /// Adds `amount` seconds to the remaining time, extending the timer.
    ///
    /// If the timer was finished, it re-activates with the extended time.
    pub fn extend(&mut self, amount: f64) {
        if amount <= 0.0 {
            return;
        }
        self.time_left += amount;
        self.active = true;
    }

    /// Resets the timer to its initial state (full wait time, un-paused, active).
    pub fn reset(&mut self) {
        self.time_left = self.wait_time;
        self.active = true;
        self.paused = false;
    }

    /// Like [`tick`](Timer::tick) but also emits a named signal via
    /// [`Events::emit`](optic_window::Events::emit) when the timer finishes.
    ///
    /// Returns `true` if the timer finished this frame.
    pub fn tick_and_emit(&mut self, dt: f64, name: &str, events: &mut optic_window::Events) -> bool {
        let finished = self.tick(dt);
        if finished {
            events.emit(name);
        }
        finished
    }

    /// Pauses the timer. A paused timer does not advance when [`tick`](Timer::tick)
    /// is called, but retains its remaining time.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes the timer from a paused state.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Returns `true` if the timer is actively counting down (not paused, not finished).
    pub fn is_running(&self) -> bool {
        self.active && !self.paused
    }

    /// Returns `true` if the timer is paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Returns `true` if the timer has not yet finished.
    ///
    /// A paused timer is still considered active. Use [`is_running`](Timer::is_running)
    /// to check if the timer is actively counting down.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Activates (un-pauses) the timer.
    pub fn start(&mut self) {
        self.active = true;
        self.paused = false;
    }

    /// Pauses the timer.
    pub fn stop(&mut self) {
        self.paused = true;
    }

    /// Returns `true` if the timer repeats after each completion.
    pub fn is_looping(&self) -> bool {
        self.repeating
    }

    /// Sets whether the timer repeats after each completion.
    pub fn set_looping(&mut self, repeating: bool) {
        self.repeating = repeating;
    }

    /// Returns the remaining time in seconds.
    pub fn time_left(&self) -> f64 {
        self.time_left
    }

    /// Returns the elapsed time (wait_time - time_left) in seconds.
    pub fn elapsed(&self) -> f64 {
        (self.wait_time - self.time_left).max(0.0)
    }

    /// Returns the completion progress as a `0.0..=1.0` ratio.
    pub fn progress(&self) -> f64 {
        if self.wait_time <= 0.0 {
            return if self.active { 0.0 } else { 1.0 };
        }
        (1.0 - self.time_left / self.wait_time).clamp(0.0, 1.0)
    }

    /// Returns the total wait time.
    pub fn wait_time(&self) -> f64 {
        self.wait_time
    }

    /// Sets a new wait time and resets the remaining time to the new value.
    pub fn set_wait_time(&mut self, wait_time: f64) {
        let wt = wait_time.max(0.0);
        self.wait_time = wt;
        self.time_left = wt;
        self.active = true;
        self.paused = false;
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
    /// Returns [`Err`] if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> OpticResult<()> {
        if index >= self.timers.len() {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                &format!("index {index} out of bounds (count: {})", self.timers.len()),
            ));
        }
        self.timers.remove(index);
        Ok(())
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
    pub fn tick_all(&mut self, dt: f64) -> Vec<usize> {
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
    pub fn tick_and_emit_all(&mut self, dt: f64, name: &str, events: &mut optic_window::Events) {
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
