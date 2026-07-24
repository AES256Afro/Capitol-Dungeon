//! Persistence: JSON files on desktop, localStorage in the browser
//! (via a tiny JS plugin registered in web/index.html).

use crate::content::Content;
use crate::world::{ItemInst, ItemStack, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct RunSave {
    pub depth: i32,
    pub hp: i32,
    pub mp: i32,
    pub base_maxhp: i32,
    pub base_maxmp: i32,
    pub base_atk: i32,
    pub base_def: i32,
    pub base_spd: f32,
    pub xp: i64,
    pub level: i32,
    pub gold: i64,
    pub skill_points: i32,
    pub skills: Vec<String>,
    // legacy pre-affix fields; still read so old saves migrate cleanly
    pub inventory: Vec<(String, i32)>,
    pub equipment: Vec<(String, String)>,
    // current format: (item id, prefix, suffix, qty) / (slot, id, prefix, suffix)
    pub inventory2: Vec<(String, Option<String>, Option<String>, i32)>,
    pub equipment2: Vec<(String, String, Option<String>, Option<String>)>,
    pub quests_active: Vec<(String, i64)>,
    pub quests_done: Vec<String>,
    pub commune: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct SaveData {
    pub stats: HashMap<String, i64>,
    pub unlocked: Vec<String>,
    pub run: Option<RunSave>,
}

// ---------- storage backends ----------

#[cfg(not(target_arch = "wasm32"))]
pub fn store_raw(key: &str, val: &str) {
    let _ = std::fs::create_dir_all("data");
    let _ = std::fs::write(format!("data/{}.json", key), val);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_raw(key: &str) -> Option<String> {
    std::fs::read_to_string(format!("data/{}.json", key)).ok()
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn cd_save(key_ptr: *const u8, key_len: u32, val_ptr: *const u8, val_len: u32);
    fn cd_load_len(key_ptr: *const u8, key_len: u32) -> u32;
    fn cd_load_copy(dst_ptr: *mut u8, cap: u32);
}

#[cfg(target_arch = "wasm32")]
pub fn store_raw(key: &str, val: &str) {
    unsafe {
        cd_save(
            key.as_ptr(),
            key.len() as u32,
            val.as_ptr(),
            val.len() as u32,
        );
    }
}

#[cfg(target_arch = "wasm32")]
pub fn fetch_raw(key: &str) -> Option<String> {
    unsafe {
        let n = cd_load_len(key.as_ptr(), key.len() as u32);
        if n == 0 {
            return None;
        }
        let mut buf = vec![0u8; n as usize];
        cd_load_copy(buf.as_mut_ptr(), n);
        String::from_utf8(buf).ok()
    }
}

// ---------- game save ----------

const SAVE_KEY: &str = "save";

pub fn snapshot(world: &World, include_run: bool) -> SaveData {
    SaveData {
        stats: world.stats.clone(),
        unlocked: world.unlocked.iter().cloned().collect(),
        run: if include_run {
            Some(RunSave {
                depth: world.depth,
                hp: world.player.hp,
                mp: world.player.mp,
                base_maxhp: world.player.base_maxhp,
                base_maxmp: world.player.base_maxmp,
                base_atk: world.player.base_atk,
                base_def: world.player.base_def,
                base_spd: world.player.base_spd,
                xp: world.player.xp,
                level: world.player.level,
                gold: world.player.gold,
                skill_points: world.player.skill_points,
                skills: world.player.skills.clone(),
                inventory: Vec::new(),
                equipment: Vec::new(),
                inventory2: world
                    .player
                    .inventory
                    .iter()
                    .map(|s| (s.inst.id.clone(), s.inst.prefix.clone(), s.inst.suffix.clone(), s.qty))
                    .collect(),
                equipment2: world
                    .player
                    .equipment
                    .iter()
                    .map(|(k, v)| (k.clone(), v.id.clone(), v.prefix.clone(), v.suffix.clone()))
                    .collect(),
                quests_active: world.quests_active.clone(),
                quests_done: world.quests_done.iter().cloned().collect(),
                commune: world.commune.clone(),
            })
        } else {
            None
        },
    }
}

pub fn write(sd: &SaveData) {
    if let Ok(json) = serde_json::to_string(sd) {
        store_raw(SAVE_KEY, &json);
    }
}

pub fn read() -> Option<SaveData> {
    let raw = fetch_raw(SAVE_KEY)?;
    serde_json::from_str(&raw).ok()
}

/// Restore profile (always) and, when present, the run: the player resumes at
/// the start of their saved depth with gear intact — roguelite checkpointing.
pub fn apply(world: &mut World, content: &Content, sd: &SaveData) -> bool {
    world.stats = sd.stats.clone();
    world.unlocked = sd.unlocked.iter().cloned().collect();
    let Some(run) = &sd.run else { return false };
    world.player.hp = run.hp.max(1);
    world.player.mp = run.mp;
    world.player.base_maxhp = run.base_maxhp.max(10);
    world.player.base_maxmp = run.base_maxmp.max(5);
    world.player.base_atk = run.base_atk.max(1);
    world.player.base_def = run.base_def.max(0);
    world.player.base_spd = if run.base_spd > 10.0 { run.base_spd } else { 92.0 };
    world.player.xp = run.xp.max(0);
    world.player.level = run.level.max(1);
    world.player.gold = run.gold.max(0);
    world.player.skill_points = run.skill_points.max(0);
    world.player.skills = run.skills.clone();
    // saves from before the skill system existed: grant the points they earned
    if world.player.skills.is_empty() && run.skill_points == 0 {
        world.player.skill_points = (run.level - 1).max(0);
    }
    if !run.inventory2.is_empty() || !run.equipment2.is_empty() {
        world.player.inventory = run
            .inventory2
            .iter()
            .map(|(id, pre, suf, qty)| ItemStack {
                inst: ItemInst { id: id.clone(), prefix: pre.clone(), suffix: suf.clone() },
                qty: *qty,
            })
            .collect();
        world.player.equipment = run
            .equipment2
            .iter()
            .map(|(slot, id, pre, suf)| {
                (slot.clone(), ItemInst { id: id.clone(), prefix: pre.clone(), suffix: suf.clone() })
            })
            .collect();
    } else {
        // migrate a pre-affix save
        world.player.inventory = run
            .inventory
            .iter()
            .map(|(id, qty)| ItemStack { inst: ItemInst::plain(id), qty: *qty })
            .collect();
        world.player.equipment = run
            .equipment
            .iter()
            .map(|(slot, id)| (slot.clone(), ItemInst::plain(id)))
            .collect();
    }
    world.quests_active = run.quests_active.clone();
    world.quests_done = run.quests_done.iter().cloned().collect();
    world.commune = run.commune.clone();
    world.depth = run.depth.max(1);
    world.load_level(content, None);
    true
}
