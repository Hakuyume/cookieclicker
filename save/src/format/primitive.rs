use super::{Decode, Encode};
use crate::error::Error;
use std::fmt;

impl Decode<'_> for () {
    #[tracing::instrument(err)]
    fn decode(_: &str) -> Result<Self, Error> {
        Ok(())
    }
}
impl Encode for () {
    fn encode(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Decode<'_> for bool {
    #[tracing::instrument(err)]
    fn decode(value: &str) -> Result<Self, Error> {
        match value {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(Error::InvalidData),
        }
    }
}
impl Encode for bool {
    fn encode(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if *self { 1 } else { 0 })
    }
}

impl<'a> Decode<'a> for &'a str {
    #[tracing::instrument(err)]
    fn decode(value: &'a str) -> Result<Self, Error> {
        Ok(value)
    }
}
impl Encode for &str {
    fn encode(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Decode<'_> for String {
    #[tracing::instrument(err)]
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.to_owned())
    }
}
impl Encode for String {
    fn encode(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

macro_rules! display_from_str {
    ($ty:ty) => {
        impl Decode<'_> for $ty {
            #[tracing::instrument(err)]
            fn decode(value: &str) -> Result<Self, Error> {
                Ok(value.parse()?)
            }
        }
        impl Encode for $ty {
            fn encode(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{self}")
            }
        }
    };
}
display_from_str!(u8);
display_from_str!(u64);
display_from_str!(usize);
display_from_str!(i64);
display_from_str!(f64);
