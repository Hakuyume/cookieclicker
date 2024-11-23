use super::Decode;
use crate::error::Error;

impl<T> Decode<T> for ()
where
    T: Sized,
{
    fn decode(_: T) -> Result<Self, Error> {
        Ok(())
    }
}

impl Decode<char> for bool {
    fn decode(value: char) -> Result<Self, Error> {
        match value {
            '0' => Ok(false),
            '1' => Ok(true),
            _ => Err(Error::Bool),
        }
    }
}

impl Decode<&str> for bool {
    fn decode(value: &str) -> Result<Self, Error> {
        match value {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(Error::Bool),
        }
    }
}

impl<'a> Decode<&'a str> for &'a str {
    fn decode(value: &'a str) -> Result<Self, Error> {
        Ok(value)
    }
}

impl Decode<&str> for String {
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.to_owned())
    }
}

macro_rules! from_str {
    ($ty:ty) => {
        impl Decode<&str> for $ty {
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
