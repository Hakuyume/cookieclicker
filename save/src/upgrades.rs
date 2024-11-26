use crate::error::Error;
use crate::format;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Upgrade {
    pub unlocked: bool,
    pub bought: bool,
}

pub(crate) struct Custom;

impl format::FormatAs<'_, Vec<Upgrade>> for Custom {
    #[tracing::instrument(err)]
    fn decode_as(value: &str) -> Result<Vec<Upgrade>, Error> {
        format::chars(value)
            .tuples()
            .map(|(unlocked, bought)| {
                let unlocked = format::Format::decode(unlocked)?;
                let bought = format::Format::decode(bought)?;
                Ok(Upgrade { unlocked, bought })
            })
            .collect()
    }

    fn encode_as(value: &Vec<Upgrade>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for v in value {
            format::Format::encode(&v.unlocked, f)?;
            format::Format::encode(&v.bought, f)?;
        }
        Ok(())
    }
}
