mod none_as_empty;
mod none_as_negative;
mod primitive;
mod timestamp;

use crate::error::Error;
use base64::prelude::{Engine, BASE64_STANDARD};
pub(crate) use none_as_empty::NoneAsEmpty;
pub(crate) use none_as_negative::NoneAsNegative;
pub(crate) use save_derive::{Decode, Encode};
use std::fmt;
pub(crate) use timestamp::Timestamp;

#[tracing::instrument(err)]
pub fn decode(value: &str) -> Result<super::Save, Error> {
    let value = urlencoding::decode(value)?;
    let value = value.trim_end_matches("!END!");
    let value = BASE64_STANDARD.decode(value)?;
    let value = String::from_utf8(value)?;
    tracing::debug!(value);
    Decode::decode(&value)
}

#[tracing::instrument]
pub fn encode(value: &super::Save) -> String {
    struct Display<'a>(&'a super::Save);
    impl fmt::Display for Display<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Encode::encode(self.0, f)
        }
    }

    let value = Display(value).to_string();
    tracing::debug!(value);
    let mut value = BASE64_STANDARD.encode(&value);
    value.push_str("!END!");
    urlencoding::encode(&value).into_owned()
}

pub(crate) trait Decode<'a>: Sized {
    fn decode(value: &'a str) -> Result<Self, Error>;
}
pub(crate) trait DecodeAs<'a, T> {
    fn decode_as(value: &'a str) -> Result<T, Error>;
}

pub(crate) trait Encode {
    fn encode(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}
pub(crate) trait EncodeAs<T> {
    fn encode_as(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

pub(crate) struct Same;
impl<'a, T> DecodeAs<'a, T> for Same
where
    T: Decode<'a>,
{
    fn decode_as(value: &'a str) -> Result<T, Error> {
        T::decode(value)
    }
}
impl<T> EncodeAs<T> for Same
where
    T: Encode,
{
    fn encode_as(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        value.encode(f)
    }
}
