//! Audio format types shared across the library.

/// Audio format information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioFormat {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Bit depth (16, 24, etc.)
    pub bit_depth: u16,
    /// Format type
    pub format_type: AudioFormatType,
}

/// Supported audio format types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioFormatType {
    Mp3,
    Flac,
    Wav,
    Ogg,
    Aac,
    Opus,
    Unknown(String),
}

impl AudioFormatType {
    /// Create a format type from a file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp3" => Self::Mp3,
            "flac" => Self::Flac,
            "wav" => Self::Wav,
            "ogg" | "oga" => Self::Ogg,
            "aac" | "m4a" => Self::Aac,
            "opus" => Self::Opus,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Returns true if the format is supported for playback.
    pub fn is_supported(&self) -> bool {
        !matches!(self, Self::Unknown(_))
    }

    /// Returns a human-readable codec name for display.
    pub fn codec_name(&self) -> &str {
        match self {
            Self::Mp3 => "MP3",
            Self::Flac => "FLAC",
            Self::Wav => "PCM",
            Self::Ogg => "Vorbis",
            Self::Aac => "AAC",
            Self::Opus => "Opus",
            Self::Unknown(_) => "Unknown",
        }
    }

    /// Returns the format as a lowercase string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
            Self::Wav => "wav",
            Self::Ogg => "ogg",
            Self::Aac => "aac",
            Self::Opus => "opus",
            Self::Unknown(s) => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_type_from_extension() {
        assert_eq!(AudioFormatType::from_extension("mp3"), AudioFormatType::Mp3);
        assert_eq!(AudioFormatType::from_extension("MP3"), AudioFormatType::Mp3);
        assert_eq!(
            AudioFormatType::from_extension("flac"),
            AudioFormatType::Flac
        );
        assert_eq!(AudioFormatType::from_extension("ogg"), AudioFormatType::Ogg);
        assert_eq!(AudioFormatType::from_extension("oga"), AudioFormatType::Ogg);
        assert_eq!(
            AudioFormatType::from_extension("opus"),
            AudioFormatType::Opus
        );
        assert_eq!(
            AudioFormatType::from_extension("xyz"),
            AudioFormatType::Unknown("xyz".to_string())
        );
    }

    #[test]
    fn format_type_is_supported() {
        assert!(AudioFormatType::Mp3.is_supported());
        assert!(AudioFormatType::Flac.is_supported());
        assert!(AudioFormatType::Opus.is_supported());
        assert!(!AudioFormatType::Unknown("xyz".to_string()).is_supported());
    }
}
