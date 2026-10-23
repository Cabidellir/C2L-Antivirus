//! Windows-specific platform adapter. Platform-specific behavior belongs here, not in the cross-platform core.
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(windows)]
mod vault_io;
#[cfg(windows)]
pub use vault_io::*;
#[cfg(windows)]
mod child_job;
#[cfg(windows)]
pub use child_job::ChildJob;

#[cfg(windows)]
pub mod service_host;

#[cfg(windows)]
pub mod notification;
