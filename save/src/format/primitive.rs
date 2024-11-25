use super::Decode;
use crate::error::Error;

impl Decode<'_> for () {
    fn decode(_: &str) -> Result<Self, Error> {
        Ok(())
    }
}

impl Decode<'_> for bool {
    fn decode(value: &str) -> Result<Self, Error> {
        match value {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(Error::Bool),
        }
    }
}

impl<'a> Decode<'a> for &'a str {
    fn decode(value: &'a str) -> Result<Self, Error> {
        Ok(value)
    }
}

impl Decode<'_> for String {
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.to_owned())
    }
}

macro_rules! from_str {
    ($ty:ty) => {
        impl Decode<'_> for $ty {
            fn decode(value: &str) -> Result<Self, Error> {
                Ok(value.parse()?)
            }
        }
    };
}
from_str!(u8);
from_str!(u64);
from_str!(usize);
from_str!(f64);
