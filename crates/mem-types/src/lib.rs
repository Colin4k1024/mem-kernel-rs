//! Core types and traits for MemOS-compatible memory API.
//!
//! Request/response DTOs align with MemOS `product_models.py` for JSON compatibility.

mod dto;
mod entity;
mod job;
mod lifecycle;
mod traits;

pub use dto::*;
pub use entity::*;
pub use job::*;
pub use lifecycle::*;
pub use traits::*;
