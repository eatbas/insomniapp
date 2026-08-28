use crate::state::AppStatus;

/// How often the engine samples the operating system for idle state.
pub const CHECK_INTERVAL_SECS: u64 = 3;

/// Window after a simulated keypress during which a low OS idle counter is
/// attributed to our own `F15` rather than to the user.
///
/// This must stay larger than [`CHECK_INTERVAL_SECS`], otherwise a simulated
/// keypress could land between two ticks and be mistaken for real user input.
pub const USER_ACTIVITY_GRACE_SECS: u64 = 5;

const _: () = assert!(
    USER_ACTIVITY_GRACE_SECS > CHECK_INTERVAL_SECS,
    "the grace window must span at least one full tick"
);

/// The operating-system facts sampled at the start of a single engine tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineTick {
    /// Seconds since the OS last saw any user input.
    pub os_idle_secs: u64,
    pub is_session_locked: bool,
    pub is_display_on: bool,
    /// Seconds since the engine last simulated a keypress.
    pub secs_since_last_simulate: u64,
    /// Seconds since the engine started tracking the current idle period, or
    /// `None` while no idle period is being tracked.
    pub tracked_idle_secs: Option<u64>,
}

/// What the caller must do with its tracked idle start once the tick is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleTracking {
    /// Leave the tracked idle start as it is.
    Keep,
    /// Forget the tracked idle start.
    Clear,
    /// Begin tracking, backdated by the supplied number of seconds so the
    /// recorded idle period includes the time the OS had already counted.
    StartBackdated(u64),
}

/// The outcome of a single engine tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineDecision {
    pub should_simulate: bool,
    /// Whether the operating system's own power idle timers should be reset on
    /// this tick, holding off sleep and display blanking without injecting any
    /// input into the session.
    pub should_hold_awake: bool,
    pub idle_tracking: IdleTracking,
}

/// Advances `status` by one tick and reports what the caller must do next.
///
/// This is the whole decision procedure of the keep-awake engine, expressed as
/// a pure function of the sampled [`EngineTick`] and the current [`AppStatus`],
/// so that the surrounding async loop stays a thin adapter over the clock, the
/// operating system, and the input simulator.
pub fn evaluate_tick(tick: &EngineTick, status: &mut AppStatus) -> EngineDecision {
    // Genuine user input shows up as a low OS idle counter. Our own simulated
    // keypress resets that same counter, so only trust the signal once the
    // grace window following the last simulation has elapsed.
    let user_became_active = tick.os_idle_secs < USER_ACTIVITY_GRACE_SECS
        && tick.secs_since_last_simulate > USER_ACTIVITY_GRACE_SECS;

    let tracked_idle_secs = if user_became_active {
        None
    } else {
        tick.tracked_idle_secs
    };

    // Once an idle period is being tracked it, rather than the OS counter,
    // defines how long the session has been idle: the OS counter is reset by
    // every keypress the engine simulates.
    let effective_idle_secs = tracked_idle_secs.unwrap_or(tick.os_idle_secs);

    status.idle_seconds = effective_idle_secs;
    status.is_idle = effective_idle_secs >= status.idle_threshold_secs;
    status.is_session_locked = tick.is_session_locked;
    status.is_display_off = !tick.is_display_on;

    let mut idle_tracking = if user_became_active {
        IdleTracking::Clear
    } else {
        IdleTracking::Keep
    };

    if status.is_idle && tracked_idle_secs.is_none() {
        idle_tracking = IdleTracking::StartBackdated(tick.os_idle_secs);
    }

    if !status.enabled {
        idle_tracking = IdleTracking::Clear;
    }

    status.is_simulating =
        status.enabled && status.is_idle && !status.is_session_locked && !status.is_display_off;

    // The power hold is deliberately independent of `is_idle`: the operating
    // system's sleep and display timers run from the same input counter the app
    // watches, so they must be held from the moment the app is enabled rather
    // than from the moment its own threshold is crossed. It still yields to a
    // locked session and to a display the user has switched off, so the app
    // never fights a deliberate lock or blank.
    let should_hold_awake = status.enabled && !tick.is_session_locked && tick.is_display_on;

    EngineDecision {
        should_simulate: status.is_simulating
            && tick.secs_since_last_simulate >= status.simulation_interval_secs,
        should_hold_awake,
        idle_tracking,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tick describing a long-idle, unlocked session on a live display, far
    /// enough past the last simulation that the interval has elapsed.
    fn idle_tick() -> EngineTick {
        EngineTick {
            os_idle_secs: 60,
            is_session_locked: false,
            is_display_on: true,
            secs_since_last_simulate: 30,
            tracked_idle_secs: Some(60),
        }
    }

    #[test]
    fn a_disabled_app_never_simulates_and_clears_idle_tracking() {
        let mut status = AppStatus {
            enabled: false,
            ..AppStatus::default()
        };

        let decision = evaluate_tick(&idle_tick(), &mut status);

        assert!(!decision.should_simulate);
        assert!(!status.is_simulating);
        assert_eq!(decision.idle_tracking, IdleTracking::Clear);
        // The status still reports the observed idle time while disabled.
        assert!(status.is_idle);
        assert_eq!(status.idle_seconds, 60);
    }

    #[test]
    fn below_the_idle_threshold_the_session_is_not_idle() {
        let mut status = AppStatus::default();
        let tick = EngineTick {
            os_idle_secs: 29,
            tracked_idle_secs: None,
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        assert!(!status.is_idle);
        assert!(!decision.should_simulate);
        assert_eq!(decision.idle_tracking, IdleTracking::Keep);
    }

    #[test]
    fn crossing_the_threshold_starts_backdated_idle_tracking() {
        let mut status = AppStatus::default();
        let tick = EngineTick {
            os_idle_secs: 31,
            tracked_idle_secs: None,
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        assert!(status.is_idle);
        assert_eq!(status.idle_seconds, 31);
        assert_eq!(decision.idle_tracking, IdleTracking::StartBackdated(31));
        assert!(decision.should_simulate);
    }

    #[test]
    fn an_already_tracked_idle_period_is_kept() {
        let mut status = AppStatus::default();

        let decision = evaluate_tick(&idle_tick(), &mut status);

        assert_eq!(decision.idle_tracking, IdleTracking::Keep);
    }

    #[test]
    fn a_locked_session_reports_idle_but_never_simulates() {
        let mut status = AppStatus::default();
        let tick = EngineTick {
            is_session_locked: true,
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        assert!(status.is_session_locked);
        assert!(status.is_idle);
        assert!(!status.is_simulating);
        assert!(!decision.should_simulate);
    }

    #[test]
    fn an_off_display_reports_idle_but_never_simulates() {
        let mut status = AppStatus::default();
        let tick = EngineTick {
            is_display_on: false,
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        assert!(status.is_display_off);
        assert!(!status.is_simulating);
        assert!(!decision.should_simulate);
    }

    #[test]
    fn the_simulation_interval_suppresses_simulation_until_it_elapses() {
        let mut status = AppStatus::default();
        let tick = EngineTick {
            secs_since_last_simulate: status.simulation_interval_secs - 1,
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        // Still "simulating" from the user's point of view, just not this tick.
        assert!(status.is_simulating);
        assert!(!decision.should_simulate);
    }

    #[test]
    fn simulation_fires_once_the_interval_has_exactly_elapsed() {
        let mut status = AppStatus::default();
        let tick = EngineTick {
            secs_since_last_simulate: status.simulation_interval_secs,
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        assert!(decision.should_simulate);
    }

    #[test]
    fn real_user_activity_outside_the_grace_window_clears_idle_tracking() {
        let mut status = AppStatus::default();
        let tick = EngineTick {
            os_idle_secs: USER_ACTIVITY_GRACE_SECS - 1,
            secs_since_last_simulate: USER_ACTIVITY_GRACE_SECS + 1,
            tracked_idle_secs: Some(120),
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        assert_eq!(decision.idle_tracking, IdleTracking::Clear);
        // Idle time falls back to the OS counter now tracking has been dropped.
        assert_eq!(status.idle_seconds, USER_ACTIVITY_GRACE_SECS - 1);
        assert!(!status.is_idle);
        assert!(!decision.should_simulate);
    }

    #[test]
    fn a_low_idle_counter_inside_the_grace_window_is_our_own_keypress() {
        let mut status = AppStatus::default();
        let tick = EngineTick {
            os_idle_secs: 0,
            secs_since_last_simulate: USER_ACTIVITY_GRACE_SECS,
            tracked_idle_secs: Some(120),
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        // Tracking survives, so the session stays idle across our own keypress.
        assert_eq!(decision.idle_tracking, IdleTracking::Keep);
        assert_eq!(status.idle_seconds, 120);
        assert!(status.is_idle);
    }

    #[test]
    fn a_high_idle_counter_outside_the_grace_window_is_not_user_activity() {
        let mut status = AppStatus::default();
        let tick = EngineTick {
            os_idle_secs: USER_ACTIVITY_GRACE_SECS,
            secs_since_last_simulate: USER_ACTIVITY_GRACE_SECS + 1,
            tracked_idle_secs: Some(120),
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        assert_eq!(decision.idle_tracking, IdleTracking::Keep);
        assert_eq!(status.idle_seconds, 120);
    }

    #[test]
    fn user_activity_that_also_crosses_the_threshold_restarts_tracking() {
        // Contrived but reachable: activity clears tracking, and the OS idle
        // counter is itself already past the threshold.
        let mut status = AppStatus {
            idle_threshold_secs: 2,
            ..AppStatus::default()
        };
        let tick = EngineTick {
            os_idle_secs: 4,
            secs_since_last_simulate: USER_ACTIVITY_GRACE_SECS + 1,
            tracked_idle_secs: Some(120),
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        assert_eq!(decision.idle_tracking, IdleTracking::StartBackdated(4));
        assert_eq!(status.idle_seconds, 4);
    }

    #[test]
    fn every_status_field_is_refreshed_from_the_tick() {
        let mut status = AppStatus::default();
        let tick = EngineTick {
            os_idle_secs: 7,
            is_session_locked: true,
            is_display_on: false,
            secs_since_last_simulate: 99,
            tracked_idle_secs: Some(45),
        };

        evaluate_tick(&tick, &mut status);

        assert_eq!(status.idle_seconds, 45);
        assert!(status.is_idle);
        assert!(status.is_session_locked);
        assert!(status.is_display_off);
        assert!(!status.is_simulating);
    }

    #[test]
    fn engine_types_are_debug_copy_and_comparable_for_diagnostics() {
        let tick = idle_tick();
        let copied = tick;
        let decision = EngineDecision {
            should_simulate: true,
            should_hold_awake: true,
            idle_tracking: IdleTracking::StartBackdated(3),
        };
        let same_decision = EngineDecision {
            should_simulate: true,
            should_hold_awake: true,
            idle_tracking: IdleTracking::StartBackdated(3),
        };

        assert_eq!(copied, idle_tick());
        assert_eq!(decision, same_decision);
        assert!(format!("{tick:?}").contains("os_idle_secs"));
        assert!(format!("{decision:?}").contains("should_simulate"));
        assert!(format!("{decision:?}").contains("should_hold_awake"));
        assert!(format!("{:?}", IdleTracking::Keep).contains("Keep"));
        assert!(format!("{:?}", IdleTracking::Clear).contains("Clear"));
    }

    #[test]
    fn an_enabled_unlocked_session_holds_the_power_state() {
        let mut status = AppStatus::default();

        let decision = evaluate_tick(&idle_tick(), &mut status);

        assert!(decision.should_hold_awake);
    }

    #[test]
    fn the_power_state_is_held_before_the_idle_threshold_is_crossed() {
        // The OS sleep timer runs from the same input counter as the engine's
        // own idle tracking, so the hold must not wait for the threshold.
        let mut status = AppStatus::default();
        let tick = EngineTick {
            os_idle_secs: 0,
            tracked_idle_secs: None,
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        assert!(!status.is_idle);
        assert!(decision.should_hold_awake);
    }

    #[test]
    fn a_disabled_app_never_holds_the_power_state() {
        let mut status = AppStatus {
            enabled: false,
            ..AppStatus::default()
        };

        let decision = evaluate_tick(&idle_tick(), &mut status);

        assert!(!decision.should_hold_awake);
    }

    #[test]
    fn a_locked_session_never_holds_the_power_state() {
        let mut status = AppStatus::default();
        let tick = EngineTick {
            is_session_locked: true,
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        assert!(!decision.should_hold_awake);
    }

    #[test]
    fn an_off_display_never_holds_the_power_state() {
        let mut status = AppStatus::default();
        let tick = EngineTick {
            is_display_on: false,
            ..idle_tick()
        };

        let decision = evaluate_tick(&tick, &mut status);

        assert!(!decision.should_hold_awake);
    }
}
