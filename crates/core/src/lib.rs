//! Pure logic: no I/O, no toolkit. Everything here is unit tested.

pub mod status;

/// The only error consumer is a person reading the flyout, so errors are plain text.
pub type Result<T> = std::result::Result<T, String>;
