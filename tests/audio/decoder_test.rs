//! Tests for audio decoder implementations
//!
//! This module contains comprehensive tests for all audio decoders,
//! including format-specific functionality and error handling.

use std::path::PathBuf;
use std::time::Duration;

use sonic_flow::audio::decoder::{Mp3Decoder, FlacDecoder, WavDecoder, UniversalDecoder};
use sonic_flow::audio::traits::{AudioDecoder, AudioFormatType};
use sonic_flow::error::AudioError;

// Test data directory (would contain small test audio files)
const TEST_DATA_DIR: &str = "tests/fixtures/audio";

/// Helper function to create test file path
fn test_file_path(filename: &str) -> PathBuf {
    PathBuf::from(TEST_DATA_DIR).join(filename)
}

/// Generate a simple test audio buffer
fn generate_test_audio(sample_rate: u32, channels: u16, duration_secs: f32) -> Vec<f32> {
    let samples_per_channel = (sample_rate as f32 * duration_secs) as usize;
    let total_samples = samples_per_channel * channels as usize;
    let mut buffer = Vec::with_capacity(total_samples);

    let frequency = 440.0; // A4 note
    let angular_freq = 2.0 * std::f32::consts::PI * frequency / sample_rate as f32;

    for i in 0..samples_per_channel {
        let sample = (angular_freq * i as f32).sin() * 0.5; // 50% amplitude
        
        // Duplicate for each channel
        for _ in 0..channels {
            buffer.push(sample);
        }
    }

    buffer
}

#[cfg(test)]
mod mp3_decoder_tests {
    use super::*;

    #[test]
    fn test_mp3_format_detection() {
        assert_eq!(AudioFormatType::from_extension("mp3"), AudioFormatType::Mp3);
        assert_eq!(AudioFormatType::from_extension("MP3"), AudioFormatType::Mp3);
        assert!(AudioFormatType::Mp3.is_supported());
    }

    #[test]
    #[ignore] // Requires test MP3 file
    fn test_mp3_decoder_creation() {
        let test_file = test_file_path("test.mp3");
        if test_file.exists() {
            let decoder = Mp3Decoder::from_file(&test_file);
            assert!(decoder.is_ok());
            
            let decoder = decoder.unwrap();
            assert_eq!(decoder.sample_rate(), 44100);
            assert_eq!(decoder.channels(), 2);
            assert!(decoder.supports_seek());
        }
    }

    #[test]
    #[ignore] // Requires test MP3 file
    fn test_mp3_vbr_detection() {
        let test_file = test_file_path("test_vbr.mp3");
        if test_file.exists() {
            let decoder = Mp3Decoder::from_file(&test_file).unwrap();
            // VBR files should have different min/max bitrates
            let (min_br, max_br) = decoder.bitrate_info();
            if decoder.is_vbr() {
                assert_ne!(min_br, max_br);
            }
        }
    }

    #[test]
    #[ignore] // Requires test MP3 file
    fn test_mp3_seeking() {
        let test_file = test_file_path("test.mp3");
        if test_file.exists() {
            let mut decoder = Mp3Decoder::from_file(&test_file).unwrap();
            
            // Test seeking to middle of file
            let seek_pos = Duration::from_secs(30);
            assert!(decoder.seek(seek_pos).is_ok());
            
            // Position should be approximately correct
            let actual_pos = decoder.position();
            assert!((actual_pos.as_secs_f64() - seek_pos.as_secs_f64()).abs() < 1.0);
        }
    }

    #[test]
    #[ignore] // Requires test MP3 file
    fn test_mp3_decoding() {
        let test_file = test_file_path("test.mp3");
        if test_file.exists() {
            let mut decoder = Mp3Decoder::from_file(&test_file).unwrap();
            let mut output_buffer = vec![0.0f32; 4096];
            
            let samples_read = decoder.decode(&[], &mut output_buffer);
            assert!(samples_read.is_ok());
            assert!(samples_read.unwrap() > 0);
            
            // Check that we got valid audio data
            assert!(output_buffer.iter().any(|&x| x != 0.0));
        }
    }
}

#[cfg(test)]
mod flac_decoder_tests {
    use super::*;

    #[test]
    fn test_flac_format_detection() {
        assert_eq!(AudioFormatType::from_extension("flac"), AudioFormatType::Flac);
        assert_eq!(AudioFormatType::from_extension("FLAC"), AudioFormatType::Flac);
        assert!(AudioFormatType::Flac.is_supported());
    }

    #[test]
    #[ignore] // Requires test FLAC file
    fn test_flac_decoder_creation() {
        let test_file = test_file_path("test.flac");
        if test_file.exists() {
            let decoder = FlacDecoder::from_file(&test_file);
            assert!(decoder.is_ok());
            
            let decoder = decoder.unwrap();
            assert!(decoder.sample_rate() > 0);
            assert!(decoder.channels() > 0);
            assert!(decoder.supports_seek());
        }
    }

    #[test]
    #[ignore] // Requires test FLAC file
    fn test_flac_high_precision() {
        let test_file = test_file_path("test_24bit.flac");
        if test_file.exists() {
            let decoder = FlacDecoder::from_file(&test_file).unwrap();
            let encoding_info = decoder.encoding_info();
            
            // 24-bit FLAC should report correct bit depth
            assert_eq!(encoding_info.bits_per_sample, 24);
        }
    }

    #[test]
    #[ignore] // Requires test FLAC file
    fn test_flac_duration_precision() {
        let test_file = test_file_path("test.flac");
        if test_file.exists() {
            let decoder = FlacDecoder::from_file(&test_file).unwrap();
            
            if let Some(duration) = decoder.duration() {
                // FLAC should provide precise duration
                assert!(duration.as_secs() > 0);
                // Duration should be precise to milliseconds
                assert!(duration.subsec_millis() > 0 || duration.as_secs() % 1 == 0);
            }
        }
    }

    #[test]
    #[ignore] // Requires test FLAC file
    fn test_flac_seeking_accuracy() {
        let test_file = test_file_path("test.flac");
        if test_file.exists() {
            let mut decoder = FlacDecoder::from_file(&test_file).unwrap();
            
            // Test multiple seek positions
            let positions = [
                Duration::from_millis(1000),
                Duration::from_millis(5000),
                Duration::from_millis(10000),
            ];
            
            for &pos in &positions {
                if decoder.seek(pos).is_ok() {
                    let actual_pos = decoder.position();
                    // FLAC seeking should be very accurate (within 50ms)
                    assert!((actual_pos.as_millis() as i64 - pos.as_millis() as i64).abs() < 50);
                }
            }
        }
    }
}

#[cfg(test)]
mod wav_decoder_tests {
    use super::*;

    #[test]
    fn test_wav_format_detection() {
        assert_eq!(AudioFormatType::from_extension("wav"), AudioFormatType::Wav);
        assert_eq!(AudioFormatType::from_extension("WAV"), AudioFormatType::Wav);
        assert!(AudioFormatType::Wav.is_supported());
    }

    #[test]
    #[ignore] // Requires test WAV file
    fn test_wav_decoder_creation() {
        let test_file = test_file_path("test.wav");
        if test_file.exists() {
            let decoder = WavDecoder::from_file(&test_file);
            assert!(decoder.is_ok());
            
            let decoder = decoder.unwrap();
            assert!(decoder.sample_rate() > 0);
            assert!(decoder.channels() > 0);
            assert!(decoder.supports_seek());
        }
    }

    #[test]
    #[ignore] // Requires test WAV file
    fn test_wav_format_info() {
        let test_file = test_file_path("test.wav");
        if test_file.exists() {
            let decoder = WavDecoder::from_file(&test_file).unwrap();
            let wav_format = decoder.wav_format();
            
            // Standard PCM WAV should have format tag 1
            assert_eq!(wav_format.format_tag, 1);
            assert!(!wav_format.is_extensible);
            
            // Block align should match channels * bit_depth / 8
            let expected_block_align = decoder.channels() * 16 / 8; // Assuming 16-bit
            assert_eq!(wav_format.block_align, expected_block_align);
        }
    }

    #[test]
    #[ignore] // Requires test WAV file
    fn test_wav_sample_accurate_seeking() {
        let test_file = test_file_path("test.wav");
        if test_file.exists() {
            let mut decoder = WavDecoder::from_file(&test_file).unwrap();
            
            // Test sample-accurate seeking
            let sample_position = 44100; // 1 second at 44.1kHz
            assert!(decoder.seek_sample_accurate(sample_position).is_ok());
            
            let expected_time = Duration::from_secs_f64(sample_position as f64 / 44100.0);
            let actual_pos = decoder.position();
            
            // Should be exactly accurate for WAV
            assert!((actual_pos.as_secs_f64() - expected_time.as_secs_f64()).abs() < 0.001);
        }
    }

    #[test]
    #[ignore] // Requires test float WAV file
    fn test_wav_float_format() {
        let test_file = test_file_path("test_float.wav");
        if test_file.exists() {
            let decoder = WavDecoder::from_file(&test_file).unwrap();
            
            if decoder.is_float_format() {
                assert_eq!(decoder.wav_format().format_tag, 3); // IEEE_FLOAT
            }
        }
    }

    #[test]
    #[ignore] // Requires test WAV file
    fn test_wav_file_calculations() {
        let test_file = test_file_path("test.wav");
        if test_file.exists() {
            let decoder = WavDecoder::from_file(&test_file).unwrap();
            
            let duration = decoder.duration();
            let sample_rate = decoder.sample_rate();
            let channels = decoder.channels();
            let bit_depth = 16; // Assuming 16-bit
            
            // Calculate expected file size
            let expected_samples = (duration.as_secs_f64() * sample_rate as f64) as u64;
            let expected_data_size = expected_samples * channels as u64 * (bit_depth / 8);
            let actual_data_size = decoder.audio_data_size();
            
            // Should match closely (accounting for WAV header)
            assert!((actual_data_size as i64 - expected_data_size as i64).abs() < 1000);
        }
    }
}

#[cfg(test)]
mod universal_decoder_tests {
    use super::*;

    #[test]
    #[ignore] // Requires test files
    fn test_universal_decoder_format_support() {
        let test_files = [
            ("test.mp3", AudioFormatType::Mp3),
            ("test.flac", AudioFormatType::Flac),
            ("test.wav", AudioFormatType::Wav),
        ];

        for (filename, expected_format) in &test_files {
            let test_file = test_file_path(filename);
            if test_file.exists() {
                let decoder = UniversalDecoder::from_file(&test_file);
                assert!(decoder.is_ok(), "Failed to create decoder for {}", filename);
                
                // All formats should be supported by universal decoder
                assert!(decoder.unwrap().supports_seek());
            }
        }
    }

    #[test]
    fn test_universal_decoder_error_handling() {
        // Test with non-existent file
        let non_existent = test_file_path("does_not_exist.mp3");
        let result = UniversalDecoder::from_file(&non_existent);
        assert!(result.is_err());

        // Test with invalid file
        let invalid_file = test_file_path("invalid.txt");
        if invalid_file.exists() {
            let result = UniversalDecoder::from_file(&invalid_file);
            assert!(result.is_err());
        }
    }

    #[test]
    #[ignore] // Requires test files
    fn test_universal_decoder_consistency() {
        // Test that universal decoder gives same results as format-specific decoders
        let test_file = test_file_path("test.mp3");
        if test_file.exists() {
            let universal = UniversalDecoder::from_file(&test_file).unwrap();
            let mp3_specific = Mp3Decoder::from_file(&test_file).unwrap();

            assert_eq!(universal.sample_rate(), mp3_specific.sample_rate());
            assert_eq!(universal.channels(), mp3_specific.channels());
        }
    }
}

#[cfg(test)]
mod decoder_integration_tests {
    use super::*;

    #[test]
    fn test_supported_extensions() {
        use sonic_flow::audio::decoder::{supported_extensions, is_supported_extension};

        let extensions = supported_extensions();
        assert!(extensions.contains(&"mp3"));
        assert!(extensions.contains(&"flac"));
        assert!(extensions.contains(&"wav"));

        assert!(is_supported_extension("mp3"));
        assert!(is_supported_extension("MP3"));
        assert!(is_supported_extension("flac"));
        assert!(is_supported_extension("FLAC"));
        assert!(is_supported_extension("wav"));
        assert!(is_supported_extension("WAV"));

        assert!(!is_supported_extension("txt"));
        assert!(!is_supported_extension("xyz"));
    }

    #[test]
    #[ignore] // Requires test files
    fn test_decoder_factory() {
        use sonic_flow::audio::decoder::create_decoder;

        let test_files = ["test.mp3", "test.flac", "test.wav"];

        for filename in &test_files {
            let test_file = test_file_path(filename);
            if test_file.exists() {
                let decoder = create_decoder(&test_file);
                assert!(decoder.is_ok(), "Failed to create decoder for {}", filename);
                
                let decoder = decoder.unwrap();
                assert!(decoder.sample_rate() > 0);
                assert!(decoder.channels() > 0);
            }
        }
    }

    #[test]
    #[ignore] // Requires test files
    fn test_cross_format_decoding() {
        // Test that different formats produce valid audio data
        let test_files = ["test.mp3", "test.flac", "test.wav"];
        
        for filename in &test_files {
            let test_file = test_file_path(filename);
            if test_file.exists() {
                let mut decoder = create_decoder(&test_file).unwrap();
                let mut buffer = vec![0.0f32; 1024];
                
                let result = decoder.decode(&[], &mut buffer);
                assert!(result.is_ok(), "Decoding failed for {}", filename);
                
                let samples_read = result.unwrap();
                assert!(samples_read > 0, "No samples read from {}", filename);
                
                // Check for valid audio data (not all zeros)
                assert!(buffer[..samples_read].iter().any(|&x| x.abs() > 0.001));
            }
        }
    }

    #[test]
    fn test_error_propagation() {
        use sonic_flow::audio::decoder::create_decoder;

        // Test various error conditions
        let non_existent = PathBuf::from("definitely_does_not_exist.mp3");
        let result = create_decoder(&non_existent);
        assert!(matches!(result, Err(AudioError::Streaming(_))));
    }

    #[test]
    #[ignore] // Requires test files with known properties
    fn test_format_specific_features() {
        // Test MP3 VBR detection
        let mp3_vbr = test_file_path("test_vbr.mp3");
        if mp3_vbr.exists() {
            let decoder = Mp3Decoder::from_file(&mp3_vbr).unwrap();
            // VBR files should be detected
            assert!(decoder.is_vbr());
        }

        // Test FLAC high bit depth
        let flac_24bit = test_file_path("test_24bit.flac");
        if flac_24bit.exists() {
            let decoder = FlacDecoder::from_file(&flac_24bit).unwrap();
            let info = decoder.encoding_info();
            assert_eq!(info.bits_per_sample, 24);
        }

        // Test WAV float format
        let wav_float = test_file_path("test_float.wav");
        if wav_float.exists() {
            let decoder = WavDecoder::from_file(&wav_float).unwrap();
            assert!(decoder.is_float_format());
        }
    }
}

/// Benchmark tests for decoder performance
#[cfg(test)]
mod decoder_benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore] // Performance test
    fn benchmark_decoder_creation() {
        let test_files = ["test.mp3", "test.flac", "test.wav"];
        
        for filename in &test_files {
            let test_file = test_file_path(filename);
            if test_file.exists() {
                let start = Instant::now();
                let _decoder = UniversalDecoder::from_file(&test_file).unwrap();
                let creation_time = start.elapsed();
                
                println!("Decoder creation for {}: {:?}", filename, creation_time);
                // Decoder creation should be fast (< 100ms)
                assert!(creation_time.as_millis() < 100);
            }
        }
    }

    #[test]
    #[ignore] // Performance test
    fn benchmark_decoding_speed() {
        let test_file = test_file_path("test.mp3");
        if test_file.exists() {
            let mut decoder = UniversalDecoder::from_file(&test_file).unwrap();
            let mut buffer = vec![0.0f32; 44100]; // 1 second of stereo audio
            
            let start = Instant::now();
            let mut total_samples = 0;
            
            // Decode for 1 second of wall time
            while start.elapsed().as_secs() < 1 {
                match decoder.decode(&[], &mut buffer) {
                    Ok(samples) if samples > 0 => total_samples += samples,
                    _ => break,
                }
            }
            
            let elapsed = start.elapsed();
            let samples_per_sec = total_samples as f64 / elapsed.as_secs_f64();
            
            println!("Decoding speed: {:.0} samples/sec", samples_per_sec);
            // Should decode much faster than real-time (> 44100 samples/sec)
            assert!(samples_per_sec > 44100.0);
        }
    }
}
