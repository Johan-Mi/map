#![no_std]

extern crate alloc;

pub struct Map<T> {
    entries: alloc::vec::Vec<Entry<T>>,
    next_vacant: Index,
}

impl<T: core::fmt::Debug> core::fmt::Debug for Map<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> Default for Map<T> {
    fn default() -> Self {
        Self {
            entries: alloc::vec::Vec::new(),
            next_vacant: 0,
        }
    }
}

impl<T> core::ops::Index<Id<T>> for Map<T> {
    type Output = T;

    fn index(&self, index: Id<T>) -> &Self::Output {
        match &self.entries[usize(index.index)] {
            Entry::Vacant { .. } => panic!(),
            Entry::Occupied(value) => value,
        }
    }
}

impl<T> core::ops::IndexMut<Id<T>> for Map<T> {
    fn index_mut(&mut self, index: Id<T>) -> &mut Self::Output {
        match &mut self.entries[usize(index.index)] {
            Entry::Vacant { .. } => panic!(),
            Entry::Occupied(value) => value,
        }
    }
}

impl<T> Map<T> {
    /// # Panics
    ///
    /// Panics if the map is too large.
    pub fn insert(&mut self, value: T) -> Id<T> {
        let id = id(self.next_vacant);
        if let Some(entry) = self.entries.get_mut(usize(self.next_vacant)) {
            let Entry::Vacant { next_vacant } = *entry else {
                unreachable!();
            };
            *entry = Entry::Occupied(value);
            self.next_vacant = next_vacant;
        } else {
            self.entries.push(Entry::Occupied(value));
            self.next_vacant = self.next_vacant.checked_add(1).expect("map is too large");
        }
        id
    }

    pub fn retain(&mut self, mut f: impl FnMut(Id<T>, &T) -> bool) {
        #![expect(clippy::missing_panics_doc, reason = "length is checked in `insert`")]
        let mut prev = self.entries.len().try_into().unwrap();
        for (index, entry) in self.entries.iter_mut().enumerate().rev() {
            let index = index.try_into().unwrap();
            match entry {
                Entry::Vacant { next_vacant } => *next_vacant = prev,
                Entry::Occupied(value) if !f(id(index), value) => {
                    *entry = Entry::Vacant { next_vacant: prev };
                }
                Entry::Occupied(_) => {}
            }
            if matches!(entry, Entry::Vacant { .. }) {
                prev = index;
            }
        }
        self.next_vacant = prev;
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id<T>, &'_ T)> {
        core::iter::zip(0.., &self.entries).filter_map(|(index, entry)| match entry {
            Entry::Vacant { .. } => None,
            Entry::Occupied(value) => Some((id(index), value)),
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id<T>, &'_ mut T)> {
        core::iter::zip(0.., &mut self.entries).filter_map(|(index, entry)| match entry {
            Entry::Vacant { .. } => None,
            Entry::Occupied(value) => Some((id(index), value)),
        })
    }

    pub fn values(&self) -> impl Iterator<Item = &'_ T> {
        self.entries.iter().filter_map(|entry| match entry {
            Entry::Vacant { .. } => None,
            Entry::Occupied(value) => Some(value),
        })
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &'_ mut T> {
        self.entries.iter_mut().filter_map(|entry| match entry {
            Entry::Vacant { .. } => None,
            Entry::Occupied(value) => Some(value),
        })
    }
}

enum Entry<T> {
    Vacant { next_vacant: Index },
    Occupied(T),
}

pub struct Id<T> {
    index: Index,
    typed: core::marker::PhantomData<T>,
}

impl<T> core::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Id").field(&self.index).finish()
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> core::hash::Hash for Id<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

const fn id<T>(index: Index) -> Id<T> {
    let typed = core::marker::PhantomData;
    Id { index, typed }
}

type Index = u32;

const fn usize(index: Index) -> usize {
    const { assert!(size_of::<Index>() <= size_of::<usize>()) }
    index as usize
}
