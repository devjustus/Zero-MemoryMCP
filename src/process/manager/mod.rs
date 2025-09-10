//! Process management module exports

pub mod attacher;
pub mod detacher;

pub use attacher::{AttachOptions, AttachmentGuard, ProcessAttacher};
pub use detacher::{DetachOptions, ProcessDetacher};
