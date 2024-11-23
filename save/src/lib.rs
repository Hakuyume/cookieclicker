// https://cookieclicker.fandom.com/wiki/Save

mod decode;

pub use decode::decode;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    Float(#[from] std::num::ParseFloatError),
    #[error(transparent)]
    Int(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("cannot parse bool")]
    Bool,
    #[error("insufficient data")]
    InsufficientData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Save {
    pub game_version: GameVersion,
    pub run_details: RunDetails,
    pub preferences: Preferences,
    pub miscellaneous_game_data: MiscellaneousGameData,
    pub building_data: BuildingData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameVersion {
    pub game_version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunDetails {
    pub ascension_start: u64,
    pub legacy_start: u64,
    pub last_opened: u64,
    pub bakery_name: String,
    pub seed: String,
    pub you_appearance: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Preferences {
    pub particles: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MiscellaneousGameData {
    pub cookies_in_bank: f64,
    pub cookies_baked: f64,
    pub cookie_clicks: u64,
    pub total_golden_cookie_clicks: u64,
    pub hand_made_cookies: f64,
    pub total_golden_cookies_missed: u64,
    pub background_type: usize,
    pub milk_type: usize,
    pub cookies_forfeited_by_ascending: f64,
    pub grandmapocalypse_stage: usize,
    pub elder_pledges_made: u64,
    pub time_left_in_elder_pledge: u64,
    pub currently_researching: usize,
    pub time_left_in_research: u64,
    pub ascensions: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildingData {
    pub cursors: BuildingDataEntry,
    pub grandmas: BuildingDataEntry,
    pub farms: BuildingDataEntry<Garden>,
    pub mines: BuildingDataEntry,
    pub factories: BuildingDataEntry,
    pub banks: BuildingDataEntry,
    pub temples: BuildingDataEntry,
    pub wizard_towers: BuildingDataEntry,
    pub shipments: BuildingDataEntry,
    pub alchemy_labs: BuildingDataEntry,
    pub portals: BuildingDataEntry,
    pub time_machines: BuildingDataEntry,
    pub antimatter_condensers: BuildingDataEntry,
    pub prisms: BuildingDataEntry,
    pub chancemakers: BuildingDataEntry,
    pub fractal_engines: BuildingDataEntry,
    pub javascript_consoles: BuildingDataEntry,
    pub idleverses: BuildingDataEntry,
    pub cortex_bakers: BuildingDataEntry,
    pub yous: BuildingDataEntry,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildingDataEntry<M = ()> {
    pub amount_owned: u64,
    pub amount_bought: u64,
    pub cookies_produced: f64,
    pub level: usize,
    pub minigame_data: M,
    pub muted: bool,
    pub highest_amount: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Garden {
    pub time_of_next_tick: u64,
    pub soil_type: usize,
    pub time_of_next_soil_change: u64,
    pub frozen_garden: bool,
    pub harvests_this_ascension: u64,
    pub total_harvests: u64,
    pub unlocked_seeds: Vec<bool>,
    pub farm_grid_data: Vec<Option<(usize, u8)>>,
}
