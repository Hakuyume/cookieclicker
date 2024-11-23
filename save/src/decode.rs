mod primitives;

use crate::error::Error;
use base64::prelude::{Engine, BASE64_STANDARD};
use itertools::Itertools;
pub(crate) use save_derive::Decode;

#[tracing::instrument(err, ret)]
pub fn decode(value: &str) -> Result<super::Save, Error> {
    let value = urlencoding::decode(value)?;
    let value = value.trim_end_matches("!END!");
    let value = BASE64_STANDARD.decode(value)?;
    let value = String::from_utf8(value)?;
    tracing::info!(value);
    super::Save::decode(&value)
}

pub(crate) trait Decode<V>: Sized {
    fn decode(value: V) -> Result<Self, Error>;
}

impl Decode<&str> for super::Garden {
    fn decode(value: &str) -> Result<Self, Error> {
        #[derive(Decode)]
        #[decode(pat = ' ')]
        struct Segments<'a> {
            a: A,
            b: &'a str,
            c: &'a str,
        }

        #[derive(Decode)]
        #[decode(pat = ':')]
        struct A {
            time_of_next_tick: u64,
            soil_type: usize,
            time_of_next_soil_change: u64,
            frozen_garden: bool,
            harvests_this_ascension: u64,
            total_harvests: u64,
        }

        let segments = Segments::decode(value)?;
        Ok(Self {
            time_of_next_tick: segments.a.time_of_next_tick,
            soil_type: segments.a.soil_type,
            time_of_next_soil_change: segments.a.time_of_next_soil_change,
            frozen_garden: segments.a.frozen_garden,
            harvests_this_ascension: segments.a.harvests_this_ascension,
            total_harvests: segments.a.total_harvests,
            unlocked_seeds: segments
                .b
                .chars()
                .map(Decode::decode)
                .collect::<Result<Vec<_>, _>>()?,
            farm_grid_data: segments
                .c
                .split(':')
                .tuples()
                .map(|(id, age)| {
                    let id = Decode::decode(id)?;
                    let age = Decode::decode(age)?;
                    if id == 0 {
                        Ok(None)
                    } else {
                        Ok(Some(super::FarmGridData { id, age }))
                    }
                })
                .collect::<Result<Vec<_>, Error>>()?,
        })
    }
}
