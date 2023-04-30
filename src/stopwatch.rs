use std::time::{Duration, Instant};

pub struct Stopwatch {
    duration: Duration,
    cur_start: Option<Instant>,
}

impl Stopwatch {
    pub fn new() -> Stopwatch {
        Stopwatch {
            duration: Duration::new(0, 0),
            cur_start: None,
        }
    }

    pub fn start(&mut self) {
        self.cur_start = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        if let Some(start) = self.cur_start {
            self.duration += start.elapsed();
            self.cur_start = None;
        }
    }

    pub fn reset(&mut self) {
        self.duration = Duration::new(0, 0);
        self.cur_start = None;
    }

    pub fn elapsed(&self) -> Duration {
        let mut duration = self.duration;
        if let Some(start) = self.cur_start {
            duration += start.elapsed();
        }
        duration
    }

    pub fn is_running(&self) -> bool {
        self.cur_start.is_some()
    }
}
