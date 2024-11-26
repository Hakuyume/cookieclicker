use super::FormatAs;
use crate::error::Error;
use chrono::{DateTime, Utc};
use std::fmt;

pub(crate) struct Timestamp;

impl FormatAs<'_, DateTime<Utc>> for Timestamp {
    #[tracing::instrument(err)]
    fn decode_as(value: &str) -> Result<DateTime<Utc>, Error> {
        DateTime::from_timestamp_millis(value.parse()?).ok_or(Error::InvalidData)
    }

    fn encode_as(value: &DateTime<Utc>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", value.timestamp_millis())
    }
}
