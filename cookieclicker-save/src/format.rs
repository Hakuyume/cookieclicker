mod none_as;
mod standard;
mod timestamp;

use crate::error::Error;
pub(crate) use cookieclicker_save_derive::Format;
pub(crate) use none_as::{NoneAsEmpty, NoneAsNegative, NoneAsZero};
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
    fn check_inverse_hook<'b>(_: &'b str) -> anyhow::Result<()>
    where
        'b: 'a,
        Self: 'b,
    {
        Ok(())
    }
}

pub(crate) fn chars(value: &str) -> impl Iterator<Item = &str> {
    value
        .char_indices()
        .map(|(offset, c)| &value[offset..offset + c.len_utf8()])
}

#[cfg(test)]
#[tracing::instrument(err)]
pub(crate) fn check_inverse<'a, 'b, T, U>(value: &'b str) -> anyhow::Result<()>
where
    'b: 'a,
    T: Format<'a, U> + 'b,
{
    T::check_inverse_hook(value)?;
    let actual = T::display(&T::decode(value)?).to_string();
    let expected = value.to_owned();
    anyhow::ensure!(
        actual == expected,
        "actual = {actual:?}, expected = {expected:?}",
    );
    Ok(())
}
