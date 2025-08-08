//! Music library management

pub mod manager;
pub mod scanner;
pub mod metadata;
pub mod database;

pub use manager::LibraryManager;

/// Placeholder for library management
pub struct LibraryManager {
    _placeholder: (),
}

impl LibraryManager {
    pub fn new() -> crate::Result<Self> {
        Ok(Self { _placeholder: () })
    }
}
