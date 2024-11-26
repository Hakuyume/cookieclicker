use crate::error::Error;
use crate::format;
use std::fmt;

pub(crate) struct Custom;

impl format::Format<'_, Vec<bool>> for Custom {
    #[tracing::instrument(err)]
    fn decode(value: &str) -> Result<Vec<bool>, Error> {
        format::chars(value).map(format::Standard::decode).collect()
    }

    fn encode(value: &Vec<bool>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for v in value {
            format::Standard::encode(v, f)?;
        }
        Ok(())
    }
}
