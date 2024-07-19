
use kernel::prelude::*;
use core::ops::{Deref, DerefMut};
use alloc::collections::TryReserveError;

#[macro_export]
macro_rules! vec {
    // Match expressions like `vecExtra![value; count]`
    ($elem:expr; $count:expr) => {
        {
            let count = $count; // Capture the count to use in `with_capacity` and `resize`
            let mut temp_vec = VecExtra::with_capacity(count).expect("Failed to allocate memory for VecExtra"); // Handle potential allocation failure;

            for _ in 0..count {
                temp_vec.try_push($elem).expect("Vector push failed"); // Resize the vector, filling with the element
            }
            temp_vec
        }
    };

    // Match expressions like `vecExtra![value, ..]`
    ( $($x:expr),* $(,)? ) => {
        {
            let mut temp_vec = VecExtra::new();
            $(
                temp_vec.try_push($x).expect("Vector push failed");
            )*
            temp_vec
        }
    };
}

/// A wrapper around `Vec<T>` that provides additional functionality.
#[derive(Debug)]
pub struct VecExtra<T> (Vec<T>);

impl<T> DerefMut for VecExtra<T> {
    /// Returns a mutable reference to the inner `Vec<T>`.
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Deref for VecExtra<T> {
    type Target = Vec<T>;

    /// Returns an immutable reference to the inner `Vec<T>`.
    fn deref(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T> VecExtra<T> {
    /// Creates a new empty `VecExtra<T>`.
    pub fn new() -> Self {
        VecExtra(Vec::new())
    }

    /// Creates a new `VecExtra<T>` with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The desired capacity of the `VecExtra<T>`.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - If the `VecExtra<T>` is successfully created.
    /// * `Err(super::AllocError)` - If there is an allocation error.
    pub fn with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        let v = Vec::try_with_capacity(capacity);
        match v {
            Ok(v) => Ok(VecExtra(v)),
            Err(e) => Err(e),
        }
    }

}

// Implement IntoIterator for VecExtra<T>
impl<T> IntoIterator for VecExtra<T> {
    type Item = <Vec<T> as IntoIterator>::Item;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Implement IntoIterator for &VecExtra<T>
impl<'a, T> IntoIterator for &'a VecExtra<T> {
    type Item = <&'a Vec<T> as IntoIterator>::Item;
    type IntoIter = <&'a Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// Implement IntoIterator for &mut VecExtra<T>
impl<'a, T> IntoIterator for &'a mut VecExtra<T> {
    type Item = <&'a mut Vec<T> as IntoIterator>::Item;
    type IntoIter = <&'a mut Vec<T> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}


impl<T: Clone> Clone for VecExtra<T> {
    /// Creates a new `VecExtra<T>` with the same elements as the original.
    fn clone(&self) -> Self {
        let mut vec = Vec::new();
        vec.try_reserve(self.len()).unwrap();
        for elem in self.iter() {
            let _ = vec.try_push(elem.clone());
        }
        VecExtra(vec)
    }
}

impl<T> FromIterator<T> for VecExtra<T> {
    #[inline]
    /// Creates a new `VecExtra<T>` from an iterator.
    ///
    /// # Arguments
    ///
    /// * `iter` - An iterator that yields elements of type `T`.
    ///
    /// # Returns
    ///
    /// * `VecExtra<T>` - The `VecExtra<T>` containing the elements from the iterator.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> VecExtra<T> {
        let mut vec = Vec::new();
        for i in iter {
            let _ = vec.try_push(i);
        }
        VecExtra(vec)
    }
}

impl<T: AsRef<[u8]>> VecExtra<T> {
    /// Returns a byte slice representing the contents of the `VecExtra<T>`.
    ///
    /// # Returns
    ///
    /// * `&[u8]` - A byte slice representing the contents of the `VecExtra<T>`.
    pub fn as_bytes(&self) -> &[u8] {
        if let Some(first_item) = self.0.first() {
            let start_ptr = first_item.as_ref().as_ptr();
            let len = self.0.len() * core::mem::size_of::<T>();
            unsafe { core::slice::from_raw_parts(start_ptr, len) }
        } else {
            &[]
        }
    }
}