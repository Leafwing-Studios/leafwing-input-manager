//! Charges are "uses of an action".
//! Actions may only be used if at least one charge is available.

use bevy::ecs::prelude::Component;
use std::marker::PhantomData;

use crate::Actionlike;

/// A component / resource that stores the [`Charges`] for each [`Actionlike`] action of type `A`.
///
/// If [`Charges`] is set for an actions, it is only [`Actionlike::ready`] when at least one charge is availabe.
///
/// ```rust
/// use leafwing_input_manager::prelude::*;
///
/// #[derive(Actionlike, Clone)]
/// enum Action {
///     // Neither cooldowns nor charges
///     Move,
///     // Double jump: 2 charges, no cooldowns
///     Jump,
///     // Simple cooldown
///     Dash,
///     // Cooldowns and charges, replenishing one at a time
///     Spell,
/// }
///
/// impl Action {
///     fn charges() -> ChargeState<Action> {
///         // You can either use the builder pattern or the `new` init for both cooldowns and charges
///         // The differences are largely aesthetic.
///         ChargeState::default()
///             // Double jump!
///             .set(Action::Jump, Charges::replenish_all(2))
///             // Store up to 3 spells at once
///             .set(Action::Spell, Charges::replenish_one(3))
///             .build()
///     }
///
///     fn cooldowns() -> Cooldowns<Action> {
///         // Ommitted cooldowns and charges will cause the action to be treated as if it always had available cooldowns / charges to use
///         Cooldowns::new([
///             (Action::Dash, Cooldown::from_secs(2.)),
///             (Action::Spell, Cooldown::from_secs(4.5)),
///         ])
///     }
/// }
///
/// // In a real game you'd spawn a bundle with the appropriate components
/// let mut bundle = InputManagerBundle {
///     cooldowns: Action::cooldowns(),
///     charges: Action::charges(),
///     ..Default::default()
/// };
///
/// // Then, you can check if an action is ready to be used
/// if Action::Spell.ready(&bundle.charges, &bundle.cooldowns) {
///     // When you use an action, remember to trigger it!
///     Action::Spell.trigger(&mut bundle.charges, &mut bundle.cooldowns);
/// }
///
/// ```
#[derive(Component, Clone, PartialEq, Eq, Debug)]
pub struct ChargeState<A: Actionlike> {
    /// The underlying [`Charges`], stored in [`Actionlike::variants`] order.
    charges_vec: Vec<Option<Charges>>,
    _phantom: PhantomData<A>,
}

impl<A: Actionlike> Default for ChargeState<A> {
    fn default() -> Self {
        ChargeState {
            charges_vec: A::variants().map(|_| None).collect(),
            _phantom: PhantomData::default(),
        }
    }
}

/// Stores how many times an action can be used.
///
/// Charges refresh when [`Charges::refresh`] is called manually,
/// or when the corresponding cooldown expires (if the [`InputManagerPlugin`](crate::plugin::InputManagerPlugin) is added).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Charges {
    current: u8,
    max: u8,
    /// What should happen when the charges are refreshed?
    pub replenish_strat: ReplenishStrategy,
    /// How should the corresponding [`Cooldown`](crate::cooldown::Cooldown) interact with these charges?
    pub cooldown_strat: CooldownStrategy,
}

/// What happens when [`Charges`] are replenished?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplenishStrategy {
    /// A single charge will be recovered.
    ///
    /// Usually paired with [`CooldownStrategy::ConstantlyRefresh`].
    OneAtATime,
    /// All charges will be recovered.
    ///
    /// Usually paired with [`CooldownStrategy::RefreshWhenEmpty`].
    AllAtOnce,
}

/// How do these charges replenish when cooldowns are refreshed?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CooldownStrategy {
    /// Cooldowns refresh will have no effect on the charges.
    Ignore,
    /// Cooldowns will replenish charges whenever the current charges are less than the max.
    ///
    /// Usually paired with [`ReplenishStrategy::OneAtATime`].
    ConstantlyRefresh,
    /// Cooldowns will only replenish charges when 0 charges are available.
    ///
    /// Usually paired with [`ReplenishStrategy::AllAtOnce`].
    RefreshWhenEmpty,
}

impl<A: Actionlike> ChargeState<A> {
    /// Creates a new [`ChargeState`] from an iterator of `(charges, action)` pairs
    ///
    /// If a [`Charges`] is not provided for an action, that action will be treated as if a charge was always available.
    ///
    /// To create an empty [`ChargeState`] struct, use the [`Default::default`] method instead.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    /// use bevy::input::keyboard::KeyCode;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    ///     Shoot,
    ///     Dash,
    /// }
    ///
    /// let input_map = ChargeState::new([
    ///     (Action::Shoot, Charges::replenish_all(6)),
    ///     (Action::Dash, Charges::replenish_one(2)),
    /// ]);
    /// ```
    #[must_use]
    pub fn new(action_cooldown_pairs: impl IntoIterator<Item = (A, Charges)>) -> Self {
        let mut charge_state = ChargeState::default();
        for (action, charges) in action_cooldown_pairs.into_iter() {
            charge_state.set(action, charges);
        }
        charge_state
    }

    /// Is at least one charge available for `action`?
    ///
    /// Returns `true` if the underlying [`Charges`] is [`None`].
    #[inline]
    #[must_use]
    pub fn available(&self, action: A) -> bool {
        if let Some(charges) = self.get(action) {
            charges.available()
        } else {
            true
        }
    }

    /// Spends one charge for `action` if able.
    ///
    /// Returns a boolean indicating whether a charge was available.
    /// If no charges are available, `false` is returned and this call has no effect.
    ///
    /// Returns `true` if the underlying [`Charges`] is [`None`].
    #[inline]
    pub fn expend(&mut self, action: A) -> bool {
        if let Some(charges) = self.get_mut(action) {
            charges.expend()
        } else {
            true
        }
    }

    /// Replenishes charges of `action`, up to its max charges.
    ///
    /// The exact effect is determined by the [`Charges`]'s [`ReplenishStrategy`].
    /// If the `action` is not associated with a [`Charges`], this has no effect.
    #[inline]
    pub fn replenish(&mut self, action: A) {
        if let Some(charges) = self.get_mut(action) {
            charges.replenish();
        }
    }

    /// Returns a reference to the underlying [`Charges`] for `action`, if set.
    #[inline]
    #[must_use]
    pub fn get(&self, action: A) -> &Option<Charges> {
        &self.charges_vec[action.index()]
    }

    /// Returns a mutable reference to the underlying [`Charges`] for `action`, if set.
    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, action: A) -> &mut Option<Charges> {
        &mut self.charges_vec[action.index()]
    }

    /// Sets the underlying [`Charges`] for `action` to the provided value.
    ///
    /// Unless you're building a new [`ChargeState`] struct, you likely want to use [`Self::get_mut`].
    #[inline]
    pub fn set(&mut self, action: A, charges: Charges) -> &mut Self {
        let data = self.get_mut(action);
        *data = Some(charges);

        self
    }

    /// Collects a `&mut Self` into a `Self`.
    ///
    /// Used to conclude the builder pattern. Actually just calls `self.clone()`.
    #[inline]
    #[must_use]
    pub fn build(&mut self) -> Self {
        self.clone()
    }

    /// Returns an iterator of references to the underlying non-[`None`] [`Charges`]
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Charges> {
        self.charges_vec.iter().flatten()
    }

    /// Returns an iterator of mutable references to the underlying non-[`None`] [`Charges`]
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Charges> {
        self.charges_vec.iter_mut().flatten()
    }
}

impl Charges {
    /// Creates a new [`Charges`], which can be expended `max_charges` times before needing to be replenished.
    ///
    /// The current charges will be set to the max charges by default.
    #[inline]
    #[must_use]
    pub fn new(
        max_charges: u8,
        replenish_strat: ReplenishStrategy,
        cooldown_strat: CooldownStrategy,
    ) -> Charges {
        Charges {
            current: max_charges,
            max: max_charges,
            replenish_strat,
            cooldown_strat,
        }
    }

    /// Creates a new [`Charges`] with [`ReplenishStrategy::OneAtATime`] and [`CooldownStrategy::Ignore`].
    pub fn simple(max_charges: u8) -> Charges {
        Charges {
            current: max_charges,
            max: max_charges,
            replenish_strat: ReplenishStrategy::OneAtATime,
            cooldown_strat: CooldownStrategy::Ignore,
        }
    }

    /// Creates a new [`Charges`] with [`ReplenishStrategy::AllAtOnce`] and [`CooldownStrategy::Ignore`].
    pub fn ammo(max_charges: u8) -> Charges {
        Charges {
            current: max_charges,
            max: max_charges,
            replenish_strat: ReplenishStrategy::AllAtOnce,
            cooldown_strat: CooldownStrategy::Ignore,
        }
    }

    /// Creates a new [`Charges`] with [`ReplenishStrategy::OneAtATime`] and [`CooldownStrategy::ConstantlyRefresh`].
    pub fn replenish_one(max_charges: u8) -> Charges {
        Charges {
            current: max_charges,
            max: max_charges,
            replenish_strat: ReplenishStrategy::OneAtATime,
            cooldown_strat: CooldownStrategy::ConstantlyRefresh,
        }
    }

    /// Creates a new [`Charges`] with [`ReplenishStrategy::AllAtOnce`] and [`CooldownStrategy::RefreshWhenEmpty`].
    pub fn replenish_all(max_charges: u8) -> Charges {
        Charges {
            current: max_charges,
            max: max_charges,
            replenish_strat: ReplenishStrategy::AllAtOnce,
            cooldown_strat: CooldownStrategy::RefreshWhenEmpty,
        }
    }

    /// The current number of available charges
    #[inline]
    #[must_use]
    pub fn charges(&self) -> u8 {
        self.current
    }

    /// The maximum number of available charges
    #[inline]
    #[must_use]
    pub fn max_charges(&self) -> u8 {
        self.max
    }

    /// Adds `charges` to the current number of available charges
    ///
    /// This will never exceed the maximum number of charges.
    /// Returns the number of excess charges.
    #[inline]
    #[must_use]
    pub fn add_charges(&mut self, charges: u8) -> u8 {
        let new_total = self.current.saturating_add(charges);

        let excess = new_total.saturating_sub(self.max);
        self.current = new_total.min(self.max);
        excess
    }

    /// Set the current number of available charges
    ///
    /// This will never exceed the maximum number of charges.
    /// Returns the number of excess charges.
    #[inline]
    pub fn set_charges(&mut self, charges: u8) -> u8 {
        let excess = charges.saturating_sub(self.max);
        self.current = charges.min(self.max);
        excess
    }

    /// Set the maximmum number of available charges
    ///
    /// If the number of charges available is greater than this number, it will be reduced to the new cap.
    #[inline]
    pub fn set_max_charges(&mut self, max_charges: u8) {
        self.max = max_charges;
        self.current = self.current.min(self.max);
    }

    /// Is at least one charge available?
    #[inline]
    #[must_use]
    pub fn available(&self) -> bool {
        self.current > 0
    }

    /// Spends one charge for `action` if able.
    ///
    /// Returns a boolean indicating whether a charge was available.
    /// If no charges are available, `false` is returned and this call has no effect.
    #[inline]
    pub fn expend(&mut self) -> bool {
        if self.current == 0 {
            return false;
        }

        self.current = self.current.saturating_sub(1);
        true
    }

    /// Replenishes charges of `action`, up to its max charges.
    ///
    /// The exact effect is determined by the [`ReplenishStrategy`] for this struct.
    #[inline]
    pub fn replenish(&mut self) {
        let charges_to_add = match self.replenish_strat {
            ReplenishStrategy::OneAtATime => 1,
            ReplenishStrategy::AllAtOnce => self.max,
        };

        // We don't care about overflowing our charges here.
        let _ = self.add_charges(charges_to_add);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charges_start_full() {
        let charges = Charges::simple(3);
        assert_eq!(charges.charges(), 3);
        assert_eq!(charges.max_charges(), 3);
    }

    #[test]
    fn charges_available() {
        let mut charges = Charges::simple(3);
        assert!(charges.available());
        charges.set_charges(1);
        assert!(charges.available());
        charges.set_charges(0);
        assert!(!charges.available());
    }

    #[test]
    fn charges_deplete() {
        let mut charges = Charges::simple(2);
        charges.expend();
        assert_eq!(charges.charges(), 1);
        charges.expend();
        assert_eq!(charges.charges(), 0);
        charges.expend();
        assert_eq!(charges.charges(), 0);
    }

    #[test]
    fn charges_replenish_one_at_a_time() {
        let mut charges = Charges::replenish_one(3);
        charges.set_charges(0);
        assert_eq!(charges.charges(), 0);
        charges.replenish();
        assert_eq!(charges.charges(), 1);
        charges.replenish();
        assert_eq!(charges.charges(), 2);
        charges.replenish();
        assert_eq!(charges.charges(), 3);
        charges.replenish();
        assert_eq!(charges.charges(), 3);
    }

    #[test]
    fn charges_replenish_all_at_once() {
        let mut charges = Charges::replenish_all(3);
        charges.set_charges(0);
        assert_eq!(charges.charges(), 0);
        charges.replenish();
        assert_eq!(charges.charges(), 3);
    }
}
