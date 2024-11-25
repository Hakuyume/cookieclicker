use super::{DecodeAs, EncodeAs, Same};
use crate::error::Error;
use std::fmt;
use std::marker::PhantomData;

pub(crate) struct NoneAsEmpty<T = Same>(PhantomData<T>);

impl<'a, T, U> DecodeAs<'a, Option<T>> for NoneAsEmpty<U>
where
    U: DecodeAs<'a, T>,
{
    #[tracing::instrument(err)]
    fn decode_as(value: &'a str) -> Result<Option<T>, Error> {
        if value.is_empty() {
            Ok(None)
        } else {
            U::decode_as(value).map(Some)
        }
    }
}

impl<T, U> EncodeAs<Option<T>> for NoneAsEmpty<U>
where
    U: EncodeAs<T>,
{
    fn encode_as(value: &Option<T>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = value {
            U::encode_as(value, f)
        } else {
            Ok(())
        }
    }
}
