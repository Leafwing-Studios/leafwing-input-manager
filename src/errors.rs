//! Errors that may occur when working with 2D coordinates

use derive_more::{Display, Error};

/// The supplied vector-like struct was too close to zero to be converted into a rotation-like type
///
/// This error is produced when attempting to convert into a rotation-like type
/// such as a [`Rotation`] or [`Quat`](bevy::math::Quat) from a vector-like type
/// such as a [`Vec2`].
///
/// In almost all cases, the correct way to handle this error is to simply not change the rotation.
#[derive(Debug, Clone, Copy, Error, Display, PartialEq, Eq)]
pub struct NearlySingularConversion;
