use std::process::Command;

extern "C" {
    fn CGEventSourceSecondsSinceLastEventType(
        source_state_id: i32,
        event_type: u64,
    ) -> f64;
}

const CG_EVENT_SOURCE_STATE_COMBINED_SESSION: i32 = 0;
const CG_ANY_INPUT_EVENT_TYPE: u64 = u64::MAX;

pub fn get_idle_seconds() -> u64 {
    unsafe {
        CGEventSourceSecondsSinceLastEventType(
            CG_EVENT_SOURCE_STATE_COMBINED_SESSION,
            CG_ANY_INPUT_EVENT_TYPE,
        ) as u64
    }
}

pub fn is_mic_active() -> bool {
    // Check if any audio input device is actively recording via CoreAudio
    // For simplicity, check if any process has an active audio input session
    if let Ok(output) = Command::new("sh")
        .args(["-c", "ioreg -c AppleHDAEngineInput | grep -i 'IOAudioEngineState' | grep '1'"])
        .output()
    {
        if output.status.success() && !output.stdout.is_empty() {
            return true;
        }
    }

    // Fallback: check via process list for common audio capture indicators
    false
}

pub fn is_camera_active() -> bool {
    // VDCAssistant runs when the built-in camera is active
    if let Ok(output) = Command::new("pgrep").args(["-x", "VDCAssistant"]).output() {
        if output.status.success() {
            return true;
        }
    }
    // AppleCameraAssistant on newer macOS versions
    if let Ok(output) = Command::new("pgrep")
        .args(["-x", "AppleCameraAssistant"])
        .output()
    {
        if output.status.success() {
            return true;
        }
    }
    false
}
