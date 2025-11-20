use anyhow::{Context, Result};
use libc::{c_int, open, O_RDWR};
use std::ffi::CString;
use std::ptr;

// Include the generated FBInk bindings
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod fbink_bindings {
    include!(concat!(env!("OUT_DIR"), "/fbink_bindings.rs"));
}

use fbink_bindings::*;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

pub struct Display {
    fbfd: c_int,
    config: FBInkConfig,
    state: FBInkState,
}

impl Display {
    /// Initialize the display subsystem
    pub fn new() -> Result<Self> {
        log::info!("Initializing FBInk display subsystem");

        // Open the framebuffer device
        let fb_path = CString::new("/dev/fb0").unwrap();
        let fbfd = unsafe { open(fb_path.as_ptr(), O_RDWR) };

        if fbfd < 0 {
            anyhow::bail!("Failed to open /dev/fb0");
        }

        // Initialize FBInk config with defaults
        let mut config: FBInkConfig = unsafe { std::mem::zeroed() };
        config.wfm_mode = FBINK_WAVEFORM_AUTO as i8;
        config.is_cleared = false;
        config.is_centered = false;

        // Initialize FBInk
        let ret = unsafe { fbink_init(fbfd, &config) };
        if ret < 0 {
            anyhow::bail!("Failed to initialize FBInk: {}", ret);
        }

        // Get device state
        let mut state: FBInkState = unsafe { std::mem::zeroed() };
        unsafe {
            fbink_get_state(&config, &mut state);
        }

        log::info!(
            "Display initialized: {}x{} @ {} dpi, device: {}",
            state.screen_width,
            state.screen_height,
            state.screen_dpi,
            unsafe { std::ffi::CStr::from_ptr(state.device_name.as_ptr()) }
                .to_string_lossy()
        );

        Ok(Display {
            fbfd,
            config,
            state,
        })
    }

    /// Get the screen dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.state.screen_width, self.state.screen_height)
    }

    /// Clear the screen with a full refresh (GC16 waveform)
    pub fn clear(&mut self) -> Result<()> {
        log::debug!("Clearing screen with GC16 waveform");

        let mut config = self.config;
        config.is_cleared = true;
        config.wfm_mode = FBINK_WAVEFORM_GC16 as i8;

        let ret = unsafe { fbink_cls(self.fbfd, &config, ptr::null()) };

        if ret < 0 {
            anyhow::bail!("Failed to clear screen: {}", ret);
        }

        Ok(())
    }

    /// Print text to the screen
    pub fn print_text(&mut self, text: &str, row: i16, col: i16, centered: bool) -> Result<()> {
        let mut config = self.config;
        config.row = row;
        config.col = col;
        config.is_centered = centered;
        config.wfm_mode = FBINK_WAVEFORM_GC16 as i8;

        let c_text = CString::new(text).context("Invalid text string")?;
        let ret = unsafe { fbink_print(self.fbfd, c_text.as_ptr(), &config) };

        if ret < 0 {
            anyhow::bail!("Failed to print text: {}", ret);
        }

        Ok(())
    }

    /// Highlight a region by inverting pixels (A2 waveform for speed)
    pub fn highlight_region(&mut self, rect: Rect) -> Result<()> {
        log::trace!(
            "Highlighting region: ({}, {}) {}x{}",
            rect.x,
            rect.y,
            rect.width,
            rect.height
        );

        let mut config = self.config;
        config.is_inverted = true;
        config.wfm_mode = FBINK_WAVEFORM_A2 as i8; // Fast 2-bit update
        config.no_refresh = false;

        let ret = unsafe {
            fbink_refresh(
                self.fbfd,
                rect.y as u32,
                rect.x as u32,
                rect.width as u32,
                rect.height as u32,
                &config,
            )
        };

        if ret < 0 {
            anyhow::bail!("Failed to highlight region: {}", ret);
        }

        Ok(())
    }

    /// Clear a previously highlighted region
    pub fn clear_highlight(&mut self, rect: Rect) -> Result<()> {
        log::trace!(
            "Clearing highlight: ({}, {}) {}x{}",
            rect.x,
            rect.y,
            rect.width,
            rect.height
        );

        let mut config = self.config;
        config.wfm_mode = FBINK_WAVEFORM_GC16 as i8; // Full quality refresh
        config.no_refresh = false;

        let ret = unsafe {
            fbink_refresh(
                self.fbfd,
                rect.y as u32,
                rect.x as u32,
                rect.width as u32,
                rect.height as u32,
                &config,
            )
        };

        if ret < 0 {
            anyhow::bail!("Failed to clear highlight: {}", ret);
        }

        Ok(())
    }

    /// Force a full screen refresh to remove ghosting
    pub fn full_refresh(&mut self) -> Result<()> {
        log::debug!("Performing full screen refresh (GC16)");

        let mut config = self.config;
        config.wfm_mode = FBINK_WAVEFORM_GC16 as i8;
        config.is_flashing = true; // Force a flashing update

        let ret = unsafe {
            fbink_refresh(
                self.fbfd,
                0,
                0,
                self.state.screen_width,
                self.state.screen_height,
                &config,
            )
        };

        if ret < 0 {
            anyhow::bail!("Failed to perform full refresh: {}", ret);
        }

        Ok(())
    }

    /// Load a TrueType/OpenType font for rendering
    pub fn load_font(&mut self, font_path: &str) -> Result<()> {
        log::info!("Loading font: {}", font_path);

        let c_path = CString::new(font_path).context("Invalid font path")?;
        let ret = unsafe { fbink_add_ot_font(c_path.as_ptr(), self.fbfd) };

        if ret < 0 {
            anyhow::bail!("Failed to load font {}: {}", font_path, ret);
        }

        Ok(())
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        log::info!("Closing FBInk display");
        unsafe {
            fbink_close(self.fbfd);
            libc::close(self.fbfd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run on actual hardware
    fn test_display_init() {
        let display = Display::new();
        assert!(display.is_ok());
    }
}
