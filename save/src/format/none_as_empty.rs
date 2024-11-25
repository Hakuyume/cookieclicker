use super::{DecodeAs, Same};
use crate::error::Error;
use std::marker::PhantomData;

pub(crate) struct NoneAsEmpty<T = Same>(PhantomData<T>);

impl<'a, T, U> DecodeAs<'a, Option<T>> for NoneAsEmpty<U>
where
    U: DecodeAs<'a, T>,
{
    fn decode_as(value: &'a str) -> Result<Option<T>, Error> {
        if value.is_empty() {
            Ok(None)
        } else {
            U::decode_as(value).map(Some)
        }
    }
}
