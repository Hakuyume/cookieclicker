mod none_as_empty;
mod none_as_negative;
mod standard;
mod timestamp;

use crate::error::Error;
pub(crate) use none_as_empty::NoneAsEmpty;
pub(crate) use none_as_negative::NoneAsNegative;
pub(crate) use save_derive::Format;
pub(crate) use standard::Standard;
use std::fmt;
pub(crate) use timestamp::Timestamp;

pub(crate) trait Format<'a, T> {
    fn decode(value: &'a str) -> Result<T, Error>;
    fn encode(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    fn display<'b>(value: &'b T) -> impl fmt::Display + 'b
    where
        Self: 'b,
    {
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

        Display(value, Self::encode)
    }

    #[cfg(test)]
    #[tracing::instrument(err)]
    fn check_inverse<'b>(value: &'b str) -> Result<(), Error>
    where
        'b: 'a,
        Self: 'b,
    {
        let actual = Self::display(&Self::decode(value)?).to_string();
        let expected = value.to_owned();
        if actual == expected {
            Ok(())
        } else {
            Err(Error::CheckInverse { actual, expected })
        }
    }
}

pub(crate) fn chars(value: &str) -> impl Iterator<Item = &str> {
    value.split("").filter(|v| !v.is_empty())
}
