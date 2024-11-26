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

impl format::Format<'_, Vec<Upgrade>> for Custom {
    #[tracing::instrument(err)]
    fn decode(value: &str) -> Result<Vec<Upgrade>, Error> {
        format::chars(value)
            .tuples()
            .map(|(unlocked, bought)| {
                let unlocked = format::Standard::decode(unlocked)?;
                let bought = format::Standard::decode(bought)?;
                Ok(Upgrade { unlocked, bought })
            })
            .collect()
    }

    fn encode(value: &Vec<Upgrade>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for v in value {
            format::Standard::encode(&v.unlocked, f)?;
            format::Standard::encode(&v.bought, f)?;
        }
        Ok(())
    }
}
