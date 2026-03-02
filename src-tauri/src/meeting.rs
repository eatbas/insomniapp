use crate::platform;

pub fn is_in_meeting() -> bool {
    platform::is_mic_active() || platform::is_camera_active()
}
