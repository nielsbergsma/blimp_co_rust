use crate::domain::{Version, Versioned};

pub struct Transaction<I, V: Clone> {
    pub id: I,
    pub value: Option<V>,
    pub version: Version,
}

impl<I, V: Clone> Transaction<I, V> {
    pub fn new(id: I) -> Self {
        Transaction {
            id,
            value: None,
            version: Version::default(),
        }
    }

    pub fn from_versioned(id: I, versioned: Versioned<V>) -> Self {
        Self {
            id,
            version: versioned.version(),
            value: Some(versioned.value()),
        }
    }

    pub fn with_value(self, value: V) -> Self {
        Self {
            id: self.id,
            value: Some(value),
            version: self.version,
        }
    }

    pub fn expect_non_empty<E>(self, error: E) -> Result<Self, E> {
        if self.value.is_some() {
            Ok(self)
        }
        else {
            Err(error)
        }
    }

    pub fn expect_empty<E>(self, error: E) -> Result<Self, E> {
        if self.value.is_none() {
            Ok(self)
        }
        else {
            Err(error)
        }
    }

    pub fn value_or<E>(&self, error: E) -> Result<V, E> {
        match &self.value {
            Some(value) => Ok(value.clone()),
            None => Err(error)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_none()
    }

    pub fn next_versioned_value(self) -> Option<Versioned<V>> {
        if let Some(value) = self.value {
            Some(Versioned::from_version(value, 1 + self.version))
        }
        else {
            None
        }
    }
}