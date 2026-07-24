use crate::content::Content;
use crate::dungeon::{Tile, TILE};
use crate::sprites::{self, hex};
use crate::world::{World, EQUIP_SLOTS};
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct Textures {
    map: HashMap<String, Texture2D>,
}

impl Textures {
    pub fn get(&self, key: &str) -> Option<&Texture2D> {
        self.map.get(key)
    }
}

pub fn build_textures(content: &Content) -> Textures {
    let mut map = HashMap::new();
    for m in &content.mobs {
        map.insert(format!("mob:{}", m.id), sprites::build_texture(&m.sprite, &m.palette));
    }
    for i in &content.items {
        map.insert(format!("item:{}", i.id), sprites::build_texture(&i.sprite, &i.palette));
    }
    for n in &content.npcs {
        map.insert(format!("npc:{}", n.id), sprites::build_texture(&n.sprite, &n.palette));
    }
    map.insert(
        "player".to_string(),
        sprites::build_texture(&sprites::strs_to_rows(&sprites::PLAYER_SPRITE), &sprites::player_palette()),
    );
    map.insert(
        "chest".to_string(),
        sprites::build_texture(&sprites::strs_to_rows(&sprites::CHEST_SPRITE), &sprites::chest_palette()),
    );
    map.insert(
        "chest_open".to_string(),
        sprites::build_texture(&sprites::strs_to_rows(&sprites::CHEST_OPEN_SPRITE), &sprites::chest_palette()),
    );
    map.insert(
        "campfire".to_string(),
        sprites::build_texture(&sprites::strs_to_rows(&sprites::CAMPFIRE_SPRITE), &sprites::campfire_palette()),
    );
    map.insert(
        "stairs".to_string(),
        sprites::build_texture(&sprites::strs_to_rows(&sprites::STAIRS_SPRITE), &sprites::stairs_palette()),
    );
    Textures { map }
}

pub fn scale() -> f32 {
    (screen_height() / 240.0).round().max(2.0)
}

pub fn camera(world: &World) -> (f32, f32, f32) {
    let s = scale();
    let map_w = world.map.w as f32 * TILE * s;
    let map_h = world.map.h as f32 * TILE * s;
    let mut cx = world.player.x * s - screen_width() / 2.0;
    let mut cy = world.player.y * s - screen_height() / 2.0;
    if map_w > screen_width() {
        cx = cx.clamp(0.0, map_w - screen_width());
    } else {
        cx = (map_w - screen_width()) / 2.0;
    }
    if map_h > screen_height() {
        cy = cy.clamp(0.0, map_h - screen_height());
    } else {
        cy = (map_h - screen_height()) / 2.0;
    }
    (cx, cy, s)
}

// depth themes cycle so descending feels like travel
const FLOOR_THEMES: [(&str, &str, &str); 5] = [
    ("#2a2434", "#262030", "#4a4458"),
    ("#26302a", "#222c26", "#44584a"),
    ("#302626", "#2c2222", "#584444"),
    ("#262a34", "#22262f", "#44495c"),
    ("#2e2836", "#2a2432", "#514763"),
];

pub fn draw_world(world: &World, content: &Content, tex: &Textures) {
    let (cx, cy, s) = camera(world);
    let ts = TILE * s;
    let theme = FLOOR_THEMES[((world.depth - 1).max(0) as usize) % FLOOR_THEMES.len()];
    let floor_a = hex(theme.0);
    let floor_b = hex(theme.1);
    let wall_c = hex(theme.2);
    let wall_dark = Color::new(wall_c.r * 0.55, wall_c.g * 0.55, wall_c.b * 0.55, 1.0);
    let safe_a = hex("#24382c");
    let safe_b = hex("#203226");

    let x0 = (cx / ts).floor().max(0.0) as i32;
    let y0 = (cy / ts).floor().max(0.0) as i32;
    let x1 = (((cx + screen_width()) / ts).ceil() as i32).min(world.map.w as i32);
    let y1 = (((cy + screen_height()) / ts).ceil() as i32).min(world.map.h as i32);

    for ty in y0..y1 {
        for tx in x0..x1 {
            let sx = tx as f32 * ts - cx;
            let sy = ty as f32 * ts - cy;
            match world.map.tile(tx, ty) {
                Tile::Wall => {
                    // only draw walls that border something walkable (lo-fi void elsewhere)
                    let near_open = (-1..=1).any(|dy| {
                        (-1..=1).any(|dx| world.map.tile(tx + dx, ty + dy).walkable())
                    });
                    if near_open {
                        draw_rectangle(sx, sy, ts, ts, wall_dark);
                        draw_rectangle(sx, sy, ts, ts * 0.6, wall_c);
                    }
                }
                Tile::Floor | Tile::Stairs => {
                    let c = if (tx + ty) % 2 == 0 { floor_a } else { floor_b };
                    draw_rectangle(sx, sy, ts, ts, c);
                }
                Tile::Safe | Tile::Campfire => {
                    let c = if (tx + ty) % 2 == 0 { safe_a } else { safe_b };
                    draw_rectangle(sx, sy, ts, ts, c);
                }
            }
        }
    }

    // stairs + campfire sprites
    for ty in y0..y1 {
        for tx in x0..x1 {
            let sx = tx as f32 * ts - cx;
            let sy = ty as f32 * ts - cy;
            match world.map.tile(tx, ty) {
                Tile::Stairs => draw_sprite(tex, "stairs", sx + ts * 0.125, sy + ts * 0.125, ts * 0.75),
                Tile::Campfire => {
                    let flicker = ((get_time() * 9.0).sin() * 0.06 + 1.0) as f32;
                    draw_sprite(tex, "campfire", sx, sy - ts * (flicker - 1.0), ts * flicker);
                }
                _ => {}
            }
        }
    }

    // graffiti scrawls
    for g in &world.map.graffiti {
        let sx = g.x as f32 * ts - cx;
        let sy = g.y as f32 * ts - cy;
        if sx < -ts || sy < -ts || sx > screen_width() || sy > screen_height() {
            continue;
        }
        let red = hex("#e04444");
        let pink = hex("#ff7788");
        draw_line(sx + ts * 0.15, sy + ts * 0.35, sx + ts * 0.5, sy + ts * 0.2, s, red);
        draw_line(sx + ts * 0.5, sy + ts * 0.2, sx + ts * 0.85, sy + ts * 0.4, s, red);
        draw_line(sx + ts * 0.2, sy + ts * 0.65, sx + ts * 0.8, sy + ts * 0.6, s, pink);
        draw_rectangle(sx + ts * 0.42, sy + ts * 0.42, s * 2.0, s * 2.0, pink);
    }

    // chests
    for c in &world.chests {
        let sx = c.tx as f32 * ts - cx;
        let sy = c.ty as f32 * ts - cy;
        draw_sprite(tex, if c.opened { "chest_open" } else { "chest" }, sx, sy, ts);
    }

    // drops (bobbing)
    for d in &world.drops {
        let bob = (d.t * 4.0).sin() * 2.0 * s;
        let sx = d.x * s - cx - ts * 0.3;
        let sy = d.y * s - cy - ts * 0.3 + bob;
        draw_sprite(tex, &format!("item:{}", d.item), sx, sy, ts * 0.6);
    }

    // npcs
    for n in &world.npcs {
        let def = &content.npcs[n.def_idx];
        let sx = n.x * s - cx - ts * 0.5;
        let sy = n.y * s - cy - ts * 0.5;
        draw_sprite(tex, &format!("npc:{}", def.id), sx, sy, ts);
    }

    // mobs
    for m in &world.mobs {
        let def = &content.mobs[m.def_idx];
        let size = if m.boss { ts * 1.6 } else { ts };
        let sx = m.x * s - cx - size / 2.0;
        let sy = m.y * s - cy - size / 2.0;
        let tint = if m.hurt > 0.0 {
            Color::new(1.0, 0.4, 0.4, 1.0)
        } else {
            WHITE
        };
        if let Some(t) = tex.get(&format!("mob:{}", def.id)) {
            draw_texture_ex(
                t,
                sx,
                sy,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(size, size)),
                    flip_x: world.player.x < m.x,
                    ..Default::default()
                },
            );
        }
        // hp bar when damaged or aggro
        if m.aggro || m.hp < m.maxhp {
            let frac = (m.hp as f32 / m.maxhp as f32).clamp(0.0, 1.0);
            draw_rectangle(sx, sy - 4.0, size, 3.0, Color::new(0.1, 0.1, 0.1, 0.8));
            draw_rectangle(sx, sy - 4.0, size * frac, 3.0, hex("#e04444"));
        }
    }

    // player
    {
        let p = &world.player;
        let sx = p.x * s - cx - ts * 0.5;
        let sy = p.y * s - cy - ts * 0.5;
        let tint = if p.hurt > 0.0 {
            Color::new(1.0, 0.5, 0.5, 1.0)
        } else {
            WHITE
        };
        if let Some(t) = tex.get("player") {
            draw_texture_ex(
                t,
                sx,
                sy,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(ts, ts)),
                    flip_x: p.facing.x < 0.0,
                    ..Default::default()
                },
            );
        }
        // swing flash
        if p.swing > 0.0 {
            let fx = p.x * s - cx + p.facing.x * ts * 0.8;
            let fy = p.y * s - cy + p.facing.y * ts * 0.8;
            draw_circle_lines(fx, fy, ts * 0.45, s, Color::new(1.0, 1.0, 0.9, p.swing * 5.0));
        }
    }

    // spell bursts
    for b in &world.bursts {
        let frac = 1.0 - (b.t / 0.45).clamp(0.0, 1.0);
        let r = (b.radius * 0.4 + b.radius * 0.6 * frac) * s;
        let mut c = hex(&b.color);
        c.a = (b.t * 2.5).clamp(0.0, 1.0);
        draw_circle_lines(b.x * s - cx, b.y * s - cy, r, s * 1.5, c);
    }

    // floating combat text
    for f in &world.fcts {
        let c = Color::from_rgba(f.color.0, f.color.1, f.color.2, (f.t * 255.0) as u8);
        draw_text(&f.text, f.x * s - cx, f.y * s - cy, 18.0, c);
    }

    // speech bubbles: enemies broadcast propaganda, comrades reply
    for m in &world.mobs {
        if m.say_t > 0.0 && !m.say.is_empty() {
            bubble(m.x * s - cx, m.y * s - cy - ts * 0.7, &m.say, hex("#3a2020"));
        }
    }
    for n in &world.npcs {
        if n.say_t > 0.0 && !n.say.is_empty() {
            bubble(n.x * s - cx, n.y * s - cy - ts * 0.7, &n.say, hex("#20303a"));
        }
    }
}

fn draw_sprite(tex: &Textures, key: &str, x: f32, y: f32, size: f32) {
    if let Some(t) = tex.get(key) {
        draw_texture_ex(
            t,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
    }
}

pub fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if !cur.is_empty() && cur.chars().count() + 1 + word.chars().count() > max_chars {
            lines.push(cur.clone());
            cur.clear();
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn bubble(cx: f32, bottom_y: f32, text: &str, bg: Color) {
    let lines = wrap_text(text, 34);
    let fs = 16.0;
    let line_h = 16.0;
    let w = lines
        .iter()
        .map(|l| measure_text(l, None, fs as u16, 1.0).width)
        .fold(0.0_f32, f32::max)
        + 12.0;
    let h = lines.len() as f32 * line_h + 8.0;
    let x = (cx - w / 2.0).clamp(4.0, (screen_width() - w - 4.0).max(4.0));
    let y = (bottom_y - h).max(4.0);
    let mut bgc = bg;
    bgc.a = 0.92;
    draw_rectangle(x, y, w, h, bgc);
    draw_rectangle_lines(x, y, w, h, 2.0, Color::new(1.0, 1.0, 1.0, 0.5));
    for (i, l) in lines.iter().enumerate() {
        draw_text(l, x + 6.0, y + 14.0 + i as f32 * line_h, fs, WHITE);
    }
}

pub fn interaction_hint(world: &World, content: &Content) -> Option<String> {
    let px = world.player.x;
    let py = world.player.y;
    if world.map.tile_at_px(px, py) == Tile::Stairs {
        return Some("[E] Descend deeper".to_string());
    }
    let ptx = (px / TILE) as i32;
    let pty = (py / TILE) as i32;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if world.map.tile(ptx + dx, pty + dy) == Tile::Campfire {
                return Some("[E] Rest by the fire (free, always)".to_string());
            }
        }
    }
    for n in &world.npcs {
        if (n.x - px).powi(2) + (n.y - py).powi(2) < (TILE * 1.6).powi(2) {
            let def = &content.npcs[n.def_idx];
            if def.shopkeeper && world.map.has_shop {
                return Some(format!("[E] Browse the co-op ({})", def.name));
            }
            return Some(format!("[E] Talk to {}", def.name));
        }
    }
    for c in &world.chests {
        if c.opened {
            continue;
        }
        let cx = (c.tx as f32 + 0.5) * TILE;
        let cy = (c.ty as f32 + 0.5) * TILE;
        if (cx - px).powi(2) + (cy - py).powi(2) < (TILE * 1.5).powi(2) {
            return Some("[E] Open chest".to_string());
        }
    }
    for g in &world.map.graffiti {
        let gx = (g.x as f32 + 0.5) * TILE;
        let gy = (g.y as f32 + 0.5) * TILE;
        if (gx - px).powi(2) + (gy - py).powi(2) < (TILE * 2.0).powi(2) {
            return Some("[E] Read the writing on the wall".to_string());
        }
    }
    None
}

pub fn draw_hud(world: &World, content: &Content) {
    let p = &world.player;
    let (atk, def, maxhp, maxmp, _) = p.totals(content);

    // panel
    draw_rectangle(8.0, 8.0, 230.0, 86.0, Color::new(0.0, 0.0, 0.0, 0.55));
    // hp
    draw_rectangle(16.0, 16.0, 180.0, 12.0, hex("#3a1418"));
    let hfrac = (p.hp as f32 / maxhp.max(1) as f32).clamp(0.0, 1.0);
    draw_rectangle(16.0, 16.0, 180.0 * hfrac, 12.0, hex("#e04444"));
    draw_text(&format!("{}/{}", p.hp, maxhp), 200.0, 26.0, 14.0, WHITE);
    // mp
    draw_rectangle(16.0, 32.0, 180.0, 10.0, hex("#141c3a"));
    let mfrac = (p.mp as f32 / maxmp.max(1) as f32).clamp(0.0, 1.0);
    draw_rectangle(16.0, 32.0, 180.0 * mfrac, 10.0, hex("#4488e0"));
    draw_text(&format!("{}/{}", p.mp, maxmp), 200.0, 41.0, 14.0, WHITE);
    // xp
    draw_rectangle(16.0, 46.0, 180.0, 6.0, hex("#2a2a14"));
    let xfrac = (p.xp as f32 / p.xp_to_next().max(1) as f32).clamp(0.0, 1.0);
    draw_rectangle(16.0, 46.0, 180.0 * xfrac, 6.0, hex("#e0c044"));
    draw_text(
        &format!("Lv {}   ATK {}   DEF {}", p.level, atk, def),
        16.0,
        68.0,
        16.0,
        WHITE,
    );
    draw_text(
        &format!("Gold {}   Depth {}", p.gold, world.depth),
        16.0,
        86.0,
        16.0,
        hex("#ffd24a"),
    );

    // spells bar
    let known: Vec<&crate::content::SpellDef> = content
        .spells
        .iter()
        .filter(|s| s.unlock_level <= p.level)
        .collect();
    let next_locked = content
        .spells
        .iter()
        .filter(|s| s.unlock_level > p.level)
        .min_by_key(|s| s.unlock_level);
    let y = screen_height() - 54.0;
    for (i, sp) in known.iter().enumerate() {
        let x = 12.0 + i as f32 * 150.0;
        let affordable = p.mp >= sp.cost;
        let bg = if affordable {
            Color::new(0.0, 0.0, 0.0, 0.6)
        } else {
            Color::new(0.1, 0.05, 0.05, 0.6)
        };
        draw_rectangle(x, y, 142.0, 40.0, bg);
        draw_rectangle_lines(x, y, 142.0, 40.0, 2.0, hex(&sp.color));
        let tcol = if affordable { WHITE } else { GRAY };
        draw_text(&format!("[{}] {}", i + 1, sp.name), x + 6.0, y + 17.0, 15.0, tcol);
        draw_text(&format!("{} MP", sp.cost), x + 6.0, y + 33.0, 14.0, hex("#4488e0"));
    }
    if let Some(nl) = next_locked {
        let x = 12.0 + known.len() as f32 * 150.0;
        draw_rectangle(x, y, 142.0, 40.0, Color::new(0.0, 0.0, 0.0, 0.4));
        draw_text(&format!("Lv{}: {}", nl.unlock_level, nl.name), x + 6.0, y + 24.0, 14.0, GRAY);
    }

    // minimap
    let mm = 2.0;
    let mx = screen_width() - world.map.w as f32 * mm - 12.0;
    let my = 12.0;
    draw_rectangle(
        mx - 2.0,
        my - 2.0,
        world.map.w as f32 * mm + 4.0,
        world.map.h as f32 * mm + 4.0,
        Color::new(0.0, 0.0, 0.0, 0.5),
    );
    for ty in 0..world.map.h {
        for tx in 0..world.map.w {
            let t = world.map.tile(tx as i32, ty as i32);
            let c = match t {
                Tile::Wall => continue,
                Tile::Floor => Color::new(0.5, 0.48, 0.55, 0.5),
                Tile::Safe => Color::new(0.3, 0.6, 0.4, 0.6),
                Tile::Campfire => Color::new(1.0, 0.6, 0.2, 0.9),
                Tile::Stairs => Color::new(1.0, 0.9, 0.3, 0.9),
            };
            draw_rectangle(mx + tx as f32 * mm, my + ty as f32 * mm, mm, mm, c);
        }
    }
    for m in &world.mobs {
        let c = if m.boss { hex("#ff3355") } else { Color::new(0.9, 0.3, 0.3, 0.8) };
        draw_rectangle(mx + m.x / TILE * mm - 1.0, my + m.y / TILE * mm - 1.0, 2.0, 2.0, c);
    }
    draw_rectangle(
        mx + world.player.x / TILE * mm - 1.5,
        my + world.player.y / TILE * mm - 1.5,
        3.0,
        3.0,
        WHITE,
    );

    // boss bar
    if let Some(b) = world.mobs.iter().find(|m| m.boss && m.aggro) {
        let def = &content.mobs[b.def_idx];
        let w = screen_width() * 0.5;
        let x = (screen_width() - w) / 2.0;
        draw_rectangle(x, 40.0, w, 14.0, Color::new(0.1, 0.05, 0.05, 0.8));
        let frac = (b.hp as f32 / b.maxhp as f32).clamp(0.0, 1.0);
        draw_rectangle(x, 40.0, w * frac, 14.0, hex("#ff3355"));
        let td = measure_text(&def.name, None, 18, 1.0);
        draw_text(&def.name, (screen_width() - td.width) / 2.0, 34.0, 18.0, hex("#ff8899"));
    }

    // toasts
    let mut ty = screen_height() - 90.0;
    for (text, t) in world.toasts.iter() {
        let a = t.min(1.0);
        let lines = wrap_text(text, 70);
        for l in lines.iter().rev() {
            let td = measure_text(l, None, 17, 1.0);
            let x = (screen_width() - td.width) / 2.0;
            draw_rectangle(
                x - 8.0,
                ty - 15.0,
                td.width + 16.0,
                21.0,
                Color::new(0.0, 0.0, 0.0, 0.55 * a),
            );
            draw_text(l, x, ty, 17.0, Color::new(1.0, 1.0, 1.0, a));
            ty -= 22.0;
        }
    }

    // interaction hint
    if let Some(h) = interaction_hint(world, content) {
        let td = measure_text(&h, None, 19, 1.0);
        let x = (screen_width() - td.width) / 2.0;
        let y = screen_height() - 120.0;
        draw_rectangle(x - 10.0, y - 17.0, td.width + 20.0, 24.0, Color::new(0.0, 0.0, 0.0, 0.7));
        draw_text(&h, x, y, 19.0, hex("#ffe066"));
    }
}

fn panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    draw_rectangle(x, y, w, h, Color::new(0.06, 0.05, 0.09, 0.96));
    draw_rectangle_lines(x, y, w, h, 3.0, hex("#8a7f9d"));
    draw_rectangle(x, y, w, 30.0, hex("#2a2434"));
    draw_text(title, x + 12.0, y + 21.0, 20.0, hex("#ffd24a"));
}

pub fn draw_inventory(world: &World, content: &Content, tex: &Textures, sel: usize, doll: bool, doll_sel: usize) {
    let w = 720.0_f32.min(screen_width() - 40.0);
    let h = 420.0_f32.min(screen_height() - 40.0);
    let x = (screen_width() - w) / 2.0;
    let y = (screen_height() - h) / 2.0;
    panel(x, y, w, h, "Inventory & Paper Doll   [Tab] switch  [Enter] use/equip  [X] drop  [Esc] close");

    // paper doll (left)
    let p = &world.player;
    let dx = x + 16.0;
    let dy = y + 44.0;
    draw_text("EQUIPPED", dx, dy, 16.0, hex("#8a7f9d"));
    for (i, slot) in EQUIP_SLOTS.iter().enumerate() {
        let sy = dy + 10.0 + i as f32 * 44.0;
        let selected = doll && doll_sel == i;
        let border = if selected { hex("#ffd24a") } else { hex("#4a4458") };
        draw_rectangle(dx, sy, 250.0, 40.0, Color::new(0.12, 0.1, 0.16, 1.0));
        draw_rectangle_lines(dx, sy, 250.0, 40.0, 2.0, border);
        draw_text(&slot.to_uppercase(), dx + 6.0, sy + 16.0, 13.0, hex("#8a7f9d"));
        if let Some(id) = p.equipment.get(*slot) {
            if let Some(t) = tex.get(&format!("item:{}", id)) {
                draw_texture_ex(
                    t,
                    dx + 6.0,
                    sy + 15.0,
                    WHITE,
                    DrawTextureParams { dest_size: Some(vec2(22.0, 22.0)), ..Default::default() },
                );
            }
            let name = content.item(id).map(|d| d.name.clone()).unwrap_or_else(|| id.clone());
            draw_text(&name, dx + 34.0, sy + 30.0, 15.0, WHITE);
        } else {
            draw_text("—", dx + 34.0, sy + 30.0, 15.0, DARKGRAY);
        }
    }

    // stats
    let (atk, def, maxhp, maxmp, spd) = p.totals(content);
    let stx = dx;
    let sty = dy + 10.0 + 7.0 * 44.0 + 18.0;
    draw_text(
        &format!("ATK {}  DEF {}  HP {}/{}  MP {}/{}  SPD {:.0}", atk, def, p.hp, maxhp, p.mp, maxmp, spd),
        stx,
        sty,
        15.0,
        hex("#aef0c0"),
    );

    // inventory grid (right)
    let gx = x + 300.0;
    let gy = y + 54.0;
    draw_text("BACKPACK", gx, gy - 10.0, 16.0, hex("#8a7f9d"));
    let cols = 6;
    let cell = 52.0;
    for i in 0..crate::world::INV_CAP {
        let cx = gx + (i % cols) as f32 * (cell + 6.0);
        let cy = gy + (i / cols) as f32 * (cell + 6.0);
        let selected = !doll && sel == i;
        let border = if selected { hex("#ffd24a") } else { hex("#4a4458") };
        draw_rectangle(cx, cy, cell, cell, Color::new(0.12, 0.1, 0.16, 1.0));
        draw_rectangle_lines(cx, cy, cell, cell, 2.0, border);
        if let Some(st) = p.inventory.get(i) {
            if let Some(t) = tex.get(&format!("item:{}", st.id)) {
                draw_texture_ex(
                    t,
                    cx + 6.0,
                    cy + 6.0,
                    WHITE,
                    DrawTextureParams { dest_size: Some(vec2(cell - 12.0, cell - 12.0)), ..Default::default() },
                );
            }
            if st.qty > 1 {
                draw_text(&format!("{}", st.qty), cx + cell - 16.0, cy + cell - 6.0, 15.0, WHITE);
            }
        }
    }

    // detail line for selection
    let detail_y = gy + 4.0 * (cell + 6.0) + 26.0;
    let described: Option<&str> = if doll {
        p.equipment.get(EQUIP_SLOTS[doll_sel.min(6)]).map(|s| s.as_str())
    } else {
        p.inventory.get(sel).map(|s| s.id.as_str())
    };
    if let Some(id) = described {
        if let Some(d) = content.item(id) {
            for (i, l) in wrap_text(&format!("{} — {}", d.name, d.desc), 52).iter().enumerate() {
                draw_text(l, gx, detail_y + i as f32 * 18.0, 16.0, WHITE);
            }
        }
    } else if doll {
        draw_text("[Enter] on an equipped slot unequips it.", gx, detail_y, 15.0, DARKGRAY);
    }
}

pub fn draw_shop(world: &World, content: &Content, tex: &Textures, sel: usize, selling: bool) {
    let w = 640.0_f32.min(screen_width() - 40.0);
    let h = 430.0_f32.min(screen_height() - 40.0);
    let x = (screen_width() - w) / 2.0;
    let y = (screen_height() - h) / 2.0;
    let title = if selling {
        "The Co-op — SELLING   [Tab] buy  [Enter] sell  [Esc] leave"
    } else {
        "The Co-op — BUYING   [Tab] sell  [Enter] buy  [Esc] leave"
    };
    panel(x, y, w, h, title);
    draw_text(
        &format!("Your gold: {}   (all proceeds fund the free clinic)", world.player.gold),
        x + 12.0,
        y + 50.0,
        16.0,
        hex("#ffd24a"),
    );

    let list_y = y + 70.0;
    if !selling {
        for (i, id) in world.shop_stock.iter().enumerate() {
            let Some(d) = content.item(id) else { continue };
            let ry = list_y + i as f32 * 48.0;
            let selected = sel == i;
            draw_rectangle(x + 12.0, ry, w - 24.0, 42.0, Color::new(0.12, 0.1, 0.16, 1.0));
            draw_rectangle_lines(x + 12.0, ry, w - 24.0, 42.0, 2.0, if selected { hex("#ffd24a") } else { hex("#4a4458") });
            if let Some(t) = tex.get(&format!("item:{}", id)) {
                draw_texture_ex(t, x + 18.0, ry + 6.0, WHITE, DrawTextureParams { dest_size: Some(vec2(30.0, 30.0)), ..Default::default() });
            }
            let afford = world.player.gold >= d.value as i64;
            draw_text(&d.name, x + 58.0, ry + 18.0, 16.0, if afford { WHITE } else { GRAY });
            draw_text(&d.desc, x + 58.0, ry + 36.0, 13.0, hex("#8a7f9d"));
            let price = format!("{} g", d.value);
            let td = measure_text(&price, None, 16, 1.0);
            draw_text(&price, x + w - 24.0 - td.width, ry + 26.0, 16.0, if afford { hex("#ffd24a") } else { hex("#8a4444") });
        }
        if world.shop_stock.is_empty() {
            draw_text("Sold out! Redistribution complete.", x + 20.0, list_y + 20.0, 16.0, GRAY);
        }
    } else {
        for (i, st) in world.player.inventory.iter().enumerate() {
            let Some(d) = content.item(&st.id) else { continue };
            let col = i / 8;
            let row = i % 8;
            let rx = x + 12.0 + col as f32 * ((w - 24.0) / 2.0);
            let ry = list_y + row as f32 * 40.0;
            let selected = sel == i;
            let rw = (w - 36.0) / 2.0;
            draw_rectangle(rx, ry, rw, 34.0, Color::new(0.12, 0.1, 0.16, 1.0));
            draw_rectangle_lines(rx, ry, rw, 34.0, 2.0, if selected { hex("#ffd24a") } else { hex("#4a4458") });
            if let Some(t) = tex.get(&format!("item:{}", st.id)) {
                draw_texture_ex(t, rx + 4.0, ry + 4.0, WHITE, DrawTextureParams { dest_size: Some(vec2(26.0, 26.0)), ..Default::default() });
            }
            draw_text(&format!("{} x{}", d.name, st.qty), rx + 36.0, ry + 15.0, 14.0, WHITE);
            draw_text(&format!("sells {} g", d.value / 2), rx + 36.0, ry + 30.0, 13.0, hex("#ffd24a"));
        }
        if world.player.inventory.is_empty() {
            draw_text("Nothing to sell. Keep what you need!", x + 20.0, list_y + 20.0, 16.0, GRAY);
        }
    }
}

pub fn draw_achievements(world: &World, content: &Content) {
    let w = 700.0_f32.min(screen_width() - 40.0);
    let h = 460.0_f32.min(screen_height() - 40.0);
    let x = (screen_width() - w) / 2.0;
    let y = (screen_height() - h) / 2.0;
    panel(x, y, w, h, &format!(
        "Achievements ({}/{})   [Esc] close",
        world.unlocked.len(),
        content.achievements.len()
    ));
    let per_col = ((content.achievements.len() + 1) / 2).max(1);
    for (i, a) in content.achievements.iter().enumerate() {
        let col = i / per_col;
        let row = i % per_col;
        let ax = x + 16.0 + col as f32 * (w / 2.0);
        let ay = y + 56.0 + row as f32 * 48.0;
        let unlocked = world.unlocked.contains(&a.id);
        let (mark, ncol, dcol) = if unlocked {
            ("✊", hex("#ffd24a"), WHITE)
        } else {
            ("·", DARKGRAY, GRAY)
        };
        draw_text(mark, ax, ay + 14.0, 20.0, ncol);
        draw_text(&a.name, ax + 24.0, ay + 8.0, 16.0, ncol);
        draw_text(&a.desc, ax + 24.0, ay + 26.0, 13.0, dcol);
    }
}

pub fn draw_dialog(who: &str, text: &str) {
    let lines = wrap_text(text, 56);
    let h = 70.0 + lines.len() as f32 * 20.0;
    let w = 620.0_f32.min(screen_width() - 40.0);
    let x = (screen_width() - w) / 2.0;
    let y = screen_height() - h - 30.0;
    draw_rectangle(x, y, w, h, Color::new(0.05, 0.04, 0.08, 0.95));
    draw_rectangle_lines(x, y, w, h, 3.0, hex("#8a7f9d"));
    draw_text(who, x + 14.0, y + 24.0, 18.0, hex("#ffd24a"));
    for (i, l) in lines.iter().enumerate() {
        draw_text(l, x + 14.0, y + 48.0 + i as f32 * 20.0, 17.0, WHITE);
    }
    draw_text("[E / Esc] close", x + w - 120.0, y + h - 10.0, 13.0, DARKGRAY);
}

pub fn draw_menu(has_custom: bool) {
    clear_background(hex("#14101c"));
    let cx = screen_width() / 2.0;
    let title = "CAPITOL DUNGEON";
    let td = measure_text(title, None, 64, 1.0);
    draw_text(title, cx - td.width / 2.0, 130.0, 64.0, hex("#ffd24a"));
    let sub = "a lo-fi crawl against vulture capital";
    let sd = measure_text(sub, None, 20, 1.0);
    draw_text(sub, cx - sd.width / 2.0, 160.0, 20.0, hex("#8a7f9d"));

    let mut lines = vec![
        "[Enter] Descend into the dungeon".to_string(),
        "[F9] Level creator".to_string(),
    ];
    if has_custom {
        lines.insert(1, "[L] Play your custom level".to_string());
    }
    for (i, l) in lines.iter().enumerate() {
        let ld = measure_text(l, None, 24, 1.0);
        draw_text(l, cx - ld.width / 2.0, 230.0 + i as f32 * 34.0, 24.0, WHITE);
    }

    let controls = [
        "WASD/arrows move · Space attack · 1-4 spells · E interact",
        "I inventory · V achievements · Esc menu",
        "",
        "The fire is free. The clinic is free. The lessons are free.",
        "Everything else, the goblins financialized. Go fix that.",
    ];
    for (i, l) in controls.iter().enumerate() {
        let ld = measure_text(l, None, 17, 1.0);
        draw_text(l, cx - ld.width / 2.0, 380.0 + i as f32 * 24.0, 17.0, hex("#8a7f9d"));
    }
}

pub fn draw_dead(world: &World) {
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.1, 0.0, 0.0, 0.75));
    let cx = screen_width() / 2.0;
    let t = "THE MARKET CLAIMED YOU";
    let td = measure_text(t, None, 48, 1.0);
    draw_text(t, cx - td.width / 2.0, 200.0, 48.0, hex("#ff5566"));
    let s = format!(
        "Depth {} · Level {} · {} exploiters defeated · {} gold in the strike fund",
        world.depth,
        world.player.level,
        world.stats.get("kills").copied().unwrap_or(0),
        world.player.gold
    );
    let sd = measure_text(&s, None, 20, 1.0);
    draw_text(&s, cx - sd.width / 2.0, 250.0, 20.0, WHITE);
    let m = "But the movement doesn't die with one member.";
    let md = measure_text(m, None, 20, 1.0);
    draw_text(m, cx - md.width / 2.0, 290.0, 20.0, hex("#8a7f9d"));
    let r = "[R] Rise again (achievements persist)   [Esc] Menu";
    let rd = measure_text(r, None, 22, 1.0);
    draw_text(r, cx - rd.width / 2.0, 350.0, 22.0, hex("#ffd24a"));
}
