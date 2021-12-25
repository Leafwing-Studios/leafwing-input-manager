use crate::Actionlike;
use bevy::prelude::*;
use bevy::utils::HashMap;

/// Resource that stores the currently and recently pressed actions
///
/// Abstracts over all of the various input methods and bindings
#[derive(Component)]
pub struct ActionState<A: Actionlike> {
    pressed: HashMap<A, bool>,
    pressed_this_tick: HashMap<A, bool>,
    just_pressed: HashMap<A, bool>,
    just_released: HashMap<A, bool>,
}

impl<A: Actionlike> ActionState<A> {
    /// Clears all just_pressed and just_released state
    ///
    /// Also clears the internal field `pressed_this_tick`, so inputs can be correctly released
    pub fn tick(&mut self) {
        self.just_pressed = Self::default_map();
        self.just_released = Self::default_map();
        self.pressed_this_tick = Self::default_map();
    }

    /// Press the `action`
    pub fn press(&mut self, action: A) {
        self.pressed.insert(action, true);
        self.pressed_this_tick.insert(action, true);
        if !self.pressed(action) {
            self.just_pressed.insert(action, true);
        }
    }

    /// Release the `action`
    pub fn release(&mut self, action: A) {
        self.pressed.insert(action, false);

        if self.pressed(action) {
            self.just_released.insert(action, true);
        }
    }

    /// Releases all actions
    pub fn release_all(&mut self) {
        for action in A::iter() {
            self.release(action);
        }
    }

    /// Is an action currently pressed?
    pub fn pressed(&self, action: A) -> bool {
        *self.pressed.get(&action).unwrap()
    }

    /// Was this action pressed since the last time [tick](ActionState::tick) was called?
    pub fn just_pressed(&self, action: A) -> bool {
        *self.just_pressed.get(&action).unwrap()
    }

    /// Is an action currently released?
    ///
    /// This is always the logical negation of [pressed](ActionState::pressed)
    pub fn released(&self, action: A) -> bool {
        !*self.pressed.get(&action).unwrap()
    }

    /// Was this action pressed since the last time [tick](ActionState::tick) was called?
    pub fn just_released(&self, action: A) -> bool {
        *self.just_pressed.get(&action).unwrap()
    }

    /// Release all actions that were not pressed this tick
    pub fn release_unpressed(&mut self) {
        for action in A::iter() {
            if !*self.pressed_this_tick.get(&action).unwrap() {
                self.release(action);
            }
        }
    }

    /// Creates a Hashmap with all of the possible A variants as keys, and false as the values
    fn default_map() -> HashMap<A, bool> {
        // PERF: optimize construction through pre-allocation or constification
        let mut map = HashMap::default();

        for action in A::iter() {
            map.insert(action, false);
        }
        map
    }
}

impl<A: Actionlike> Default for ActionState<A> {
    fn default() -> Self {
        Self {
            pressed: Self::default_map(),
            pressed_this_tick: Self::default_map(),
            just_pressed: Self::default_map(),
            just_released: Self::default_map(),
        }
    }
}
