//! A combination of buttons, pressed simultaneously.

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::user_inputs::UserInput;

/// A combined input with two inner [`UserInput`]s.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize)]
pub struct Combined2Inputs<'a, U1, U2>
where
    U1: UserInput<'a> + Deserialize<'a>,
    U2: UserInput<'a> + Deserialize<'a>,
{
    inner: (U1, U2),
    _phantom_data: PhantomData<(&'a U1, &'a U2)>,
}

/// A combined input with three inner [`UserInput`]s.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize)]
pub struct Combined3Inputs<'a, U1, U2, U3>
where
    U1: UserInput<'a> + Deserialize<'a>,
    U2: UserInput<'a> + Deserialize<'a>,
    U3: UserInput<'a> + Deserialize<'a>,
{
    inner: (U1, U2, U3),
    _phantom_data: PhantomData<(&'a U1, &'a U2, &'a U3)>,
}
