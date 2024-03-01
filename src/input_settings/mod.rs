//! Utilities for configuring input settings and processors.
//!
//! This crate provides utilities to configure input settings and process various types of inputs in applications or games.
//!
//! # General Input Settings
//!
//! The input settings modules provide utilities for configuring various aspects of input behavior.
//!
//! - [`single_axis_settings`]: Settings for single-axis input.
//! - [`dual_axis_settings`]: Settings for dual-axis input.
//!
//! # General Input Processors
//!
//! The input processors modules offer functionality for processing different types of inputs.
//!
//! - [`common_processors`]: Utilities for various input processing methods.
//! - [`deadzones`]: Utilities for various deadzone regions.

pub use self::dual_axis_settings::*;
pub use self::single_axis_settings::*;

pub use self::common_processors::*;
pub use self::deadzones::*;

pub mod dual_axis_settings;
pub mod single_axis_settings;

pub mod common_processors;
pub mod deadzones;
