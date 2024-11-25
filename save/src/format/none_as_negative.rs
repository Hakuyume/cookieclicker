use super::DecodeAs;
use crate::error::Error;

pub(crate) struct NoneAsNegative;

impl DecodeAs<'_, Option<u64>> for NoneAsNegative {
    fn decode_as(value: &str) -> Result<Option<u64>, Error> {
        let value = value.parse::<i64>()?;
        if value >= 0 {
            Ok(Some(value as _))
        } else {
            Ok(None)
        }
    }
}
