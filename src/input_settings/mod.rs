//! Utilities for configuring input settings and processors.
//!
//! This crate provides utilities to configure input settings and process various types of inputs in applications or games.
//!
//! # General Input Settings
//!
//! The input settings modules provide utilities for configuring various aspects of input behavior.
//!
//! - [`single_axis_settings`]: Utilities for configuring settings related to single-axis inputs.
//! - [`dual_axis_settings`]: Utilities for configuring settings related to dual-axis inputs.
//!
//! # General Input Processors
//!
//! The input processors modules offer functionality for processing different types of inputs.
//!
//! - [`common_processors`]: Utilities for processing all kinds of inputs.
//! - [`deadzone_processors`]: Utilities for deadzone handling in input processing.

pub use self::dual_axis_settings::*;
pub use self::single_axis_settings::*;

pub use self::common_processors::*;
pub use self::deadzone_processors::*;

pub mod dual_axis_settings;
pub mod single_axis_settings;

pub mod common_processors;
pub mod deadzone_processors;
