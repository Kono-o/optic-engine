use std::time::Instant;

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

    pub fn fps(&self) -> f64 { self.fps }
    pub fn delta(&self) -> f64 { self.delta }
    pub fn elapsed(&self) -> f64 { self.elapsed }

    pub fn now(&self) -> f64 {
        Instant::now().duration_since(self.start_time).as_secs_f64()
    }
    pub fn now_ms(&self) -> u64 {
        Instant::now().duration_since(self.start_time).as_millis() as u64
    }
    pub fn now_as_ms(&self) -> u64 {
        self.now_ms()
    }
    pub fn now_as_ns(&self) -> u64 {
        Instant::now().duration_since(self.start_time).as_nanos() as u64
    }

    pub fn sleep(&self, secs: f64) {
        std::thread::sleep(std::time::Duration::from_secs_f64(secs));
    }
    pub fn sleep_ms(&self, millis: u64) {
        std::thread::sleep(std::time::Duration::from_millis(millis));
    }
    pub fn sleep_ns(&self, nanos: u64) {
        std::thread::sleep(std::time::Duration::from_nanos(nanos));
    }
}
