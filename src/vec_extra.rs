use ::kernel::prelude::*;
use core::ops::Deref;
use core::ops::DerefMut;

#[macro_export]
macro_rules! vec {
    // Match expressions like `vecExtra![value; count]`
    ($elem:expr; $count:expr) => {
        {
            let count = $count; // Capture the count to use in `with_capacity` and `resize`
            let mut temp_vec = VecExtra::with_capacity(count, GFP_KERNEL);
            temp_vec.resize(count, $elem); // Resize the vector, filling with the element
            temp_vec.expect("Vector resize failed")
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
    pub fn with_capacity(capacity: usize) -> Result<Self, super::AllocError> {
        let v = Vec::with_capacity(capacity, GFP_KERNEL);
        match v {
            Ok(v) => Ok(VecExtra(v)),
            Err(e) => Err(e),
        }
    }
}

impl<T: Clone> Clone for VecExtra<T> {
    /// Creates a new `VecExtra<T>` with the same elements as the original.
    fn clone(&self) -> Self {
        let mut vec = Vec::new();
        vec.try_reserve(self.len()).unwrap();
        for elem in self.iter() {
            let _ = vec.push(elem.clone(), GFP_KERNEL);
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
            let _ = vec.push(i, GFP_KERNEL);
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