//! Error types for sonic-core.

use thiserror::Error;

/// Top-level error type for the sonic-core library.
#[derive(Error, Debug)]
pub enum Error {
    /// Audio subsystem error
    #[error("audio error: {0}")]
    Audio(#[from] AudioError),

    /// File system I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic application error
    #[error("{0}")]
    Application(String),
}

/// Audio subsystem errors.
#[derive(Error, Debug)]
pub enum AudioError {
    /// Audio device error
    #[error("device error: {0}")]
    Device(String),

    /// Unsupported audio format
    #[error("unsupported format: {format}")]
    UnsupportedFormat {
        /// The unsupported format identifier
        format: String,
    },

    /// Audio buffer underrun during playback
    #[error("buffer underrun")]
    BufferUnderrun,

    /// Invalid state transition
    #[error("invalid state transition: {from} -> {to}")]
    InvalidState {
        /// State transitioning from
        from: String,
        /// State transitioning to
        to: String,
    },

    /// Decode error (codec or container level)
    #[error("decode error: {0}")]
    Decode(String),

    /// Streaming / decoding error
    #[error("streaming error: {0}")]
    Streaming(String),

    /// Metadata extraction error
    #[error("metadata error: {0}")]
    Metadata(String),
}

/// Result alias using the top-level [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;
