//! Audio metadata extraction functionality
//!
//! This module provides comprehensive metadata extraction from various audio formats,
//! including ID3 tags, FLAC comments, and embedded artwork.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::AudioError;

/// Comprehensive audio track metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrackMetadata {
    // Basic information
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub year: Option<u32>,
    pub track_number: Option<u32>,
    pub track_total: Option<u32>,
    pub disc_number: Option<u32>,
    pub disc_total: Option<u32>,

    // Extended information
    pub genre: Option<String>,
    pub composer: Option<String>,
    pub performer: Option<String>,
    pub conductor: Option<String>,
    pub comment: Option<String>,

    // Technical information
    pub duration: Option<Duration>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub bit_depth: Option<u16>,

    // File information
    pub file_size: Option<u64>,
    pub format: Option<String>,
    pub encoding: Option<String>,

    // Artwork
    pub artwork: Option<ArtworkInfo>,

    // Additional tags (format-specific)
    pub custom_tags: std::collections::HashMap<String, String>,
}

/// Album artwork information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtworkInfo {
    pub mime_type: String,
    pub description: Option<String>,
    pub artwork_type: ArtworkType,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub data: Vec<u8>,
}

/// Types of embedded artwork
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArtworkType {
    FrontCover,
    BackCover,
    LeafletPage,
    Media,
    LeadArtist,
    Artist,
    Conductor,
    Band,
    Composer,
    Lyricist,
    RecordingLocation,
    DuringRecording,
    DuringPerformance,
    MovieScreenCapture,
    ColouredFish,
    Illustration,
    BandLogotype,
    PublisherLogotype,
    Other(String),
}

/// Metadata extraction engine
pub struct MetadataExtractor;

impl MetadataExtractor {
    /// Extract metadata from an audio file
    pub fn extract_metadata(path: &Path) -> Result<TrackMetadata, AudioError> {
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "mp3" => Self::extract_mp3_metadata(path),
            "flac" => Self::extract_flac_metadata(path),
            "wav" => Self::extract_wav_metadata(path),
            "ogg" => Self::extract_ogg_metadata(path),
            "m4a" | "aac" => Self::extract_m4a_metadata(path),
            _ => Self::extract_generic_metadata(path),
        }
    }

    /// Extract MP3 metadata using ID3 tags
    fn extract_mp3_metadata(path: &Path) -> Result<TrackMetadata, AudioError> {
        use id3::{Tag, TagLike};

        let tag = Tag::read_from_path(path)
            .map_err(|e| AudioError::Metadata(format!("Failed to read ID3 tag: {}", e)))?;

        let mut metadata = TrackMetadata {
            title: tag.title().map(ToString::to_string),
            artist: tag.artist().map(ToString::to_string),
            album: tag.album().map(ToString::to_string),
            album_artist: tag.album_artist().map(ToString::to_string),
            year: tag.year().map(|y| y as u32),
            track_number: tag.track(),
            track_total: tag.total_tracks(),
            disc_number: tag.disc(),
            disc_total: tag.total_discs(),
            genre: tag.genre().map(ToString::to_string),
            ..TrackMetadata::default()
        };

        // Extended tags
        for frame in tag.frames() {
            match frame.id() {
                "TCOM" => metadata.composer = frame.content().text().map(|s| s.to_string()),
                "TPE3" => metadata.conductor = frame.content().text().map(|s| s.to_string()),
                "TPE4" => metadata.performer = frame.content().text().map(|s| s.to_string()),
                "COMM" => metadata.comment = frame.content().text().map(|s| s.to_string()),
                _ => {
                    if let Some(text) = frame.content().text() {
                        metadata
                            .custom_tags
                            .insert(frame.id().to_string(), text.to_string());
                    }
                }
            }
        }

        // Extract artwork
        if let Some(picture) = tag.pictures().next() {
            metadata.artwork = Some(ArtworkInfo {
                mime_type: picture.mime_type.clone(),
                description: Some(picture.description.clone()),
                artwork_type: Self::convert_id3_picture_type(picture.picture_type),
                width: None, // Would need image parsing
                height: None,
                data: picture.data.clone(),
            });
        }

        // Get file information
        let file_info = std::fs::metadata(path)
            .map_err(|e| AudioError::Metadata(format!("Failed to get file info: {}", e)))?;
        metadata.file_size = Some(file_info.len());
        metadata.format = Some("MP3".to_string());

        Ok(metadata)
    }

    /// Extract FLAC metadata using Vorbis comments
    fn extract_flac_metadata(path: &Path) -> Result<TrackMetadata, AudioError> {
        use metaflac::Tag;

        let tag = Tag::read_from_path(path)
            .map_err(|e| AudioError::Metadata(format!("Failed to read FLAC metadata: {}", e)))?;

        let mut metadata = TrackMetadata::default();

        // Get Vorbis comments
        if let Some(vorbis_comments) = tag.vorbis_comments() {
            for (key, values) in vorbis_comments.comments.iter() {
                if let Some(value) = values.first() {
                    match key.to_uppercase().as_str() {
                        "TITLE" => metadata.title = Some(value.clone()),
                        "ARTIST" => metadata.artist = Some(value.clone()),
                        "ALBUM" => metadata.album = Some(value.clone()),
                        "ALBUMARTIST" => metadata.album_artist = Some(value.clone()),
                        "DATE" | "YEAR" => {
                            metadata.year = value.parse().ok();
                        }
                        "TRACKNUMBER" => {
                            metadata.track_number = value.parse().ok();
                        }
                        "TRACKTOTAL" => {
                            metadata.track_total = value.parse().ok();
                        }
                        "DISCNUMBER" => {
                            metadata.disc_number = value.parse().ok();
                        }
                        "DISCTOTAL" => {
                            metadata.disc_total = value.parse().ok();
                        }
                        "GENRE" => metadata.genre = Some(value.clone()),
                        "COMPOSER" => metadata.composer = Some(value.clone()),
                        "PERFORMER" => metadata.performer = Some(value.clone()),
                        "CONDUCTOR" => metadata.conductor = Some(value.clone()),
                        "COMMENT" => metadata.comment = Some(value.clone()),
                        _ => {
                            metadata.custom_tags.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }

        // Get STREAMINFO block for technical details
        if let Some(streaminfo) = tag.get_streaminfo() {
            metadata.sample_rate = Some(streaminfo.sample_rate);
            metadata.channels = Some(streaminfo.num_channels as u16);
            metadata.bit_depth = Some(streaminfo.bits_per_sample as u16);

            if streaminfo.total_samples > 0 {
                let duration_secs = streaminfo.total_samples as f64 / streaminfo.sample_rate as f64;
                metadata.duration = Some(Duration::from_secs_f64(duration_secs));
            }
        }

        // Extract artwork from PICTURE blocks
        for block in tag.blocks() {
            if let metaflac::block::BlockType::Picture = block.block_type()
                && let metaflac::block::Block::Picture(picture) = block
            {
                metadata.artwork = Some(ArtworkInfo {
                    mime_type: picture.mime_type.clone(),
                    description: Some(picture.description.clone()),
                    artwork_type: Self::convert_flac_picture_type(picture.picture_type),
                    width: Some(picture.width),
                    height: Some(picture.height),
                    data: picture.data.clone(),
                });
                break; // Use first picture found
            }
        }

        // Get file information
        let file_info = std::fs::metadata(path)
            .map_err(|e| AudioError::Metadata(format!("Failed to get file info: {}", e)))?;
        metadata.file_size = Some(file_info.len());
        metadata.format = Some("FLAC".to_string());
        metadata.encoding = Some("Lossless".to_string());

        Ok(metadata)
    }

    /// Extract WAV metadata (basic RIFF info)
    fn extract_wav_metadata(path: &Path) -> Result<TrackMetadata, AudioError> {
        use hound::WavReader;

        let reader = WavReader::open(path)
            .map_err(|e| AudioError::Metadata(format!("Failed to open WAV file: {}", e)))?;

        let spec = reader.spec();
        let duration_secs = f64::from(reader.duration()) / f64::from(spec.sample_rate);
        let mut metadata = TrackMetadata {
            sample_rate: Some(spec.sample_rate),
            channels: Some(spec.channels),
            bit_depth: Some(spec.bits_per_sample),
            duration: Some(Duration::from_secs_f64(duration_secs)),
            ..TrackMetadata::default()
        };

        // Get file information
        let file_info = std::fs::metadata(path)
            .map_err(|e| AudioError::Metadata(format!("Failed to get file info: {}", e)))?;
        metadata.file_size = Some(file_info.len());
        metadata.format = Some("WAV".to_string());
        metadata.encoding = Some("PCM".to_string());

        // Calculate bitrate
        let bitrate = spec.sample_rate * spec.channels as u32 * spec.bits_per_sample as u32;
        metadata.bitrate = Some(bitrate);

        // WAV files might have LIST chunks with metadata, but hound doesn't expose them
        // For full WAV metadata support, we'd need a more comprehensive parser

        Ok(metadata)
    }

    /// Extract OGG Vorbis metadata
    fn extract_ogg_metadata(_path: &Path) -> Result<TrackMetadata, AudioError> {
        // Placeholder for OGG Vorbis metadata extraction
        // Would use lewton or similar crate
        Ok(TrackMetadata::default())
    }

    /// Extract M4A/AAC metadata
    fn extract_m4a_metadata(_path: &Path) -> Result<TrackMetadata, AudioError> {
        // Placeholder for M4A metadata extraction
        // Would use mp4parse or similar crate
        Ok(TrackMetadata::default())
    }

    /// Extract metadata using Symphonia (fallback)
    fn extract_generic_metadata(path: &Path) -> Result<TrackMetadata, AudioError> {
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = std::fs::File::open(path)
            .map_err(|e| AudioError::Metadata(format!("Failed to open file: {}", e)))?;

        let source = Box::new(file);
        let mss = MediaSourceStream::new(source, Default::default());

        let mut hint = Hint::new();
        if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
            hint.with_extension(extension);
        }

        let mut probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| AudioError::Metadata(format!("Failed to probe format: {}", e)))?;

        let mut metadata = TrackMetadata::default();

        // Extract metadata from Symphonia
        if let Some(symphonia_metadata) = probed.metadata.get()
            && let Some(current) = symphonia_metadata.current()
        {
            for tag in current.tags() {
                match tag.key.as_str() {
                    "TITLE" => metadata.title = Some(tag.value.to_string()),
                    "ARTIST" => metadata.artist = Some(tag.value.to_string()),
                    "ALBUM" => metadata.album = Some(tag.value.to_string()),
                    "DATE" => metadata.year = tag.value.to_string().parse().ok(),
                    "TRACK" => metadata.track_number = tag.value.to_string().parse().ok(),
                    "GENRE" => metadata.genre = Some(tag.value.to_string()),
                    _ => {
                        metadata
                            .custom_tags
                            .insert(tag.key.clone(), tag.value.to_string());
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// Convert ID3 picture type to our enum
    fn convert_id3_picture_type(pic_type: id3::frame::PictureType) -> ArtworkType {
        match pic_type {
            id3::frame::PictureType::CoverFront => ArtworkType::FrontCover,
            id3::frame::PictureType::CoverBack => ArtworkType::BackCover,
            id3::frame::PictureType::Leaflet => ArtworkType::LeafletPage,
            id3::frame::PictureType::Media => ArtworkType::Media,
            id3::frame::PictureType::LeadArtist => ArtworkType::LeadArtist,
            id3::frame::PictureType::Artist => ArtworkType::Artist,
            id3::frame::PictureType::Conductor => ArtworkType::Conductor,
            id3::frame::PictureType::Band => ArtworkType::Band,
            id3::frame::PictureType::Composer => ArtworkType::Composer,
            id3::frame::PictureType::Lyricist => ArtworkType::Lyricist,
            id3::frame::PictureType::RecordingLocation => ArtworkType::RecordingLocation,
            id3::frame::PictureType::DuringRecording => ArtworkType::DuringRecording,
            id3::frame::PictureType::DuringPerformance => ArtworkType::DuringPerformance,
            id3::frame::PictureType::ScreenCapture => ArtworkType::MovieScreenCapture,
            id3::frame::PictureType::BrightFish => ArtworkType::ColouredFish,
            id3::frame::PictureType::Illustration => ArtworkType::Illustration,
            id3::frame::PictureType::BandLogo => ArtworkType::BandLogotype,
            id3::frame::PictureType::PublisherLogo => ArtworkType::PublisherLogotype,
            _ => ArtworkType::Other("Unknown".to_string()),
        }
    }

    /// Convert FLAC picture type to our enum
    fn convert_flac_picture_type(pic_type: metaflac::block::PictureType) -> ArtworkType {
        match pic_type {
            metaflac::block::PictureType::CoverFront => ArtworkType::FrontCover,
            metaflac::block::PictureType::CoverBack => ArtworkType::BackCover,
            metaflac::block::PictureType::Media => ArtworkType::Media,
            metaflac::block::PictureType::LeadArtist => ArtworkType::LeadArtist,
            metaflac::block::PictureType::Artist => ArtworkType::Artist,
            metaflac::block::PictureType::Conductor => ArtworkType::Conductor,
            metaflac::block::PictureType::Band => ArtworkType::Band,
            metaflac::block::PictureType::Composer => ArtworkType::Composer,
            metaflac::block::PictureType::Lyricist => ArtworkType::Lyricist,
            metaflac::block::PictureType::RecordingLocation => ArtworkType::RecordingLocation,
            metaflac::block::PictureType::DuringRecording => ArtworkType::DuringRecording,
            metaflac::block::PictureType::DuringPerformance => ArtworkType::DuringPerformance,
            metaflac::block::PictureType::Illustration => ArtworkType::Illustration,
            metaflac::block::PictureType::BandLogo => ArtworkType::BandLogotype,
            metaflac::block::PictureType::PublisherLogo => ArtworkType::PublisherLogotype,
            _ => ArtworkType::Other("Unknown".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artwork_type_conversion() {
        assert_eq!(
            MetadataExtractor::convert_id3_picture_type(id3::frame::PictureType::CoverFront),
            ArtworkType::FrontCover
        );
    }

    #[test]
    fn test_metadata_default() {
        let metadata = TrackMetadata::default();
        assert!(metadata.title.is_none());
        assert!(metadata.artist.is_none());
        assert!(metadata.custom_tags.is_empty());
    }

    // Additional tests would include:
    // - Metadata extraction from sample files
    // - Artwork extraction and validation
    // - Format-specific metadata handling
    // - Error handling for corrupt files
}
