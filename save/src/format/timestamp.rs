use super::DecodeAs;
use crate::error::Error;
use chrono::{DateTime, Utc};

pub(crate) struct Timestamp;

impl DecodeAs<'_, DateTime<Utc>> for Timestamp {
    #[tracing::instrument(err)]
    fn decode_as(value: &str) -> Result<DateTime<Utc>, Error> {
        DateTime::from_timestamp_millis(value.parse()?).ok_or(Error::TimestampOutOfRange)
    }
}
