mod djb2;

use kernel::prelude::*;
use kernel::vec;
use alloc::vecExtra::VecExtra;

use core::hash::{Hash, Hasher};
use crate::hashset::djb2::DJB2Hasher;

#[derive(Debug, Clone)]
pub(crate) struct HashSet<T, H = DJB2Hasher>
    where T: Clone
{
    buckets: VecExtra<Option<T>>,
    capacity: usize,
    size: usize,
    hasher: H,
}

// Define the iterator struct
pub struct HashSetIter<'a, T>
where T: Clone
{
    hashset: &'a HashSet<T>,
    current_pos: usize,
}

// Implementing Iterator for HashSetIter
impl<'a, T: Clone> Iterator for HashSetIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_pos < self.hashset.capacity {
            match &self.hashset.buckets[self.current_pos] {
                Some(item) => {
                    self.current_pos += 1; // Move to the next item for the next call
                    return Some(item);
                },
                None => self.current_pos += 1, // Continue searching
            }
        }
        None // End of buckets
    }
}

// Needed for doing `for h in myHashSet { ... }``
impl<'a, T: Clone> IntoIterator for &'a HashSet<T> {
    type Item = &'a T;
    type IntoIter = HashSetIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        HashSetIter {
            hashset: self,
            current_pos: 0,
        }
    }
}

// Needed for `::collect::<HashSet<_>>`
impl<'a, T: Clone + Eq + Hash + Copy + core::fmt::Debug> FromIterator<T> for HashSet<T> {
    fn from_iter<I: for<'b> IntoIterator<Item=T> + Sized>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut set = HashSet::with_capacity(iter.size_hint().0);
        for item in iter {
            set.insert(item);
        }
        set
    }
}

impl<T: Clone + Eq + Hash + Copy + core::fmt::Debug> HashSet<T, DJB2Hasher> {
    pub fn with_capacity(capacity: usize) -> Self {
        HashSet {
            buckets: vec![None; capacity],
            capacity,
            size: 0,
            hasher: DJB2Hasher::new(),
        }
    }
}

impl<T: Clone + Eq + Hash + core::fmt::Debug, H: Hasher> HashSet<T, H> {
    pub fn with_hasher_and_capacity(hasher: H, capacity: usize) -> Self {
        HashSet {
            buckets: vec![None; capacity],
            capacity,
            size: 0,
            hasher,
        }
    }

    pub fn hash(&mut self, item: &T) -> u64 {
        item.hash(&mut self.hasher);
        let ret = self.hasher.finish() as u64;
        ret
    }

    fn calculate_index(&mut self, item: &T) -> usize {
        usize::try_from(self.hash(item)).unwrap() % self.capacity
    }

    pub fn insert(&mut self, item: T) {
        if self.size >= self.capacity {
            self.resize();
        }

        let mut index = self.calculate_index(&item);
        while let Some(ref existing) = self.buckets[index] {
            if *existing == item {
                // Item already exists, do nothing
                return;
            }
            // Quadratic probing:
            index = (index + 1) % self.capacity;
        }
        self.buckets[index] = Some(item);
        self.size += 1;
    }

    fn resize(&mut self) {
        let new_cap = self.capacity *2;
        let mut new_buckets = vec![None; new_cap];
        let current_items: VecExtra<_> = self.buckets.iter().cloned().collect();

        for item_option in current_items {
            if let Some(item) = item_option {
                let mut index =  usize::try_from(self.hash(&item)).unwrap() % new_cap;
                while let Some(ref existing) = new_buckets[index] {
                    if *existing == item {
                        // Item already exists, do nothing
                        return;
                    }
                    // Quadratic probing:
                    index = (index + 1) % new_cap;
                }
                new_buckets[index] = Some(item.clone());
            }
        }
        self.buckets = new_buckets;
        self.capacity = new_cap;
    }

    pub fn contains(&mut self, item: &T) -> bool {
        let start_index = usize::try_from(self.hash(item)).unwrap() % self.capacity;
        let mut index = start_index;
        loop {
            match self.buckets[index] {
                Some(ref existing) if existing == item => {
                    pr_info!("Found item: {:?}", existing);
                    pr_info!("Item matches: {:?} = {:?}", existing, item);
                    return true;
                },
                Some(_) if (index + 1) % self.capacity == start_index => {
                    pr_info!("Item not found");
                    return false;
                },
                _ => {
                    index = (index + 1) % self.capacity;
                    pr_info!("index: {}", index);
                },
            }
            if index == start_index {
                return false;
            }
        }
    }

    pub fn remove(&mut self, item: &T) {
        let mut index = usize::try_from(self.hash(item)).unwrap() % self.capacity;
        let start_index = index as usize;
        loop {
            match self.buckets[index] {
                Some(ref existing) if *existing == *item => {
                    self.buckets[index] = None; // Remove the item
                                                // Optionally, rehash items in subsequent buckets until an empty bucket is found
                                                // to maintain the integrity of the hash table. This part is omitted for brevity.
                    self.size -= 1;
                    return;
                }
                None => return, // Item not found
                _ => {
                    index = (index + 1) % self.capacity;
                    if index == start_index { // All buckets checked, item not found
                        return;
                    }
                }
            }
        }
    }

    pub fn is_empty(&mut self) -> bool {
        self.size == 0
    }
    pub fn len(&mut self) -> usize {
        self.size
    }

    // get all the elements that are in the first set but not the second.
    pub fn difference(&mut self, other: &mut HashSet<T, H>) -> VecExtra<T> {
        let mut diff = VecExtra::new();
        let buckets = self.buckets.clone();
        for bucket in &buckets {
            if let Some(ref item) = bucket {
                if !other.contains(item) {
                    diff.push(item.clone(), GFP_KERNEL);
                    pr_info!("Pushing {:?}", item);
                }
            }
        }
        diff
    }

    // get all the unique elements in both sets.
    pub fn union(&mut self, other: &mut HashSet<T, H>) -> VecExtra<T> {
        let mut union = VecExtra::new();
        for bucket in &self.buckets {
            if let Some(ref item) = bucket {
                union.push(item.clone(), GFP_KERNEL);
            }
        }
        for bucket in &other.buckets {
            if let Some(ref item) = bucket {
                if !self.contains(item) {
                    union.push(item.clone(), GFP_KERNEL);
                }
            }
        }
        union
    }

    // get all the elements that are only in both sets.
    pub fn intersection(&mut self, other: &mut HashSet<T, H>) -> VecExtra<T> {
        let mut intersection = VecExtra::new();
        for bucket in &self.buckets {
            if let Some(ref item) = bucket {
                if other.contains(item) {
                    intersection.push(item.clone(), GFP_KERNEL);
                }
            }
        }
        intersection
    }

    // Get all the elements that are in one set or the other, but not both.
    pub fn symmetric_difference(&mut self, other: &mut HashSet<T, H>) -> VecExtra<T> {
        let mut sym_diff = VecExtra::new();
        for bucket in &self.buckets {
            if let Some(ref item) = bucket {
                if !other.contains(item) {
                    sym_diff.push(item.clone(), GFP_KERNEL);
                }
            }
        }
        for bucket in &other.buckets {
            if let Some(ref item) = bucket {
                if !self.contains(item) {
                    sym_diff.push(item.clone(), GFP_KERNEL);
                }
            }
        }
        sym_diff
    }

    pub fn drain(&mut self) {
        self.buckets = vec![None; self.capacity];
        self.size = 0;
    }
}
