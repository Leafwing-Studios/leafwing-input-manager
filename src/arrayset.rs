//! A module for the [ArraySet] data type, a simple array-backed set storage

/// A set-like data structure with a fixed maximum size
///
/// These data structure does not require the [Hash] trait,
/// and instead uses linear iteration to find entries.
/// Iteration order is guaranteed to be stable, and elements are not re-compressed upon removal.
///
/// Principally, this data structure should be used for relatively small sets,
/// where iteration performance, stable-order, stack-allocation and uniqueness
/// are more important than insertion or look-up speed.
/// Iteration, insertion and checking whether an element is in the set are O(CAP).
/// Indexing is O(1), as is removing an element at a specific index.
///
/// The values are stored as [Option]s within an array,
/// so niche optimization can significantly reduce memory footprint.
///
/// The maximum size of this type is given by the const-generic type parameter `CAP`.
/// Entries in this structure are guaranteed to be unique.
#[derive(Debug, Clone, Copy)]
pub struct ArraySet<T: PartialEq + Clone + Copy, const CAP: usize> {
    storage: [Option<T>; CAP],
}

impl<T: PartialEq + Clone + Copy, const CAP: usize> Default for ArraySet<T, CAP> {
    fn default() -> Self {
        Self {
            storage: [None; CAP],
        }
    }
}

impl<T: PartialEq + Clone + Copy, const CAP: usize> ArraySet<T, CAP> {
    /// Create a new empty [ArraySet] where all values are the same
    ///
    /// The capacity is given by the generic parameter `CAP`.
    #[must_use]
    pub fn new(value: T) -> Self {
        ArraySet {
            storage: [Some(value); CAP],
        }
    }

    /// Returns the index of the next filled slot, if any
    pub fn next_index(&self) -> Option<usize> {
        for i in 0..CAP {
            if self.storage[i].is_some() {
                return Some(i);
            }
        }
        None
    }

    /// Returns the index of the next empty slot, if any
    pub fn next_empty_index(&self) -> Option<usize> {
        for i in 0..CAP {
            if self.storage[i].is_none() {
                return Some(i);
            }
        }
        None
    }

    /// Insert a new element to the set
    ///
    /// PANICS: will panic if the set is full before insertion.
    pub fn insert(&mut self, element: T) {
        // Always insert
        if let Err(InsertionError::Overfull) = self.try_insert(element) {
            // But panic if the set was full
            panic!("Inserting this element would have overflowed the set!")
        }
    }

    /// Insert a new element to the set at the provided index
    ///
    /// This is a very fast O(1) operation, as we do not need to check for duplicates or find a free slot.
    ///
    /// Returns `Some(T)` if an element was found at that index, or `None` if no element was there.
    ///
    /// PANICS: panics if the provided index is larger than CAP.
    pub fn insert_at(&mut self, element: T, index: usize) -> Option<T> {
        assert!(index <= CAP);

        let preexisting_element = self.remove_at(index);
        self.storage[index] = Some(element);

        preexisting_element
    }

    /// Attempt to insert a new element to the set
    ///
    /// Returns Ok if this succeeds, or an error if this failed due to either capacity or a duplicate entry.
    pub fn try_insert(&mut self, element: T) -> Result<(), InsertionError> {
        if self.contains(&element) {
            return Err(InsertionError::Duplicate);
        }

        let next_empty_index = self.next_empty_index();

        if let Some(index) = next_empty_index {
            self.insert_at(element, index);
            Ok(())
        } else {
            Err(InsertionError::Overfull)
        }
    }

    /// Is the provided element in the set?
    #[must_use]
    pub fn contains(self, element: &T) -> bool {
        for existing_element in self {
            if *element == existing_element {
                return true;
            }
        }
        false
    }

    /// Returns a reference to the provided index of the underlying array
    ///
    /// Returns `Some(&T)` if the index is in-bounds and has an element
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if let Some(reference) = &self.storage[index] {
            Some(reference)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the provided index of the underlying array
    ///
    /// Returns `Some(&mut T)` if the index is in-bounds and has an element
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if let Some(reference) = &mut self.storage[index] {
            Some(reference)
        } else {
            None
        }
    }

    /// Removes the element from the set, if it exists
    ///
    /// Returns `Some(index, T)` for the first matching element found, or `None` if no matching element is found
    pub fn remove(&mut self, element: &T) -> Option<(usize, T)> {
        for index in 0..CAP {
            if let Some(existing_element) = &self.storage[index] {
                if *element == *existing_element {
                    let removed_element = self.remove_at(index).unwrap();
                    return Some((index, removed_element));
                }
            }
        }
        None
    }

    /// Removes the element at the provided index
    ///
    /// Returns `Some(T)` if an element was found at that index, or `None` if no element was there.
    ///
    /// PANICS: panics if the provided index is larger than CAP.
    pub fn remove_at(&mut self, index: usize) -> Option<T> {
        assert!(index <= CAP);

        let removed = self.storage[index];
        self.storage[index] = None;
        removed
    }

    /// Return the capacity of the [ArraySet]
    #[must_use]
    pub fn capacity(&self) -> usize {
        CAP
    }

    /// Returns the current number of elements in the [ArraySet]
    #[must_use]
    pub fn len(&self) -> usize {
        self.storage.iter().filter(|e| e.is_some()).count()
    }

    /// Are there exactly 0 elements in the [ArraySet]?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.storage.len() == 0
    }

    /// Removes all elements from the set without allocation
    pub fn clear(&mut self) {
        self.storage = [None; CAP];
    }
}

impl<T: PartialEq + Clone + Copy, const CAP: usize> Iterator for ArraySet<T, CAP> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut item = None;

        while item.is_none() {
            // Get the next item, and return it if it is non-empty
            if let Some(filled_item) = self.storage.get(0) {
                item = *filled_item;
            }
        }

        item
    }
}

impl<T: PartialEq + Clone + Copy, const CAP: usize> FromIterator<T> for ArraySet<T, CAP> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set: ArraySet<T, CAP> = ArraySet::default();
        for element in iter {
            set.insert(element);
        }
        set
    }
}

impl<T: PartialEq + Clone + Copy, const CAP: usize> PartialEq for ArraySet<T, CAP> {
    /// Uses an inefficient O(n^2) approach to avoid introducing additional trait bounds
    fn eq(&self, other: &Self) -> bool {
        // Two sets cannot be equal if their cardinality differs
        if self.len() != other.len() {
            return false;
        }

        for item in *self {
            let mut match_found = false;
            for other_item in *other {
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

/// An error returned when attempting to insert into a [ArraySet]
#[derive(Debug)]
pub enum InsertionError {
    /// The set was full before insertion was attempted
    Overfull,
    /// A matching entry already existed
    Duplicate,
}
