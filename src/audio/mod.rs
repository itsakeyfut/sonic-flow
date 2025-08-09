//! Audio engine and processing

pub mod traits;
pub mod engine;
pub mod decoder;
pub mod renderer;
pub mod effects;
pub mod analysis;

pub use engine::AudioEngine;

/// Placeholder for now - will be implemented in audio engine feature
pub struct AudioEngine {
    _placeholder: (),
}

impl AudioEngine {
    pub fn new() -> crate::Result<Self> {
        Ok(Self { _placeholder: () })
    }
}
