use macroquad::prelude::*;
use std::collections::HashMap;

pub fn hex(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    if s.len() < 6 {
        return WHITE;
    }
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255);
    Color::from_rgba(r, g, b, 255)
}

/// Turn rows of palette characters into a nearest-filtered pixel texture.
/// '.' and ' ' are transparent; every other char looks up its hex color.
pub fn build_texture(rows: &[String], palette: &HashMap<String, String>) -> Texture2D {
    let h = rows.len().max(1);
    let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(1).max(1);
    let mut img = Image::gen_image_color(w as u16, h as u16, Color::from_rgba(0, 0, 0, 0));
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if ch == '.' || ch == ' ' {
                continue;
            }
            let c = palette
                .get(&ch.to_string())
                .map(|v| hex(v))
                .unwrap_or(WHITE);
            img.set_pixel(x as u32, y as u32, c);
        }
    }
    let t = Texture2D::from_image(&img);
    t.set_filter(FilterMode::Nearest);
    t
}

pub const PLAYER_SPRITE: [&str; 12] = [
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

pub fn player_palette() -> HashMap<String, String> {
    [
        ("h", "#6a3a1a"),
        ("a", "#d9a97a"),
        ("e", "#221a2a"),
        ("c", "#8a2a3a"),
        ("r", "#cc2233"),
        ("b", "#3a4a6a"),
        ("s", "#2a2a2a"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

pub const CHEST_SPRITE: [&str; 12] = [
    "............",
    "............",
    "............",
    "..aaaaaaaa..",
    ".abbbbbbbba.",
    ".aaaaaaaaaa.",
    ".abbbggbbba.",
    ".abbbggbbba.",
    ".abbbbbbbba.",
    ".aaaaaaaaaa.",
    "............",
    "............",
];

pub const CHEST_OPEN_SPRITE: [&str; 12] = [
    "............",
    ".aaaaaaaaaa.",
    ".a........a.",
    ".aaaaaaaaaa.",
    ".abbbbbbbba.",
    ".aaaaaaaaaa.",
    ".abbbggbbba.",
    ".abbbbbbbba.",
    ".abbbbbbbba.",
    ".aaaaaaaaaa.",
    "............",
    "............",
];

pub fn chest_palette() -> HashMap<String, String> {
    [("a", "#7a5230"), ("b", "#a0783c"), ("g", "#ffd24a")]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

pub const CAMPFIRE_SPRITE: [&str; 12] = [
    "............",
    "............",
    ".....y......",
    "....yoy.....",
    "....yoy..y..",
    "...yoooy.o..",
    "...yorroy...",
    "..yorrrroy..",
    "..sorrrros..",
    ".sswwwwwwss.",
    ".ws.wwww.sw.",
    "............",
];

pub fn campfire_palette() -> HashMap<String, String> {
    [
        ("y", "#ffe066"),
        ("o", "#ff9933"),
        ("r", "#e04422"),
        ("w", "#7a5230"),
        ("s", "#4e3018"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

pub const STAIRS_SPRITE: [&str; 12] = [
    "aaaaaaaaaaaa",
    "abbbbbbbbbba",
    "abssssssssba",
    "abs......sba",
    "abs.dddd.sba",
    "abs.dddd.sba",
    "abs..dd..sba",
    "abs..dd..sba",
    "abs......sba",
    "abssssssssba",
    "abbbbbbbbbba",
    "aaaaaaaaaaaa",
];

pub fn stairs_palette() -> HashMap<String, String> {
    [
        ("a", "#3a3440"),
        ("b", "#524a5c"),
        ("s", "#262030"),
        ("d", "#0e0a14"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

pub fn strs_to_rows(rows: &[&str]) -> Vec<String> {
    rows.iter().map(|s| s.to_string()).collect()
}
