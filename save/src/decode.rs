use crate::Error;
use base64::prelude::{Engine, BASE64_STANDARD};
use itertools::Itertools;
use std::str;

#[tracing::instrument(err, ret)]
pub fn decode(value: &str) -> Result<super::Save, Error> {
    let value = urlencoding::decode(value)?;
    let value = value.trim_end_matches("!END!");
    let value = BASE64_STANDARD.decode(value)?;
    let value = String::from_utf8(value)?;
    tracing::info!(value);
    super::Save::decode(&value)
}

trait Decode<V>: Sized {
    fn decode(value: V) -> Result<Self, Error>;
}

impl<T> Decode<T> for ()
where
    T: Sized,
{
    fn decode(_: T) -> Result<Self, Error> {
        Ok(())
    }
}

impl Decode<char> for bool {
    fn decode(value: char) -> Result<Self, Error> {
        match value {
            '0' => Ok(false),
            '1' => Ok(true),
            _ => Err(Error::Bool),
        }
    }
}

impl Decode<&str> for bool {
    fn decode(value: &str) -> Result<Self, Error> {
        match value {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(Error::Bool),
        }
    }
}

impl Decode<&str> for u8 {
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.parse()?)
    }
}

impl Decode<&str> for u64 {
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.parse()?)
    }
}

impl Decode<&str> for usize {
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.parse()?)
    }
}

impl Decode<&str> for f64 {
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.parse()?)
    }
}

impl<'a> Decode<&'a str> for &'a str {
    fn decode(value: &'a str) -> Result<Self, Error> {
        Ok(value)
    }
}

impl Decode<&str> for String {
    fn decode(value: &str) -> Result<Self, Error> {
        Ok(value.to_owned())
    }
}

trait IteratorExt: Iterator {
    fn next_decode<T>(&mut self) -> Result<T, Error>
    where
        T: Decode<Self::Item>;
}
impl<I> IteratorExt for I
where
    I: Iterator,
{
    fn next_decode<T>(&mut self) -> Result<T, Error>
    where
        T: Decode<Self::Item>,
    {
        T::decode(self.next().ok_or(Error::InsufficientData)?)
    }
}

impl Decode<&str> for super::Save {
    fn decode(value: &str) -> Result<Self, Error> {
        let mut segments = value.split('|');
        Ok(Self {
            game_version: segments.next_decode()?,
            run_details: segments.by_ref().skip(1).next_decode()?,
            preferences: segments.next_decode()?,
            miscellaneous_game_data: segments.next_decode()?,
            building_data: segments.next_decode()?,
        })
    }
}

impl Decode<&str> for super::GameVersion {
    fn decode(value: &str) -> Result<Self, Error> {
        let mut segments = value.split(';');
        Ok(Self {
            game_version: segments.next_decode()?,
        })
    }
}

impl Decode<&str> for super::RunDetails {
    fn decode(value: &str) -> Result<Self, Error> {
        let mut segments = value.split(';');
        Ok(Self {
            ascension_start: segments.next_decode()?,
            legacy_start: segments.next_decode()?,
            last_opened: segments.next_decode()?,
            bakery_name: segments.next_decode()?,
            seed: segments.next_decode()?,
            you_appearance: segments.next_decode()?,
        })
    }
}

impl Decode<&str> for super::Preferences {
    fn decode(value: &str) -> Result<Self, Error> {
        let mut segments = value.chars();
        Ok(Self {
            particles: segments.next_decode()?,
        })
    }
}

impl Decode<&str> for super::MiscellaneousGameData {
    fn decode(value: &str) -> Result<Self, Error> {
        let mut segments = value.split(';');
        Ok(Self {
            cookies_in_bank: segments.next_decode()?,
            cookies_baked: segments.next_decode()?,
            cookie_clicks: segments.next_decode()?,
            total_golden_cookie_clicks: segments.next_decode()?,
            hand_made_cookies: segments.next_decode()?,
            total_golden_cookies_missed: segments.next_decode()?,
            background_type: segments.next_decode()?,
            milk_type: segments.next_decode()?,
            cookies_forfeited_by_ascending: segments.next_decode()?,
            grandmapocalypse_stage: segments.next_decode()?,
            elder_pledges_made: segments.next_decode()?,
            time_left_in_elder_pledge: segments.next_decode()?,
            currently_researching: segments.next_decode()?,
            time_left_in_research: segments.next_decode()?,
            ascensions: segments.next_decode()?,
        })
    }
}

impl Decode<&str> for super::BuildingData {
    fn decode(value: &str) -> Result<Self, Error> {
        let mut segments = value.split(';');
        Ok(Self {
            cursors: segments.next_decode()?,
            grandmas: segments.next_decode()?,
            farms: segments.next_decode()?,
            mines: segments.next_decode()?,
            factories: segments.next_decode()?,
            banks: segments.next_decode()?,
            temples: segments.next_decode()?,
            wizard_towers: segments.next_decode()?,
            shipments: segments.next_decode()?,
            alchemy_labs: segments.next_decode()?,
            portals: segments.next_decode()?,
            time_machines: segments.next_decode()?,
            antimatter_condensers: segments.next_decode()?,
            prisms: segments.next_decode()?,
            chancemakers: segments.next_decode()?,
            fractal_engines: segments.next_decode()?,
            javascript_consoles: segments.next_decode()?,
            idleverses: segments.next_decode()?,
            cortex_bakers: segments.next_decode()?,
            yous: segments.next_decode()?,
        })
    }
}

impl<'a, M> Decode<&'a str> for super::BuildingDataEntry<M>
where
    M: Decode<&'a str>,
{
    fn decode(value: &'a str) -> Result<Self, Error> {
        let mut segments = value.split(',');
        Ok(Self {
            amount_owned: segments.next_decode()?,
            amount_bought: segments.next_decode()?,
            cookies_produced: segments.next_decode()?,
            level: segments.next_decode()?,
            minigame_data: segments.next_decode()?,
            muted: segments.next_decode()?,
            highest_amount: segments.next_decode()?,
        })
    }
}

impl Decode<&str> for super::Garden {
    fn decode(value: &str) -> Result<Self, Error> {
        let mut segments = value.split(' ');
        let mut segments_a = segments.next_decode::<&str>()?.split(':');
        let segments_b = segments.next_decode::<&str>()?.chars();
        let segments_c = segments.next_decode::<&str>()?.split(':');
        Ok(Self {
            time_of_next_tick: segments_a.next_decode()?,
            soil_type: segments_a.next_decode()?,
            time_of_next_soil_change: segments_a.next_decode()?,
            frozen_garden: segments_a.next_decode()?,
            harvests_this_ascension: segments_a.next_decode()?,
            total_harvests: segments_a.next_decode()?,
            unlocked_seeds: segments_b
                .map(|value| Decode::decode(value))
                .collect::<Result<Vec<_>, _>>()?,
            farm_grid_data: segments_c
                .tuples()
                .map(|(id, age)| {
                    let id = Decode::decode(id)?;
                    let age = Decode::decode(age)?;
                    if id == 0 {
                        Ok(None)
                    } else {
                        Ok(Some((id, age)))
                    }
                })
                .collect::<Result<Vec<_>, Error>>()?,
        })
    }
}
