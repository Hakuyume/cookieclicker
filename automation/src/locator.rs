use fantoccini::Locator;
use std::sync::OnceLock;
use strum::VariantArray;

pub const PROMPT_TEXTAREA: Locator = Locator::Id("textareaPrompt");
pub const PROMPT_OPTION0: Locator = Locator::Id("promptOption0");

pub const LANG_SELECT_ENGLISH: Locator = Locator::Id("langSelect-EN");

pub const BIG_COOKIE: Locator = Locator::Id("bigCookie");
pub const MENU_CLOSE: Locator = Locator::Css("#menu > .menuClose");

pub const OPTIONS: Locator = Locator::Id("prefsButton");
pub const IMPORT_SAVE: Locator = Locator::LinkText("Import save");
pub const IMPORT_SAVE_TEXT: Locator = PROMPT_TEXTAREA;
pub const IMPORT_SAVE_LOAD: Locator = PROMPT_OPTION0;

pub const STORE_BUY_ALL_UPGRADES: Locator = Locator::Id("storeBuyAllButton");
pub const STORE_BUIK10: Locator = Locator::Id("storeBulk10");

#[derive(Clone, Copy, Debug, strum::VariantArray)]
pub enum Building {
    Cursor,
    Grandmas,
    Farms,
    Mines,
    Factory,
    Bank,
    Temple,
    WizardTower,
    Shipment,
    AlchemyLab,
    Portal,
    TimeMachine,
    AntimatterCondenser,
    Prism,
    Chancemaker,
    FractalEngine,
    JavascriptConsole,
    Idleverse,
    CortexBaker,
    You,
}
pub fn store_building(building: Building) -> Locator<'static> {
    static IDS: OnceLock<Vec<String>> = OnceLock::new();
    let ids = IDS.get_or_init(|| {
        (0..Building::VARIANTS.len())
            .map(|i| format!("product{i}"))
            .collect()
    });
    Locator::Id(&ids[building as usize])
}
