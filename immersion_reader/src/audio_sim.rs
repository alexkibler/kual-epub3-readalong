// Simulator audio backend for development/testing on macOS
// Uses standard audio output (autoaudiosink) instead of bluealsa

use anyhow::{Context, Result};
use gstreamer as gst;
use gst::prelude::*;
use std::path::Path;
use std::thread;
use std::time::Duration;

pub struct AudioPlayer {
    pipeline: gst::Pipeline,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        log::info!("Initializing simulator audio subsystem (autoaudiosink)");

        gst::init().context("Failed to initialize GStreamer")?;

        let pipeline = gst::Pipeline::new(Some("audio-player"));

        Ok(AudioPlayer { pipeline })
    }

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

        // Use autoaudiosink for automatic audio output selection
        let audiosink = gst::ElementFactory::make("autoaudiosink")
            .build()
            .context("Failed to create autoaudiosink element")?;

        // Add elements to the pipeline
        self.pipeline
            .add_many(&[&src, &decodebin, &audioconvert, &audioresample, &audiosink])
            .context("Failed to add elements to pipeline")?;

        // Link static elements
        src.link(&decodebin)
            .context("Failed to link filesrc to decodebin")?;

        // Link convert -> resample -> sink
        gst::Element::link_many(&[&audioconvert, &audioresample, &audiosink])
            .context("Failed to link audio processing chain")?;

        // Connect decodebin's pad-added signal
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

        // Set pipeline to PAUSED state
        self.pipeline
            .set_state(gst::State::Paused)
            .context("Failed to set pipeline to PAUSED state")?;

        // Wait for state change
        let (_, _, _) = self
            .pipeline
            .state(gst::ClockTime::from_seconds(5))
            .context("Failed to wait for PAUSED state")?;

        log::info!("Audio file loaded and ready to play");
        Ok(())
    }

    pub fn play(&self) -> Result<()> {
        log::debug!("Starting playback");

        self.pipeline
            .set_state(gst::State::Playing)
            .context("Failed to start playback")?;

        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        log::debug!("Pausing playback");

        self.pipeline
            .set_state(gst::State::Paused)
            .context("Failed to pause playback")?;

        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        log::debug!("Stopping playback");

        self.pipeline
            .set_state(gst::State::Null)
            .context("Failed to stop playback")?;

        Ok(())
    }

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

    pub fn position_ms(&self) -> Result<u64> {
        let position = self
            .pipeline
            .query_position::<gst::ClockTime>()
            .context("Failed to query position")?;

        Ok(position.mseconds())
    }

    pub fn duration_ms(&self) -> Result<u64> {
        let duration = self
            .pipeline
            .query_duration::<gst::ClockTime>()
            .context("Failed to query duration")?;

        Ok(duration.mseconds())
    }

    pub fn is_playing(&self) -> bool {
        let (_, current_state, _) = self.pipeline.state(gst::ClockTime::from_mseconds(0));
        current_state == gst::State::Playing
    }

    pub fn wait_for_eos(&self) -> Result<()> {
        log::debug!("Waiting for end of stream");

        let bus = self.pipeline.bus().context("Pipeline has no bus")?;

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

    pub fn retry_bluetooth_connection(&mut self, _max_retries: u32) -> Result<()> {
        log::info!("Bluetooth retry not needed in simulator mode");
        Ok(())
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        log::info!("Shutting down audio player");
        let _ = self.stop();
    }
}
