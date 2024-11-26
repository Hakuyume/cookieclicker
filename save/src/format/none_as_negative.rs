use super::Format;
use crate::error::Error;
use std::fmt;

pub(crate) struct NoneAsNegative;

macro_rules! impl_ {
    ($u:ty, $i:ty) => {
        impl Format<'_, Option<$u>> for NoneAsNegative {
            #[tracing::instrument(err)]
            fn decode(value: &str) -> Result<Option<$u>, Error> {
                let value = value.parse::<$i>()?;
                if value >= 0 {
                    Ok(Some(value as _))
                } else {
                    Ok(None)
                }
            }

            fn encode(value: &Option<$u>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if let Some(value) = value {
                    write!(f, "{value}")
                } else {
                    write!(f, "-1")
                }
            }
        }
    };
}
impl_!(u64, i64);
impl_!(usize, isize);
