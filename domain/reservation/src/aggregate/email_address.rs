use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use prelude::parse::{end, is_a, one_of, Parser, sym};

#[derive(Error, Debug, PartialEq)]
pub enum EmailParseError {
    #[error("malformed value")]
    MalformedValue,
}

#[derive(Error, Debug, PartialEq)]
pub enum EmailVerificationError {
    #[error("challenge don't match")]
    ChallengeDontMatch,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EmailAddress {
    Unverified(String),
    Verified(String),
}

impl EmailAddress {
    fn parser<'a>() -> Parser<'a, &'a str> {
        // according to rfc5322
        fn dot_atom_text<'a>() -> Parser<'a, char> {
            is_a(|c| c.is_ascii_alphanumeric()) | one_of("!#$%&'*+-/=?^_`{|}~")
        }
        fn domain_text<'a>() -> Parser<'a, char> {
            is_a(|c| c.is_ascii_alphanumeric())
        }

        let local_part = (dot_atom_text().repeat(1..64) + (sym('.') * dot_atom_text().repeat(1..))).repeat(0..64);
        let domain = (domain_text().repeat(1..255) + (sym('.') + domain_text().repeat(1..255))).repeat(0..64);
        (local_part - sym('@') + domain + end::<char>()).collect()
    }

    pub fn is_verified(&self) -> bool {
        match self {
            EmailAddress::Unverified(_) => false,
            EmailAddress::Verified(_) => true,
        }
    }

    pub fn verify_challenge(&self) -> Option<String> {
        if let EmailAddress::Unverified(address) = &self {
            // simplistic implementation, easy guessable
            let mut hasher = DefaultHasher::new();
            address.hash(&mut hasher);

            Some(hasher.finish().to_string())
        }
        else {
            None
        }
    }

    pub fn verify(self, challenge: String) -> Result<Self, EmailVerificationError> {
        let counter_challenge = self.verify_challenge().clone();

        match self {
            Self::Unverified(address) => {
                if counter_challenge == Some(challenge) {
                    Ok(Self::Verified(address))
                }
                else {
                    Err(EmailVerificationError::ChallengeDontMatch)
                }
            }
            Self::Verified(_) => {
                Ok(self)
            }
        }
    }
}

impl FromStr for EmailAddress {
    type Err = EmailParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.to_lowercase();
        let parsed = Self::parser()
            .parse_str(&input)
            .map_err(|_| EmailParseError::MalformedValue)?
            .to_owned();

        Ok(Self::Unverified(parsed))
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailAddress::Unverified(address) => f.write_str(address),
            EmailAddress::Verified(address) => f.write_str(address),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::{EmailAddress, EmailParseError, EmailVerificationError};

    #[test]
    fn parsed_values_are_unverified() {
        let result: Result<EmailAddress, EmailParseError> = "n.bergsma@internet.com".parse();
        assert_eq!(result, Ok(EmailAddress::Unverified("n.bergsma@internet.com".to_lowercase())));
    }

    #[test]
    fn is_case_insensitive() {
        let address1: EmailAddress = "n.bergsma@internet.com".parse().unwrap();
        let address2: EmailAddress = "N.BERGSMA@INTERNET.COM".parse().unwrap();

        assert_eq!(address1, address2);
    }

    #[test]
    fn errors_on_malformed_values() {
        // missing @-symbol
        let result: Result<EmailAddress, EmailParseError> = "n.bergsma.internet.com".parse();
        assert_eq!(result, Err(EmailParseError::MalformedValue));

        // missing tld
        let result: Result<EmailAddress, EmailParseError> = "n.bergsma@internet".parse();
        assert_eq!(result, Err(EmailParseError::MalformedValue));
    }

    #[test]
    fn is_serializable() {
        let original: EmailAddress = "n.bergsma@internet.com".parse().unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: EmailAddress = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn verify_an_unverified_by_a_challenge() {
        let before: EmailAddress = "n.bergsma@internet.com".parse().unwrap();
        assert_eq!(before, EmailAddress::Unverified("n.bergsma@internet.com".to_owned()));

        let challenge = before.verify_challenge().unwrap();
        let after = before.verify(challenge).unwrap();
        assert_eq!(after, EmailAddress::Verified("n.bergsma@internet.com".to_owned()));
    }

    #[test]
    fn can_test_if_verified() {
        let before: EmailAddress = "n.bergsma@internet.com".parse().unwrap();
        assert!(!before.is_verified());

        let challenge = before.verify_challenge().unwrap();
        let after = before.verify(challenge).unwrap();
        assert!(after.is_verified());
    }

    #[test]
    fn errors_on_malformed_challenge() {
        let before: EmailAddress = "n.bergsma@internet.com".parse().unwrap();
        assert_eq!(before, EmailAddress::Unverified("n.bergsma@internet.com".to_owned()));

        let result = before.verify("0000".to_owned());
        assert_eq!(result, Err(EmailVerificationError::ChallengeDontMatch));
    }
}