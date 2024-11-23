use crate::error::Error;
use super::Decode;

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

impl Decode<&str> for u8 {
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.parse()?)
    }
}

impl Decode<&str> for u64 {
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.parse()?)
    }
}

impl Decode<&str> for usize {
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.parse()?)
    }
}

impl Decode<&str> for f64 {
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.parse()?)
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
