//! Shared synthetic fixtures for the integration-test layer.
//!
//! Behaviour/logic (Category 1) and round-trip/invariant (Category 2) tests
//! must not depend on the *contents* of the real committed fixtures — the
//! character files under `TestData/`, the canonical data files under `data/`,
//! or `docs/lgo_TEMPLATE_gearReady.toml`. Updating any of those for legitimate
//! reasons should never break a test that is only checking install-layout
//! routing, CLI behaviour, parsing, ordering, or reporting logic.
//!
//! These helpers let those tests own **minimal synthetic** inputs. The numbers
//! here are the tests' own — they are deliberately not "the real game data".
//! Only the small set of Category-3 ground-truth tests keeps reading the real
//! fixtures.

#![allow(dead_code)]

use std::path::Path;

/// Minimal synthetic base-stat derivation table. Every class the loader
/// insists on is present, but only Lore-master carries non-empty coefficient
/// rows — the only class the logic tests exercise. Tests own these numbers.
pub const DERIVATIONS_JSON: &str = r#"{
  "Beorning": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
  "Brawler": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
  "Burglar": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
  "Captain": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
  "Champion": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
  "Guardian": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
  "Hunter": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
  "Lore-master": {
    "Might": { "CriticalRating": 1.5, "Finesse": 1.5, "TacticalMastery": 2.0, "Parry": 1.0 },
    "Agility": { "CriticalRating": 2.0, "Finesse": 1.0, "TacticalMastery": 2.0, "Evade": 1.0 },
    "Vitality": { "Morale": 4.5 },
    "Will": { "CriticalRating": 1.0, "TacticalMastery": 3.0, "Resistance": 1.0, "Evade": 2.0, "PhysicalMitigation": 1.0, "TacticalMitigation": 1.0 },
    "Fate": { "Power": 1.5 }
  },
  "Mariner": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
  "Minstrel": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
  "Rune-keeper": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} },
  "Warden": { "Might": {}, "Agility": {}, "Vitality": {}, "Will": {}, "Fate": {} }
}"#;

/// Minimal synthetic items DB. It maps only the handful of `Test *` item names
/// the layout/resolve tests reference to canonical slots — one entry per slot
/// family so divider-ordering tests still have something to order.
pub const ITEMS_JSON: &str = r#"{
  "Test Helm": { "name": "Test Helm", "slot": "Head" },
  "Test Robe": { "name": "Test Robe", "slot": "Chest" },
  "Test Leggings": { "name": "Test Leggings", "slot": "Legs" },
  "Test Gloves": { "name": "Test Gloves", "slot": "Hands" },
  "Test Boots": { "name": "Test Boots", "slot": "Feet" },
  "Test Shoulders": { "name": "Test Shoulders", "slot": "Shoulders" },
  "Test Cloak": { "name": "Test Cloak", "slot": "Back" },
  "Test Bracelet": { "name": "Test Bracelet", "slot": "Wrist" },
  "Test Necklace": { "name": "Test Necklace", "slot": "Neck" },
  "Test Ring": { "name": "Test Ring", "slot": "Finger" },
  "Test Earring": { "name": "Test Earring", "slot": "Ear" },
  "Test Trinket": { "name": "Test Trinket", "slot": "Pocket" },
  "Test Staff": { "name": "Test Staff", "slot": "Main-hand" },
  "Test Rune-stone": { "name": "Test Rune-stone", "slot": "Off-hand", "either_hand": true },
  "Test Bow": { "name": "Test Bow", "slot": "Ranged" },
  "Test Tome": { "name": "Test Tome", "slot": "Class Item" }
}"#;

/// Synthetic raw Base stats a plugindata export carries. Tests that pass these
/// through `[InnateStats]` assert against these constants — their own data —
/// rather than the real character's numbers.
pub const PLUGIN_MIGHT: i64 = 5300;
pub const PLUGIN_AGILITY: i64 = 2650;
pub const PLUGIN_VITALITY: i64 = 10200;
pub const PLUGIN_WILL: i64 = 7950;
pub const PLUGIN_FATE: i64 = 4000;

/// Write the synthetic derivation table into
/// `<data_dir>/base_stat_derivations.json`, creating `data_dir` if needed.
pub fn write_derivations(data_dir: &Path) {
    std::fs::create_dir_all(data_dir).expect("create data dir");
    std::fs::write(
        data_dir.join("base_stat_derivations.json"),
        DERIVATIONS_JSON,
    )
    .expect("write synthetic derivations");
}

/// Write the synthetic items DB into `<data_dir>/lgo_items.json`, creating
/// `data_dir` if needed.
pub fn write_items(data_dir: &Path) {
    std::fs::create_dir_all(data_dir).expect("create data dir");
    std::fs::write(data_dir.join("lgo_items.json"), ITEMS_JSON).expect("write synthetic items DB");
}

/// A synthetic plugindata (`.plugindata`) export body for `character`. Carries
/// the five raw Base stats plus the `lgo-gearlist-2` measured fields, so that
/// `resolve-slots` writes both `[InnateStats]` and `[MeasuredStats]`.
pub fn plugindata_body(character: &str) -> String {
    format!(
        "return \n{{\n\
\t[\"class\"] = \"Lore-master\",\n\
\t[\"character\"] = \"{character}\",\n\
\t[\"level\"] = 160.000000,\n\
\t[\"maxMorale\"] = 100000.000000,\n\
\t[\"maxPower\"] = 20000.000000,\n\
\t[\"activeEffects\"] = 0.000000,\n\
\t[\"equipped\"] = \n\t{{\n\t\t[1.000000] = \"Test Helm\"\n\t}},\n\
\t[\"baseStats\"] = \n\t{{\n\
\t\t[\"GetBaseMight\"] = {might}.000000,\n\
\t\t[\"GetBaseAgility\"] = {agility}.000000,\n\
\t\t[\"GetBaseVitality\"] = {vitality}.000000,\n\
\t\t[\"GetBaseWill\"] = {will}.000000,\n\
\t\t[\"GetBaseFate\"] = {fate}.000000\n\t}}\n}}\n",
        might = PLUGIN_MIGHT,
        agility = PLUGIN_AGILITY,
        vitality = PLUGIN_VITALITY,
        will = PLUGIN_WILL,
        fate = PLUGIN_FATE,
    )
}

/// A synthetic bookmarklet `gearStats.toml` export. It carries several items
/// spanning multiple canonical slot families (so divider-ordering invariants
/// have something to order), a deliberately unknown item that keeps an
/// `# UNRESOLVED:` comment (so unresolved-handling invariants have a subject),
/// and an Ear item whose slot string is the bookmarklet typo `Ears (1)` (so
/// typo-canonicalisation is exercised). The item names are the synthetic
/// `Test *` names the shared items DB knows.
pub const SYNTH_GEARSTATS: &str = r#"# LGO gear stats file — generated by bookmarklet
character          = "Thalya"
class              = "Lore-master"

[[item]]
slot               = "Unknown"
name               = "Test Helm"
Armour                = 6000
CriticalRating        = 26000

[[item]]
slot               = "Unknown"
name               = "Test Robe"
Armour                = 20000
CriticalRating        = 22000

[[item]]
slot               = "Unknown"
name               = "Test Leggings"
Armour                = 15000
Finesse               = 12000

[[item]]
slot               = "Ears (1)"
name               = "Test Earring"
CriticalRating        = 9000
TacticalMastery       = 8000

[[item]]
slot               = "Unknown"
name               = "Mystery Relic"
# UNRESOLVED: legendary item — you should hand-edit stats
Armour                = 0
CriticalRating        = 0
"#;

/// The shared synthetic items DB, parsed from [`ITEMS_JSON`].
pub fn items_db() -> lgo::slot_resolver::ItemsDb {
    lgo::slot_resolver::ItemsDb::from_json_str(ITEMS_JSON, Path::new("<synthetic-items>"))
        .expect("synthetic items DB must parse")
}
