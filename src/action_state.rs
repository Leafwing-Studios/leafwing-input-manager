//! This module contains [`ActionState`] and its supporting methods and impls.

use crate::Actionlike;
use crate::{axislike::DualAxisData, buttonlike::ButtonState};

use bevy::ecs::{component::Component, entity::Entity};
use bevy::math::Vec2;
use bevy::prelude::{Event, Resource};
use bevy::reflect::Reflect;
use bevy::utils::hashbrown::hash_set::Iter;
use bevy::utils::{Duration, Entry, HashMap, HashSet, Instant};
use serde::{Deserialize, Serialize};
use std::iter::Once;

/// Metadata about an [`Actionlike`] action
///
/// If a button is released, its `reasons_pressed` should be empty.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct ActionData {
    /// Is the action pressed or released?
    pub state: ButtonState,
    /// The "value" of the binding that triggered the action.
    ///
    /// See [`ActionState::value`] for more details.
    ///
    /// **Warning:** this value may not be bounded as you might expect.
    /// Consider clamping this to account for multiple triggering inputs.
    pub value: f32,
    /// The [`DualAxisData`] of the binding that triggered the action.
    pub axis_pair: Option<DualAxisData>,
    /// When was the button pressed / released, and how long has it been held for?
    pub timing: Timing,
    /// Was this action consumed by [`ActionState::consume`]?
    ///
    /// Actions that are consumed cannot be pressed again until they are explicitly released.
    /// This ensures that consumed actions are not immediately re-pressed by continued inputs.
    pub consumed: bool,
}

/// Stores the canonical input-method-agnostic representation of the inputs received
///
/// Can be used as either a resource or as a [`Component`] on entities that you wish to control directly from player input.
///
/// # Example
/// ```rust
/// use bevy::reflect::Reflect;
/// use leafwing_input_manager::prelude::*;
/// use bevy::utils::Instant;
///
/// #[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
/// enum Action {
///     Left,
///     Right,
///     Jump,
/// }
///
/// let mut action_state = ActionState::<Action>::default();
///
/// // Typically, this is done automatically by the `InputManagerPlugin` from user inputs
/// // using the `ActionState::update` method
/// action_state.press(&Action::Jump);
///
/// assert!(action_state.pressed(&Action::Jump));
/// assert!(action_state.just_pressed(&Action::Jump));
/// assert!(action_state.released(&Action::Left));
///
/// // Resets just_pressed and just_released
/// let t0 = Instant::now();
/// let t1 = Instant::now();
///
///  action_state.tick(t1, t0);
/// assert!(action_state.pressed(&Action::Jump));
/// assert!(!action_state.just_pressed(&Action::Jump));
///
/// action_state.release(&Action::Jump);
/// assert!(!action_state.pressed(&Action::Jump));
/// assert!(action_state.released(&Action::Jump));
/// assert!(action_state.just_released(&Action::Jump));
///
/// let t2 = Instant::now();
/// action_state.tick(t2, t1);
/// assert!(action_state.released(&Action::Jump));
/// assert!(!action_state.just_released(&Action::Jump));
/// ```
#[derive(Resource, Component, Clone, Debug, PartialEq, Serialize, Deserialize, Reflect)]
pub struct ActionState<A: Actionlike> {
    /// The [`ActionData`] of each action
    ///
    /// The position in this vector corresponds to [`Actionlike::index`].
    action_data: HashMap<A, ActionData>,
}

// The derive does not work unless A: Default,
// so we have to implement it manually
impl<A: Actionlike> Default for ActionState<A> {
    fn default() -> Self {
        Self {
            action_data: HashMap::default(),
        }
    }
}

impl<A: Actionlike> ActionState<A> {
    /// Updates the [`ActionState`] based on a vector of [`ActionData`], ordered by [`Actionlike::id`](Actionlike).
    ///
    /// The `action_data` is typically constructed from [`InputMap::which_pressed`](crate::input_map::InputMap),
    /// which reads from the assorted [`Input`](bevy::input::Input) resources.
    pub fn update(&mut self, action_data: HashMap<A, ActionData>) {
        for (action, action_datum) in action_data {
            match self.action_data.entry(action) {
                Entry::Occupied(occupied_entry) => {
                    let entry = occupied_entry.into_mut();

                    match action_datum.state {
                        ButtonState::JustPressed => entry.state.press(),
                        ButtonState::Pressed => entry.state.press(),
                        ButtonState::JustReleased => entry.state.release(),
                        ButtonState::Released => entry.state.release(),
                    }

                    entry.axis_pair = action_datum.axis_pair;
                    entry.value = action_datum.value;
                }
                Entry::Vacant(empty_entry) => {
                    empty_entry.insert(action_datum.clone());
                }
            }
        }
    }

    /// Advances the time for all actions
    ///
    /// The underlying [`Timing`] and [`ButtonState`] will be advanced according to the `current_instant`.
    /// - if no [`Instant`] is set, the `current_instant` will be set as the initial time at which the button was pressed / released
    /// - the [`Duration`] will advance to reflect elapsed time
    ///
    ///
    /// # Example
    /// ```rust
    /// use bevy::prelude::Reflect;
    /// use leafwing_input_manager::prelude::*;
    /// use leafwing_input_manager::buttonlike::ButtonState;
    /// use bevy::utils::Instant;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// let mut action_state = ActionState::<Action>::default();
    ///
    /// // Actions start released
    /// assert!(action_state.released(&Action::Jump));
    /// assert!(!action_state.just_released(&Action::Run));
    ///
    /// // Ticking time moves causes buttons that were just released to no longer be just released
    /// let t0 = Instant::now();
    /// let t1 = Instant::now();
    ///
    /// action_state.tick(t1, t0);
    /// assert!(action_state.released(&Action::Jump));
    /// assert!(!action_state.just_released(&Action::Jump));
    ///
    /// action_state.press(&Action::Jump);
    /// assert!(action_state.just_pressed(&Action::Jump));
    ///
    /// // Ticking time moves causes buttons that were just pressed to no longer be just pressed
    /// let t2 = Instant::now();
    ///
    /// action_state.tick(t2, t1);
    /// assert!(action_state.pressed(&Action::Jump));
    /// assert!(!action_state.just_pressed(&Action::Jump));
    /// ```
    pub fn tick(&mut self, current_instant: Instant, previous_instant: Instant) {
        // Advanced the ButtonState
        self.action_data
            .iter_mut()
            .for_each(|(_, ad)| ad.state.tick());

        // Advance the Timings
        self.action_data.iter_mut().for_each(|(_, ad)| {
            // Durations should not advance while actions are consumed
            if !ad.consumed {
                ad.timing.tick(current_instant, previous_instant);
            }
        });
    }

    /// A reference to the [`ActionData`] of the corresponding `action` if populated.
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    #[inline]
    #[must_use]
    pub fn action_data(&self, action: &A) -> Option<&ActionData> {
        self.action_data.get(action)
    }

    /// A mutable reference of the [`ActionData`] of the corresponding `action` if populated.
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    #[inline]
    #[must_use]
    pub fn action_data_mut(&mut self, action: &A) -> Option<&mut ActionData> {
        self.action_data.get_mut(action)
    }

    /// Get the value associated with the corresponding `action` if present.
    ///
    /// Different kinds of bindings have different ways of calculating the value:
    ///
    /// - Binary buttons will have a value of `0.0` when the button is not pressed, and a value of
    /// `1.0` when the button is pressed.
    /// - Some axes, such as an analog stick, will have a value in the range `-1.0..=1.0`.
    /// - Some axes, such as a variable trigger, will have a value in the range `0.0..=1.0`.
    /// - Some buttons will also return a value in the range `0.0..=1.0`, such as analog gamepad
    /// triggers which may be tracked as buttons or axes. Examples of these include the Xbox LT/RT
    /// triggers and the Playstation L2/R2 triggers. See also the `axis_inputs` example in the
    /// repository.
    /// - Dual axis inputs will return the magnitude of its [`DualAxisData`] and will be in the range
    /// `0.0..=1.0`.
    /// - Chord inputs will return the value of its first input.
    ///
    /// If multiple inputs trigger the same game action at the same time, the value of each
    /// triggering input will be added together.
    ///
    /// # Warnings
    ///
    /// This value will be 0. if the action has never been pressed or released.
    ///
    /// This value may not be bounded as you might expect.
    /// Consider clamping this to account for multiple triggering inputs,
    /// typically using the [`clamped_value`](Self::clamped_value) method instead.
    pub fn value(&self, action: &A) -> f32 {
        match self.action_data(action) {
            Some(action_data) => action_data.value,
            None => 0.0,
        }
    }

    /// Get the value associated with the corresponding `action`, clamped to `[-1.0, 1.0]`.
    ///
    /// # Warning
    ///
    /// This value will be 0. if the action has never been pressed or released.
    pub fn clamped_value(&self, action: &A) -> f32 {
        self.value(action).clamp(-1., 1.)
    }

    /// Get the [`DualAxisData`] from the binding that triggered the corresponding `action`.
    ///
    /// Only certain events such as [`VirtualDPad`][crate::axislike::VirtualDPad] and
    /// [`DualAxis`][crate::axislike::DualAxis] provide an [`DualAxisData`], and this
    /// will return [`None`] for other events.
    ///
    /// Chord inputs will return the [`DualAxisData`] of it's first input.
    ///
    /// If multiple inputs with an axis pair trigger the same game action at the same time, the
    /// value of each axis pair will be added together.
    ///
    /// # Warning
    ///
    /// These values may not be bounded as you might expect.
    /// Consider clamping this to account for multiple triggering inputs,
    /// typically using the [`clamped_axis_pair`](Self::clamped_axis_pair) method instead.
    pub fn axis_pair(&self, action: &A) -> Option<DualAxisData> {
        let action_data = self.action_data(action)?;
        action_data.axis_pair
    }

    /// Get the [`DualAxisData`] associated with the corresponding `action`, clamped to `[-1.0, 1.0]`.
    pub fn clamped_axis_pair(&self, action: &A) -> Option<DualAxisData> {
        self.axis_pair(action)
            .map(|pair| DualAxisData::new(pair.x().clamp(-1.0, 1.0), pair.y().clamp(-1.0, 1.0)))
    }

    /// Manually sets the [`ActionData`] of the corresponding `action`
    ///
    /// You should almost always use more direct methods, as they are simpler and less error-prone.
    ///
    /// However, this method can be useful for testing,
    /// or when transferring [`ActionData`] between action states.
    ///
    /// # Example
    /// ```rust
    /// use bevy::prelude::Reflect;
    /// use leafwing_input_manager::prelude::*;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
    /// enum AbilitySlot {
    ///     Slot1,
    ///     Slot2,
    /// }
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// let mut ability_slot_state = ActionState::<AbilitySlot>::default();
    /// let mut action_state = ActionState::<Action>::default();
    ///
    /// // Extract the state from the ability slot
    /// let slot_1_state = ability_slot_state.action_data(&AbilitySlot::Slot1);
    ///
    /// // And transfer it to the actual ability that we care about
    /// // without losing timing information
    /// if let Some(state) = slot_1_state {
    ///    action_state.set_action_data(Action::Run, state.clone());
    /// }
    /// ```
    #[inline]
    pub fn set_action_data(&mut self, action: A, data: ActionData) {
        self.action_data.insert(action, data);
    }

    /// Press the `action`
    ///
    /// No initial instant or reasons why the button was pressed will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    pub fn press(&mut self, action: &A) {
        let action_data = match self.action_data_mut(action) {
            Some(action_data) => action_data,
            None => {
                self.set_action_data(action.clone(), ActionData::default());
                self.action_data_mut(action).unwrap()
            }
        };

        // Consumed actions cannot be pressed until they are released
        if action_data.consumed {
            return;
        }

        if action_data.state.released() {
            action_data.timing.flip();
        }

        action_data.state.press();
    }

    /// Release the `action`
    ///
    /// No initial instant will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    pub fn release(&mut self, action: &A) {
        let action_data = match self.action_data_mut(action) {
            Some(action_data) => action_data,
            None => {
                self.set_action_data(action.clone(), ActionData::default());
                self.action_data_mut(action).unwrap()
            }
        };

        // Once released, consumed actions can be pressed again
        action_data.consumed = false;

        if action_data.state.pressed() {
            action_data.timing.flip();
        }

        action_data.state.release();
    }

    /// Consumes the `action`
    ///
    /// The action will be released, and will not be able to be pressed again
    /// until it would have otherwise been released by [`ActionState::release`],
    /// [`ActionState::release_all`] or [`ActionState::update`].
    ///
    /// No initial instant will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    ///
    /// # Example
    ///
    /// ```rust
    /// use bevy::prelude::Reflect;
    /// use leafwing_input_manager::prelude::*;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
    /// enum Action {
    ///     Eat,
    ///     Sleep,
    /// }
    ///
    /// let mut action_state = ActionState::<Action>::default();
    ///
    /// action_state.press(&Action::Eat);
    /// assert!(action_state.pressed(&Action::Eat));
    ///
    /// // Consuming actions releases them
    /// action_state.consume(&Action::Eat);
    /// assert!(action_state.released(&Action::Eat));
    ///
    /// // Doesn't work, as the action was consumed
    /// action_state.press(&Action::Eat);
    /// assert!(action_state.released(&Action::Eat));
    ///
    /// // Releasing consumed actions allows them to be pressed again
    /// action_state.release(&Action::Eat);
    /// action_state.press(&Action::Eat);
    /// assert!(action_state.pressed(&Action::Eat));
    /// ```
    #[inline]
    pub fn consume(&mut self, action: &A) {
        let action_data = match self.action_data_mut(action) {
            Some(action_data) => action_data,
            None => {
                self.set_action_data(action.clone(), ActionData::default());
                self.action_data_mut(action).unwrap()
            }
        };

        // This is the only difference from action_state.release(&action)
        action_data.consumed = true;
        action_data.state.release();
        action_data.timing.flip();
    }

    /// Consumes all actions
    #[inline]
    pub fn consume_all(&mut self) {
        for action in self.keys() {
            self.consume(&action);
        }
    }

    /// Releases all actions
    pub fn release_all(&mut self) {
        for action in self.keys() {
            self.release(&action);
        }
    }

    /// Is this `action` currently consumed?
    #[inline]
    #[must_use]
    pub fn consumed(&self, action: &A) -> bool {
        match self.action_data(action) {
            Some(action_data) => action_data.consumed,
            None => false,
        }
    }

    /// Is this `action` currently pressed?
    #[inline]
    #[must_use]
    pub fn pressed(&self, action: &A) -> bool {
        match self.action_data(action) {
            Some(action_data) => action_data.state.pressed(),
            None => false,
        }
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    #[inline]
    #[must_use]
    pub fn just_pressed(&self, action: &A) -> bool {
        match self.action_data(action) {
            Some(action_data) => action_data.state.just_pressed(),
            None => false,
        }
    }

    /// Is this `action` currently released?
    ///
    /// This is always the logical negation of [pressed](ActionState::pressed)
    #[inline]
    #[must_use]
    pub fn released(&self, action: &A) -> bool {
        match self.action_data(action) {
            Some(action_data) => action_data.state.released(),
            None => true,
        }
    }

    /// Was this `action` released since the last time [tick](ActionState::tick) was called?
    #[inline]
    #[must_use]
    pub fn just_released(&self, action: &A) -> bool {
        match self.action_data(action) {
            Some(action_data) => action_data.state.just_released(),
            None => false,
        }
    }

    #[must_use]
    /// Which actions are currently pressed?
    pub fn get_pressed(&self) -> Vec<A> {
        self.action_data
            .iter()
            .filter(|(_action, data)| data.state.pressed())
            .map(|(action, _data)| action.clone())
            .collect()
    }

    #[must_use]
    /// Which actions were just pressed?
    pub fn get_just_pressed(&self) -> Vec<A> {
        self.action_data
            .iter()
            .filter(|(_action, data)| data.state.just_pressed())
            .map(|(action, _data)| action.clone())
            .collect()
    }

    #[must_use]
    /// Which actions are currently released?
    pub fn get_released(&self) -> Vec<A> {
        self.action_data
            .iter()
            .filter(|(_action, data)| data.state.released())
            .map(|(action, _data)| action.clone())
            .collect()
    }

    #[must_use]
    /// Which actions were just released?
    pub fn get_just_released(&self) -> Vec<A> {
        self.action_data
            .iter()
            .filter(|(_action, data)| data.state.just_released())
            .map(|(action, _data)| action.clone())
            .collect()
    }

    /// The [`Instant`] that the action was last pressed or released
    ///
    ///
    ///
    /// If the action was pressed or released since the last time [`ActionState::tick`] was called
    /// the value will be [`None`].
    /// This ensures that all of our actions are assigned a timing and duration
    /// that corresponds exactly to the start of a frame, rather than relying on idiosyncratic timing.
    ///
    /// This will also be [`None`] if the action was never pressed or released.
    pub fn instant_started(&self, action: &A) -> Option<Instant> {
        let action_data = self.action_data(action)?;
        action_data.timing.instant_started
    }

    /// The [`Duration`] for which the action has been held or released
    ///
    /// This will be [`Duration::ZERO`] if the action was never pressed or released.
    pub fn current_duration(&self, action: &A) -> Duration {
        let Some(action_data) = self.action_data(action) else {
            return Duration::ZERO;
        };
        action_data.timing.current_duration
    }

    /// The [`Duration`] for which the action was last held or released
    ///
    /// This is a snapshot of the [`ActionState::current_duration`] state at the time
    /// the action was last pressed or released.
    ///
    /// This will be [`Duration::ZERO`] if the action was never pressed or released.
    pub fn previous_duration(&self, action: &A) -> Duration {
        let Some(action_data) = self.action_data(action) else {
            return Duration::ZERO;
        };
        action_data.timing.previous_duration
    }

    /// Applies an [`ActionDiff`] (usually received over the network) to the [`ActionState`].
    ///
    /// This lets you reconstruct an [`ActionState`] from a stream of [`ActionDiff`]s
    pub fn apply_diff(&mut self, action_diff: &ActionDiff<A>) {
        match action_diff {
            ActionDiff::Pressed { action } => {
                self.press(action);
                // Pressing will initialize the ActionData if it doesn't exist
                self.action_data_mut(action).unwrap().value = 1.;
            }
            ActionDiff::Released { action } => {
                self.release(action);
                // Releasing will initialize the ActionData if it doesn't exist
                let action_data = self.action_data_mut(action).unwrap();
                action_data.value = 0.;
                action_data.axis_pair = None;
            }
            ActionDiff::ValueChanged { action, value } => {
                self.press(action);
                // Pressing will initialize the ActionData if it doesn't exist
                self.action_data_mut(action).unwrap().value = *value;
            }
            ActionDiff::AxisPairChanged { action, axis_pair } => {
                self.press(action);
                let action_data = self.action_data_mut(action).unwrap();
                // Pressing will initialize the ActionData if it doesn't exist
                action_data.axis_pair = Some(DualAxisData::from_xy(*axis_pair));
                action_data.value = axis_pair.length();
            }
        };
    }

    /// Returns an owned list of the [`Actionlike`] keys in this [`ActionState`].
    #[inline]
    #[must_use]
    pub fn keys(&self) -> Vec<A> {
        self.action_data.keys().cloned().collect()
    }
}

/// A component that allows the attached entity to drive the [`ActionState`] of the associated entity
///
/// # Examples
///
/// By default, [`update_action_state_from_interaction`](crate::systems::update_action_state_from_interaction) uses this component
/// in order to connect `bevy::ui` buttons to the corresponding `ActionState`.
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// #[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
/// enum DanceDance {
///     Left,
///     Right,
///     Up,
///     Down,
/// }
///
/// // Spawn entity to track dance inputs
/// let mut world = World::new();
/// let dance_tracker = world
///     .spawn(ActionState::<DanceDance>::default())
///     .id();
///
/// // Spawn a button, which is wired up to the dance tracker
/// // When used with InputManagerPlugin<DanceDance>, this button will press the DanceDance::Left action when it is pressed.
/// world
///     .spawn(ButtonBundle::default())
///     // This component links the button to the entity with the `ActionState` component
///     .insert(ActionStateDriver {
///         action: DanceDance::Left,
///         targets: dance_tracker.into(),
///     });
///```
///
/// Writing your own systems that use the [`ActionStateDriver`] component is easy,
/// although this should be reserved for cases where the entity whose value you want to check
/// is distinct from the entity whose [`ActionState`] you want to set.
/// Check the source code of [`update_action_state_from_interaction`](crate::systems::update_action_state_from_interaction) for an example of how this is done.
#[derive(Debug, Component, Clone, PartialEq, Eq)]
pub struct ActionStateDriver<A: Actionlike> {
    /// The action triggered by this entity
    pub action: A,
    /// The entity whose action state should be updated
    pub targets: ActionStateDriverTarget,
}

/// Represents the entities that an ``ActionStateDriver`` targets.
#[derive(Debug, Component, Clone, PartialEq, Eq)]
pub enum ActionStateDriverTarget {
    /// No targets
    None,
    /// Single target
    Single(Entity),
    /// Multiple targets
    Multi(HashSet<Entity>),
}

impl ActionStateDriverTarget {
    /// Get an iterator for the entities targeted.
    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        match self {
            Self::None => ActionStateDriverTargetIterator::None,
            Self::Single(entity) => {
                ActionStateDriverTargetIterator::Single(std::iter::once(entity))
            }
            Self::Multi(entities) => ActionStateDriverTargetIterator::Multi(entities.iter()),
        }
    }

    /// Insert an entity as a target.
    #[inline(always)]
    pub fn insert(&mut self, entity: Entity) {
        // Don't want to copy a bunch of logic, switch out the ref, then replace it
        // rust doesn't like in place replacement
        *self = std::mem::replace(self, Self::None).with(entity);
    }

    /// Remove an entity as a target if it's in the target set.
    #[inline(always)]
    pub fn remove(&mut self, entity: Entity) {
        // see insert
        *self = std::mem::replace(self, Self::None).without(entity);
    }

    /// Add an entity as a target.
    #[inline(always)]
    pub fn add(&mut self, entities: impl Iterator<Item = Entity>) {
        for entity in entities {
            self.insert(entity)
        }
    }

    /// Get the number of targets.
    #[inline(always)]
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Single(_) => 1,
            Self::Multi(targets) => targets.len(),
        }
    }

    /// Returns true if there are no targets.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Add an entity as a target using a builder style pattern.
    #[inline(always)]
    pub fn with(mut self, entity: Entity) -> Self {
        match self {
            Self::None => Self::Single(entity),
            Self::Single(og) => Self::Multi(HashSet::from([og, entity])),
            Self::Multi(ref mut targets) => {
                targets.insert(entity);
                self
            }
        }
    }

    /// Remove an entity as a target if it's in the set using a builder style pattern.
    pub fn without(self, entity: Entity) -> Self {
        match self {
            Self::None => Self::None,
            Self::Single(_) => Self::None,
            Self::Multi(mut targets) => {
                targets.remove(&entity);
                Self::from_iter(targets)
            }
        }
    }
}

impl From<Entity> for ActionStateDriverTarget {
    fn from(value: Entity) -> Self {
        Self::Single(value)
    }
}

impl From<()> for ActionStateDriverTarget {
    fn from(_value: ()) -> Self {
        Self::None
    }
}

impl FromIterator<Entity> for ActionStateDriverTarget {
    fn from_iter<T: IntoIterator<Item = Entity>>(iter: T) -> Self {
        let entities = HashSet::from_iter(iter);

        match entities.len() {
            0 => Self::None,
            1 => Self::Single(entities.into_iter().next().unwrap()),
            _ => Self::Multi(entities),
        }
    }
}

impl<'a> FromIterator<&'a Entity> for ActionStateDriverTarget {
    fn from_iter<T: IntoIterator<Item = &'a Entity>>(iter: T) -> Self {
        let entities = HashSet::from_iter(iter.into_iter().cloned());

        match entities.len() {
            0 => Self::None,
            1 => Self::Single(entities.into_iter().next().unwrap()),
            _ => Self::Multi(entities),
        }
    }
}

enum ActionStateDriverTargetIterator<'a> {
    None,
    Single(Once<&'a Entity>),
    Multi(Iter<'a, Entity>),
}

impl<'a> Iterator for ActionStateDriverTargetIterator<'a> {
    type Item = &'a Entity;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::None => None,
            Self::Single(iter) => iter.next(),
            Self::Multi(iter) => iter.next(),
        }
    }
}

/// Stores information about when an action was pressed or released
///
/// This struct is principally used as a field on [`ActionData`],
/// which itself lives inside an [`ActionState`].
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub struct Timing {
    /// The [`Instant`] at which the button was pressed or released
    /// Recorded as the [`Time`](bevy::time::Time) at the start of the tick after the state last changed.
    /// If this is none, [`Timing::tick`] has not been called yet.
    #[serde(skip)]
    pub instant_started: Option<Instant>,
    /// The [`Duration`] for which the button has been pressed or released.
    ///
    /// This begins at [`Duration::ZERO`] when [`ActionState::update`] is called.
    pub current_duration: Duration,
    /// The [`Duration`] for which the button was pressed or released before the state last changed.
    pub previous_duration: Duration,
}

impl PartialOrd for Timing {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.current_duration.partial_cmp(&other.current_duration)
    }
}

impl Timing {
    /// Advances the `current_duration` of this timer
    ///
    /// If the `instant_started` is None, it will be set to the current time.
    /// This design allows us to ensure that the timing is always synchronized with the start of each frame.
    pub fn tick(&mut self, current_instant: Instant, previous_instant: Instant) {
        if let Some(instant_started) = self.instant_started {
            self.current_duration = current_instant - instant_started;
        } else {
            self.current_duration = current_instant - previous_instant;
            self.instant_started = Some(previous_instant);
        }
    }

    /// Flips the metaphorical hourglass, storing `current_duration` in `previous_duration` and resetting `instant_started`
    ///
    /// This method is called whenever actions are pressed or released
    pub fn flip(&mut self) {
        self.previous_duration = self.current_duration;
        self.current_duration = Duration::ZERO;
        self.instant_started = None;
    }
}

/// Will store an `ActionDiff` as well as what generated it (either an Entity, or nothing if the
/// input actions are represented by a `Resource`)
///
/// These are typically accessed using the `Events<ActionDiffEvent>` resource.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Event)]
pub struct ActionDiffEvent<A: Actionlike> {
    /// If some: the entity that has the `ActionState<A>` component
    /// If none: `ActionState<A>` is a Resource, not a component
    pub owner: Option<Entity>,
    /// The `ActionDiff` that was generated
    pub action_diffs: Vec<ActionDiff<A>>,
}

/// Stores presses and releases of buttons without timing information
///
/// These are typically accessed using the `Events<ActionDiffEvent>` resource.
/// Uses a minimal storage format, in order to facilitate transport over the network.
///
/// An `ActionState` can be fully reconstructed from a stream of `ActionDiff`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActionDiff<A: Actionlike> {
    /// The action was pressed
    Pressed {
        /// The value of the action
        action: A,
    },
    /// The action was released
    Released {
        /// The value of the action
        action: A,
    },
    /// The value of the action changed
    ValueChanged {
        /// The value of the action
        action: A,
        /// The new value of the action
        value: f32,
    },
    /// The axis pair of the action changed
    AxisPairChanged {
        /// The value of the action
        action: A,
        /// The new value of the axis
        axis_pair: Vec2,
    },
}

#[cfg(test)]
mod tests {
    use crate as leafwing_input_manager;
    use crate::input_mocking::MockInput;
    use bevy::prelude::{Entity, Reflect};
    use leafwing_input_manager_macros::Actionlike;

    use super::ActionStateDriverTarget;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
    enum Action {
        Run,
        Jump,
        Hide,
    }

    #[test]
    fn press_lifecycle() {
        use crate::action_state::ActionState;
        use crate::clashing_inputs::ClashStrategy;
        use crate::input_map::InputMap;
        use crate::input_streams::InputStreams;
        use bevy::input::InputPlugin;
        use bevy::prelude::*;
        use bevy::utils::{Duration, Instant};

        let mut app = App::new();
        app.add_plugins(InputPlugin);

        // Action state
        let mut action_state = ActionState::<Action>::default();

        // Input map
        let mut input_map = InputMap::default();
        input_map.insert(Action::Run, KeyCode::R);

        // Starting state
        let input_streams = InputStreams::from_world(&app.world, None);
        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(!action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));

        // Pressing
        app.send_input(KeyCode::R);
        // Process the input events into Input<KeyCode> data
        app.update();
        let input_streams = InputStreams::from_world(&app.world, None);

        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(action_state.pressed(&Action::Run));
        assert!(action_state.just_pressed(&Action::Run));
        assert!(!action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(!action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));

        // Releasing
        app.release_input(KeyCode::R);
        app.update();
        let input_streams = InputStreams::from_world(&app.world, None);

        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(!action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(action_state.released(&Action::Run));
        assert!(action_state.just_released(&Action::Run));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(!action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));
    }

    #[test]
    fn time_tick_ticks_away() {
        use crate::action_state::ActionState;
        use bevy::utils::{Duration, Instant};

        let mut action_state = ActionState::<Action>::default();

        // Actions start released (but not just released)
        assert!(action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Jump));

        // Ticking causes buttons that were just released to no longer be just released
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        assert!(action_state.released(&Action::Jump));
        assert!(!action_state.just_released(&Action::Jump));
        action_state.press(&Action::Jump);
        assert!(action_state.just_pressed(&Action::Jump));

        // Ticking causes buttons that were just pressed to no longer be just pressed
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        assert!(action_state.pressed(&Action::Jump));
        assert!(!action_state.just_pressed(&Action::Jump));
    }

    #[test]
    fn durations() {
        use crate::action_state::ActionState;
        use bevy::utils::{Duration, Instant};

        let mut action_state = ActionState::<Action>::default();

        // Actions start released
        assert!(action_state.released(&Action::Jump));
        assert_eq!(action_state.instant_started(&Action::Jump), None,);
        assert_eq!(action_state.current_duration(&Action::Jump), Duration::ZERO);
        assert_eq!(
            action_state.previous_duration(&Action::Jump),
            Duration::ZERO
        );

        // Pressing a button swaps the state
        action_state.press(&Action::Jump);
        assert!(action_state.pressed(&Action::Jump));
        assert_eq!(action_state.instant_started(&Action::Jump), None);
        assert_eq!(action_state.current_duration(&Action::Jump), Duration::ZERO);
        assert_eq!(
            action_state.previous_duration(&Action::Jump),
            Duration::ZERO
        );

        // Ticking time sets the instant for the new state
        let t0 = Instant::now();
        let t1 = t0 + Duration::new(1, 0);

        action_state.tick(t1, t0);
        assert_eq!(action_state.instant_started(&Action::Jump), Some(t0));
        assert_eq!(action_state.current_duration(&Action::Jump), t1 - t0);
        assert_eq!(
            action_state.previous_duration(&Action::Jump),
            Duration::ZERO
        );

        // Time passes
        let t2 = t1 + Duration::new(5, 0);

        // The duration is updated
        action_state.tick(t2, t1);
        assert_eq!(action_state.instant_started(&Action::Jump), Some(t0));
        assert_eq!(action_state.current_duration(&Action::Jump), t2 - t0);
        assert_eq!(
            action_state.previous_duration(&Action::Jump),
            Duration::ZERO
        );

        // Releasing again, swapping the current duration to the previous one
        action_state.release(&Action::Jump);
        assert_eq!(action_state.instant_started(&Action::Jump), None);
        assert_eq!(action_state.current_duration(&Action::Jump), Duration::ZERO);
        assert_eq!(action_state.previous_duration(&Action::Jump), t2 - t0);
    }

    #[test]
    fn action_state_driver_targets() {
        let mut target = ActionStateDriverTarget::from(());

        assert_eq!(0, target.len());

        target.insert(Entity::from_raw(0));
        assert_eq!(1, target.len());

        target.insert(Entity::from_raw(1));
        assert_eq!(2, target.len());

        target.remove(Entity::from_raw(0));
        assert_eq!(1, target.len());

        target.remove(Entity::from_raw(1));
        assert_eq!(0, target.len());

        target = target.with(Entity::from_raw(0));
        assert_eq!(1, target.len());

        target = target.without(Entity::from_raw(0));
        assert_eq!(0, target.len());

        target.add(
            [
                Entity::from_raw(0),
                Entity::from_raw(1),
                Entity::from_raw(2),
            ]
            .iter()
            .cloned(),
        );
        assert_eq!(3, target.len());

        let mut sum = 0;
        for entity in target.iter() {
            sum += entity.index();
        }
        assert_eq!(3, sum);
    }
}
