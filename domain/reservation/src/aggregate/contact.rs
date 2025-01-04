use serde::{Deserialize, Serialize};
use crate::aggregate::{EmailAddress, EmailVerificationError, PersonFullName, PhoneNumber};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Contact {
    pub name: PersonFullName,
    pub email: EmailAddress,
    pub phone: Option<PhoneNumber>,
}

impl Contact {
    pub fn new(name: PersonFullName, email: EmailAddress, phone: Option<PhoneNumber>) -> Self {
        Self {
            name,
            email,
            phone,
        }
    }

    pub fn set_phone(self, phone: PhoneNumber) -> Self {
        Self {
            phone: Some(phone),
            ..self
        }
    }

    pub fn phone_is_present(&self) -> bool {
        self.phone.is_some()
    }

    pub fn email_is_verified(&self) -> bool {
        self.email.is_verified()
    }

    pub fn email_verify_challenge(&self) -> Option<String> {
        self.email.verify_challenge()
    }

    pub fn verify_email(self, challenge: String) -> Result<Contact, EmailVerificationError> {
        Ok(Self {
            email: self.email.verify(challenge)?,
            ..self
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::{Contact, EmailAddress, PersonFullName, PhoneNumber};

    #[test]
    fn can_create_new_from_name_email() {
        let contact = Contact::new(name(), email(), None);

        assert_eq!(contact.name, name());
        assert_eq!(contact.email, email());
        assert_eq!(contact.phone, None)
    }

    #[test]
    fn equals_by_values() {
        let contact1 = Contact::new(name(), email(), None);
        let contact2 = Contact::new(name(), email(), None);
        assert_eq!(contact1, contact2);

        let contact3 = Contact::new(name2(), email(), None);
        assert_ne!(contact1, contact3);
    }

    #[test]
    fn can_update_phone_number() {
        let before = Contact::new(name(), email(), None);
        assert!(!before.phone_is_present());

        let after = before.set_phone(phone());
        assert_eq!(after.phone, Some(phone()));
        assert!(after.phone_is_present());
    }

    #[test]
    fn can_verify_email_address() {
        let before = Contact::new(name(), email(), None);
        assert!(!before.email_is_verified());

        let challenge = before.email_verify_challenge().unwrap();
        let after = before.verify_email(challenge).unwrap();
        assert!(after.email_is_verified());
    }

    #[test]
    fn is_serializable() {
        let original = Contact::new(name(), email(), Some(phone()));
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Contact = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.email, deserialized.email);
        assert_eq!(original.phone, deserialized.phone)
    }

    fn name() -> PersonFullName {
        "Niels Bergsma".parse().unwrap()
    }

    fn name2() -> PersonFullName {
        "Karina Sands".parse().unwrap()
    }

    fn email() -> EmailAddress {
        "n.bergsma@internet.com".parse().unwrap()
    }

    fn phone() -> PhoneNumber {
        "+31653321799".parse().unwrap()
    }
}