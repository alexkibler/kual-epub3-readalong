// Simulator display backend for development/testing on macOS
// Uses terminal output to visualize the display

use anyhow::{Context, Result};
use crossterm::{
    cursor, execute, queue, style,
    terminal::{self, ClearType},
};
use std::io::{stdout, Write};
use std::sync::Mutex;

use crate::display::Rect;

pub struct Display {
    width: u32,
    height: u32,
    content: Mutex<Vec<String>>,
}

impl Display {
    pub fn new() -> Result<Self> {
        log::info!("Initializing simulator display (terminal)");

        // Set up terminal
        terminal::enable_raw_mode().context("Failed to enable raw mode")?;

        let (cols, rows) = terminal::size().unwrap_or((80, 24));

        let mut stdout = stdout();
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::Hide
        )?;

        log::info!("Simulator display: {} cols x {} rows", cols, rows);

        // Simulate Kindle screen dimensions in terminal cells
        // Kindle: 1448x1072, we'll map to terminal rows
        Ok(Display {
            width: cols as u32,
            height: rows as u32,
            content: Mutex::new(vec![String::new(); rows as usize]),
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn clear(&mut self) -> Result<()> {
        log::debug!("Clearing simulator display");

        let mut stdout = stdout();
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        let mut content = self.content.lock().unwrap();
        content.iter_mut().for_each(|line| line.clear());

        Ok(())
    }

    pub fn print_text(&mut self, text: &str, row: i16, col: i16, centered: bool) -> Result<()> {
        let mut stdout = stdout();

        let actual_row = row.max(0) as u16;
        let actual_col = if centered {
            let text_len = text.len() as u16;
            (self.width as u16 / 2).saturating_sub(text_len / 2)
        } else {
            col.max(0) as u16
        };

        queue!(
            stdout,
            cursor::MoveTo(actual_col, actual_row),
            style::Print(text)
        )?;

        stdout.flush()?;

        // Store in content buffer
        if let Ok(mut content) = self.content.lock() {
            if (actual_row as usize) < content.len() {
                content[actual_row as usize] = text.to_string();
            }
        }

        log::trace!("Printed: '{}' at ({}, {})", text, actual_row, actual_col);

        Ok(())
    }

    pub fn highlight_region(&mut self, rect: Rect) -> Result<()> {
        log::trace!(
            "HIGHLIGHT: ({}, {}) {}x{} [SIMULATED - showing as inverse]",
            rect.x,
            rect.y,
            rect.width,
            rect.height
        );

        // In simulator, we'll just print a visual indicator
        let mut stdout = stdout();

        // Draw a box to represent the highlight
        let row = (rect.y as f32 / 1072.0 * self.height as f32) as u16;
        let col = (rect.x as f32 / 1448.0 * self.width as f32) as u16;
        let width = (rect.width as f32 / 1448.0 * self.width as f32).max(1.0) as u16;

        queue!(
            stdout,
            cursor::MoveTo(col, row),
            style::SetAttribute(style::Attribute::Reverse),
            style::Print("▓".repeat(width as usize)),
            style::SetAttribute(style::Attribute::Reset)
        )?;

        stdout.flush()?;

        Ok(())
    }

    pub fn clear_highlight(&mut self, rect: Rect) -> Result<()> {
        log::trace!("CLEAR HIGHLIGHT: ({}, {}) {}x{}", rect.x, rect.y, rect.width, rect.height);

        // Clear the highlight by redrawing the area
        let mut stdout = stdout();

        let row = (rect.y as f32 / 1072.0 * self.height as f32) as u16;
        let col = (rect.x as f32 / 1448.0 * self.width as f32) as u16;
        let width = (rect.width as f32 / 1448.0 * self.width as f32).max(1.0) as u16;

        queue!(
            stdout,
            cursor::MoveTo(col, row),
            style::Print(" ".repeat(width as usize))
        )?;

        stdout.flush()?;

        Ok(())
    }

    pub fn full_refresh(&mut self) -> Result<()> {
        log::debug!("Full refresh (simulated)");

        let mut stdout = stdout();
        execute!(stdout, terminal::Clear(ClearType::All))?;

        Ok(())
    }

    pub fn load_font(&mut self, _font_path: &str) -> Result<()> {
        log::debug!("Font loading not needed in simulator");
        Ok(())
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        log::info!("Closing simulator display");
        let _ = terminal::disable_raw_mode();
        let _ = execute!(stdout(), cursor::Show);
    }
}
