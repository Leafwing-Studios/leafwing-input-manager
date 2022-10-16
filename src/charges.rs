//! Charges are "uses of an action".
//! Actions may only be used if at least one charge is available.

use std::marker::PhantomData;

use bevy::ecs::prelude::Component;

use crate::Actionlike;

/// Store the [`Charges`] for each [`Actionlike`] action of type `A`.
#[derive(Component, Clone, PartialEq, Eq, Debug)]
pub struct ActionCharges<A: Actionlike> {
    /// The underlying [`Charges`], stored in [`Actionlike::variants`] order.
    charges_vec: Vec<Option<Charges>>,
    _phantom: PhantomData<A>,
}

impl<A: Actionlike> Default for ActionCharges<A> {
    fn default() -> Self {
        ActionCharges {
            charges_vec: Vec::default(),
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

impl<A: Actionlike> ActionCharges<A> {
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
    /// Unless you're building a new [`ActionCharges`] struct, you likely want to use [`Self::get_mut`].
    #[inline]
    #[must_use]
    pub fn set(&mut self, action: A, charges: Charges) {
        let data = self.get_mut(action);
        *data = Some(charges);
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

    /// Creates a new [`Charges`] with [`ReplenishStrategy::OneAtATime`] and [`CooldownStrategy::ConstantlyRefresh`].
    pub fn new_replenish_one(max_charges: u8) -> Charges {
        Charges {
            current: max_charges,
            max: max_charges,
            replenish_strat: ReplenishStrategy::OneAtATime,
            cooldown_strat: CooldownStrategy::ConstantlyRefresh,
        }
    }

    /// Creates a new [`Charges`] with [`ReplenishStrategy::AllAtOnce`] and [`CooldownStrategy::RefreshWhenEmpty`].
    pub fn new_replenish_all(max_charges: u8) -> Charges {
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
