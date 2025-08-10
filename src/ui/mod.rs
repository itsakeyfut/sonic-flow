//! User interface layer
//!
//! Slint-based UI implementations with Rust integration

pub mod bindings;

pub use bindings::MainWindowBinding;

use crate::error::{Result, UiError};

pub struct UiSystem {
    main_window: MainWindowBinding,
}

impl UiSystem {
    /// Create a new UI system
    pub fn new(event_bus: crate::app::EventBus) -> Result<Self> {
        let main_window = MainWindowBinding::new(event_bus)?;

        Ok(Self { main_window })
    }

    /// Get main window binding
    pub fn main_window(&self) -> &MainWindowBinding {
        &self.main_window
    }

    /// Run UI system
    pub fn run(&self) -> Result<()> {
        self.main_window.run()
    }
}
