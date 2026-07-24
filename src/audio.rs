//! Chiptune SFX synthesized at startup — no asset files, pure lo-fi.

use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};

const RATE: u32 = 22050;

pub struct Sfx {
    swing: Option<Sound>,
    hit: Option<Sound>,
    kill: Option<Sound>,
    hurt: Option<Sound>,
    pickup: Option<Sound>,
    levelup: Option<Sound>,
    cast: Option<Sound>,
    chest: Option<Sound>,
    rest: Option<Sound>,
    ui: Option<Sound>,
}

fn wav_bytes(samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

/// Square-wave sweep from f0 to f1 over dur seconds with exponential decay.
fn sweep(f0: f32, f1: f32, dur: f32, vol: f32) -> Vec<i16> {
    let n = (RATE as f32 * dur) as usize;
    let mut phase = 0.0f32;
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let f = f0 + (f1 - f0) * t;
            phase += f / RATE as f32;
            let sq = if phase.fract() < 0.5 { 1.0 } else { -1.0 };
            let env = (1.0 - t).powf(1.5);
            (sq * env * vol * i16::MAX as f32 * 0.5) as i16
        })
        .collect()
}

/// White-noise burst with decay (impacts, whooshes).
fn noise(dur: f32, vol: f32) -> Vec<i16> {
    let n = (RATE as f32 * dur) as usize;
    let mut seed = 0x12345678u32;
    (0..n)
        .map(|i| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let r = (seed >> 16) as i16 as f32 / i16::MAX as f32;
            let t = i as f32 / n as f32;
            let env = (1.0 - t).powf(2.0);
            (r * env * vol * i16::MAX as f32 * 0.5) as i16
        })
        .collect()
}

fn mix(parts: &[Vec<i16>]) -> Vec<i16> {
    let len = parts.iter().map(|p| p.len()).max().unwrap_or(0);
    (0..len)
        .map(|i| {
            let sum: i32 = parts.iter().map(|p| *p.get(i).unwrap_or(&0) as i32).sum();
            sum.clamp(i16::MIN as i32, i16::MAX as i32) as i16
        })
        .collect()
}

fn chain(parts: &[Vec<i16>]) -> Vec<i16> {
    parts.concat()
}

async fn load(samples: Vec<i16>) -> Option<Sound> {
    load_sound_from_bytes(&wav_bytes(&samples)).await.ok()
}

impl Sfx {
    pub async fn load_all() -> Sfx {
        Sfx {
            swing: load(noise(0.08, 0.35)).await,
            hit: load(mix(&[noise(0.07, 0.6), sweep(120.0, 70.0, 0.09, 0.5)])).await,
            kill: load(mix(&[noise(0.12, 0.5), sweep(320.0, 60.0, 0.28, 0.6)])).await,
            hurt: load(sweep(200.0, 55.0, 0.2, 0.55)).await,
            pickup: load(sweep(420.0, 880.0, 0.09, 0.4)).await,
            levelup: load(chain(&[
                sweep(392.0, 392.0, 0.09, 0.4),
                sweep(494.0, 494.0, 0.09, 0.4),
                sweep(587.0, 587.0, 0.16, 0.45),
            ]))
            .await,
            cast: load(sweep(220.0, 950.0, 0.16, 0.45)).await,
            chest: load(chain(&[sweep(520.0, 520.0, 0.07, 0.4), sweep(700.0, 700.0, 0.12, 0.4)])).await,
            rest: load(chain(&[sweep(330.0, 330.0, 0.2, 0.25), sweep(440.0, 440.0, 0.3, 0.25)])).await,
            ui: load(sweep(600.0, 600.0, 0.05, 0.3)).await,
        }
    }

    fn play(s: &Option<Sound>, volume: f32) {
        if let Some(s) = s {
            play_sound(s, PlaySoundParams { looped: false, volume });
        }
    }

    pub fn swing(&self) { Self::play(&self.swing, 0.5); }
    pub fn hit(&self) { Self::play(&self.hit, 0.7); }
    pub fn kill(&self) { Self::play(&self.kill, 0.8); }
    pub fn hurt(&self) { Self::play(&self.hurt, 0.7); }
    pub fn pickup(&self) { Self::play(&self.pickup, 0.5); }
    pub fn levelup(&self) { Self::play(&self.levelup, 0.6); }
    pub fn cast(&self) { Self::play(&self.cast, 0.55); }
    pub fn chest(&self) { Self::play(&self.chest, 0.55); }
    pub fn rest(&self) { Self::play(&self.rest, 0.5); }
    pub fn ui(&self) { Self::play(&self.ui, 0.4); }
}
