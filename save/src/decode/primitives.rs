use super::{Decoder, Standard};
use crate::error::Error;

impl Decoder<'_, ()> for Standard {
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(_: &str) -> Result<(), Error> {
        Ok(())
    }
}

impl Decoder<'_, bool> for Standard {
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(value: &str) -> Result<bool, Error> {
        match value {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(Error::Bool),
        }
    }
}

impl<'a> Decoder<'a, &'a str> for Standard {
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(value: &'a str) -> Result<&'a str, Error> {
        Ok(value)
    }
}

impl Decoder<'_, String> for Standard {
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(value: &str) -> Result<String, Error> {
        Ok(value.to_owned())
    }
}

macro_rules! from_str {
    ($ty:ty) => {
        impl Decoder<'_, $ty> for Standard {
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
