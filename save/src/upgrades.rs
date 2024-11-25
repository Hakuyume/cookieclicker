use crate::error::Error;
use crate::format::{Decode, DecodeAs};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Upgrade {
    pub unlocked: bool,
    pub bought: bool,
}

pub(crate) struct Custom;

impl DecodeAs<'_, Vec<Upgrade>> for Custom {
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
