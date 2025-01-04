use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::aggregate::{AccommodationId, AccommodationName, Picture, Place};
use prelude::collection::{SortedSet};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum AccommodationError {
    #[error("too few pictures")]
    TooFewPictures,

    #[error("too many pictures")]
    TooManyPictures,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Accommodation {
    pub id: AccommodationId,
    pub name: AccommodationName,
    pub place: Place,
    pub pictures: SortedSet<Picture>,
}

impl Accommodation {
    pub fn build(id: AccommodationId, name: AccommodationName, place: Place, pictures: SortedSet<Picture>) -> Result<Self, AccommodationError> {
        if pictures.is_empty() {
            return Err(AccommodationError::TooFewPictures);
        }
        if pictures.len() > 50 {
            return Err(AccommodationError::TooManyPictures);
        }

        Ok(Self {
            id,
            name,
            place,
            pictures,
        })
    }
}

impl PartialEq for Accommodation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use prelude::collection::SortedSet;
    use crate::aggregate::{Accommodation, AccommodationError, AccommodationId, AccommodationName, Picture, Place};

    #[test]
    fn is_buildable() {
        let accommodation = Accommodation::build(id(), name(), place(), SortedSet::singleton(picture()));
        assert!(accommodation.is_ok());
    }

    #[test]
    fn errors_on_malformed_input() {
        // too few pictures
        let accommodation = Accommodation::build(id(), name(), place(), SortedSet::empty());
        assert_eq!(accommodation, Err(AccommodationError::TooFewPictures));

        // too many pictures
        let accommodation = Accommodation::build(id(), name(), place(), pictures(60));
        assert_eq!(accommodation, Err(AccommodationError::TooManyPictures));
    }

    #[test]
    fn is_serializable() {
        let original = Accommodation::build(id(), name(), place(), SortedSet::singleton(picture())).unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Accommodation = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.pictures, deserialized.pictures);
    }

    #[test]
    fn equals_by_id() {
        let accommodation1 = Accommodation::build(id(), name(), place(), SortedSet::singleton(picture())).unwrap();
        let accommodation2 = Accommodation::build(id(), name2(), place(), SortedSet::singleton(picture2())).unwrap();
        assert_eq!(accommodation1, accommodation2);

        let accommodation3 = Accommodation::build(id2(), name(), place(), SortedSet::singleton(picture())).unwrap();
        assert_ne!(accommodation1, accommodation3);
    }

    fn id() -> AccommodationId {
        "5EPFciXgSxB70tAE8iERl6".parse().unwrap()
    }

    fn id2() -> AccommodationId {
        "5EPFciXgSxB70tAE8iERl7".parse().unwrap()
    }

    fn name() -> AccommodationName {
        "Farsund Fjordhotel".parse().unwrap()
    }

    fn name2() -> AccommodationName {
        "Bergen Resort".parse().unwrap()
    }

    fn place() -> Place {
        Place::new(
            "Farsund, Norway".parse().unwrap(),
            "u4kf6x".parse().unwrap(),
        )
    }

    fn picture() -> Picture {
        Picture::build(
            "https://www.visitnorway.com/img/farsund.jpg".parse().unwrap(),
            "Farsund Resort".to_owned(),
        ).unwrap()
    }

    fn picture2() -> Picture {
        Picture::build(
            "https://www.visitnorway.com/img/bergen.jpg".parse().unwrap(),
            "Bergen Resort".to_owned(),
        ).unwrap()
    }

    fn pictures(count: u8) -> SortedSet<Picture> {
        let mut pictures = SortedSet::empty();
        for index in 0..count {
            let picture = Picture::build(
                format!("https://www.visitnorway.com/img/{}.jpg", index).parse().unwrap(),
                "Bergen Resort".to_owned(),
            ).unwrap();

            pictures = pictures.insert(picture);
        }
        pictures
    }
}