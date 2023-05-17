use serde::Deserialize;

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use chrono::Utc;
use serde::Serialize;

const MAX_UNDO_STACK_SIZE: usize = 100;

#[derive(Default, Clone)]
pub struct Stopwatch {
    duration: Duration,
    cur_start: Option<Instant>,
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

impl Serialize for Stopwatch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serializable: StopwatchSerializable = self.into();
        serializable.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Stopwatch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        StopwatchSerializable::deserialize(deserializer).map(|s| s.into())
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

impl From<StopwatchSerializable> for Stopwatch {
    fn from(stopwatch: StopwatchSerializable) -> Self {
        Self::from(&stopwatch)
    }
}

#[derive(Default, Serialize, Deserialize)]
struct StopwatchSerializable {
    duration_secs: u64,
    duration_nanos: u32,
    cur_start_millis_utc: Option<i64>,
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

#[derive(Serialize, Deserialize)]
pub struct StopwatchStack {
    undo_stack: VecDeque<Stopwatch>,
    redo_stack: VecDeque<Stopwatch>,
    current: Stopwatch,
}

impl StopwatchStack {
    pub fn new() -> Self {
        Self {
            undo_stack: Default::default(),
            redo_stack: Default::default(),
            current: Default::default(),
        }
    }

    pub fn current(&self) -> &Stopwatch {
        &self.current
    }

    pub fn current_mut(&mut self) -> &mut Stopwatch {
        &mut self.current
    }

    pub fn push(&mut self) {
        self.undo_stack.push_back(self.current.clone());

        while self.undo_stack.len() > MAX_UNDO_STACK_SIZE {
            self.undo_stack.pop_front();
        }

        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(stopwatch) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(self.current.clone());
            self.current = stopwatch;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(stopwatch) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(self.current.clone());
            self.current = stopwatch;
            true
        } else {
            false
        }
    }
}

impl Default for StopwatchStack {
    fn default() -> Self {
        Self::new()
    }
}
