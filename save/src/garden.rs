use crate::error::Error;
use crate::format::{Decode, DecodeAs};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Garden {
    pub time_of_next_tick: u64,
    pub soil_type: usize,
    pub time_of_next_soil_change: u64,
    pub frozen_garden: bool,
    pub harvests_this_ascension: u64,
    pub total_harvests: u64,
    pub unlocked_seeds: Vec<bool>,
    pub farm_grid_data: Vec<Option<FarmGridData>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FarmGridData {
    pub id: usize,
    pub age: u8,
}

#[derive(Debug, Decode)]
#[format(split = ' ')]
#[allow(dead_code)]
struct Format {
    inner: Inner,
    #[format(as = Custom)]
    unlocked_seeds: Vec<bool>,
    #[format(as = Custom)]
    farm_grid_data: Vec<Option<FarmGridData>>,
}

#[derive(Debug, Decode)]
#[format(split = ':')]
struct Inner {
    time_of_next_tick: u64,
    soil_type: usize,
    time_of_next_soil_change: u64,
    frozen_garden: bool,
    harvests_this_ascension: u64,
    total_harvests: u64,
}

struct Custom;

impl DecodeAs<'_, Vec<bool>> for Custom {
    fn decode_as(value: &str) -> Result<Vec<bool>, Error> {
        value
            .split("")
            .filter(|s| !s.is_empty())
            .map(Decode::decode)
            .collect()
    }
}

impl DecodeAs<'_, Vec<Option<FarmGridData>>> for Custom {
    fn decode_as(value: &str) -> Result<Vec<Option<FarmGridData>>, Error> {
        value
            .split(':')
            .tuples()
            .map(|(id, age)| {
                let id = Decode::decode(id)?;
                let age = Decode::decode(age)?;
                if id == 0 {
                    Ok(None)
                } else {
                    Ok(Some(FarmGridData { id, age }))
                }
            })
            .collect()
    }
}

impl Decode<'_> for Garden {
    fn decode(value: &str) -> Result<Self, Error> {
        let Format {
            inner:
                Inner {
                    time_of_next_tick,
                    soil_type,
                    time_of_next_soil_change,
                    frozen_garden,
                    harvests_this_ascension,
                    total_harvests,
                },
            unlocked_seeds,
            farm_grid_data,
        } = Decode::decode(value)?;
        Ok(Self {
            time_of_next_tick,
            soil_type,
            time_of_next_soil_change,
            frozen_garden,
            harvests_this_ascension,
            total_harvests,
            unlocked_seeds,
            farm_grid_data,
        })
    }
}
