use super::{Format, Standard};
use crate::error::Error;
use std::fmt;
use std::marker::PhantomData;

pub(crate) struct NoneAsEmpty<T = Standard>(PhantomData<T>);

impl<'a, T, U> Format<'a, Option<T>> for NoneAsEmpty<U>
where
    U: Format<'a, T>,
{
    #[tracing::instrument(err)]
    fn decode(value: &'a str) -> Result<Option<T>, Error> {
        if value.is_empty() {
            Ok(None)
        } else {
            U::decode(value).map(Some)
        }
    }

    fn encode(value: &Option<T>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = value {
            U::encode(value, f)
        } else {
            Ok(())
        }
    }
}
