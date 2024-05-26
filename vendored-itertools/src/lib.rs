//! Extra iterator adaptors, functions and macros.
//!
//! To extend [`Iterator`] with methods in this crate, import
//! the [`Itertools`] trait:
//!
//! ```
//! use itertools::Itertools;
//! ```
//!
//! ## Rust Version
//!
//! This version of itertools requires Rust 1.43.1 or later.

use std::collections::HashMap;
use std::hash::Hash;

/// The concrete iterator types.
pub mod structs {
    pub use crate::groupbylazy::{Chunk, ChunkBy, Chunks, Group, Groups, IntoChunks};
}

mod group_map;
mod groupbylazy;

use groupbylazy::ChunkBy;

/// An [`Iterator`] blanket implementation that provides extra adaptors and
/// methods.
///
/// This trait defines a number of methods. They are divided into two groups:
///
/// * *Adaptors* take an iterator and parameter as input, and return
/// a new iterator value. These are listed first in the trait. An example
/// of an adaptor is [`.interleave()`](Itertools::interleave)
///
/// * *Regular methods* are those that don't return iterators and instead
/// return a regular value of some other kind.
/// [`.next_tuple()`](Itertools::next_tuple) is an example and the first regular
/// method in the list.
pub trait Itertools: Iterator {
    // adaptors

    /// Return an *iterable* that can group iterator elements.
    /// Consecutive elements that map to the same key (“runs”), are assigned
    /// to the same group.
    ///
    /// `ChunkBy` is the storage for the lazy grouping operation.
    ///
    /// If the groups are consumed in order, or if each group's iterator is
    /// dropped without keeping it around, then `ChunkBy` uses no
    /// allocations.  It needs allocations only if several group iterators
    /// are alive at the same time.
    ///
    /// This type implements [`IntoIterator`] (it is **not** an iterator
    /// itself), because the group iterators need to borrow from this
    /// value. It should be stored in a local variable or temporary and
    /// iterated.
    ///
    /// Iterator element type is `(K, Group)`: the group's key and the
    /// group iterator.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// // chunk data into runs of larger than zero or not.
    /// let data = vec![1, 3, -2, -2, 1, 0, 1, 2];
    /// // chunks:     |---->|------>|--------->|
    ///
    /// // Note: The `&` is significant here, `ChunkBy` is iterable
    /// // only by reference. You can also call `.into_iter()` explicitly.
    /// let mut data_grouped = Vec::new();
    /// for (key, chunk) in &data.into_iter().chunk_by(|elt| *elt >= 0) {
    ///     data_grouped.push((key, chunk.collect()));
    /// }
    /// assert_eq!(data_grouped, vec![(true, vec![1, 3]), (false, vec![-2, -2]), (true, vec![1, 0, 1, 2])]);
    /// ```
    fn chunk_by<K, F>(self, key: F) -> ChunkBy<K, Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> K,
        K: PartialEq,
    {
        groupbylazy::new(self, key)
    }

    /// Return a `HashMap` of keys mapped to `Vec`s of values. Keys and values
    /// are taken from `(Key, Value)` tuple pairs yielded by the input iterator.
    ///
    /// Essentially a shorthand for `.into_grouping_map().collect::<Vec<_>>()`.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let data = vec![(0, 10), (2, 12), (3, 13), (0, 20), (3, 33), (2, 42)];
    /// let lookup = data.into_iter().into_group_map();
    ///
    /// assert_eq!(lookup[&0], vec![10, 20]);
    /// assert_eq!(lookup.get(&1), None);
    /// assert_eq!(lookup[&2], vec![12, 42]);
    /// assert_eq!(lookup[&3], vec![13, 33]);
    /// ```
    fn into_group_map<K, V>(self) -> HashMap<K, Vec<V>>
    where
        Self: Iterator<Item = (K, V)> + Sized,
        K: Hash + Eq,
    {
        group_map::into_group_map(self)
    }
}

impl<T> Itertools for T where T: Iterator + ?Sized {}
