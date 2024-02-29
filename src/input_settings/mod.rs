//! Utilities for configuring input settings and processors.
//!
//! # General Input Settings
//!
//! - [`axislike_settings`]: Utilities for configuring settings related to axis-like inputs.
//!
//! # General Input Processors
//!
//! - [`common_processors`]: Utilities for processing all kinds of inputs.
//! - [`deadzone_processors`]: Utilities for deadzone handling in input processing.

pub use self::axislike_settings::*;
pub use self::common_processors::*;
pub use self::deadzone_processors::*;

pub mod axislike_settings;

pub mod common_processors;
pub mod deadzone_processors;
