//! Pure logic: no I/O, no toolkit. Everything here is unit tested.

pub mod client;
pub mod profiles;
pub mod realm;
pub mod settings;
pub mod status;

/// The only error consumer is a person reading the flyout, so errors are plain text.
pub type Result<T> = std::result::Result<T, String>;
