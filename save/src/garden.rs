use crate::error::Error;
use crate::format;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
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

#[derive(format::Format)]
#[format(split = ' ')]
struct Format<'a> {
    inner: Inner,
    #[format(as = Custom)]
    unlocked_seeds: Cow<'a, [bool]>,
    #[format(as = Custom)]
    farm_grid_data: Cow<'a, [Option<FarmGridData>]>,
}

#[derive(format::Format)]
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

impl<'a> format::FormatAs<'_, Cow<'a, [bool]>> for Custom {
    #[tracing::instrument(err)]
    fn decode_as(value: &str) -> Result<Cow<'a, [bool]>, Error> {
        format::chars(value).map(format::Format::decode).collect()
    }

    fn encode_as(value: &Cow<'_, [bool]>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for v in value.as_ref() {
            format::Format::encode(v, f)?;
        }
        Ok(())
    }
}

impl<'a> format::FormatAs<'_, Cow<'a, [Option<FarmGridData>]>> for Custom {
    #[tracing::instrument(err)]
    fn decode_as(value: &str) -> Result<Cow<'a, [Option<FarmGridData>]>, Error> {
        value
            .split(':')
            .tuples()
            .map(|(id, age)| {
                let id = format::Format::decode(id)?;
                let age = format::Format::decode(age)?;
                if id == 0 {
                    Ok(None)
                } else {
                    Ok(Some(FarmGridData { id, age }))
                }
            })
            .collect()
    }

    fn encode_as(
        value: &Cow<'a, [Option<FarmGridData>]>,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        for (i, v) in value.iter().enumerate() {
            if i > 0 {
                write!(f, ":")?;
            }
            if let Some(FarmGridData { id, age }) = v {
                format::Format::encode(id, f)?;
                write!(f, ":")?;
                format::Format::encode(age, f)?;
            } else {
                write!(f, "0:0")?;
            }
        }
        Ok(())
    }
}

impl format::Format<'_> for Garden {
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
        } = format::Format::decode(value)?;
        Ok(Self {
            time_of_next_tick,
            soil_type,
            time_of_next_soil_change,
            frozen_garden,
            harvests_this_ascension,
            total_harvests,
            unlocked_seeds: unlocked_seeds.into(),
            farm_grid_data: farm_grid_data.into(),
        })
    }

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
        format::Format::encode(
            &Format {
                inner: Inner {
                    time_of_next_tick,
                    soil_type,
                    time_of_next_soil_change,
                    frozen_garden,
                    harvests_this_ascension,
                    total_harvests,
                },
                unlocked_seeds: unlocked_seeds.into(),
                farm_grid_data: farm_grid_data.into(),
            },
            f,
        )
    }
}
