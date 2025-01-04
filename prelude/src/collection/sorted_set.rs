use std::vec::IntoIter;
use std::slice::Iter;
use serde::{Deserialize, Serialize};

/// Set-like container, where order of element are sorted according to insertion order
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SortedSet<T: PartialEq + Clone>(Vec<T>);

impl<T: PartialEq + Clone> SortedSet<T> {
    pub fn empty() -> Self {
        SortedSet(Vec::new())
    }

    pub fn singleton(element: T) -> Self {
        Self::empty().insert(element)
    }

    pub fn from_vec(elements: Vec<T>) -> Self {
        elements.into_iter()
            .fold(SortedSet::empty(), |set, item| set.insert(item))
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn contains(&self, element: &T) -> bool {
        self.0.contains(element)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    #[inline]
    pub fn to_vec(self) -> Vec<T> {
        self.0
    }

    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> IntoIter<T> {
        self.0.into_iter()
    }

    #[allow(clippy::manual_find)]
    pub fn find<P>(&self, p: P) -> Option<&T>
        where P: Fn(&T) -> bool
    {
        for element in self.0.iter() {
            if p(element) {
                return Some(element);
            }
        }
        None
    }

    /// Insert element;
    /// - if already exist, it's replaced  (order remains intact)
    /// - if element not exists, it's added to the 'back'
    pub fn insert(self, element: T) -> Self {
        let mut elements= Vec::new();
        let mut replaced = false;

        for existing_item in self.0.into_iter() {
            if existing_item == element {
                replaced = true;
                elements.push(element.clone())
            }
            else {
                elements.push(existing_item);
            }
        }

        if !replaced {
            elements.push(element);
        }

        Self(elements)
    }
}

impl<T: PartialEq + Clone> PartialEq for SortedSet<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        for element in self.0.iter() {
            if !other.0.contains(element) {
                return false;
            }
        }

        true
    }
}