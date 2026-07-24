//! Character editor: hairstyle + palette customization for the player sprite.

use crate::sprites;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const HAIR_COLORS: [(&str, &str); 8] = [
    ("Chestnut", "#6a3a1a"),
    ("Black", "#241a14"),
    ("Blond", "#d8b04a"),
    ("Red", "#a03a20"),
    ("Silver", "#b8b8c0"),
    ("Blue", "#3a5a9a"),
    ("Green", "#3a7a4a"),
    ("Pink", "#c05a8a"),
];

pub const SKIN_TONES: [(&str, &str); 8] = [
    ("Tan", "#d9a97a"),
    ("Deep", "#7a4a2a"),
    ("Rich", "#5c3a20"),
    ("Warm", "#b8825a"),
    ("Olive", "#c8a070"),
    ("Fair", "#ecc8a0"),
    ("Cool", "#a87858"),
    ("Umber", "#8a5a38"),
];

pub const SHIRT_COLORS: [(&str, &str); 8] = [
    ("Rebel Red", "#8a2a3a"),
    ("Forest", "#2a6a3a"),
    ("Union Blue", "#2a4a8a"),
    ("Plum", "#5a2a6a"),
    ("Rust", "#8a4a20"),
    ("Charcoal", "#3a3a42"),
    ("Teal", "#20686a"),
    ("Mustard", "#9a7a20"),
];

pub const PANTS_COLORS: [(&str, &str); 8] = [
    ("Denim", "#3a4a6a"),
    ("Brown", "#5c4630"),
    ("Black", "#26262c"),
    ("Olive", "#4a5230"),
    ("Gray", "#5a5a64"),
    ("Wine", "#5c2a34"),
    ("Navy", "#242e50"),
    ("Sand", "#8a7a54"),
];

pub const STYLES: [&str; 3] = ["Classic", "Long", "Mohawk"];

const SPRITE_CLASSIC: [&str; 12] = [
    "...hhhhhh...",
    "..hhhhhhhh..",
    "..haaaaaah..",
    "..haeaaeah..",
    "...aaaaaa...",
    "....aaaa....",
    "..rcccccrc..",
    ".c.cccccc.c.",
    "...cccccc...",
    "...bbbbbb...",
    "...bb..bb...",
    "..sbb..bbs..",
];

const SPRITE_LONG: [&str; 12] = [
    "...hhhhhh...",
    "..hhhhhhhh..",
    ".hhaaaaaahh.",
    ".hhaeaaeahh.",
    ".hhaaaaaahh.",
    ".h..aaaa..h.",
    "..rcccccrc..",
    ".c.cccccc.c.",
    "...cccccc...",
    "...bbbbbb...",
    "...bb..bb...",
    "..sbb..bbs..",
];

const SPRITE_MOHAWK: [&str; 12] = [
    ".....hh.....",
    ".....hh.....",
    "..aahhhhaa..",
    "..haeaaeah..",
    "...aaaaaa...",
    "....aaaa....",
    "..rcccccrc..",
    ".c.cccccc.c.",
    "...cccccc...",
    "...bbbbbb...",
    "...bb..bb...",
    "..sbb..bbs..",
];

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
#[serde(default)]
pub struct PlayerLook {
    pub style: usize,
    pub hair: usize,
    pub skin: usize,
    pub shirt: usize,
    pub pants: usize,
}

impl PlayerLook {
    pub fn sprite_rows(&self) -> Vec<String> {
        let rows = match self.style % STYLES.len() {
            1 => &SPRITE_LONG,
            2 => &SPRITE_MOHAWK,
            _ => &SPRITE_CLASSIC,
        };
        sprites::strs_to_rows(rows)
    }

    pub fn palette(&self) -> HashMap<String, String> {
        [
            ("h", HAIR_COLORS[self.hair % HAIR_COLORS.len()].1),
            ("a", SKIN_TONES[self.skin % SKIN_TONES.len()].1),
            ("e", "#221a2a"),
            ("c", SHIRT_COLORS[self.shirt % SHIRT_COLORS.len()].1),
            ("r", "#cc2233"),
            ("b", PANTS_COLORS[self.pants % PANTS_COLORS.len()].1),
            ("s", "#2a2a2a"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }

    pub fn build_texture(&self) -> Texture2D {
        sprites::build_texture(&self.sprite_rows(), &self.palette())
    }
}

pub fn save(look: &PlayerLook) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::create_dir_all("data");
        let _ = std::fs::write(
            "data/player_look.json",
            serde_json::to_string_pretty(look).unwrap_or_default(),
        );
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = look;
    }
}

pub async fn load() -> PlayerLook {
    match macroquad::file::load_string("data/player_look.json").await {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => PlayerLook::default(),
    }
}

const ROWS: usize = 5;

fn row_label(look: &PlayerLook, row: usize) -> (&'static str, &'static str) {
    match row {
        0 => ("Hairstyle", STYLES[look.style % STYLES.len()]),
        1 => ("Hair", HAIR_COLORS[look.hair % HAIR_COLORS.len()].0),
        2 => ("Skin", SKIN_TONES[look.skin % SKIN_TONES.len()].0),
        3 => ("Shirt", SHIRT_COLORS[look.shirt % SHIRT_COLORS.len()].0),
        _ => ("Pants", PANTS_COLORS[look.pants % PANTS_COLORS.len()].0),
    }
}

fn cycle(look: &mut PlayerLook, row: usize, dir: i32) {
    fn step(v: usize, n: usize, dir: i32) -> usize {
        ((v as i32 + dir).rem_euclid(n as i32)) as usize
    }
    match row {
        0 => look.style = step(look.style, STYLES.len(), dir),
        1 => look.hair = step(look.hair, HAIR_COLORS.len(), dir),
        2 => look.skin = step(look.skin, SKIN_TONES.len(), dir),
        3 => look.shirt = step(look.shirt, SHIRT_COLORS.len(), dir),
        _ => look.pants = step(look.pants, PANTS_COLORS.len(), dir),
    }
}

fn row_rect(row: usize) -> (f32, f32, f32, f32) {
    let w = 420.0_f32.min(screen_width() - 40.0);
    let x = screen_width() / 2.0 - w / 2.0 + 110.0;
    let y = 150.0 + row as f32 * 56.0;
    (x, y, w - 110.0, 46.0)
}

/// Returns true when the look changed (caller rebuilds the player texture).
pub fn update(look: &mut PlayerLook, sel: &mut usize, taps: &[Vec2]) -> bool {
    let mut changed = false;
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        *sel = sel.checked_sub(1).unwrap_or(ROWS - 1);
    }
    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        *sel = (*sel + 1) % ROWS;
    }
    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
        cycle(look, *sel, -1);
        changed = true;
    }
    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
        cycle(look, *sel, 1);
        changed = true;
    }
    for t in taps {
        for row in 0..ROWS {
            let (x, y, w, h) = row_rect(row);
            if t.x >= x && t.x <= x + w && t.y >= y && t.y <= y + h {
                *sel = row;
                cycle(look, row, if t.x < x + w / 2.0 { -1 } else { 1 });
                changed = true;
            }
        }
    }
    changed
}

pub fn draw(look: &PlayerLook, sel: usize, player_tex: &Texture2D) {
    clear_background(crate::sprites::hex("#14101c"));
    let cx = screen_width() / 2.0;
    let title = "WHO DESCENDS?";
    let td = measure_text(title, None, 40, 1.0);
    draw_text(title, cx - td.width / 2.0, 80.0, 40.0, crate::sprites::hex("#ffd24a"));

    // big preview
    let size = 120.0;
    draw_texture_ex(
        player_tex,
        cx - size / 2.0 - 190.0,
        170.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            ..Default::default()
        },
    );

    for row in 0..ROWS {
        let (x, y, w, h) = row_rect(row);
        let selected = row == sel;
        let border = if selected {
            crate::sprites::hex("#ffd24a")
        } else {
            crate::sprites::hex("#4a4458")
        };
        draw_rectangle(x, y, w, h, Color::new(0.12, 0.1, 0.16, 1.0));
        draw_rectangle_lines(x, y, w, h, 2.0, border);
        let (label, value) = row_label(look, row);
        draw_text(label, x + 12.0, y + 20.0, 16.0, crate::sprites::hex("#8a7f9d"));
        draw_text("<", x + 12.0, y + 40.0, 18.0, if selected { WHITE } else { GRAY });
        let vd = measure_text(value, None, 18, 1.0);
        draw_text(value, x + w / 2.0 - vd.width / 2.0, y + 40.0, 18.0, WHITE);
        let gd = measure_text(">", None, 18, 1.0);
        draw_text(">", x + w - 12.0 - gd.width, y + 40.0, 18.0, if selected { WHITE } else { GRAY });
    }

    let help = "arrows / tap to customize · [Enter or Esc] done (saved on desktop)";
    let hd = measure_text(help, None, 17, 1.0);
    draw_text(help, cx - hd.width / 2.0, 150.0 + ROWS as f32 * 56.0 + 40.0, 17.0, crate::sprites::hex("#8a7f9d"));
}
