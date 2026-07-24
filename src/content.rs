use serde::Deserialize;
use std::collections::HashMap;

// ---------- definitions (all moddable via data/*.json) ----------

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct MobDef {
    pub id: String,
    pub name: String,
    pub hp: i32,
    pub atk: i32,
    pub def: i32,
    pub speed: f32,
    pub xp: i32,
    pub gold_min: i32,
    pub gold_max: i32,
    pub tier: i32,
    pub boss: bool,
    pub lines: Vec<String>,
    pub drops: Vec<DropDef>,
    pub palette: HashMap<String, String>,
    pub sprite: Vec<String>,
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct DropDef {
    pub item: String,
    pub chance: f32,
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub kind: String, // weapon|offhand|head|chest|legs|boots|ring|potion
    pub atk: i32,
    pub def: i32,
    pub hp: i32,
    pub mp: i32,
    pub spd: i32,
    pub heal: i32,
    pub mana: i32,
    pub value: i32,
    pub tier: i32,
    pub wclass: String, // dagger|sword|hammer|spear|scythe ("" = unarmed/sword)
    pub palette: HashMap<String, String>,
    pub sprite: Vec<String>,
}

impl ItemDef {
    pub fn is_equippable(&self) -> bool {
        matches!(
            self.kind.as_str(),
            "weapon" | "offhand" | "head" | "chest" | "legs" | "boots" | "ring"
        )
    }
    pub fn is_usable(&self) -> bool {
        self.kind == "potion"
    }
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct NpcDef {
    pub id: String,
    pub name: String,
    pub shopkeeper: bool,
    pub fighter: bool,
    pub hp: i32,
    pub atk: i32,
    pub lines: Vec<String>,
    pub palette: HashMap<String, String>,
    pub sprite: Vec<String>,
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct BanterDef {
    pub a: String,
    pub b: String,
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct SpellDef {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub cost: i32,
    pub damage: i32,
    pub radius: f32,
    pub heal: i32,
    pub range: f32,
    pub unlock_level: i32,
    pub color: String,
}

#[derive(Deserialize, Clone, Copy, Default)]
#[serde(default)]
pub struct StatBlock {
    pub atk: i32,
    pub def: i32,
    pub hp: i32,
    pub mp: i32,
    pub spd: i32,
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct AffixDef {
    pub id: String,
    pub name: String,
    pub value_mult: f32,
    pub stats: StatBlock,
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct SkillDef {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub branch: usize, // column in the tree
    pub tier: i32,     // row in the tree
    pub requires: String,
    pub stat: String, // atk|def|hp|mp|spd|crit|dodge|mana_regen|gold|xp|focus
    pub amount: f32,
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct AchievementDef {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub stat: String,
    pub threshold: i64,
}

#[derive(Deserialize, Default)]
struct MobsFile {
    mobs: Vec<MobDef>,
}
#[derive(Deserialize, Default)]
struct ItemsFile {
    items: Vec<ItemDef>,
}
#[derive(Deserialize, Default)]
struct NpcsFile {
    npcs: Vec<NpcDef>,
}
#[derive(Deserialize, Default)]
struct SpellsFile {
    spells: Vec<SpellDef>,
}
#[derive(Deserialize, Default)]
struct AchievementsFile {
    achievements: Vec<AchievementDef>,
}
#[derive(Deserialize, Default)]
struct GraffitiFile {
    graffiti: Vec<String>,
}
#[derive(Deserialize, Default)]
struct BanterFile {
    banter: Vec<BanterDef>,
}
#[derive(Deserialize, Default)]
struct SkillsFile {
    skills: Vec<SkillDef>,
}
#[derive(Deserialize, Default)]
struct EmotesFile {
    npc: Vec<String>,
    mob: Vec<String>,
}
#[derive(Deserialize, Default)]
struct AffixesFile {
    prefixes: Vec<AffixDef>,
    suffixes: Vec<AffixDef>,
}

pub struct Content {
    pub mobs: Vec<MobDef>,
    pub items: Vec<ItemDef>,
    pub npcs: Vec<NpcDef>,
    pub spells: Vec<SpellDef>,
    pub achievements: Vec<AchievementDef>,
    pub graffiti: Vec<String>,
    pub banter: Vec<BanterDef>,
    pub skills: Vec<SkillDef>,
    pub emotes_npc: Vec<String>,
    pub emotes_mob: Vec<String>,
    pub prefixes: Vec<AffixDef>,
    pub suffixes: Vec<AffixDef>,
}

impl Content {
    pub fn item(&self, id: &str) -> Option<&ItemDef> {
        self.items.iter().find(|i| i.id == id)
    }
    pub fn prefix(&self, id: &str) -> Option<&AffixDef> {
        self.prefixes.iter().find(|a| a.id == id)
    }
    pub fn suffix(&self, id: &str) -> Option<&AffixDef> {
        self.suffixes.iter().find(|a| a.id == id)
    }
}

// Embedded defaults so the game always runs; files on disk (or served over
// http for the wasm build) override them — that's the modding hook.
const DEFAULT_MOBS: &str = include_str!("../data/mobs.json");
const DEFAULT_ITEMS: &str = include_str!("../data/items.json");
const DEFAULT_NPCS: &str = include_str!("../data/npcs.json");
const DEFAULT_SPELLS: &str = include_str!("../data/spells.json");
const DEFAULT_ACHIEVEMENTS: &str = include_str!("../data/achievements.json");
const DEFAULT_GRAFFITI: &str = include_str!("../data/graffiti.json");
const DEFAULT_BANTER: &str = include_str!("../data/banter.json");
const DEFAULT_SKILLS: &str = include_str!("../data/skills.json");
const DEFAULT_EMOTES: &str = include_str!("../data/emotes.json");
const DEFAULT_AFFIXES: &str = include_str!("../data/affixes.json");

async fn load_or(path: &str, fallback: &str) -> String {
    match macroquad::file::load_string(path).await {
        Ok(s) => s,
        Err(_) => fallback.to_string(),
    }
}

fn parse<T: for<'de> Deserialize<'de> + Default>(src: &str, fallback: &str, what: &str) -> T {
    match serde_json::from_str::<T>(src) {
        Ok(v) => v,
        Err(e) => {
            macroquad::logging::warn!("bad {} json ({}), using defaults", what, e);
            serde_json::from_str::<T>(fallback).unwrap_or_default()
        }
    }
}

pub async fn load_content() -> Content {
    let mobs_s = load_or("data/mobs.json", DEFAULT_MOBS).await;
    let items_s = load_or("data/items.json", DEFAULT_ITEMS).await;
    let npcs_s = load_or("data/npcs.json", DEFAULT_NPCS).await;
    let spells_s = load_or("data/spells.json", DEFAULT_SPELLS).await;
    let ach_s = load_or("data/achievements.json", DEFAULT_ACHIEVEMENTS).await;
    let graf_s = load_or("data/graffiti.json", DEFAULT_GRAFFITI).await;
    let banter_s = load_or("data/banter.json", DEFAULT_BANTER).await;
    let skills_s = load_or("data/skills.json", DEFAULT_SKILLS).await;
    let emotes_s = load_or("data/emotes.json", DEFAULT_EMOTES).await;
    let affixes_s = load_or("data/affixes.json", DEFAULT_AFFIXES).await;

    let mobs: MobsFile = parse(&mobs_s, DEFAULT_MOBS, "mobs");
    let items: ItemsFile = parse(&items_s, DEFAULT_ITEMS, "items");
    let npcs: NpcsFile = parse(&npcs_s, DEFAULT_NPCS, "npcs");
    let spells: SpellsFile = parse(&spells_s, DEFAULT_SPELLS, "spells");
    let ach: AchievementsFile = parse(&ach_s, DEFAULT_ACHIEVEMENTS, "achievements");
    let graf: GraffitiFile = parse(&graf_s, DEFAULT_GRAFFITI, "graffiti");
    let banter: BanterFile = parse(&banter_s, DEFAULT_BANTER, "banter");
    let skills: SkillsFile = parse(&skills_s, DEFAULT_SKILLS, "skills");
    let emotes: EmotesFile = parse(&emotes_s, DEFAULT_EMOTES, "emotes");
    let affixes: AffixesFile = parse(&affixes_s, DEFAULT_AFFIXES, "affixes");

    Content {
        mobs: mobs.mobs,
        items: items.items,
        npcs: npcs.npcs,
        spells: spells.spells,
        achievements: ach.achievements,
        graffiti: graf.graffiti,
        banter: banter.banter,
        skills: skills.skills,
        emotes_npc: emotes.npc,
        emotes_mob: emotes.mob,
        prefixes: affixes.prefixes,
        suffixes: affixes.suffixes,
    }
}
