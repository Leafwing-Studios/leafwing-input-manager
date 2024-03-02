//! Input settings and processors.
//!
//! This crate provides utilities to configure input settings and process various types of inputs in applications or games.
//!
//! # General Input Settings
//!
//! The input settings modules provide utilities for configuring various aspects of input behavior.
//!
//! - [`single_axis_settings`]: Contains [`SingleAxisSettings`] and its processing pipeline.
//! - [`dual_axis_settings`]: Contains [`DualAxisSettings`] and its processing pipeline.
//!
//! # General Input Processors
//!
//! The input processors modules offer functionality for processing different types of inputs.
//!
//! - [`common_processors`]: Contains various input processing methods, e.g., limiting, normalization, etc.
//! - [`deadzones`]: Contains various deadzone regions, e.g., 1D-deadzone, 2D-deadzone, etc.

pub use self::dual_axis_settings::*;
pub use self::single_axis_settings::*;

pub use self::common_processors::*;
pub use self::deadzones::*;

pub mod dual_axis_settings;
pub mod single_axis_settings;

pub mod common_processors;
pub mod deadzones;
