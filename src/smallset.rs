//! A module for the [SmallSet] data type, an [ArrayVec]-backed set storage

use arrayvec::ArrayVec;

/// A set-like data structure with a fixed maximum size
///
/// These data structure does not require the [Hash] trait,
/// and instead uses linear iteration to find entries.
/// Iteration order is not guaranteed to be stable.
/// Principally, this data structure should be used for small sets,
/// where iteration performance, stack-allocation and uniqueness are more important than
/// lookup and insertion speed.
/// This set takes ownership of the values passed in,
/// so it is also better suited to small, [Copy]-enabled elements.
///
/// The maximum size of this type is given by the const-generic type parameter `CAP`.
/// Entries in this structure are guaranteed to be unique.
#[derive(Debug, Default, Clone)]
pub struct SmallSet<T: PartialEq + Clone, const CAP: usize> {
    storage: ArrayVec<T, CAP>,
}

impl<T: PartialEq + Clone, const CAP: usize> SmallSet<T, CAP> {
    /// Create a new empty [SmallSet]
    ///
    /// The capacity is given by the generic parameter `CAP`.
    #[must_use]
    pub fn new() -> Self {
        SmallSet {
            storage: ArrayVec::new(),
        }
    }

    /// Insert a new element to the set
    ///
    /// PANICS: will panic if the set is full before insertion.
    pub fn insert(&mut self, element: T) {
        // Always insert
        if let Err(InsertionError::Overfull) = self.try_insert(element) {
            // But panic if the set was full
            panic!("Set was full before insertion!")
        }
    }

    /// Attempt to insert a new element to the set
    ///
    /// Returns Ok if this succeeds, or an error if this failed due to either capacity or a duplicate entry.
    pub fn try_insert(&mut self, element: T) -> Result<(), InsertionError> {
        if self.len() == self.capacity() {
            return Err(InsertionError::Overfull);
        }

        for existing_element in self.storage.iter() {
            if element == *existing_element {
                return Err(InsertionError::Duplicate);
            }
        }

        // SAFE: capacity was just checked, and capacity of storage and self always match
        unsafe {
            self.storage.push_unchecked(element);
        }

        Ok(())
    }

    /// Checks if the provided element is in the set
    #[must_use]
    pub fn contains(self, element: &T) -> bool {
        for existing_element in self.storage.iter() {
            if *element == *existing_element {
                return true;
            }
        }
        false
    }

    /// Attempt to get a reference to the provided element from the set
    ///
    /// Returns `Some(&T)` to the first matching element found, or `None` if no matching element is found
    #[must_use]
    pub fn get_mut(&mut self, element: &T) -> Option<&mut T> {
        for existing_element in self.storage.iter_mut() {
            if *element == *existing_element {
                return Some(existing_element);
            }
        }
        None
    }

    /// Removes the element from the set, if it exists.
    ///
    /// Returns `Some(T)` to the first matching element found, or `None` if no matching element is found
    pub fn remove(&mut self, element: &T) -> Option<T> {
        let mut matching_index = None;
        for i in 0..self.capacity() {
            let existing_element = &self.storage[i];
            if *element == *existing_element {
                matching_index = Some(i);
                break;
            }
        }

        if let Some(i) = matching_index {
            let matching_element = self.storage.remove(i);
            Some(matching_element)
        } else {
            None
        }
    }

    /// Returns the current number of elements in the [SmallSet]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.storage.len() == 0
    }

    /// Returns the current number of elements in the [SmallSet]
    #[must_use]
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// Return the capacity of the [SmallSet]
    #[must_use]
    pub fn capacity(&self) -> usize {
        CAP
    }

    /// Removes all elements from the set
    pub fn clear(&mut self) {
        self.storage.clear();
    }
}

impl<T: PartialEq + Clone, const CAP: usize> IntoIterator for SmallSet<T, CAP> {
    type Item = T;
    type IntoIter = arrayvec::IntoIter<T, CAP>;

    fn into_iter(self) -> Self::IntoIter {
        self.storage.into_iter()
    }
}

impl<T: PartialEq + Clone, const CAP: usize> PartialEq for SmallSet<T, CAP> {
    /// Uses an inefficient O(n^2) approach to avoid introducing additional trait bounds
    fn eq(&self, other: &Self) -> bool {
        // Two sets cannot be equal if their cardinality differs
        if self.len() != other.len() {
            return false;
        }

        for item in self.clone() {
            let mut match_found = false;
            for other_item in other.clone() {
                // If a match can be found, we do not need to find another match for `item`
                if item == other_item {
                    match_found = true;
                    break;
                }
            }
            // If no match can be found, the sets cannot match
            if !match_found {
                return false;
            }
        }
        // Matches must be found for all items in the set for the them to be equal
        true
    }
}

/// An error returned when attempting to insert into a [SmallSet]
#[derive(Debug)]
pub enum InsertionError {
    /// The set was full before insertion was attempted
    Overfull,
    /// A matching entry already existed
    Duplicate,
}
