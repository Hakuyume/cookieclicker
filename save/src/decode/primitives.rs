use super::{Decoder, Standard};
use crate::error::Error;

impl<T> Decoder<T, ()> for Standard
where
    T: Sized,
{
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(_: T) -> Result<(), Error> {
        Ok(())
    }
}

impl Decoder<char, bool> for Standard {
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(value: char) -> Result<bool, Error> {
        match value {
            '0' => Ok(false),
            '1' => Ok(true),
            _ => Err(Error::Bool),
        }
    }
}

impl Decoder<&str, bool> for Standard {
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(value: &str) -> Result<bool, Error> {
        match value {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(Error::Bool),
        }
    }
}

impl<'a> Decoder<&'a str, &'a str> for Standard {
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(value: &'a str) -> Result<&'a str, Error> {
        Ok(value)
    }
}

impl Decoder<&str, String> for Standard {
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(value: &str) -> Result<String, Error> {
        Ok(value.to_owned())
    }
}

macro_rules! from_str {
    ($ty:ty) => {
        impl Decoder<&str, $ty> for Standard {
            #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
            fn decode(value: &str) -> Result<$ty, Error> {
                Ok(value.parse()?)
            }
        }
    };
}
from_str!(u8);
from_str!(u64);
from_str!(usize);
from_str!(f64);
