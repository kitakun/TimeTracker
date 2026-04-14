use crate::platform;
use crate::platform::types::ActiveWindowInfo;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub const DEFAULT_IDLE_THRESHOLD_SECS: u64 = 300; // 5 minutes
pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 5;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrackingState {
    Running,
    Paused,
    Idle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySnapshot {
    pub timestamp: String,
    pub state: TrackingState,
    pub window: Option<ActiveWindowInfo>,
    pub idle_secs: u64,
}

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub idle_threshold_secs: u64,
    pub poll_interval_secs: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            idle_threshold_secs: DEFAULT_IDLE_THRESHOLD_SECS,
            poll_interval_secs: DEFAULT_POLL_INTERVAL_SECS,
        }
    }
}

#[derive(Debug)]
pub struct ActivityMonitor {
    pub config: MonitorConfig,
    state: Arc<Mutex<MonitorState>>,
}

#[derive(Debug)]
struct MonitorState {
    tracking: TrackingState,
    last_snapshot: Option<ActivitySnapshot>,
    manually_paused: bool,
    idle_detection_enabled: bool,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            tracking: TrackingState::default(),
            last_snapshot: None,
            manually_paused: false,
            idle_detection_enabled: true,
        }
    }
}

impl Default for TrackingState {
    fn default() -> Self {
        TrackingState::Paused
    }
}

impl ActivityMonitor {
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(MonitorState::default())),
        }
    }

    pub fn start(&self) -> tokio::sync::broadcast::Sender<ActivitySnapshot> {
        let (tx, _rx) = tokio::sync::broadcast::channel(64);
        let tx_clone = tx.clone();
        let config = self.config.clone();
        let state = Arc::clone(&self.state);

        thread::spawn(move || {
            // SystemTime is a wall-clock that advances during system sleep,
            // unlike Instant (which may be suspended while the CPU is halted).
            let mut last_poll_wall = std::time::SystemTime::now();

            loop {
                thread::sleep(Duration::from_secs(config.poll_interval_secs));

                let now_wall = std::time::SystemTime::now();
                let real_elapsed_secs = now_wall
                    .duration_since(last_poll_wall)
                    .unwrap_or_default()
                    .as_secs();
                last_poll_wall = now_wall;

                // If the real gap between polls is more than 3× the expected
                // interval, the system almost certainly woke from sleep.
                // In that case we suppress the idle signal so the in-progress
                // session is not discarded as "idle during sleep time".
                let just_woke_from_sleep =
                    real_elapsed_secs > config.poll_interval_secs.saturating_mul(3);

                let (manually_paused, idle_detection_enabled) = {
                    let s = state.lock().unwrap();
                    (s.manually_paused, s.idle_detection_enabled)
                };

                let idle_secs = if just_woke_from_sleep {
                    0 // treat wake-from-sleep as instant activity, not long idle
                } else {
                    platform::get_idle_seconds()
                };
                let window = platform::get_active_window();

                let tracking = if manually_paused {
                    TrackingState::Paused
                } else if idle_detection_enabled && idle_secs >= config.idle_threshold_secs {
                    TrackingState::Idle
                } else {
                    TrackingState::Running
                };

                let snapshot = ActivitySnapshot {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    state: tracking.clone(),
                    window,
                    idle_secs,
                };

                {
                    let mut s = state.lock().unwrap();
                    s.tracking = tracking;
                    s.last_snapshot = Some(snapshot.clone());
                }

                let _ = tx_clone.send(snapshot);
            }
        });

        tx
    }

    pub fn pause(&self) {
        let mut s = self.state.lock().unwrap();
        s.manually_paused = true;
        s.tracking = TrackingState::Paused;
    }

    pub fn resume(&self) {
        let mut s = self.state.lock().unwrap();
        s.manually_paused = false;
    }

    pub fn set_idle_detection_enabled(&self, enabled: bool) {
        self.state.lock().unwrap().idle_detection_enabled = enabled;
    }

    pub fn is_paused(&self) -> bool {
        self.state.lock().unwrap().manually_paused
    }

    pub fn current_state(&self) -> TrackingState {
        self.state.lock().unwrap().tracking.clone()
    }

    pub fn last_snapshot(&self) -> Option<ActivitySnapshot> {
        self.state.lock().unwrap().last_snapshot.clone()
    }
}
