use anyhow::{Context, Result};
use gstreamer as gst;
use gst::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Represents the audio playback subsystem
pub struct AudioPlayer {
    pipeline: gst::Pipeline,
    bluealsa_device: String,
}

impl AudioPlayer {
    /// Initialize the GStreamer audio player with bluealsa device discovery
    pub fn new() -> Result<Self> {
        log::info!("Initializing GStreamer audio subsystem");

        // Initialize GStreamer
        gst::init().context("Failed to initialize GStreamer")?;

        // Discover bluealsa device
        let bluealsa_device = Self::discover_bluealsa_device()
            .context("Failed to discover bluealsa device")?;

        log::info!("Using Bluetooth audio device: {}", bluealsa_device);

        // Create an empty pipeline
        let pipeline = gst::Pipeline::new(Some("audio-player"));

        Ok(AudioPlayer {
            pipeline,
            bluealsa_device,
        })
    }

    /// Discover the bluealsa PCM device
    fn discover_bluealsa_device() -> Result<String> {
        log::debug!("Scanning for bluealsa ALSA devices");

        // Try common bluealsa device names
        let candidates = vec![
            "bluealsa",
            "bluealsa:HCI=hci0",
            "bluealsa:DEV=XX:XX:XX:XX:XX:XX", // Would need MAC address
        ];

        // For now, we'll use a simple approach - just use "bluealsa"
        // In production, you'd parse `aplay -L` output
        for device in &candidates {
            log::debug!("Checking device: {}", device);
            // You could verify the device exists here
            // For now, we'll just return the first candidate
            return Ok(device.to_string());
        }

        anyhow::bail!("No bluealsa device found. Is Bluetooth audio connected?");
    }

    /// Load an audio file and prepare the pipeline
    pub fn load(&mut self, audio_path: &Path) -> Result<()> {
        log::info!("Loading audio file: {}", audio_path.display());

        if !audio_path.exists() {
            anyhow::bail!("Audio file does not exist: {}", audio_path.display());
        }

        // Stop any existing playback
        self.stop()?;

        // Create pipeline elements
        let src = gst::ElementFactory::make("filesrc")
            .property("location", audio_path.to_str().unwrap())
            .build()
            .context("Failed to create filesrc element")?;

        let decodebin = gst::ElementFactory::make("decodebin")
            .build()
            .context("Failed to create decodebin element")?;

        let audioconvert = gst::ElementFactory::make("audioconvert")
            .build()
            .context("Failed to create audioconvert element")?;

        let audioresample = gst::ElementFactory::make("audioresample")
            .build()
            .context("Failed to create audioresample element")?;

        let alsasink = gst::ElementFactory::make("alsasink")
            .property("device", &self.bluealsa_device)
            .build()
            .context("Failed to create alsasink element")?;

        // Add elements to the pipeline
        self.pipeline
            .add_many(&[&src, &decodebin, &audioconvert, &audioresample, &alsasink])
            .context("Failed to add elements to pipeline")?;

        // Link static elements
        src.link(&decodebin)
            .context("Failed to link filesrc to decodebin")?;

        // Link convert -> resample -> sink
        gst::Element::link_many(&[&audioconvert, &audioresample, &alsasink])
            .context("Failed to link audio processing chain")?;

        // Connect decodebin's pad-added signal (dynamic linking)
        let audioconvert_weak = audioconvert.downgrade();
        decodebin.connect_pad_added(move |_, src_pad| {
            let Some(audioconvert) = audioconvert_weak.upgrade() else {
                return;
            };

            let sink_pad = audioconvert
                .static_pad("sink")
                .expect("audioconvert has no sink pad");

            if sink_pad.is_linked() {
                log::debug!("Sink pad already linked, ignoring");
                return;
            }

            let caps = src_pad.current_caps().unwrap();
            let structure = caps.structure(0).unwrap();
            let name = structure.name();

            if name.starts_with("audio/") {
                if let Err(e) = src_pad.link(&sink_pad) {
                    log::error!("Failed to link decodebin to audioconvert: {:?}", e);
                } else {
                    log::debug!("Successfully linked audio pad");
                }
            }
        });

        // Set pipeline to PAUSED state (ready to play)
        self.pipeline
            .set_state(gst::State::Paused)
            .context("Failed to set pipeline to PAUSED state")?;

        // Wait for state change to complete
        let (_, _, _) = self
            .pipeline
            .state(gst::ClockTime::from_seconds(5))
            .context("Failed to wait for PAUSED state")?;

        log::info!("Audio file loaded and ready to play");
        Ok(())
    }

    /// Start playback from the current position
    pub fn play(&self) -> Result<()> {
        log::debug!("Starting playback");

        self.pipeline
            .set_state(gst::State::Playing)
            .context("Failed to start playback")?;

        Ok(())
    }

    /// Pause playback
    pub fn pause(&self) -> Result<()> {
        log::debug!("Pausing playback");

        self.pipeline
            .set_state(gst::State::Paused)
            .context("Failed to pause playback")?;

        Ok(())
    }

    /// Stop playback and reset position
    pub fn stop(&self) -> Result<()> {
        log::debug!("Stopping playback");

        self.pipeline
            .set_state(gst::State::Null)
            .context("Failed to stop playback")?;

        Ok(())
    }

    /// Seek to a specific position in milliseconds
    pub fn seek(&self, position_ms: u64) -> Result<()> {
        log::debug!("Seeking to {}ms", position_ms);

        let position = gst::ClockTime::from_mseconds(position_ms);

        self.pipeline
            .seek_simple(
                gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE,
                position,
            )
            .context("Failed to seek")?;

        Ok(())
    }

    /// Get the current playback position in milliseconds
    pub fn position_ms(&self) -> Result<u64> {
        let position = self
            .pipeline
            .query_position::<gst::ClockTime>()
            .context("Failed to query position")?;

        Ok(position.mseconds())
    }

    /// Get the total duration in milliseconds
    pub fn duration_ms(&self) -> Result<u64> {
        let duration = self
            .pipeline
            .query_duration::<gst::ClockTime>()
            .context("Failed to query duration")?;

        Ok(duration.mseconds())
    }

    /// Check if playback is currently active
    pub fn is_playing(&self) -> bool {
        let (_, current_state, _) = self.pipeline.state(gst::ClockTime::from_mseconds(0));
        current_state == gst::State::Playing
    }

    /// Wait for the pipeline to reach EOS (end of stream)
    pub fn wait_for_eos(&self) -> Result<()> {
        log::debug!("Waiting for end of stream");

        let bus = self
            .pipeline
            .bus()
            .context("Pipeline has no bus")?;

        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            use gst::MessageView;

            match msg.view() {
                MessageView::Eos(..) => {
                    log::info!("Reached end of stream");
                    break;
                }
                MessageView::Error(err) => {
                    anyhow::bail!(
                        "Error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                }
                MessageView::StateChanged(state_changed) => {
                    if msg.src() == Some(self.pipeline.upcast_ref()) {
                        log::trace!(
                            "Pipeline state changed: {:?} -> {:?}",
                            state_changed.old(),
                            state_changed.current()
                        );
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Retry connection to Bluetooth device if it's asleep
    pub fn retry_bluetooth_connection(&mut self, max_retries: u32) -> Result<()> {
        log::warn!("Attempting to reconnect to Bluetooth device");

        for attempt in 1..=max_retries {
            log::info!("Bluetooth connection attempt {}/{}", attempt, max_retries);

            match Self::discover_bluealsa_device() {
                Ok(device) => {
                    self.bluealsa_device = device;
                    log::info!("Successfully reconnected to Bluetooth device");
                    return Ok(());
                }
                Err(_) => {
                    if attempt < max_retries {
                        let wait_time = Duration::from_secs(2u64.pow(attempt - 1));
                        log::debug!("Waiting {:?} before retry", wait_time);
                        thread::sleep(wait_time);
                    }
                }
            }
        }

        anyhow::bail!("Failed to connect to Bluetooth device after {} attempts", max_retries);
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        log::info!("Shutting down audio player");
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run on actual hardware with GStreamer
    fn test_audio_player_init() {
        let player = AudioPlayer::new();
        assert!(player.is_ok());
    }
}
