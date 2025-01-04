use serde::{Deserialize, Serialize};

pub type Version = u32;

#[derive(Serialize, Deserialize)]
pub struct Versioned<T>(T, Version);

impl<T> Versioned<T> {
    pub fn new(value: T) -> Versioned<T> {
        Versioned(value, 1u32)
    }

    pub fn from_version(value:T, version: Version) -> Versioned<T> {
        Versioned(value, version)
    }

    pub fn value(self) -> T {
        self.0
    }

    pub fn value_ref(&self) -> &T {
        &self.0
    }

    pub fn version(&self) -> Version {
        self.1
    }
}
