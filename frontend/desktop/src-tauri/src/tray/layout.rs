/// Gap between the monitor's left edge and the window.
pub const WINDOW_LEFT_MARGIN: i32 = 24;

/// Gap between the monitor's bottom edge and the window, leaving room for the
/// taskbar that hosts the tray icon.
pub const WINDOW_BOTTOM_MARGIN: i32 = 72;

/// A position in physical pixels, mirroring `tauri::PhysicalPosition<i32>`.
///
/// Keeping the geometry free of Tauri types lets the placement rule be tested
/// without a windowing system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Places the main window near the bottom-left corner of the given monitor.
///
/// `monitor_origin` is the monitor's top-left corner in the virtual desktop, so
/// the result is correct on secondary monitors with a non-zero origin.
pub fn main_window_position(
    monitor_origin: Point,
    monitor_height: u32,
    window_height: u32,
) -> Point {
    let bottom_aligned_y =
        monitor_origin.y + monitor_height as i32 - window_height as i32 - WINDOW_BOTTOM_MARGIN;

    Point {
        x: monitor_origin.x + WINDOW_LEFT_MARGIN,
        // A window taller than the monitor would otherwise be pushed above the
        // monitor's top edge, where its title bar cannot be reached.
        y: bottom_aligned_y.max(monitor_origin.y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGIN: Point = Point { x: 0, y: 0 };

    #[test]
    fn the_window_sits_above_the_taskbar_on_the_primary_monitor() {
        let position = main_window_position(ORIGIN, 1080, 600);

        assert_eq!(
            position,
            Point {
                x: WINDOW_LEFT_MARGIN,
                y: 1080 - 600 - WINDOW_BOTTOM_MARGIN,
            }
        );
    }

    #[test]
    fn the_monitor_origin_offsets_both_axes_on_a_secondary_monitor() {
        let origin = Point { x: 1920, y: -200 };

        let position = main_window_position(origin, 1080, 600);

        assert_eq!(
            position,
            Point {
                x: 1920 + WINDOW_LEFT_MARGIN,
                y: -200 + 1080 - 600 - WINDOW_BOTTOM_MARGIN,
            }
        );
    }

    #[test]
    fn a_window_taller_than_the_monitor_is_clamped_to_the_monitor_top() {
        let origin = Point { x: 10, y: 50 };

        let position = main_window_position(origin, 600, 1080);

        assert_eq!(position, Point { x: 34, y: 50 });
    }

    #[test]
    fn a_window_that_exactly_fills_the_usable_height_touches_the_monitor_top() {
        let position = main_window_position(ORIGIN, 1080, 1080 - WINDOW_BOTTOM_MARGIN as u32);

        assert_eq!(position.y, 0);
    }

    #[test]
    fn point_is_debug_copy_and_comparable() {
        let point = Point { x: 3, y: 4 };
        let copied = point;

        assert_eq!(copied, Point { x: 3, y: 4 });
        assert!(format!("{point:?}").contains('3'));
    }
}
