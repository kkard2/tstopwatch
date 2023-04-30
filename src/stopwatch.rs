use std::time::{Duration, Instant};

use chrono::Utc;

pub struct Stopwatch {
    duration: Duration,
    cur_start: Option<Instant>,
}

#[derive(Serialize, Deserialize)]
pub struct StopwatchSerializable {
    duration_secs: u64,
    duration_nanos: u32,
    cur_start_millis_utc: Option<i64>,
}

impl Default for StopwatchSerializable {
    fn default() -> Self {
        Self {
            duration_secs: 0,
            duration_nanos: 0,
            cur_start_millis_utc: None,
        }
    }
}

impl Stopwatch {
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

impl From<&Stopwatch> for StopwatchSerializable {
    fn from(stopwatch: &Stopwatch) -> Self {
        let duration = stopwatch.elapsed();
        StopwatchSerializable {
            duration_secs: duration.as_secs(),
            duration_nanos: duration.subsec_nanos(),
            cur_start_millis_utc: if stopwatch.is_running() {
                Some(Utc::now().timestamp_millis())
            } else {
                None
            },
        }
    }
}

impl From<&StopwatchSerializable> for Stopwatch {
    fn from(stopwatch: &StopwatchSerializable) -> Self {
        let now = Utc::now().timestamp_millis();
        let mut duration = Duration::new(stopwatch.duration_secs, stopwatch.duration_nanos);
        duration +=
            Duration::from_millis((now - stopwatch.cur_start_millis_utc.unwrap_or(now)) as u64);
        Stopwatch {
            duration,
            cur_start: if stopwatch.cur_start_millis_utc.is_some() {
                Some(Instant::now())
            } else {
                None
            },
        }
    }
}
