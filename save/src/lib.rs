// https://cookieclicker.fandom.com/wiki/Save

mod error;
mod escape;
mod format;
mod garden;
mod upgrades;

use chrono::{DateTime, Utc};
pub use error::Error;
use format::Format as _;
pub use garden::{FarmGridData, Garden};
use serde::{Deserialize, Serialize};
pub use upgrades::Upgrade;

#[tracing::instrument(err)]
pub fn decode(value: &str) -> Result<Save, Error> {
    format::Standard::decode(&escape::decode(value)?)
}

#[tracing::instrument]
pub fn encode(value: &Save) -> String {
    escape::encode(&format::Standard::display(value).to_string())
}

#[derive(Clone, Debug, Deserialize, Serialize, format::Format)]
#[format(split = '|')]
pub struct Save {
    pub game_version: GameVersion,
    #[serde(skip)]
    pub empty: (),
    pub run_details: RunDetails,
    pub preferences: Preferences,
    pub miscellaneous_game_data: MiscellaneousGameData,
    pub building_data: BuildingData,
    #[format(with = upgrades::Custom)]
    pub upgrades: Vec<Upgrade>,
}

#[derive(Clone, Debug, Deserialize, Serialize, format::Format)]
#[format(split = ';')]
pub struct GameVersion {
    pub game_version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, format::Format)]
#[format(split = ';')]
pub struct RunDetails {
    #[format(with = format::Timestamp)]
    pub ascension_start: DateTime<Utc>,
    #[format(with = format::Timestamp)]
    pub legacy_start: DateTime<Utc>,
    #[format(with = format::Timestamp)]
    pub last_opened: DateTime<Utc>,
    pub bakery_name: String,
    pub seed: String,
    pub you_appearance: YouAppearance,
}

#[derive(Clone, Debug, Deserialize, Serialize, format::Format)]
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

#[derive(Clone, Debug, Deserialize, Serialize, format::Format)]
pub struct Preferences {
    pub particles: bool,
    pub numbers: bool,
    pub auto_save: bool,
    pub auto_update: bool,
    pub milk: bool,
    pub faancy_graphics: bool,
    pub closing_warning: bool,
    pub cursors: bool,
    pub defocus: bool,
    pub short_numbers: bool,
    pub fast_notes: bool,
    pub wobbly_cookie: bool,
    pub alt_font: bool,
    pub css_filters: bool,
    pub alt_cookie_sound: bool,
    pub icon_crates: bool,
    pub backup_warning: bool,
    pub extra_buttons: bool,
    pub lump_confirmation: bool,
    pub custom_grandmas: bool,
    pub sleep_mode: bool,
    pub enable_cloud_saving: bool,
    pub music_in_background: bool,
    pub scary_stuff_off: bool,
    pub fullscreen: bool,
    pub screan_reader: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, format::Format)]
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
    #[format(with = format::NoneAsNegative)]
    pub time_left_in_research: Option<u64>,
    pub ascensions: u64,
    pub golden_cookie_clicks: u64,
    pub cookies_sucked_by_wrinklers: f64,
    pub wrinklers_popped: u64,
    pub santa_level: usize,
    pub reindeer_clicked: u64,
    #[format(with = format::NoneAsNegative)]
    pub time_left_in_season: Option<u64>,
    pub season_switcher_uses: u64,
    #[format(with = format::NoneAsEmpty)]
    pub current_season: Option<String>,
    pub cookies_contained_in_wrinklers: f64,
    pub number_of_wrinklers: u64,
    pub prestive_level: f64,
    pub heavenly_chips: f64,
    pub heavenly_chips_spent: f64,
    heavenly_cookies: String,
    pub ascension_mode: bool,
    #[format(with = format::NoneAsNegative)]
    pub permanent_upgrade_i: Option<usize>,
    #[format(with = format::NoneAsNegative)]
    pub permanent_upgrade_ii: Option<usize>,
    #[format(with = format::NoneAsNegative)]
    pub permanent_upgrade_iii: Option<usize>,
    #[format(with = format::NoneAsNegative)]
    pub permanent_upgrade_iv: Option<usize>,
    #[format(with = format::NoneAsNegative)]
    pub permanent_upgrade_v: Option<usize>,
    pub dragon_level: usize,
    pub dragon_aura: usize,
    pub dragon_aura_2: usize,
    pub golden_cookie_chime_type: usize,
    pub volume: u64,
    pub number_of_shiny_wrinklers: u64,
    pub cookies_in_shiny_wrinklers: f64,
    pub sugar_lumps: u64,
    pub total_sugar_lumps_made: u64,
    #[format(with = format::Timestamp)]
    pub time_of_start_of_sugar_lump: DateTime<Utc>,
    #[format(with = format::Timestamp)]
    pub time_of_last_minigame_refill: DateTime<Utc>,
    pub sugar_lump_type: usize,
    pub upgrades_in_vault: String,
    pub heralds: u64,
    #[serde(skip)]
    pub to_do_0: (),
    #[serde(skip)]
    pub to_do_1: (),
    #[serde(skip)]
    pub to_do_2: (),
    pub music_volume: u64,
    pub cookies_sent: f64,
    pub cookies_received: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, format::Format)]
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

#[derive(Clone, Debug, Deserialize, Serialize, format::Format)]
#[format(split = ',')]
pub struct BuildingDataEntry<M = ()> {
    pub amount_owned: u64,
    pub amount_bought: u64,
    pub cookies_produced: f64,
    pub level: usize,
    #[format(with = format::NoneAsEmpty)]
    pub minigame_data: Option<M>,
    pub muted: bool,
    pub highest_amount: u64,
}

#[cfg(test)]
mod tests {
    use crate::{escape, format};

    #[test]
    #[tracing_test::traced_test]
    fn test_save() {
        let value = escape::decode(include_str!("samples/00.txt")).unwrap();
        <format::Standard as format::Format<'_, super::Save>>::check_inverse(&value).unwrap();
    }
}
