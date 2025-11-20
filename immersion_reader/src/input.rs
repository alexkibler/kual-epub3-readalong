use anyhow::{Context, Result};
use evdev::{Device, EventType, InputEventKind, AbsoluteAxisType};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchEvent {
    Tap { x: i32, y: i32 },
    SwipeLeft,
    SwipeRight,
    SwipeUp,
    SwipeDown,
}

pub struct InputHandler {
    device: Device,
    last_x: i32,
    last_y: i32,
    touch_down: bool,
    start_x: i32,
    start_y: i32,
}

impl InputHandler {
    /// Initialize the input handler by discovering the touchscreen device
    pub fn new() -> Result<Self> {
        log::info!("Initializing touchscreen input handler");

        let device = Self::discover_touchscreen()
            .context("Failed to discover touchscreen device")?;

        log::info!("Using input device: {}", device.name().unwrap_or("unknown"));

        Ok(InputHandler {
            device,
            last_x: 0,
            last_y: 0,
            touch_down: false,
            start_x: 0,
            start_y: 0,
        })
    }

    /// Discover the touchscreen device by scanning /dev/input/event*
    fn discover_touchscreen() -> Result<Device> {
        log::debug!("Scanning for touchscreen input devices");

        let input_dir = PathBuf::from("/dev/input");

        if !input_dir.exists() {
            anyhow::bail!("/dev/input directory not found");
        }

        // Scan all event devices
        for entry in fs::read_dir(&input_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with("event") {
                    log::trace!("Checking device: {}", path.display());

                    if let Ok(device) = Device::open(&path) {
                        // Check if this device has touchscreen capabilities
                        if Self::is_touchscreen(&device) {
                            log::info!("Found touchscreen: {}", path.display());
                            return Ok(device);
                        }
                    }
                }
            }
        }

        anyhow::bail!("No touchscreen device found in /dev/input");
    }

    /// Check if a device is a touchscreen by examining its capabilities
    fn is_touchscreen(device: &Device) -> bool {
        // Check for absolute axis events (touchscreen characteristic)
        if let Some(abs_axes) = device.supported_absolute_axes() {
            // A touchscreen typically has ABS_X, ABS_Y, and ABS_PRESSURE
            let has_x = abs_axes.contains(AbsoluteAxisType::ABS_X) ||
                       abs_axes.contains(AbsoluteAxisType::ABS_MT_POSITION_X);
            let has_y = abs_axes.contains(AbsoluteAxisType::ABS_Y) ||
                       abs_axes.contains(AbsoluteAxisType::ABS_MT_POSITION_Y);

            if has_x && has_y {
                log::debug!(
                    "Device {} has touchscreen capabilities",
                    device.name().unwrap_or("unknown")
                );
                return true;
            }
        }

        false
    }

    /// Read and process the next touch event (blocking)
    pub fn read_event(&mut self) -> Result<Option<TouchEvent>> {
        // Fetch events from the device
        for ev in self.device.fetch_events()? {
            match ev.kind() {
                InputEventKind::AbsAxis(axis) => {
                    let value = ev.value();

                    match axis {
                        AbsoluteAxisType::ABS_X | AbsoluteAxisType::ABS_MT_POSITION_X => {
                            self.last_x = value;
                        }
                        AbsoluteAxisType::ABS_Y | AbsoluteAxisType::ABS_MT_POSITION_Y => {
                            self.last_y = value;
                        }
                        _ => {}
                    }
                }
                InputEventKind::Key(key) => {
                    // BTN_TOUCH indicates touch down/up
                    if key == evdev::Key::BTN_TOUCH {
                        if ev.value() == 1 {
                            // Touch down
                            self.touch_down = true;
                            self.start_x = self.last_x;
                            self.start_y = self.last_y;
                            log::trace!("Touch down at ({}, {})", self.start_x, self.start_y);
                        } else if ev.value() == 0 {
                            // Touch up - determine if it's a tap or swipe
                            self.touch_down = false;
                            let event = self.process_touch_up();
                            log::debug!("Touch event: {:?}", event);
                            return Ok(event);
                        }
                    }
                }
                InputEventKind::Synchronization(_) => {
                    // Sync event - marks the end of an input report
                    // We don't need to do anything here
                }
                _ => {}
            }
        }

        Ok(None)
    }

    /// Process touch up event to determine tap vs swipe
    fn process_touch_up(&self) -> Option<TouchEvent> {
        let dx = self.last_x - self.start_x;
        let dy = self.last_y - self.start_y;

        let distance = ((dx * dx + dy * dy) as f64).sqrt();

        // Threshold for distinguishing tap from swipe (in device units)
        const SWIPE_THRESHOLD: f64 = 100.0;
        const SWIPE_ANGLE_THRESHOLD: f64 = 0.5; // ~26 degrees

        if distance < SWIPE_THRESHOLD {
            // It's a tap
            return Some(TouchEvent::Tap {
                x: self.last_x,
                y: self.last_y,
            });
        }

        // It's a swipe - determine direction
        let abs_dx = dx.abs() as f64;
        let abs_dy = dy.abs() as f64;

        if abs_dx > abs_dy * SWIPE_ANGLE_THRESHOLD {
            // Horizontal swipe
            if dx > 0 {
                return Some(TouchEvent::SwipeRight);
            } else {
                return Some(TouchEvent::SwipeLeft);
            }
        } else if abs_dy > abs_dx * SWIPE_ANGLE_THRESHOLD {
            // Vertical swipe
            if dy > 0 {
                return Some(TouchEvent::SwipeDown);
            } else {
                return Some(TouchEvent::SwipeUp);
            }
        }

        None
    }

    /// Poll for events without blocking
    pub fn poll_event(&mut self) -> Result<Option<TouchEvent>> {
        // Try to read events without blocking
        match self.read_event() {
            Ok(event) => Ok(event),
            Err(e) => {
                // Check if it's a "would block" error
                if e.to_string().contains("EAGAIN") || e.to_string().contains("would block") {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run on actual hardware
    fn test_input_handler_init() {
        let handler = InputHandler::new();
        assert!(handler.is_ok());
    }
}
