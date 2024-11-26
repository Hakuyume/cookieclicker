use super::{Format, Standard};
use crate::error::Error;
use std::fmt;
use std::marker::PhantomData;

macro_rules! none_as {
    ($name:ident, $none:literal) => {
        pub(crate) struct $name<T = Standard>(PhantomData<T>);

        impl<'a, T, U> Format<'a, Option<T>> for $name<U>
        where
            U: Format<'a, T>,
        {
            #[tracing::instrument(err)]
            fn decode(value: &'a str) -> Result<Option<T>, Error> {
                if value == $none {
                    Ok(None)
                } else {
                    U::decode(value).map(Some)
                }
            }

            fn encode(value: &Option<T>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if let Some(value) = value {
                    U::encode(value, f)
                } else {
                    write!(f, $none)
                }
            }
        }
    };
}
none_as!(NoneAsEmpty, "");
none_as!(NoneAsNegative, "-1");
none_as!(NoneAsZero, "0");
