use crate::error::Error;
use crate::format::{self, Decode, DecodeAs, Encode, EncodeAs};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Garden {
    pub time_of_next_tick: DateTime<Utc>,
    pub soil_type: usize,
    pub time_of_next_soil_change: DateTime<Utc>,
    pub frozen_garden: bool,
    pub harvests_this_ascension: u64,
    pub total_harvests: u64,
    pub unlocked_seeds: Vec<bool>,
    pub farm_grid_data: Vec<Option<FarmGridData>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FarmGridData {
    pub id: usize,
    pub age: u64,
}

#[derive(Decode, Encode)]
#[format(split = ' ')]
struct Format<T, U> {
    inner: Inner,
    #[format(as = Custom)]
    unlocked_seeds: T,
    #[format(as = Custom)]
    farm_grid_data: U,
}

#[derive(Decode, Encode)]
#[format(split = ':')]
struct Inner {
    #[format(as = format::Timestamp)]
    time_of_next_tick: DateTime<Utc>,
    soil_type: usize,
    #[format(as = format::Timestamp)]
    time_of_next_soil_change: DateTime<Utc>,
    frozen_garden: bool,
    harvests_this_ascension: u64,
    total_harvests: u64,
}

struct Custom;

impl DecodeAs<'_, Vec<bool>> for Custom {
    #[tracing::instrument(err)]
    fn decode_as(value: &str) -> Result<Vec<bool>, Error> {
        format::chars(value).map(Decode::decode).collect()
    }
}

impl EncodeAs<&Vec<bool>> for Custom {
    fn encode_as(value: &&Vec<bool>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for v in *value {
            Encode::encode(v, f)?;
        }
        Ok(())
    }
}

impl DecodeAs<'_, Vec<Option<FarmGridData>>> for Custom {
    #[tracing::instrument(err)]
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

impl EncodeAs<&Vec<Option<FarmGridData>>> for Custom {
    fn encode_as(value: &&Vec<Option<FarmGridData>>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, v) in value.iter().enumerate() {
            if i > 0 {
                write!(f, ":")?;
            }
            if let Some(FarmGridData { id, age }) = v {
                Encode::encode(id, f)?;
                write!(f, ":")?;
                Encode::encode(age, f)?;
            } else {
                write!(f, "0:0")?;
            }
        }
        Ok(())
    }
}

impl Decode<'_> for Garden {
    #[tracing::instrument(err)]
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

impl Encode for Garden {
    fn encode(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            time_of_next_tick,
            soil_type,
            time_of_next_soil_change,
            frozen_garden,
            harvests_this_ascension,
            total_harvests,
            ref unlocked_seeds,
            ref farm_grid_data,
        } = *self;
        Encode::encode(
            &Format {
                inner: Inner {
                    time_of_next_tick,
                    soil_type,
                    time_of_next_soil_change,
                    frozen_garden,
                    harvests_this_ascension,
                    total_harvests,
                },
                unlocked_seeds,
                farm_grid_data,
            },
            f,
        )
    }
}
