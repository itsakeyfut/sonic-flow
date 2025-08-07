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
