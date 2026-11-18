//! The pure decision behind the tray registration watcher.
//!
//! Windows drops every tray icon when Explorer restarts, and the tray library
//! adds its icon back with a single attempt whose failure it ignores. A shell
//! too busy to answer therefore leaves the app running with no icon, and this
//! app has no other way in: its window stays hidden until the icon is clicked.
//!
//! [`RegistrationTracker::observe`] takes one look at the registration and
//! answers with what to do about it. It performs nothing, so the rule is tested
//! without a windowing system; `watcher` is the adapter that acts on it.

use std::time::Duration;

/// How often a registered icon is looked at again.
pub const STEADY_CHECK_INTERVAL: Duration = Duration::from_secs(60);

/// Waits after the first failed looks, in order. Every tray call runs on the
/// main thread and a shell that does not answer can hold each one for seconds,
/// so the retry backs off; later looks use [`STEADY_CHECK_INTERVAL`].
pub const RETRY_DELAYS: [Duration; 4] = [
    Duration::from_secs(5),
    Duration::from_secs(10),
    Duration::from_secs(20),
    Duration::from_secs(40),
];

/// What one look at the icon's registration found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    /// The shell holds the icon.
    Registered,
    /// The shell does not hold the icon.
    Missing,
}

/// What the watcher does after one look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decision {
    /// Ask the shell to add the icon again.
    pub register_again: bool,
    /// How long to wait before the next look.
    pub next_check: Duration,
}

/// Counts the failed looks since the icon was last seen registered.
#[derive(Default)]
pub struct RegistrationTracker {
    failed_looks: usize,
}

impl RegistrationTracker {
    /// Decides what one look at the registration calls for.
    pub fn observe(&mut self, probe: Probe) -> Decision {
        match probe {
            Probe::Registered => {
                self.failed_looks = 0;
                Decision {
                    register_again: false,
                    next_check: STEADY_CHECK_INTERVAL,
                }
            }
            Probe::Missing => {
                let next_check = RETRY_DELAYS
                    .get(self.failed_looks)
                    .copied()
                    .unwrap_or(STEADY_CHECK_INTERVAL);
                self.failed_looks = self.failed_looks.saturating_add(1);
                Decision {
                    register_again: true,
                    next_check,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Feeds `count` failed looks in a row and returns each decision in order.
    fn missing_looks(tracker: &mut RegistrationTracker, count: usize) -> Vec<Decision> {
        (0..count)
            .map(|_| tracker.observe(Probe::Missing))
            .collect()
    }

    #[test]
    fn a_registered_icon_is_left_alone_and_looked_at_again_later() {
        let mut tracker = RegistrationTracker::default();

        for _ in 0..3 {
            assert_eq!(
                tracker.observe(Probe::Registered),
                Decision {
                    register_again: false,
                    next_check: STEADY_CHECK_INTERVAL,
                }
            );
        }
    }

    #[test]
    fn every_failed_look_asks_for_the_icon_again() {
        let mut tracker = RegistrationTracker::default();

        let decisions = missing_looks(&mut tracker, 20);

        assert!(decisions.iter().all(|decision| decision.register_again));
    }

    #[test]
    fn the_retry_backs_off_to_the_steady_interval_and_stays_there() {
        let mut tracker = RegistrationTracker::default();

        let waits: Vec<Duration> = missing_looks(&mut tracker, 7)
            .iter()
            .map(|decision| decision.next_check)
            .collect();

        assert_eq!(
            waits,
            [
                Duration::from_secs(5),
                Duration::from_secs(10),
                Duration::from_secs(20),
                Duration::from_secs(40),
                STEADY_CHECK_INTERVAL,
                STEADY_CHECK_INTERVAL,
                STEADY_CHECK_INTERVAL,
            ]
        );
    }

    #[test]
    fn the_retry_never_waits_longer_than_the_steady_interval() {
        assert!(RETRY_DELAYS
            .iter()
            .all(|delay| *delay <= STEADY_CHECK_INTERVAL));
    }

    #[test]
    fn a_restored_icon_is_left_alone_again() {
        let mut tracker = RegistrationTracker::default();
        missing_looks(&mut tracker, 3);

        assert_eq!(
            tracker.observe(Probe::Registered),
            Decision {
                register_again: false,
                next_check: STEADY_CHECK_INTERVAL,
            }
        );
    }

    #[test]
    fn a_second_loss_starts_the_back_off_over() {
        let mut tracker = RegistrationTracker::default();
        missing_looks(&mut tracker, 9);
        tracker.observe(Probe::Registered);

        assert_eq!(
            tracker.observe(Probe::Missing),
            Decision {
                register_again: true,
                next_check: RETRY_DELAYS[0],
            }
        );
    }

    #[test]
    fn the_failed_look_count_saturates_instead_of_wrapping() {
        let mut tracker = RegistrationTracker {
            failed_looks: usize::MAX,
        };

        assert_eq!(
            tracker.observe(Probe::Missing),
            Decision {
                register_again: true,
                next_check: STEADY_CHECK_INTERVAL,
            }
        );
        // A wrapped count would restart the back-off at its shortest wait.
        assert_eq!(
            tracker.observe(Probe::Missing).next_check,
            STEADY_CHECK_INTERVAL
        );
    }

    #[test]
    fn registration_types_are_debug_copy_and_comparable_for_diagnostics() {
        let probe = Probe::Missing;
        let copied_probe = probe;
        let decision = RegistrationTracker::default().observe(probe);
        let copied_decision = decision;

        assert_eq!(copied_probe, Probe::Missing);
        assert_ne!(copied_probe, Probe::Registered);
        assert_eq!(copied_decision, decision);
        assert!(format!("{:?}", Probe::Registered).contains("Registered"));
        assert!(format!("{probe:?}").contains("Missing"));
        assert!(format!("{decision:?}").contains("register_again"));
        assert!(format!("{decision:?}").contains("next_check"));
    }
}
