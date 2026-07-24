use crate::dungeon::{self, CustomLevel, Map, Tile, MAP_H, MAP_W, TILE};
use crate::sprites::hex;
use macroquad::prelude::*;

const BRUSHES: [&str; 12] = [
    "Floor", "Wall", "Safe floor", "Campfire", "Stairs down", "Chest", "Graffiti",
    "Mob T1", "Mob T2", "Mob T3", "Boss (T4)", "NPC",
];

pub struct Editor {
    pub map: Map,
    pub brush: usize,
    pub status: String,
}

fn blank_map() -> Map {
    let mut m = Map {
        w: MAP_W,
        h: MAP_H,
        tiles: vec![Tile::Wall; MAP_W * MAP_H],
        rooms: Vec::new(),
        spawn: (3.5 * TILE, 3.5 * TILE),
        graffiti: Vec::new(),
        chest_spots: Vec::new(),
        mob_spots: Vec::new(),
        npc_spots: Vec::new(),
        has_shop: true,
    };
    for y in 2..8 {
        for x in 2..10 {
            m.set(x, y, Tile::Safe);
        }
    }
    m.set(5, 4, Tile::Campfire);
    m
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            map: blank_map(),
            brush: 0,
            status: "Paint with the mouse. [G] generates a start point.".to_string(),
        }
    }

    fn clear_markers(&mut self, x: usize, y: usize) {
        self.map.graffiti.retain(|g| !(g.x == x && g.y == y));
        self.map.chest_spots.retain(|&(cx, cy)| !(cx == x && cy == y));
        self.map.mob_spots.retain(|&(mx, my, _)| !(mx == x && my == y));
        self.map.npc_spots.retain(|&(nx, ny)| !(nx == x && ny == y));
    }

    fn paint(&mut self, x: usize, y: usize) {
        if x == 0 || y == 0 || x >= self.map.w - 1 || y >= self.map.h - 1 {
            return; // keep the border sealed
        }
        self.clear_markers(x, y);
        let (xi, yi) = (x as i32, y as i32);
        match self.brush {
            0 => self.map.set(xi, yi, Tile::Floor),
            1 => self.map.set(xi, yi, Tile::Wall),
            2 => self.map.set(xi, yi, Tile::Safe),
            3 => self.map.set(xi, yi, Tile::Campfire),
            4 => self.map.set(xi, yi, Tile::Stairs),
            5 => {
                self.map.set(xi, yi, Tile::Floor);
                self.map.chest_spots.push((x, y));
            }
            6 => {
                self.map.set(xi, yi, Tile::Wall);
                self.map.graffiti.push(dungeon::Graffiti { x, y, text_idx: (x * 7 + y * 13) % 16 });
            }
            7..=10 => {
                self.map.set(xi, yi, Tile::Floor);
                self.map.mob_spots.push((x, y, (self.brush - 6) as i32));
            }
            11 => {
                self.map.set(xi, yi, Tile::Safe);
                self.map.npc_spots.push((x, y));
            }
            _ => {}
        }
    }

    /// Returns Some(level) when the user wants to test-play the level.
    pub fn update(&mut self, graffiti_count: usize) -> Option<CustomLevel> {
        // brush select
        let keys = [
            KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5,
            KeyCode::Key6, KeyCode::Key7, KeyCode::Key8, KeyCode::Key9, KeyCode::Key0,
        ];
        for (i, k) in keys.iter().enumerate() {
            if is_key_pressed(*k) {
                self.brush = i;
            }
        }
        if is_key_pressed(KeyCode::LeftBracket) && self.brush > 0 {
            self.brush -= 1;
        }
        if is_key_pressed(KeyCode::RightBracket) && self.brush < BRUSHES.len() - 1 {
            self.brush += 1;
        }
        if is_key_pressed(KeyCode::N) {
            self.map = blank_map();
            self.status = "Fresh canvas. The dungeon is yours, comrade.".to_string();
        }
        if is_key_pressed(KeyCode::G) {
            self.map = dungeon::generate(1, graffiti_count);
            self.status = "Generated a level to remix.".to_string();
        }

        let (ts, ox, oy) = self.view();
        let (mx, my) = mouse_position();
        let tx = ((mx - ox) / ts).floor() as i32;
        let ty = ((my - oy) / ts).floor() as i32;
        let in_bounds = tx >= 0 && ty >= 0 && (tx as usize) < self.map.w && (ty as usize) < self.map.h;

        if in_bounds && is_mouse_button_down(MouseButton::Left) {
            self.paint(tx as usize, ty as usize);
        }
        if in_bounds && is_mouse_button_down(MouseButton::Right) {
            let (x, y) = (tx as usize, ty as usize);
            if x > 0 && y > 0 && x < self.map.w - 1 && y < self.map.h - 1 {
                self.clear_markers(x, y);
                self.map.set(tx, ty, Tile::Wall);
            }
        }
        if in_bounds && is_key_pressed(KeyCode::P) {
            self.map.spawn = ((tx as f32 + 0.5) * TILE, (ty as f32 + 0.5) * TILE);
            self.status = "Player spawn set.".to_string();
        }

        if is_key_pressed(KeyCode::F5) {
            let level = dungeon::to_custom(&self.map, "custom");
            self.save(&level);
        }
        if is_key_pressed(KeyCode::T) {
            return Some(dungeon::to_custom(&self.map, "custom"));
        }
        None
    }

    fn save(&mut self, level: &CustomLevel) {
        let json = serde_json::to_string_pretty(level).unwrap_or_default();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = std::fs::create_dir_all("data");
            match std::fs::write("data/custom_level.json", &json) {
                Ok(_) => self.status = "Saved to data/custom_level.json — [L] on the menu plays it.".to_string(),
                Err(e) => self.status = format!("Save failed: {}", e),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            macroquad::logging::info!("CUSTOM LEVEL JSON:\n{}", json);
            self.status = "Level JSON printed to the browser console (F12) — save it as data/custom_level.json.".to_string();
        }
    }

    fn view(&self) -> (f32, f32, f32) {
        let ts = ((screen_width() - 260.0) / self.map.w as f32)
            .min((screen_height() - 40.0) / self.map.h as f32)
            .floor()
            .max(4.0);
        (ts, 20.0, 30.0)
    }

    pub fn draw(&self) {
        clear_background(hex("#100c18"));
        let (ts, ox, oy) = self.view();

        for y in 0..self.map.h {
            for x in 0..self.map.w {
                let sx = ox + x as f32 * ts;
                let sy = oy + y as f32 * ts;
                let c = match self.map.tile(x as i32, y as i32) {
                    Tile::Wall => hex("#3a3444"),
                    Tile::Floor => hex("#221e2c"),
                    Tile::Safe => hex("#1e3226"),
                    Tile::Campfire => hex("#a05a20"),
                    Tile::Stairs => hex("#c0a030"),
                };
                draw_rectangle(sx, sy, ts - 1.0, ts - 1.0, c);
            }
        }
        for g in &self.map.graffiti {
            draw_text("g", ox + g.x as f32 * ts + 1.0, oy + g.y as f32 * ts + ts - 1.0, ts + 2.0, hex("#ff7788"));
        }
        for &(x, y) in &self.map.chest_spots {
            draw_text("C", ox + x as f32 * ts + 1.0, oy + y as f32 * ts + ts - 1.0, ts + 2.0, hex("#ffd24a"));
        }
        for &(x, y, tier) in &self.map.mob_spots {
            let c = match tier {
                1 => hex("#7fbf7f"),
                2 => hex("#e0a040"),
                3 => hex("#e05050"),
                _ => hex("#ff2266"),
            };
            draw_text(&tier.to_string(), ox + x as f32 * ts + 1.0, oy + y as f32 * ts + ts - 1.0, ts + 2.0, c);
        }
        for &(x, y) in &self.map.npc_spots {
            draw_text("n", ox + x as f32 * ts + 1.0, oy + y as f32 * ts + ts - 1.0, ts + 2.0, hex("#7fd4c9"));
        }
        let (sx, sy) = self.map.spawn;
        draw_text(
            "@",
            ox + (sx / TILE).floor() * ts + 1.0,
            oy + (sy / TILE).floor() * ts + ts - 1.0,
            ts + 2.0,
            WHITE,
        );

        // hover highlight
        let (mx, my) = mouse_position();
        let tx = ((mx - ox) / ts).floor();
        let ty = ((my - oy) / ts).floor();
        if tx >= 0.0 && ty >= 0.0 && (tx as usize) < self.map.w && (ty as usize) < self.map.h {
            draw_rectangle_lines(ox + tx * ts, oy + ty * ts, ts, ts, 2.0, WHITE);
        }

        // sidebar
        let px = ox + self.map.w as f32 * ts + 16.0;
        draw_text("LEVEL CREATOR", px, 46.0, 22.0, hex("#ffd24a"));
        for (i, b) in BRUSHES.iter().enumerate() {
            let sel = i == self.brush;
            let label = if i < 10 {
                format!("[{}] {}", (i + 1) % 10, b)
            } else {
                format!("[ ] {}", b)
            };
            draw_text(
                &label,
                px,
                80.0 + i as f32 * 22.0,
                17.0,
                if sel { hex("#ffe066") } else { GRAY },
            );
        }
        let help = [
            "",
            "LMB paint · RMB erase",
            "[ / ] cycle brush",
            "P: set spawn at cursor",
            "G: generate  N: blank",
            "F5: save level",
            "T: test-play level",
            "Esc: back to menu",
        ];
        for (i, l) in help.iter().enumerate() {
            draw_text(l, px, 90.0 + (BRUSHES.len() + i) as f32 * 22.0, 15.0, hex("#8a7f9d"));
        }
        draw_text(&self.status, 20.0, screen_height() - 12.0, 15.0, hex("#aef0c0"));
    }
}
