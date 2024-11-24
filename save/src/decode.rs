mod primitives;

use crate::error::Error;
use base64::prelude::{Engine, BASE64_STANDARD};
use itertools::Itertools;
pub(crate) use save_derive::Decode;
use std::fmt::Debug;
use std::marker::PhantomData;

#[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
pub fn decode(value: &str) -> Result<super::Save, Error> {
    let value = urlencoding::decode(value)?;
    let value = value.trim_end_matches("!END!");
    let value = BASE64_STANDARD.decode(value)?;
    let value = String::from_utf8(value)?;
    <Standard as Decoder<_, super::Save>>::decode(&value)
}

pub(crate) trait Decoder<V, T> {
    fn decode(value: V) -> Result<T, Error>;
}

pub(crate) struct Standard;

pub(crate) struct NoneAsEmpty<D = Standard>(PhantomData<D>);
impl<'a, T, D> Decoder<&'a str, Option<T>> for NoneAsEmpty<D>
where
    T: Debug,
    D: Decoder<&'a str, T>,
{
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(value: &'a str) -> Result<Option<T>, Error> {
        if value.is_empty() {
            Ok(None)
        } else {
            D::decode(value).map(Some)
        }
    }
}

pub(crate) struct NoneAsNegative;
impl Decoder<&str, Option<u64>> for NoneAsNegative {
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(value: &str) -> Result<Option<u64>, Error> {
        let value = value.parse::<i64>()?;
        if value >= 0 {
            Ok(Some(value as _))
        } else {
            Ok(None)
        }
    }
}

impl Decoder<&str, super::Garden> for Standard {
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(value: &str) -> Result<super::Garden, Error> {
        #[derive(Debug, Decode)]
        #[decode(pat = ' ')]
        struct Segments<'a> {
            a: A,
            b: &'a str,
            c: &'a str,
        }

        #[derive(Debug, Decode)]
        #[decode(pat = ':')]
        struct A {
            time_of_next_tick: u64,
            soil_type: usize,
            time_of_next_soil_change: u64,
            frozen_garden: bool,
            harvests_this_ascension: u64,
            total_harvests: u64,
        }

        let segments = <Standard as Decoder<_, Segments>>::decode(value)?;
        Ok(super::Garden {
            time_of_next_tick: segments.a.time_of_next_tick,
            soil_type: segments.a.soil_type,
            time_of_next_soil_change: segments.a.time_of_next_soil_change,
            frozen_garden: segments.a.frozen_garden,
            harvests_this_ascension: segments.a.harvests_this_ascension,
            total_harvests: segments.a.total_harvests,
            unlocked_seeds: segments
                .b
                .chars()
                .map(Standard::decode)
                .collect::<Result<Vec<_>, _>>()?,
            farm_grid_data: segments
                .c
                .split(':')
                .tuples()
                .map(|(id, age)| {
                    let id = Standard::decode(id)?;
                    let age = Standard::decode(age)?;
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

impl Decoder<&str, Vec<super::Upgrade>> for Standard {
    #[tracing::instrument(err, ret(level = tracing::Level::DEBUG))]
    fn decode(value: &str) -> Result<Vec<super::Upgrade>, Error> {
        value
            .chars()
            .tuples()
            .map(|(unlocked, bought)| {
                let unlocked = Standard::decode(unlocked)?;
                let bought = Standard::decode(bought)?;
                Ok(super::Upgrade { unlocked, bought })
            })
            .collect()
    }
}
