//! Error handling for the Sonic Flow.
//! 
//! This module provides a hierarchical error system that allows for
//! proper error propagation and recovery strategies.

use thiserror::Error;

/// The main error type for the Sonic Flow application.
/// 
/// This error type aggregates all possible errors that can occur
/// within the application, providing a unified interface for error handling.
#[derive(Error, Debug)]
pub enum Error {
    /// Audio engine related errors
    #[error("Audio engine error: {0}")]
    Audio(#[from] AudioError),

    /// Database related errors
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    /// File system I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Plugin system errors
    #[error("Plugin error: {plugin} - {message}")]
    Plugin { plugin: String, message: String },

    /// UI related errors
    #[error("UI error: {0}")]
    Ui(#[from] UiError),

    /// Playlist management errors
    #[error("Playlist error: {0}")]
    Playlist(#[from] PlaylistError),

    /// Library management errors
    #[error("Library error: {0}")]
    Library(#[from] LibraryError),

    /// Visualizer errors
    #[error("Visualizer error: {0}")]
    Visualizer(#[from] VisualizerError),

    /// Generic application errors
    #[error("Application error: {0}")]
    Application(String),
}

/// Audio engine specific errors
#[derive(Error, Debug)]
pub enum AudioError {
    /// Audio decoder related errors
    #[error("Decoder error: {0}")]
    Decoder(#[from] DecoderError),

    /// Audio device related errors
    #[error("Device error: {0}")]
    Device(String),

    /// Unsupported audio format
    #[error("Format not supported: {format}")]
    UnsupportedFormat { format: String },

    /// Audio buffer underrun
    #[error("Buffer underrun")]
    BufferUnderrun,

    /// Invalid state transition
    #[error("Invalid state transition: {from} -> {to}")]
    InvalidState { from: String, to: String },

    /// Audio streaming error
    #[error("Streaming error: {0}")]
    Streaming(String),

    /// Effects processing error
    #[error("Effects processing error: {0}")]
    Effects(String),
}

/// Audio decoder specific errors
#[derive(Error, Debug)]
pub enum DecoderError {
    /// Failed to initialize decoder
    #[error("Failed to initialize decoder for format: {format}")]
    InitializationFailed { format: String },

    /// Corrupted audio data
    #[error("Corrupted audio data: {0}")]
    CorruptedData(String),

    ///Seek operation failed
    #[error("Seek operation failed: {0}")]
    SeekFailed(String),

    /// End of stream reached
    #[error("End of stream")]
    EndOfStream,
}

/// Database related errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// Database connection error
    #[error("Connection error: {0}")]
    Connection(#[from] sqlx::Error),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// Query execution error
    #[error("Query error: {0}")]
    Query(String),

    /// Data consistency error
    #[error("Data consistency error: {0}")]
    Consistency(String),
}

/// Configuration related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    /// Invalid configuration format
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(#[from] toml::de::Error),

    /// Missing required configuration
    #[error("Missing required configuration: {key}")]
    MissingRequired { key: String },

    /// Invalid configuration value
    #[error("Invalid configuration value for {key}: {value}")]
    InvalidState { key: String, value: String },

    /// Configuration serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),
}

/// UI related errors
#[derive(Error, Debug)]
pub enum UiError {
    /// Slint compilation error
    #[error("Slint compilation error: {0}")]
    SlintCompilation(String),

    /// Component not found
    #[error("Component not found: {component}")]
    ComponentNotFound { component: String },

    /// Theme loading error
    #[error("Theme loading error: {0}")]
    ThemeLoading(String),

    /// UI event handling error
    #[error("Event handling error: {0}")]
    EventHandling(String),
}

/// Playlist management errors
#[derive(Error, Debug)]
pub enum PlaylistError {
    /// Playlist not found
    #[error("Playlist not found: {id}")]
    NotFound { id: crate::PlaylistId },

    /// Track not in playlist
    #[error("Track not in playlist: {track_id}")]
    TrackNotFound { track_id: crate::TrackId },

    /// Invalid playlist format
    #[error("Invalid playlist format: {0}")]
    InvalidFormat(String),

    /// Playlist operation failed
    #[error("Playlist operation failed: {0}")]
    OperationFailed(String),
}

/// Library management errors
#[derive(Error, Debug)]
pub enum LibraryError {
    /// Track not found in library
    #[error("Track not found: {id}")]
    TrackNotFound { id: crate::TrackId },

    /// Metadata extraction failed
    #[error("Metadata extraction failed: {0}")]
    MetadataExtraction(String),

    /// Library scan failed
    #[error("Library scan failed: {0}")]
    ScanFailed(String),

    /// File watch error
    #[error("File watch error: {0}")]
    FileWatch(#[from] notify::Error),
}

/// Visualizer related errors
#[derive(Error, Debug)]
pub enum VisualizerError {
    /// Plugin not found
    #[error("Visualizer plugin not found: {name}")]
    PluginNotFound { name: String },

    /// Rendering error
    #[error("Rendering error: {0}")]
    Rendering(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Performance constraint violation
    #[error("Performance constraint violation: {constraint}")]
    PerformanceViolation { constraint: String },
}

/// Specialized Result type for the application
pub type Result<T> = std::result::Result<T, Error>;

/// Trait for error recovery strategies
pub trait ErrorRecovery {
    /// The error type this recovery strategy handles
    type Error;

    /// Check if the error can be recovered from
    fn can_recover(&self, error: &Self::Error) -> bool;

    /// Attempt to recover from the error
    fn recover(&mut self, error: Self::Error) -> std::result::Result<(), Self::Error>;
}

/// Erro severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Warning - operation can continue with degraded functionality
    Warning,
    /// Error - operation failed but application can recover
    Error,
    /// Critical - application state is compromised
    Critical,
}
