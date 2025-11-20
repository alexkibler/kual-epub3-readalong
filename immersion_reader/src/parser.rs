use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Represents a single synchronization point between audio and text
#[derive(Debug, Clone)]
pub struct SyncNode {
    pub id: String,
    pub text_element_id: String,
    pub audio_src: String,
    pub audio_start_ms: u64,
    pub audio_end_ms: u64,
}

/// Represents a chapter in the EPUB with its media overlay
#[derive(Debug, Clone)]
pub struct Chapter {
    pub title: String,
    pub xhtml_path: PathBuf,
    pub smil_path: Option<PathBuf>,
    pub audio_path: Option<PathBuf>,
    pub sync_nodes: Vec<SyncNode>,
}

/// Represents a text element's position on screen
#[derive(Debug, Clone, Copy)]
pub struct TextBounds {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Main EPUB document structure
pub struct EpubDocument {
    pub title: String,
    pub chapters: Vec<Chapter>,
    extract_dir: PathBuf,
}

impl EpubDocument {
    /// Open and parse an EPUB file
    pub fn open(epub_path: &Path) -> Result<Self> {
        log::info!("Opening EPUB: {}", epub_path.display());

        if !epub_path.exists() {
            anyhow::bail!("EPUB file not found: {}", epub_path.display());
        }

        // Create temporary extraction directory
        let extract_dir = PathBuf::from("/tmp/immersion_reader_epub");
        if extract_dir.exists() {
            std::fs::remove_dir_all(&extract_dir)?;
        }
        std::fs::create_dir_all(&extract_dir)?;

        // Extract EPUB (it's just a ZIP file)
        Self::extract_epub(epub_path, &extract_dir)?;

        // Parse the OPF file to get the manifest and spine
        let (title, chapters) = Self::parse_opf(&extract_dir)?;

        log::info!("Successfully opened EPUB: {} ({} chapters)", title, chapters.len());

        Ok(EpubDocument {
            title,
            chapters,
            extract_dir,
        })
    }

    /// Extract EPUB ZIP file
    fn extract_epub(epub_path: &Path, extract_dir: &Path) -> Result<()> {
        log::debug!("Extracting EPUB to {}", extract_dir.display());

        let file = File::open(epub_path)?;
        let mut archive = ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = extract_dir.join(file.mangled_name());

            if file.is_dir() {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        log::debug!("EPUB extracted successfully");
        Ok(())
    }

    /// Parse the OPF file to get book metadata and spine
    fn parse_opf(extract_dir: &Path) -> Result<(String, Vec<Chapter>)> {
        log::debug!("Parsing OPF file");

        // First, find the OPF file by reading META-INF/container.xml
        let container_path = extract_dir.join("META-INF/container.xml");
        let opf_path = Self::find_opf_path(&container_path, extract_dir)?;

        log::debug!("OPF file: {}", opf_path.display());

        // Parse the OPF file
        let opf_content = std::fs::read_to_string(&opf_path)?;
        let mut reader = Reader::from_str(&opf_content);
        reader.trim_text(true);

        let mut title = String::from("Unknown");
        let mut manifest: HashMap<String, PathBuf> = HashMap::new();
        let mut spine_ids: Vec<String> = Vec::new();
        let mut media_overlays: HashMap<String, String> = HashMap::new();

        let mut buf = Vec::new();
        let mut in_metadata = false;
        let mut in_manifest = false;
        let mut in_spine = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"metadata" => in_metadata = true,
                        b"manifest" => in_manifest = true,
                        b"spine" => in_spine = true,
                        b"dc:title" if in_metadata => {
                            if let Ok(Event::Text(t)) = reader.read_event_into(&mut buf) {
                                title = t.unescape().unwrap_or_default().to_string();
                            }
                        }
                        b"item" if in_manifest => {
                            let mut id = String::new();
                            let mut href = String::new();
                            let mut media_overlay = String::new();

                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    match attr.key.as_ref() {
                                        b"id" => id = String::from_utf8_lossy(&attr.value).to_string(),
                                        b"href" => href = String::from_utf8_lossy(&attr.value).to_string(),
                                        b"media-overlay" => media_overlay = String::from_utf8_lossy(&attr.value).to_string(),
                                        _ => {}
                                    }
                                }
                            }

                            if !id.is_empty() && !href.is_empty() {
                                let full_path = opf_path.parent().unwrap().join(&href);
                                manifest.insert(id.clone(), full_path);

                                if !media_overlay.is_empty() {
                                    media_overlays.insert(id, media_overlay);
                                }
                            }
                        }
                        b"itemref" if in_spine => {
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    if attr.key.as_ref() == b"idref" {
                                        let idref = String::from_utf8_lossy(&attr.value).to_string();
                                        spine_ids.push(idref);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    match e.name().as_ref() {
                        b"metadata" => in_metadata = false,
                        b"manifest" => in_manifest = false,
                        b"spine" => in_spine = false,
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("Error parsing OPF: {:?}", e)),
                _ => {}
            }
            buf.clear();
        }

        // Build chapters from spine
        let mut chapters = Vec::new();
        for (idx, spine_id) in spine_ids.iter().enumerate() {
            if let Some(xhtml_path) = manifest.get(spine_id) {
                let smil_path = media_overlays
                    .get(spine_id)
                    .and_then(|smil_id| manifest.get(smil_id).cloned());

                let chapter = Chapter {
                    title: format!("Chapter {}", idx + 1),
                    xhtml_path: xhtml_path.clone(),
                    smil_path,
                    audio_path: None,
                    sync_nodes: Vec::new(),
                };

                chapters.push(chapter);
            }
        }

        log::debug!("Found {} chapters", chapters.len());
        Ok((title, chapters))
    }

    /// Find the OPF file path from container.xml
    fn find_opf_path(container_path: &Path, extract_dir: &Path) -> Result<PathBuf> {
        let content = std::fs::read_to_string(container_path)?;
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"rootfile" => {
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            if attr.key.as_ref() == b"full-path" {
                                let path = String::from_utf8_lossy(&attr.value);
                                return Ok(extract_dir.join(path.as_ref()));
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("Error parsing container.xml: {:?}", e)),
                _ => {}
            }
            buf.clear();
        }

        anyhow::bail!("Could not find OPF path in container.xml");
    }

    /// Parse a SMIL file to extract synchronization points
    pub fn parse_smil(smil_path: &Path) -> Result<Vec<SyncNode>> {
        log::debug!("Parsing SMIL file: {}", smil_path.display());

        let content = std::fs::read_to_string(smil_path)?;
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);

        let mut sync_nodes = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"par" => {
                    // Parse <par> element which contains <text> and <audio>
                    let mut id = String::new();
                    let mut text_src = String::new();
                    let mut audio_src = String::new();
                    let mut clip_begin = String::new();
                    let mut clip_end = String::new();

                    // This is simplified - in real SMIL, text and audio are child elements
                    // We'd need to read ahead to get them
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            match attr.key.as_ref() {
                                b"id" => id = String::from_utf8_lossy(&attr.value).to_string(),
                                _ => {}
                            }
                        }
                    }

                    // Read child elements
                    // (Simplified - a full parser would handle this better)

                    if !id.is_empty() {
                        let start_ms = Self::parse_smil_time(&clip_begin);
                        let end_ms = Self::parse_smil_time(&clip_end);

                        sync_nodes.push(SyncNode {
                            id: id.clone(),
                            text_element_id: text_src,
                            audio_src,
                            audio_start_ms: start_ms,
                            audio_end_ms: end_ms,
                        });
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("Error parsing SMIL: {:?}", e)),
                _ => {}
            }
            buf.clear();
        }

        log::debug!("Found {} sync points", sync_nodes.len());
        Ok(sync_nodes)
    }

    /// Parse SMIL time format (e.g., "0:05.123" or "5.123s")
    fn parse_smil_time(time_str: &str) -> u64 {
        if time_str.is_empty() {
            return 0;
        }

        // Handle formats like "123.456s" or "0:02:03.456"
        if time_str.ends_with('s') {
            let num_str = time_str.trim_end_matches('s');
            if let Ok(secs) = num_str.parse::<f64>() {
                return (secs * 1000.0) as u64;
            }
        }

        // Handle MM:SS.mmm or HH:MM:SS.mmm
        let parts: Vec<&str> = time_str.split(':').collect();
        match parts.len() {
            1 => {
                // Just seconds
                if let Ok(secs) = parts[0].parse::<f64>() {
                    return (secs * 1000.0) as u64;
                }
            }
            2 => {
                // MM:SS.mmm
                if let (Ok(mins), Ok(secs)) = (parts[0].parse::<u64>(), parts[1].parse::<f64>()) {
                    return mins * 60_000 + (secs * 1000.0) as u64;
                }
            }
            3 => {
                // HH:MM:SS.mmm
                if let (Ok(hours), Ok(mins), Ok(secs)) = (
                    parts[0].parse::<u64>(),
                    parts[1].parse::<u64>(),
                    parts[2].parse::<f64>(),
                ) {
                    return hours * 3_600_000 + mins * 60_000 + (secs * 1000.0) as u64;
                }
            }
            _ => {}
        }

        0
    }

    /// Get the file path for a resource relative to the EPUB
    pub fn get_resource_path(&self, relative_path: &str) -> PathBuf {
        self.extract_dir.join(relative_path)
    }
}

impl Drop for EpubDocument {
    fn drop(&mut self) {
        // Clean up extracted files
        if self.extract_dir.exists() {
            let _ = std::fs::remove_dir_all(&self.extract_dir);
            log::debug!("Cleaned up EPUB extraction directory");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_smil_time() {
        assert_eq!(EpubDocument::parse_smil_time("5.5s"), 5500);
        assert_eq!(EpubDocument::parse_smil_time("1:30"), 90000);
        assert_eq!(EpubDocument::parse_smil_time("0:01:30.500"), 90500);
    }
}
