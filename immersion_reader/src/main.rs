mod audio;
mod display;
mod input;
mod parser;

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use audio::AudioPlayer;
use display::{Display, Rect};
use input::{InputHandler, TouchEvent};
use parser::{Chapter, EpubDocument, SyncNode};

/// Main application state
struct ImmersionReader {
    display: Display,
    audio: AudioPlayer,
    input: InputHandler,
    epub: EpubDocument,
    current_chapter: usize,
    current_highlight: Option<Rect>,
}

impl ImmersionReader {
    /// Initialize the application
    fn new(epub_path: PathBuf) -> Result<Self> {
        log::info!("Initializing Immersion Reader");

        // Initialize subsystems
        let display = Display::new().context("Failed to initialize display")?;
        let audio = AudioPlayer::new().context("Failed to initialize audio")?;
        let input = InputHandler::new().context("Failed to initialize input")?;
        let epub = EpubDocument::open(&epub_path).context("Failed to open EPUB")?;

        Ok(ImmersionReader {
            display,
            audio,
            input,
            epub,
            current_chapter: 0,
            current_highlight: None,
        })
    }

    /// Run the main application loop
    fn run(&mut self) -> Result<()> {
        log::info!("Starting Immersion Reader");

        // Show the title screen
        self.show_title_screen()?;

        // Main reading loop
        loop {
            // Load the current chapter
            if let Err(e) = self.load_chapter(self.current_chapter) {
                log::error!("Failed to load chapter: {}", e);
                self.display.print_text(&format!("Error: {}", e), 10, 0, true)?;
                thread::sleep(Duration::from_secs(3));
                break;
            }

            // Play the chapter with synchronization
            match self.play_chapter_with_sync() {
                Ok(ChapterResult::Completed) => {
                    log::info!("Chapter completed");
                    self.current_chapter += 1;
                    if self.current_chapter >= self.epub.chapters.len() {
                        log::info!("Reached end of book");
                        break;
                    }
                }
                Ok(ChapterResult::NextChapter) => {
                    self.current_chapter += 1;
                    if self.current_chapter >= self.epub.chapters.len() {
                        self.current_chapter = self.epub.chapters.len() - 1;
                    }
                }
                Ok(ChapterResult::PreviousChapter) => {
                    if self.current_chapter > 0 {
                        self.current_chapter -= 1;
                    }
                }
                Ok(ChapterResult::Exit) => {
                    log::info!("User requested exit");
                    break;
                }
                Err(e) => {
                    log::error!("Error playing chapter: {}", e);
                    break;
                }
            }
        }

        // Cleanup
        self.display.clear()?;
        self.display.print_text("Goodbye!", 10, 0, true)?;
        thread::sleep(Duration::from_secs(1));

        Ok(())
    }

    /// Show the title screen
    fn show_title_screen(&mut self) -> Result<()> {
        self.display.clear()?;
        self.display.print_text(&self.epub.title, 5, 0, true)?;
        self.display.print_text("Tap to start reading", 7, 0, true)?;
        self.display.print_text(&format!("{} chapters", self.epub.chapters.len()), 9, 0, true)?;

        // Wait for tap
        loop {
            if let Some(TouchEvent::Tap { .. }) = self.input.read_event()? {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    }

    /// Load a chapter and prepare for playback
    fn load_chapter(&mut self, chapter_idx: usize) -> Result<()> {
        if chapter_idx >= self.epub.chapters.len() {
            anyhow::bail!("Chapter index out of bounds");
        }

        let chapter = &self.epub.chapters[chapter_idx];
        log::info!("Loading chapter {}: {}", chapter_idx, chapter.title);

        // Clear the display
        self.display.clear()?;
        self.display.print_text(&chapter.title, 2, 0, true)?;

        // TODO: Render XHTML content to the display
        // For MVP, we'll show a simplified version
        self.display.print_text("Chapter content...", 5, 2, false)?;

        // Load audio if available
        if let Some(ref audio_path) = chapter.audio_path {
            self.audio.load(audio_path)?;
        } else if let Some(ref smil_path) = chapter.smil_path {
            // Parse SMIL to find audio file
            let sync_nodes = EpubDocument::parse_smil(smil_path)?;
            if let Some(node) = sync_nodes.first() {
                let audio_path = self.epub.get_resource_path(&node.audio_src);
                self.audio.load(&audio_path)?;
            }
        }

        Ok(())
    }

    /// Play chapter with audio-text synchronization
    fn play_chapter_with_sync(&mut self) -> Result<ChapterResult> {
        log::info!("Playing chapter with synchronization");

        let chapter = self.epub.chapters[self.current_chapter].clone();

        // Parse SMIL if available
        let sync_nodes = if let Some(ref smil_path) = chapter.smil_path {
            EpubDocument::parse_smil(smil_path)?
        } else {
            log::warn!("No SMIL file available for this chapter");
            Vec::new()
        };

        // Start playback
        self.audio.play()?;

        let mut current_node_idx = 0;

        // Main synchronization loop
        loop {
            // Check for input events (non-blocking)
            if let Some(event) = self.input.poll_event()? {
                match event {
                    TouchEvent::Tap { .. } => {
                        // Toggle play/pause
                        if self.audio.is_playing() {
                            log::info!("Pausing playback");
                            self.audio.pause()?;
                        } else {
                            log::info!("Resuming playback");
                            self.audio.play()?;
                        }
                    }
                    TouchEvent::SwipeRight => {
                        log::info!("Next chapter requested");
                        return Ok(ChapterResult::NextChapter);
                    }
                    TouchEvent::SwipeLeft => {
                        log::info!("Previous chapter requested");
                        return Ok(ChapterResult::PreviousChapter);
                    }
                    TouchEvent::SwipeUp => {
                        // Skip forward 10 seconds
                        if let Ok(pos) = self.audio.position_ms() {
                            let _ = self.audio.seek(pos + 10_000);
                        }
                    }
                    TouchEvent::SwipeDown => {
                        // Skip backward 10 seconds
                        if let Ok(pos) = self.audio.position_ms() {
                            if pos >= 10_000 {
                                let _ = self.audio.seek(pos - 10_000);
                            } else {
                                let _ = self.audio.seek(0);
                            }
                        }
                    }
                }
            }

            // Get current playback position
            let position_ms = match self.audio.position_ms() {
                Ok(pos) => pos,
                Err(_) => {
                    // Playback might have ended
                    log::info!("Audio playback ended");
                    return Ok(ChapterResult::Completed);
                }
            };

            // Check if we need to update the highlight
            if current_node_idx < sync_nodes.len() {
                let node = &sync_nodes[current_node_idx];

                if position_ms >= node.audio_start_ms && position_ms < node.audio_end_ms {
                    // We're in this sync node - highlight it
                    self.update_highlight_for_node(node)?;
                } else if position_ms >= node.audio_end_ms {
                    // Move to next sync node
                    self.clear_current_highlight()?;
                    current_node_idx += 1;
                }
            }

            // Small delay to avoid busy-waiting
            thread::sleep(Duration::from_millis(50));
        }
    }

    /// Update the text highlight for a sync node
    fn update_highlight_for_node(&mut self, node: &SyncNode) -> Result<()> {
        // Clear previous highlight
        self.clear_current_highlight()?;

        // Calculate the bounding box for the text element
        // For MVP, we'll use a simple fixed position
        // In production, you'd parse the XHTML and calculate actual positions
        let rect = self.calculate_text_bounds(&node.text_element_id);

        // Highlight the region
        self.display.highlight_region(rect)?;
        self.current_highlight = Some(rect);

        log::trace!("Highlighted text element: {}", node.text_element_id);

        Ok(())
    }

    /// Clear the current text highlight
    fn clear_current_highlight(&mut self) -> Result<()> {
        if let Some(rect) = self.current_highlight {
            self.display.clear_highlight(rect)?;
            self.current_highlight = None;
        }
        Ok(())
    }

    /// Calculate text bounds for a given element ID
    /// This is a simplified MVP implementation
    fn calculate_text_bounds(&self, _element_id: &str) -> Rect {
        // In a real implementation, you would:
        // 1. Parse the XHTML
        // 2. Find the element by ID
        // 3. Perform layout to get the bounding box
        // 4. Convert to screen coordinates

        // For MVP, return a fixed region
        let (width, height) = self.display.dimensions();
        Rect {
            x: 50,
            y: 200,
            width: (width as u16) - 100,
            height: 30,
        }
    }
}

#[derive(Debug)]
enum ChapterResult {
    Completed,
    NextChapter,
    PreviousChapter,
    Exit,
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    log::info!("=== Immersion Reader for Kindle ===");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <epub_file>", args[0]);
        eprintln!("Example: {} /mnt/us/documents/book.epub", args[0]);
        std::process::exit(1);
    }

    let epub_path = PathBuf::from(&args[1]);

    // Initialize and run the application
    let mut app = ImmersionReader::new(epub_path)?;
    app.run()?;

    log::info!("Application exited successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run on actual hardware
    fn test_app_init() {
        let epub_path = PathBuf::from("/mnt/us/documents/test.epub");
        let app = ImmersionReader::new(epub_path);
        // Will fail if EPUB doesn't exist, but tests the initialization chain
    }
}
