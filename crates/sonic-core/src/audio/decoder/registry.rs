//! Decoder registry — selects the appropriate decoder for a given file path.
//!
//! Currently all supported formats are handled by [`SymphoniaDecoder`], so
//! this module acts as a thin factory. It exists as an extension point for
//! future format-specific overrides.

use std::path::Path;

use crate::audio::traits::AudioFormatType;
use crate::error::AudioError;

use super::{AudioDecoder, SymphoniaDecoder};

/// Open an audio file and return a boxed [`AudioDecoder`].
///
/// The format is determined from the file extension. Returns
/// [`AudioError::UnsupportedFormat`] for unknown extensions before attempting
/// to open the file.
pub fn open(path: &Path) -> Result<Box<dyn AudioDecoder>, AudioError> {
    let ext =
        path.extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| AudioError::UnsupportedFormat {
                format: "no extension".into(),
            })?;

    let format_type = AudioFormatType::from_extension(ext);
    if !format_type.is_supported() {
        return Err(AudioError::UnsupportedFormat {
            format: ext.to_string(),
        });
    }

    let decoder = SymphoniaDecoder::open(path)?;
    Ok(Box::new(decoder))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn unsupported_extension_rejected_before_io() {
        let result = open(&PathBuf::from("music.xyz"));
        assert!(matches!(result, Err(AudioError::UnsupportedFormat { .. })));
    }

    #[test]
    fn missing_extension_rejected() {
        let result = open(&PathBuf::from("music_no_ext"));
        assert!(matches!(result, Err(AudioError::UnsupportedFormat { .. })));
    }
}
