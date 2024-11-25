use crate::error::Error;
use crate::format::{self, Decode, DecodeAs, Encode, EncodeAs};
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
        format::chars(value)
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
        for v in value {
            Encode::encode(&v.unlocked, f)?;
            Encode::encode(&v.bought, f)?;
        }
        Ok(())
    }
}
