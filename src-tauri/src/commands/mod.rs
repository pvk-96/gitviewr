pub mod export;
pub mod recent;
pub mod repository;
pub mod system;

use serde::Serialize;

/// Payload emitted to the frontend while analysis is running.
#[derive(Clone, Serialize)]
pub struct ProgressEvent {
    pub phase: String,
}
