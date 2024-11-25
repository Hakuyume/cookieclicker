// https://cookieclicker.fandom.com/wiki/Save

mod error;
mod format;
mod garden;
mod upgrades;

use chrono::{DateTime, Utc};
pub use error::Error;
pub use format::{decode, encode};
pub use garden::{FarmGridData, Garden};
use serde::{Deserialize, Serialize};
pub use upgrades::Upgrade;

#[allow(clippy::manual_non_exhaustive)]
#[derive(Clone, Debug, Deserialize, Serialize, format::Decode, format::Encode)]
#[format(split = '|')]
pub struct Save {
    pub game_version: GameVersion,
    #[format(skip = 1)]
    pub run_details: RunDetails,
    pub preferences: Preferences,
    pub miscellaneous_game_data: MiscellaneousGameData,
    pub building_data: BuildingData,
    #[format(as = upgrades::Custom)]
    pub upgrades: Vec<Upgrade>,
}

#[derive(Clone, Debug, Deserialize, Serialize, format::Decode, format::Encode)]
#[format(split = ';')]
pub struct GameVersion {
    pub game_version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, format::Decode, format::Encode)]
#[format(split = ';')]
pub struct RunDetails {
    #[format(as = format::Timestamp)]
    pub ascension_start: DateTime<Utc>,
    #[format(as = format::Timestamp)]
    pub legacy_start: DateTime<Utc>,
    #[format(as = format::Timestamp)]
    pub last_opened: DateTime<Utc>,
    pub bakery_name: String,
    pub seed: String,
    pub you_appearance: YouAppearance,
}

#[derive(Clone, Debug, Deserialize, Serialize, format::Decode, format::Encode)]
#[format(split = ',')]
pub struct YouAppearance {
    pub hair: usize,
    pub hair_color: usize,
    pub skin_color: usize,
    pub head_shape: usize,
    pub face: usize,
    pub extra_a: usize,
    pub extra_b: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, format::Decode, format::Encode)]
pub struct Preferences {
    pub particles: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, format::Decode, format::Encode)]
#[format(split = ';')]
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
    #[format(as = format::NoneAsNegative)]
    pub time_left_in_research: Option<u64>,
    pub ascensions: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, format::Decode, format::Encode)]
#[format(split = ';')]
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

#[derive(Clone, Debug, Deserialize, Serialize, format::Decode, format::Encode)]
#[format(split = ',')]
pub struct BuildingDataEntry<M = ()> {
    pub amount_owned: u64,
    pub amount_bought: u64,
    pub cookies_produced: f64,
    pub level: usize,
    #[format(as = format::NoneAsEmpty)]
    pub minigame_data: Option<M>,
    pub muted: bool,
    pub highest_amount: u64,
}
