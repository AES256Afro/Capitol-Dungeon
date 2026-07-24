//! Global, persisted player settings (volume, screen shake, music).

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static VOLUME_PCT: AtomicU32 = AtomicU32::new(100);
static SHAKE_PCT: AtomicU32 = AtomicU32::new(100);
static MUSIC_ON: AtomicBool = AtomicBool::new(true);

pub fn volume() -> f32 {
    VOLUME_PCT.load(Ordering::Relaxed) as f32 / 100.0
}
pub fn volume_pct() -> u32 {
    VOLUME_PCT.load(Ordering::Relaxed)
}
pub fn set_volume_pct(v: u32) {
    VOLUME_PCT.store(v.min(200), Ordering::Relaxed);
}
pub fn shake() -> f32 {
    SHAKE_PCT.load(Ordering::Relaxed) as f32 / 100.0
}
pub fn shake_pct() -> u32 {
    SHAKE_PCT.load(Ordering::Relaxed)
}
pub fn set_shake_pct(v: u32) {
    SHAKE_PCT.store(v.min(200), Ordering::Relaxed);
}
pub fn music_on() -> bool {
    MUSIC_ON.load(Ordering::Relaxed)
}
pub fn set_music_on(v: bool) {
    MUSIC_ON.store(v, Ordering::Relaxed);
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct SettingsFile {
    volume: u32,
    shake: u32,
    music: bool,
}

pub fn save() {
    let sf = SettingsFile {
        volume: volume_pct(),
        shake: shake_pct(),
        music: music_on(),
    };
    if let Ok(json) = serde_json::to_string(&sf) {
        crate::save::store_raw("settings", &json);
    }
}

pub fn load() {
    if let Some(raw) = crate::save::fetch_raw("settings") {
        if let Ok(sf) = serde_json::from_str::<SettingsFile>(&raw) {
            set_volume_pct(sf.volume.clamp(0, 200));
            set_shake_pct(sf.shake.clamp(0, 200));
            set_music_on(sf.music);
        }
    }
}
