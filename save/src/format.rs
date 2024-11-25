mod none_as_empty;
mod none_as_negative;
mod primitive;
mod timestamp;

use crate::error::Error;
use base64::prelude::{Engine, BASE64_STANDARD};
pub(crate) use none_as_empty::NoneAsEmpty;
pub(crate) use none_as_negative::NoneAsNegative;
pub(crate) use save_derive::Decode;
pub(crate) use timestamp::Timestamp;

#[tracing::instrument(err)]
pub fn decode(value: &str) -> Result<super::Save, Error> {
    let value = urlencoding::decode(value)?;
    let value = value.trim_end_matches("!END!");
    let value = BASE64_STANDARD.decode(value)?;
    let value = String::from_utf8(value)?;
    Decode::decode(&value)
}

pub(crate) trait Decode<'a>: Sized {
    fn decode(value: &'a str) -> Result<Self, Error>;
}

pub(crate) trait DecodeAs<'a, T> {
    fn decode_as(value: &'a str) -> Result<T, Error>;
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
