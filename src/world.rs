use crate::content::Content;
use crate::dungeon::{self, CustomLevel, Map, Tile, TILE};
use macroquad::prelude::{vec2, Vec2};
use macroquad::rand::gen_range;
use std::collections::{HashMap, HashSet, VecDeque};

pub const EQUIP_SLOTS: [&str; 7] = ["weapon", "offhand", "head", "chest", "legs", "boots", "ring"];
pub const INV_CAP: usize = 24;

#[derive(Clone)]
pub struct ItemStack {
    pub id: String,
    pub qty: i32,
}

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub facing: Vec2,
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
    pub attack_cd: f32,
    pub swing: f32,
    pub hurt: f32,
    pub inventory: Vec<ItemStack>,
    pub equipment: HashMap<String, String>,
}

impl Player {
    pub fn new() -> Self {
        Player {
            x: 0.0,
            y: 0.0,
            facing: vec2(0.0, 1.0),
            hp: 40,
            mp: 20,
            base_maxhp: 40,
            base_maxmp: 20,
            base_atk: 3,
            base_def: 0,
            base_spd: 92.0,
            xp: 0,
            level: 1,
            gold: 25,
            attack_cd: 0.0,
            swing: 0.0,
            hurt: 0.0,
            inventory: vec![
                ItemStack { id: "bread".into(), qty: 2 },
                ItemStack { id: "rusty_shiv".into(), qty: 1 },
            ],
            equipment: HashMap::new(),
        }
    }

    /// (atk, def, maxhp, maxmp, spd) with equipment bonuses applied.
    pub fn totals(&self, c: &Content) -> (i32, i32, i32, i32, f32) {
        let mut atk = self.base_atk;
        let mut def = self.base_def;
        let mut hp = self.base_maxhp;
        let mut mp = self.base_maxmp;
        let mut spd = self.base_spd;
        for id in self.equipment.values() {
            if let Some(it) = c.item(id) {
                atk += it.atk;
                def += it.def;
                hp += it.hp;
                mp += it.mp;
                spd += it.spd as f32;
            }
        }
        (atk, def, hp, mp, spd)
    }

    pub fn xp_to_next(&self) -> i64 {
        (24.0 * (self.level as f64).powf(1.5)) as i64
    }

    pub fn add_item(&mut self, id: &str) -> bool {
        if let Some(st) = self.inventory.iter_mut().find(|s| s.id == id) {
            st.qty += 1;
            return true;
        }
        if self.inventory.len() >= INV_CAP {
            return false;
        }
        self.inventory.push(ItemStack { id: id.to_string(), qty: 1 });
        true
    }

    pub fn remove_item(&mut self, idx: usize) {
        if idx < self.inventory.len() {
            self.inventory[idx].qty -= 1;
            if self.inventory[idx].qty <= 0 {
                self.inventory.remove(idx);
            }
        }
    }
}

pub struct Mob {
    pub def_idx: usize,
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub maxhp: i32,
    pub atk: i32,
    pub def: i32,
    pub speed: f32,
    pub attack_cd: f32,
    pub say: String,
    pub say_t: f32,
    pub say_cd: f32,
    pub aggro: bool,
    pub hurt: f32,
    pub boss: bool,
}

pub struct Npc {
    pub def_idx: usize,
    pub x: f32,
    pub y: f32,
    pub say: String,
    pub say_t: f32,
}

pub struct Chest {
    pub tx: usize,
    pub ty: usize,
    pub opened: bool,
}

pub struct Drop {
    pub item: String,
    pub x: f32,
    pub y: f32,
    pub t: f32,
}

pub struct Fct {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub t: f32,
    pub color: (u8, u8, u8),
}

pub struct Burst {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub t: f32,
    pub color: String,
}

pub struct World {
    pub map: Map,
    pub player: Player,
    pub mobs: Vec<Mob>,
    pub npcs: Vec<Npc>,
    pub chests: Vec<Chest>,
    pub drops: Vec<Drop>,
    pub fcts: Vec<Fct>,
    pub bursts: Vec<Burst>,
    pub depth: i32,
    pub stats: HashMap<String, i64>,
    pub unlocked: HashSet<String>,
    pub toasts: VecDeque<(String, f32)>,
    pub shop_stock: Vec<String>,
    pub mp_regen: f32,
}

pub enum Interaction {
    Dialog { who: String, text: String },
    Shop,
    Descend,
    Rested,
    ChestLoot { text: String },
    None,
}

fn scale_hp(base: i32, depth: i32) -> i32 {
    (base as f32 * (1.0 + 0.13 * (depth - 1) as f32)) as i32
}
fn scale_atk(base: i32, depth: i32) -> i32 {
    (base as f32 * (1.0 + 0.09 * (depth - 1) as f32)) as i32
}

impl World {
    pub fn new(content: &Content) -> Self {
        let mut w = World {
            map: dungeon::generate(1, content.graffiti.len()),
            player: Player::new(),
            mobs: Vec::new(),
            npcs: Vec::new(),
            chests: Vec::new(),
            drops: Vec::new(),
            fcts: Vec::new(),
            bursts: Vec::new(),
            depth: 1,
            stats: HashMap::new(),
            unlocked: HashSet::new(),
            toasts: VecDeque::new(),
            shop_stock: Vec::new(),
            mp_regen: 0.0,
        };
        w.populate(content);
        w
    }

    pub fn descend(&mut self, content: &Content, custom: Option<&CustomLevel>) {
        self.depth += 1;
        self.load_level(content, custom);
        self.bump_stat_max("depth", self.depth as i64, content);
        self.toast(format!("Depth {} — the air smells of unpaid invoices.", self.depth));
    }

    pub fn load_level(&mut self, content: &Content, custom: Option<&CustomLevel>) {
        self.map = custom
            .and_then(|c| dungeon::from_custom(c, content.graffiti.len()))
            .unwrap_or_else(|| dungeon::generate(self.depth, content.graffiti.len()));
        self.mobs.clear();
        self.npcs.clear();
        self.chests.clear();
        self.drops.clear();
        self.fcts.clear();
        self.bursts.clear();
        self.populate(content);
    }

    fn populate(&mut self, content: &Content) {
        self.player.x = self.map.spawn.0;
        self.player.y = self.map.spawn.1;

        for &(tx, ty, tier) in &self.map.mob_spots {
            let pool: Vec<usize> = content
                .mobs
                .iter()
                .enumerate()
                .filter(|(_, m)| m.tier == tier)
                .map(|(i, _)| i)
                .collect();
            if pool.is_empty() {
                continue;
            }
            let def_idx = pool[gen_range(0, pool.len() as i32) as usize];
            let def = &content.mobs[def_idx];
            self.mobs.push(Mob {
                def_idx,
                x: (tx as f32 + 0.5) * TILE,
                y: (ty as f32 + 0.5) * TILE,
                hp: scale_hp(def.hp, self.depth),
                maxhp: scale_hp(def.hp, self.depth),
                atk: scale_atk(def.atk, self.depth),
                def: def.def,
                speed: def.speed,
                attack_cd: 0.0,
                say: String::new(),
                say_t: 0.0,
                say_cd: gen_range(1.0_f32, 5.0_f32),
                aggro: false,
                hurt: 0.0,
                boss: def.boss,
            });
        }

        // NPCs: shopkeeper first if this level has a shop, then random comrades
        let mut spots = self.map.npc_spots.clone();
        if self.map.has_shop {
            if let Some(sk) = content.npcs.iter().position(|n| n.shopkeeper) {
                if let Some((tx, ty)) = spots.pop() {
                    self.npcs.push(Npc {
                        def_idx: sk,
                        x: (tx as f32 + 0.5) * TILE,
                        y: (ty as f32 + 0.5) * TILE,
                        say: String::new(),
                        say_t: 0.0,
                    });
                }
            }
        }
        let civilians: Vec<usize> = content
            .npcs
            .iter()
            .enumerate()
            .filter(|(_, n)| !n.shopkeeper)
            .map(|(i, _)| i)
            .collect();
        for (tx, ty) in spots {
            if civilians.is_empty() {
                break;
            }
            let def_idx = civilians[gen_range(0, civilians.len() as i32) as usize];
            self.npcs.push(Npc {
                def_idx,
                x: (tx as f32 + 0.5) * TILE,
                y: (ty as f32 + 0.5) * TILE,
                say: String::new(),
                say_t: 0.0,
            });
        }

        for &(tx, ty) in &self.map.chest_spots {
            self.chests.push(Chest { tx, ty, opened: false });
        }

        // co-op stock: at-cost gear appropriate to the depth
        self.shop_stock.clear();
        if self.map.has_shop {
            let max_tier = 1 + self.depth / 2;
            let pool: Vec<&str> = content
                .items
                .iter()
                .filter(|i| i.tier <= max_tier)
                .map(|i| i.id.as_str())
                .collect();
            let mut guard = 0;
            while self.shop_stock.len() < 6.min(pool.len()) && guard < 100 {
                guard += 1;
                let pick = pool[gen_range(0, pool.len() as i32) as usize];
                if !self.shop_stock.iter().any(|s| s == pick) {
                    self.shop_stock.push(pick.to_string());
                }
            }
        }
    }

    pub fn toast(&mut self, text: String) {
        self.toasts.push_back((text, 4.0));
        if self.toasts.len() > 4 {
            self.toasts.pop_front();
        }
    }

    pub fn add_stat(&mut self, key: &str, amount: i64, content: &Content) {
        *self.stats.entry(key.to_string()).or_insert(0) += amount;
        self.check_achievements(content);
    }

    pub fn bump_stat_max(&mut self, key: &str, value: i64, content: &Content) {
        let e = self.stats.entry(key.to_string()).or_insert(0);
        if value > *e {
            *e = value;
        }
        self.check_achievements(content);
    }

    fn check_achievements(&mut self, content: &Content) {
        let mut newly: Vec<String> = Vec::new();
        for a in &content.achievements {
            if self.unlocked.contains(&a.id) {
                continue;
            }
            let v = self.stats.get(&a.stat).copied().unwrap_or(0);
            if v >= a.threshold {
                self.unlocked.insert(a.id.clone());
                newly.push(format!("🏆 Achievement: {} — {}", a.name, a.desc));
            }
        }
        for t in newly {
            self.toast(t);
        }
    }

    fn fct(&mut self, x: f32, y: f32, text: String, color: (u8, u8, u8)) {
        self.fcts.push(Fct { x, y, text, t: 1.0, color });
    }

    // ---------- per-frame simulation ----------

    pub fn update(&mut self, dt: f32, content: &Content, move_dir: Vec2, attack: bool, cast: Option<usize>) {
        let (atk_total, def_total, maxhp, maxmp, spd) = self.player.totals(content);
        self.player.hp = self.player.hp.min(maxhp);
        self.player.mp = self.player.mp.min(maxmp);

        // mp trickle
        self.mp_regen += dt;
        if self.mp_regen > 1.6 {
            self.mp_regen = 0.0;
            self.player.mp = (self.player.mp + 1).min(maxmp);
        }

        // movement, axis-separated so we slide along walls
        let hw = 5.0;
        if move_dir.length_squared() > 0.0 {
            let d = move_dir.normalize();
            self.player.facing = d;
            let nx = self.player.x + d.x * spd * dt;
            if self.map.box_free(nx, self.player.y, hw, hw) {
                self.player.x = nx;
            }
            let ny = self.player.y + d.y * spd * dt;
            if self.map.box_free(self.player.x, ny, hw, hw) {
                self.player.y = ny;
            }
        }

        self.player.attack_cd = (self.player.attack_cd - dt).max(0.0);
        self.player.swing = (self.player.swing - dt).max(0.0);
        self.player.hurt = (self.player.hurt - dt).max(0.0);

        // melee swing
        if attack && self.player.attack_cd <= 0.0 {
            self.player.attack_cd = 0.38;
            self.player.swing = 0.18;
            let reach = self.player.facing * 13.0;
            let px = self.player.x + reach.x;
            let py = self.player.y + reach.y;
            let mut hits: Vec<usize> = Vec::new();
            for (i, m) in self.mobs.iter().enumerate() {
                let dx = m.x - px;
                let dy = m.y - py;
                if dx * dx + dy * dy < 15.0 * 15.0 {
                    hits.push(i);
                }
            }
            for i in hits {
                let dmg = (atk_total - self.mobs[i].def + gen_range(0, 3)).max(1);
                self.damage_mob(i, dmg, content);
            }
        }

        // spells
        if let Some(slot) = cast {
            self.cast_spell(slot, content);
        }

        // mobs
        let px = self.player.x;
        let py = self.player.y;
        let mut dmg_to_player = 0;
        for m in &mut self.mobs {
            let def = &content.mobs[m.def_idx];
            m.attack_cd = (m.attack_cd - dt).max(0.0);
            m.hurt = (m.hurt - dt).max(0.0);
            m.say_t = (m.say_t - dt).max(0.0);
            let dx = px - m.x;
            let dy = py - m.y;
            let dist2 = dx * dx + dy * dy;
            let aggro_r = if m.boss { TILE * 12.0 } else { TILE * 7.5 };
            m.aggro = dist2 < aggro_r * aggro_r && !self.map.tile_at_px(px, py).safe();

            if m.aggro {
                // capitalist propaganda broadcast
                m.say_cd -= dt;
                if m.say_cd <= 0.0 && !def.lines.is_empty() {
                    m.say = def.lines[gen_range(0, def.lines.len() as i32) as usize].clone();
                    m.say_t = 3.0;
                    m.say_cd = gen_range(4.0_f32, 9.0_f32);
                }

                let dist = dist2.sqrt().max(0.001);
                if dist > 12.0 {
                    let step = m.speed * dt;
                    let sx = m.x + dx / dist * step;
                    let sy = m.y + dy / dist * step;
                    let mhw = 5.0;
                    if self.map.box_free(sx, m.y, mhw, mhw) && !self.map.tile_at_px(sx, m.y).safe() {
                        m.x = sx;
                    }
                    if self.map.box_free(m.x, sy, mhw, mhw) && !self.map.tile_at_px(m.x, sy).safe() {
                        m.y = sy;
                    }
                }
                if dist < 15.0 && m.attack_cd <= 0.0 {
                    m.attack_cd = 0.9;
                    let dmg = (m.atk - def_total / 2 + gen_range(0, 2)).max(1);
                    dmg_to_player += dmg;
                }
            }
        }
        if dmg_to_player > 0 {
            self.player.hp -= dmg_to_player;
            self.player.hurt = 0.25;
            let (x, y) = (self.player.x, self.player.y);
            self.fct(x, y - 10.0, format!("-{}", dmg_to_player), (255, 80, 80));
        }

        // pick up drops by walking over them
        let mut picked: Vec<usize> = Vec::new();
        for (i, d) in self.drops.iter_mut().enumerate() {
            d.t += dt;
            let dx = d.x - px;
            let dy = d.y - py;
            if dx * dx + dy * dy < 12.0 * 12.0 {
                picked.push(i);
            }
        }
        for i in picked.into_iter().rev() {
            let id = self.drops[i].item.clone();
            if self.player.add_item(&id) {
                let name = content.item(&id).map(|d| d.name.clone()).unwrap_or(id.clone());
                self.toast(format!("Picked up: {}", name));
                self.drops.remove(i);
            }
        }

        // effects decay
        for f in &mut self.fcts {
            f.t -= dt;
            f.y -= 18.0 * dt;
        }
        self.fcts.retain(|f| f.t > 0.0);
        for b in &mut self.bursts {
            b.t -= dt;
        }
        self.bursts.retain(|b| b.t > 0.0);
        for n in &mut self.npcs {
            n.say_t = (n.say_t - dt).max(0.0);
        }
        if let Some(front) = self.toasts.front_mut() {
            front.1 -= dt;
            if front.1 <= 0.0 {
                self.toasts.pop_front();
            }
        }

        // xp already applied on kill; check level-ups
        while self.player.xp >= self.player.xp_to_next() {
            self.player.xp -= self.player.xp_to_next();
            self.player.level += 1;
            self.player.base_maxhp += 8;
            self.player.base_maxmp += 4;
            self.player.base_atk += 1;
            if self.player.level % 3 == 0 {
                self.player.base_def += 1;
            }
            let (_, _, mh, mm, _) = self.player.totals(content);
            self.player.hp = mh;
            self.player.mp = mm;
            self.toast(format!(
                "Level {}! The movement grows stronger.",
                self.player.level
            ));
            self.bump_stat_max("level", self.player.level as i64, content);
            for s in &content.spells {
                if s.unlock_level == self.player.level {
                    self.toast(format!("✨ New spell: {} — {}", s.name, s.desc));
                }
            }
        }
        self.bump_stat_max("gold", self.player.gold, content);
    }

    fn damage_mob(&mut self, i: usize, dmg: i32, content: &Content) {
        if i >= self.mobs.len() {
            return;
        }
        self.mobs[i].hp -= dmg;
        self.mobs[i].hurt = 0.2;
        let (mx, my) = (self.mobs[i].x, self.mobs[i].y);
        self.fct(mx, my - 10.0, format!("{}", dmg), (255, 230, 120));
        if self.mobs[i].hp <= 0 {
            self.kill_mob(i, content);
        }
    }

    fn kill_mob(&mut self, i: usize, content: &Content) {
        let m = self.mobs.remove(i);
        let def = content.mobs[m.def_idx].clone();
        let gold = if def.gold_max > def.gold_min {
            gen_range(def.gold_min, def.gold_max + 1) as i64
        } else {
            def.gold_min as i64
        };
        self.player.gold += gold;
        self.player.xp += def.xp as i64;
        if gold > 0 {
            self.fct(m.x, m.y - 4.0, format!("+{}g", gold), (255, 210, 74));
        }
        for d in &def.drops {
            if gen_range(0.0_f32, 1.0_f32) < d.chance {
                self.drops.push(Drop {
                    item: d.item.clone(),
                    x: m.x + gen_range(-6.0_f32, 6.0_f32),
                    y: m.y + gen_range(-6.0_f32, 6.0_f32),
                    t: 0.0,
                });
            }
        }
        self.add_stat("kills", 1, content);
        if def.boss {
            self.add_stat("bosses", 1, content);
            self.toast(format!("{} liquidated. The dungeon breathes easier.", def.name));
        }
    }

    fn cast_spell(&mut self, slot: usize, content: &Content) {
        let known: Vec<usize> = content
            .spells
            .iter()
            .enumerate()
            .filter(|(_, s)| s.unlock_level <= self.player.level)
            .map(|(i, _)| i)
            .collect();
        let Some(&si) = known.get(slot) else { return };
        let spell = content.spells[si].clone();
        if self.player.mp < spell.cost {
            self.toast(format!("Not enough MP for {}.", spell.name));
            return;
        }

        let px = self.player.x;
        let py = self.player.y;

        if spell.damage > 0 && spell.radius <= 0.0 {
            // single target: nearest mob in range
            let range = spell.range * TILE;
            let mut best: Option<(usize, f32)> = None;
            for (i, m) in self.mobs.iter().enumerate() {
                let d2 = (m.x - px).powi(2) + (m.y - py).powi(2);
                if d2 < range * range && best.map(|(_, bd)| d2 < bd).unwrap_or(true) {
                    best = Some((i, d2));
                }
            }
            let Some((ti, _)) = best else {
                self.toast("No exploiter in range.".to_string());
                return;
            };
            self.player.mp -= spell.cost;
            let (tx, ty) = (self.mobs[ti].x, self.mobs[ti].y);
            self.bursts.push(Burst { x: tx, y: ty, radius: 14.0, t: 0.3, color: spell.color.clone() });
            self.damage_mob(ti, spell.damage, content);
        } else if spell.damage > 0 && spell.radius > 0.0 {
            self.player.mp -= spell.cost;
            let r = spell.radius * TILE;
            self.bursts.push(Burst { x: px, y: py, radius: r, t: 0.45, color: spell.color.clone() });
            let hits: Vec<usize> = self
                .mobs
                .iter()
                .enumerate()
                .filter(|(_, m)| (m.x - px).powi(2) + (m.y - py).powi(2) < r * r)
                .map(|(i, _)| i)
                .collect();
            for i in hits.into_iter().rev() {
                self.damage_mob(i, spell.damage, content);
            }
        } else if spell.heal > 0 {
            self.player.mp -= spell.cost;
        } else {
            return;
        }

        if spell.heal > 0 {
            let (_, _, maxhp, _, _) = self.player.totals(content);
            self.player.hp = (self.player.hp + spell.heal).min(maxhp);
            self.bursts.push(Burst { x: px, y: py, radius: 16.0, t: 0.4, color: "#4fdc7f".to_string() });
            self.fct(px, py - 12.0, format!("+{}", spell.heal), (100, 240, 140));
        }
    }

    // ---------- interactions (E key) ----------

    pub fn interact(&mut self, content: &Content) -> Interaction {
        let px = self.player.x;
        let py = self.player.y;

        // stairs
        if self.map.tile_at_px(px, py) == Tile::Stairs {
            return Interaction::Descend;
        }

        // campfire rest
        let ptx = (px / TILE) as i32;
        let pty = (py / TILE) as i32;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if self.map.tile(ptx + dx, pty + dy) == Tile::Campfire {
                    let (_, _, maxhp, maxmp, _) = self.player.totals(content);
                    self.player.hp = maxhp;
                    self.player.mp = maxmp;
                    self.add_stat("rests", 1, content);
                    return Interaction::Rested;
                }
            }
        }

        // npc
        let mut best_npc: Option<(usize, f32)> = None;
        for (i, n) in self.npcs.iter().enumerate() {
            let d2 = (n.x - px).powi(2) + (n.y - py).powi(2);
            if d2 < (TILE * 1.6).powi(2) && best_npc.map(|(_, bd)| d2 < bd).unwrap_or(true) {
                best_npc = Some((i, d2));
            }
        }
        if let Some((i, _)) = best_npc {
            let def = &content.npcs[self.npcs[i].def_idx];
            if def.shopkeeper && self.map.has_shop {
                return Interaction::Shop;
            }
            let line = if def.lines.is_empty() {
                "Solidarity, friend.".to_string()
            } else {
                def.lines[gen_range(0, def.lines.len() as i32) as usize].clone()
            };
            self.npcs[i].say = line.clone();
            self.npcs[i].say_t = 4.0;
            self.add_stat("talks", 1, content);
            return Interaction::Dialog { who: def.name.clone(), text: line };
        }

        // chest
        for ci in 0..self.chests.len() {
            let c = &self.chests[ci];
            if c.opened {
                continue;
            }
            let cx = (c.tx as f32 + 0.5) * TILE;
            let cy = (c.ty as f32 + 0.5) * TILE;
            if (cx - px).powi(2) + (cy - py).powi(2) < (TILE * 1.5).powi(2) {
                self.chests[ci].opened = true;
                let gold = (gen_range(8, 20) + self.depth * 4) as i64;
                self.player.gold += gold;
                let mut text = format!("+{} gold for the strike fund", gold);
                if gen_range(0, 100) < 55 {
                    let max_tier = 1 + self.depth / 2;
                    let pool: Vec<&str> = content
                        .items
                        .iter()
                        .filter(|i| i.tier <= max_tier)
                        .map(|i| i.id.as_str())
                        .collect();
                    if !pool.is_empty() {
                        let id = pool[gen_range(0, pool.len() as i32) as usize].to_string();
                        if self.player.add_item(&id) {
                            let name = content.item(&id).map(|d| d.name.clone()).unwrap_or(id);
                            text = format!("{}, and: {}", text, name);
                        }
                    }
                }
                self.add_stat("chests", 1, content);
                return Interaction::ChestLoot { text };
            }
        }

        // graffiti on adjacent walls
        let mut found_graffiti: Option<usize> = None;
        for (gi, g) in self.map.graffiti.iter().enumerate() {
            let gx = (g.x as f32 + 0.5) * TILE;
            let gy = (g.y as f32 + 0.5) * TILE;
            if (gx - px).powi(2) + (gy - py).powi(2) < (TILE * 2.0).powi(2) {
                found_graffiti = Some(gi);
                break;
            }
        }
        if let Some(gi) = found_graffiti {
            let idx = self.map.graffiti[gi].text_idx;
            let text = content
                .graffiti
                .get(idx)
                .cloned()
                .unwrap_or_else(|| "SOLIDARITY".to_string());
            let key = format!("graffiti_seen_{}", idx);
            if self.stats.get(&key).copied().unwrap_or(0) == 0 {
                self.stats.insert(key, 1);
                self.add_stat("graffiti", 1, content);
            }
            return Interaction::Dialog { who: "Graffiti on the wall".to_string(), text };
        }

        Interaction::None
    }
}
