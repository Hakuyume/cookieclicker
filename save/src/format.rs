mod none_as_empty;
mod none_as_negative;
mod primitive;
mod timestamp;

use crate::error::Error;
pub(crate) use none_as_empty::NoneAsEmpty;
pub(crate) use none_as_negative::NoneAsNegative;
pub(crate) use save_derive::Format;
use std::fmt;
pub(crate) use timestamp::Timestamp;

pub(crate) trait Format<'a>: Sized {
    fn decode(value: &'a str) -> Result<Self, Error>;
    fn encode(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

pub(crate) trait FormatAs<'a, T>: Sized {
    fn decode_as(value: &'a str) -> Result<T, Error>;
    fn encode_as(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

pub(crate) struct Same;
impl<'a, T> FormatAs<'a, T> for Same
where
    T: Format<'a>,
{
    fn decode_as(value: &'a str) -> Result<T, Error> {
        Format::decode(value)
    }
    fn encode_as(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Format::encode(value, f)
    }
}

pub(crate) fn chars(value: &str) -> impl Iterator<Item = &str> {
    value.split("").filter(|v| !v.is_empty())
}

pub(crate) trait FormatExt<'a>: Format<'a> {
    fn display(&self) -> impl fmt::Display + '_ {
        struct Display<'a, T, F>(&'a T, F)
        where
            T: ?Sized;
        impl<T, F> fmt::Display for Display<'_, T, F>
        where
            T: ?Sized,
            F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.1(self.0, f)
            }
        }

        Display(self, Self::encode)
    }
}
impl<'a, T> FormatExt<'a> for T where T: Format<'a> {}
