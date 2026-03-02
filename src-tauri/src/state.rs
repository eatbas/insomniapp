use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub enabled: bool,
    pub is_idle: bool,
    pub idle_seconds: u64,
    pub is_in_meeting: bool,
    pub is_simulating: bool,
    pub idle_threshold_secs: u64,
    pub simulation_interval_secs: u64,
}

impl Default for AppStatus {
    fn default() -> Self {
        Self {
            enabled: true,
            is_idle: false,
            idle_seconds: 0,
            is_in_meeting: false,
            is_simulating: false,
            idle_threshold_secs: 60,
            simulation_interval_secs: 30,
        }
    }
}

pub struct AppState {
    pub status: Mutex<AppStatus>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            status: Mutex::new(AppStatus::default()),
        }
    }
}
