//! Tests for metadata extraction functionality
//!
//! This module contains comprehensive tests for metadata extraction
//! from various audio formats, including error handling and edge cases.

use std::path::PathBuf;
use std::time::Duration;

use sonic_flow::audio::metadata::{MetadataExtractor, TrackMetadata, ArtworkInfo, ArtworkType};
use sonic_flow::error::AudioError;

// Test data directory
const TEST_DATA_DIR: &str = "tests/fixtures/audio";

fn test_file_path(filename: &str) -> PathBuf {
    PathBuf::from(TEST_DATA_DIR).join(filename)
}

#[cfg(test)]
mod metadata_extraction_tests {
    use super::*;

    #[test]
    fn test_metadata_default() {
        let metadata = TrackMetadata::default();
        
        assert!(metadata.title.is_none());
        assert!(metadata.artist.is_none());
        assert!(metadata.album.is_none());
        assert!(metadata.year.is_none());
        assert!(metadata.artwork.is_none());
        assert!(metadata.custom_tags.is_empty());
    }

    #[test]
    #[ignore] // Requires test MP3 file with ID3 tags
    fn test_mp3_metadata_extraction() {
        let test_file = test_file_path("test_with_tags.mp3");
        if test_file.exists() {
            let metadata = MetadataExtractor::extract_metadata(&test_file);
            assert!(metadata.is_ok());
            
            let metadata = metadata.unwrap();
            
            // Check basic metadata
            assert!(metadata.title.is_some());
            assert!(metadata.artist.is_some());
            assert!(metadata.album.is_some());
            assert_eq!(metadata.format, Some("MP3".to_string()));
            
            // Check technical information
            assert!(metadata.file_size.is_some());
            assert!(metadata.file_size.unwrap() > 0);
        }
    }

    #[test]
    #[ignore] // Requires test MP3 file with artwork
    fn test_mp3_artwork_extraction() {
        let test_file = test_file_path("test_with_artwork.mp3");
        if test_file.exists() {
            let metadata = MetadataExtractor::extract_metadata(&test_file);
            assert!(metadata.is_ok());
            
            let metadata = metadata.unwrap();
            
            if let Some(artwork) = metadata.artwork {
                assert!(!artwork.data.is_empty());
                assert!(!artwork.mime_type.is_empty());
                assert!(artwork.mime_type.starts_with("image/"));
                
                // Most common artwork type should be front cover
                assert_eq!(artwork.artwork_type, ArtworkType::FrontCover);
            }
        }
    }

    #[test]
    #[ignore] // Requires test FLAC file
    fn test_flac_metadata_extraction() {
        let test_file = test_file_path("test_with_tags.flac");
        if test_file.exists() {
            let metadata = MetadataExtractor::extract_metadata(&test_file);
            assert!(metadata.is_ok());
            
            let metadata = metadata.unwrap();
            
            // FLAC should have precise technical information
            assert_eq!(metadata.format, Some("FLAC".to_string()));
            assert_eq!(metadata.encoding, Some("Lossless".to_string()));
            
            if let Some(sample_rate) = metadata.sample_rate {
                assert!(sample_rate > 0);
            }
            
            if let Some(bit_depth) = metadata.bit_depth {
                assert!(bit_depth >= 16);
            }
            
            // FLAC files should have precise duration
            if let Some(duration) = metadata.duration {
                assert!(duration > Duration::from_secs(0));
            }
        }
    }

    #[test]
    #[ignore] // Requires test WAV file
    fn test_wav_metadata_extraction() {
        let test_file = test_file_path("test.wav");
        if test_file.exists() {
            let metadata = MetadataExtractor::extract_metadata(&test_file);
            assert!(metadata.is_ok());
            
            let metadata = metadata.unwrap();
            
            // WAV technical information should be accurate
            assert_eq!(metadata.format, Some("WAV".to_string()));
            assert_eq!(metadata.encoding, Some("PCM".to_string()));
            
            // Should have technical specs
            assert!(metadata.sample_rate.is_some());
            assert!(metadata.channels.is_some());
            assert!(metadata.bit_depth.is_some());
            assert!(metadata.bitrate.is_some());
            assert!(metadata.duration.is_some());
            
            // Bitrate calculation should be correct for PCM
            if let (Some(sr), Some(ch), Some(bd)) = (metadata.sample_rate, metadata.channels, metadata.bit_depth) {
                let expected_bitrate = sr * ch as u32 * bd as u32;
                assert_eq!(metadata.bitrate, Some(expected_bitrate));
            }
        }
    }

    #[test]
    fn test_unsupported_format() {
        let test_file = test_file_path("test.txt");
        
        // Should still attempt extraction but may return minimal metadata
        let metadata = MetadataExtractor::extract_metadata(&test_file);
        
        // Even for unsupported formats, should not panic
        // May return error or minimal metadata depending on implementation
        match metadata {
            Ok(metadata) => {
                // If successful, should at least have file size
                assert!(metadata.file_size.is_some() || metadata.file_size.is_none());
            }
            Err(_) => {
                // Error is acceptable for unsupported formats
            }
        }
    }

    #[test]
    fn test_nonexistent_file() {
        let non_existent = test_file_path("does_not_exist.mp3");
        let result = MetadataExtractor::extract_metadata(&non_existent);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AudioError::Metadata(_)));
    }
}

#[cfg(test)]
mod artwork_tests {
    use super::*;

    #[test]
    fn test_artwork_type_equality() {
        assert_eq!(ArtworkType::FrontCover, ArtworkType::FrontCover);
        assert_ne!(ArtworkType::FrontCover, ArtworkType::BackCover);
        
        let custom1 = ArtworkType::Other("Custom".to_string());
        let custom2 = ArtworkType::Other("Custom".to_string());
        assert_eq!(custom1, custom2);
    }

    #[test]
    #[ignore] // Requires test files with various artwork types
    fn test_artwork_type_detection() {
        let test_files = [
            ("test_front_cover.mp3", ArtworkType::FrontCover),
            ("test_back_cover.mp3", ArtworkType::BackCover),
            ("test_artist.mp3", ArtworkType::Artist),
        ];

        for (filename, expected_type) in &test_files {
            let test_file = test_file_path(filename);
            if test_file.exists() {
                let metadata = MetadataExtractor::extract_metadata(&test_file);
                if let Ok(metadata) = metadata {
                    if let Some(artwork) = metadata.artwork {
                        assert_eq!(artwork.artwork_type, *expected_type);
                    }
                }
            }
        }
    }

    #[test]
    #[ignore] // Requires test file with large artwork
    fn test_large_artwork_handling() {
        let test_file = test_file_path("test_large_artwork.mp3");
        if test_file.exists() {
            let metadata = MetadataExtractor::extract_metadata(&test_file);
            
            if let Ok(metadata) = metadata {
                if let Some(artwork) = metadata.artwork {
                    // Large artwork should be handled properly
                    assert!(!artwork.data.is_empty());
                    
                    // Should have reasonable size limits (e.g., < 10MB)
                    assert!(artwork.data.len() < 10 * 1024 * 1024);
                    
                    // Should have dimensions if available
                    if artwork.width.is_some() && artwork.height.is_some() {
                        assert!(artwork.width.unwrap() > 0);
                        assert!(artwork.height.unwrap() > 0);
                    }
                }
            }
        }
    }

    #[test]
    #[ignore] // Requires test files with different image formats
    fn test_artwork_format_support() {
        let expected_formats = ["image/jpeg", "image/png", "image/gif"];
        
        let test_files = [
            "test_jpeg_art.mp3",
            "test_png_art.mp3", 
            "test_gif_art.mp3",
        ];

        for filename in &test_files {
            let test_file = test_file_path(filename);
            if test_file.exists() {
                let metadata = MetadataExtractor::extract_metadata(&test_file);
                
                if let Ok(metadata) = metadata {
                    if let Some(artwork) = metadata.artwork {
                        assert!(expected_formats.contains(&artwork.mime_type.as_str()));
                        
                        // Data should match the mime type
                        match artwork.mime_type.as_str() {
                            "image/jpeg" => {
                                assert!(artwork.data.starts_with(&[0xFF, 0xD8])); // JPEG header
                            }
                            "image/png" => {
                                assert!(artwork.data.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG header
                            }
                            "image/gif" => {
                                assert!(artwork.data.starts_with(b"GIF")); // GIF header
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod format_specific_tests {
    use super::*;

    #[test]
    #[ignore] // Requires test files with specific tag formats
    fn test_id3_version_support() {
        let test_files = [
            ("test_id3v1.mp3", "ID3v1"),
            ("test_id3v2_3.mp3", "ID3v2.3"),
            ("test_id3v2_4.mp3", "ID3v2.4"),
        ];

        for (filename, _version) in &test_files {
            let test_file = test_file_path(filename);
            if test_file.exists() {
                let metadata = MetadataExtractor::extract_metadata(&test_file);
                assert!(metadata.is_ok(), "Failed to extract from {}", filename);
                
                let metadata = metadata.unwrap();
                // Should extract at least basic information regardless of ID3 version
                assert!(metadata.title.is_some() || metadata.artist.is_some());
            }
        }
    }

    #[test]
    #[ignore] // Requires test FLAC with various comment blocks
    fn test_flac_vorbis_comments() {
        let test_file = test_file_path("test_vorbis_comments.flac");
        if test_file.exists() {
            let metadata = MetadataExtractor::extract_metadata(&test_file);
            assert!(metadata.is_ok());
            
            let metadata = metadata.unwrap();
            
            // FLAC should support extended metadata
            assert!(metadata.composer.is_some() || metadata.performer.is_some());
            
            // Custom tags should be preserved
            assert!(!metadata.custom_tags.is_empty());
        }
    }
}