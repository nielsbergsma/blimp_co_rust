pub fn capital<'a>() -> Parser<'a, char> {
    is_a(|c| c.is_alphabetic() && c.is_uppercase())
}

pub fn alphanumeric<'a>() -> Parser<'a, char> {
    is_a(|c| c.is_alphanumeric())
}

pub fn spaces<'a>() -> Parser<'a, char> {
    sym(' ')
}

pub use pom::utf8::{Parser, sym, end, is_a, one_of};
