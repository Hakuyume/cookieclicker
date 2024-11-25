use crate::error::Error;
use crate::format::{Decode, DecodeAs, Encode, EncodeAs};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Upgrade {
    pub unlocked: bool,
    pub bought: bool,
}

pub(crate) struct Custom;

impl DecodeAs<'_, Vec<Upgrade>> for Custom {
    #[tracing::instrument(err)]
    fn decode_as(value: &str) -> Result<Vec<Upgrade>, Error> {
        value
            .split("")
            .filter(|s| !s.is_empty())
            .tuples()
            .map(|(unlocked, bought)| {
                let unlocked = Decode::decode(unlocked)?;
                let bought = Decode::decode(bought)?;
                Ok(Upgrade { unlocked, bought })
            })
            .collect()
    }
}

impl EncodeAs<Vec<Upgrade>> for Custom {
    fn encode_as(value: &Vec<Upgrade>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for value in value {
            Encode::encode(&value.unlocked, f)?;
            Encode::encode(&value.bought, f)?;
        }
        Ok(())
    }
}
