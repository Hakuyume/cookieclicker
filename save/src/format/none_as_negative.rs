use super::{DecodeAs, EncodeAs};
use crate::error::Error;
use std::fmt;

pub(crate) struct NoneAsNegative;

impl DecodeAs<'_, Option<u64>> for NoneAsNegative {
    #[tracing::instrument(err)]
    fn decode_as(value: &str) -> Result<Option<u64>, Error> {
        let value = value.parse::<i64>()?;
        if value >= 0 {
            Ok(Some(value as _))
        } else {
            Ok(None)
        }
    }
}

impl EncodeAs<Option<u64>> for NoneAsNegative {
    fn encode_as(value: &Option<u64>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = value {
            write!(f, "{value}")
        } else {
            write!(f, "-1")
        }
    }
}
