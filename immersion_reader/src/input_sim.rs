// Simulator input backend for development/testing on macOS
// Uses keyboard instead of touchscreen

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::Duration;

pub use crate::input::TouchEvent;

pub struct InputHandler {
    // No device needed for simulator
}

impl InputHandler {
    pub fn new() -> Result<Self> {
        log::info!("Initializing simulator input (keyboard)");
        log::info!("Controls:");
        log::info!("  SPACE = Tap (play/pause)");
        log::info!("  → = Swipe Right (next chapter)");
        log::info!("  ← = Swipe Left (previous chapter)");
        log::info!("  ↑ = Swipe Up (skip forward)");
        log::info!("  ↓ = Swipe Down (skip backward)");
        log::info!("  Q/ESC = Quit");

        Ok(InputHandler {})
    }

    pub fn read_event(&mut self) -> Result<Option<TouchEvent>> {
        // Block until we get a key event
        loop {
            if let Ok(true) = event::poll(Duration::from_millis(100)) {
                if let Ok(Event::Key(key)) = event::read() {
                    return Ok(self.key_to_touch_event(key));
                }
            }
        }
    }

    pub fn poll_event(&mut self) -> Result<Option<TouchEvent>> {
        // Non-blocking check
        if let Ok(true) = event::poll(Duration::from_millis(0)) {
            if let Ok(Event::Key(key)) = event::read() {
                return Ok(self.key_to_touch_event(key));
            }
        }
        Ok(None)
    }

    fn key_to_touch_event(&self, key: KeyEvent) -> Option<TouchEvent> {
        match key.code {
            KeyCode::Char(' ') => {
                log::debug!("Key: SPACE -> Tap");
                Some(TouchEvent::Tap { x: 500, y: 500 })
            }
            KeyCode::Right => {
                log::debug!("Key: → -> Swipe Right");
                Some(TouchEvent::SwipeRight)
            }
            KeyCode::Left => {
                log::debug!("Key: ← -> Swipe Left");
                Some(TouchEvent::SwipeLeft)
            }
            KeyCode::Up => {
                log::debug!("Key: ↑ -> Swipe Up");
                Some(TouchEvent::SwipeUp)
            }
            KeyCode::Down => {
                log::debug!("Key: ↓ -> Swipe Down");
                Some(TouchEvent::SwipeDown)
            }
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                log::info!("Quit requested");
                std::process::exit(0);
            }
            _ => None,
        }
    }
}
