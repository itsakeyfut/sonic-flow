//! Core audio engine implementation
//! 
//! This module provides the main AudioEngine that handles audio playback,
//! track management, and audio processing coordination.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use parking_lot::RwLock;
use rodio::{Decoder, OutputStream, Sink};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AudioError, DecoderError};
use crate::{Result, TrackId};

use super::traits::{
    AudioFormat, AudioFormatType, PlaybackControl, PlaybackState, PlaybackStatus, TrackLoader,
    VolumeControl,
};
