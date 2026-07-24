use crate::content::Content;
use crate::dungeon::{self, CustomLevel, Map, Tile, TILE};
use macroquad::prelude::{vec2, Vec2};
use macroquad::rand::gen_range;
use std::collections::{HashMap, HashSet, VecDeque};

pub const EQUIP_SLOTS: [&str; 7] = ["weapon", "offhand", "head", "chest", "legs", "boots", "ring"];
pub const INV_CAP: usize = 24;

/// A concrete item: base definition plus optional rolled affixes.
#[derive(Clone, PartialEq, Eq)]
pub struct ItemInst {
    pub id: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

impl ItemInst {
    pub fn plain(id: &str) -> Self {
        ItemInst { id: id.to_string(), prefix: None, suffix: None }
    }
    pub fn has_affix(&self) -> bool {
        self.prefix.is_some() || self.suffix.is_some()
    }
    /// 0 = common, 1 = uncommon (one affix), 2 = rare (both affixes)
    pub fn rarity(&self) -> u8 {
        self.prefix.is_some() as u8 + self.suffix.is_some() as u8
    }
}

/// Full display name, e.g. "Vicious Rusty Shiv of the Commune".
pub fn display_name(c: &Content, inst: &ItemInst) -> String {
    let base = c.item(&inst.id).map(|d| d.name.clone()).unwrap_or_else(|| inst.id.clone());
    let mut name = String::new();
    if let Some(p) = inst.prefix.as_deref().and_then(|p| c.prefix(p)) {
        name.push_str(&p.name);
        name.push(' ');
    }
    name.push_str(&base);
    if let Some(s) = inst.suffix.as_deref().and_then(|s| c.suffix(s)) {
        name.push(' ');
        name.push_str(&s.name);
    }
    name
}

/// (atk, def, hp, mp, spd) including affix bonuses.
pub fn inst_bonus(c: &Content, inst: &ItemInst) -> (i32, i32, i32, i32, i32) {
    let mut atk = 0;
    let mut def = 0;
    let mut hp = 0;
    let mut mp = 0;
    let mut spd = 0;
    if let Some(d) = c.item(&inst.id) {
        atk += d.atk;
        def += d.def;
        hp += d.hp;
        mp += d.mp;
        spd += d.spd;
    }
    for a in [
        inst.prefix.as_deref().and_then(|p| c.prefix(p)),
        inst.suffix.as_deref().and_then(|s| c.suffix(s)),
    ]
    .into_iter()
    .flatten()
    {
        atk += a.stats.atk;
        def += a.stats.def;
        hp += a.stats.hp;
        mp += a.stats.mp;
        spd += a.stats.spd;
    }
    (atk, def, hp, mp, spd)
}

/// Gold value including affix multipliers.
pub fn inst_value(c: &Content, inst: &ItemInst) -> i32 {
    let base = c.item(&inst.id).map(|d| d.value).unwrap_or(0) as f32;
    let mut mult = 1.0;
    if let Some(p) = inst.prefix.as_deref().and_then(|p| c.prefix(p)) {
        mult *= p.value_mult.max(1.0);
    }
    if let Some(s) = inst.suffix.as_deref().and_then(|s| c.suffix(s)) {
        mult *= s.value_mult.max(1.0);
    }
    (base * mult) as i32
}

/// Weapon class handling profile: (attack cooldown, reach px, hit radius px, knockback mult).
pub fn weapon_profile(wclass: &str) -> (f32, f32, f32, f32) {
    match wclass {
        "dagger" => (0.22, 12.0, 14.0, 0.7),
        "hammer" => (0.55, 14.0, 18.0, 1.9),
        "spear" => (0.38, 22.0, 13.0, 1.0),
        "scythe" => (0.45, 11.0, 24.0, 1.2),
        "sword" => (0.32, 14.0, 17.0, 1.0),
        _ => (0.30, 12.0, 15.0, 0.8), // bare fists
    }
}

#[derive(Clone)]
pub struct ItemStack {
    pub inst: ItemInst,
    pub qty: i32,
}

/// Sound/feedback events; the main loop drains these and plays SFX.
pub enum GameEvent {
    Swing,
    Hit,
    Kill,
    Hurt,
    Pickup,
    LevelUp,
    Cast,
    Chest,
    Rest,
    Dash,
    Recruit,
    Quest,
    Boom,
}

/// Commune upgrades purchasable at the campfire: (id, name, desc, cost).
pub const COMMUNE_UPGRADES: [(&str, &str, &str, i64); 4] = [
    ("clinic_beds", "Clinic Beds", "+10 max HP, effective immediately. Healthcare infrastructure works.", 120),
    ("free_library", "Free Library", "Spells cost 10% less. Knowledge compounds better than interest.", 100),
    ("communal_forge", "Communal Forge", "Forge rerolls cost 25% less. Shared tools, shared savings.", 80),
    ("soup_kitchen", "Soup Kitchen", "Resting grants Well Fed: +2 ATK for 60 seconds.", 60),
];

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub facing: Vec2,
    pub kx: f32,
    pub ky: f32,
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
    pub dodge_t: f32,
    pub dodge_cd: f32,
    pub poison_t: f32,
    pub slow_t: f32,
    pub poison_tick: f32,
    pub skill_points: i32,
    pub skills: Vec<String>,
    pub inventory: Vec<ItemStack>,
    pub equipment: HashMap<String, ItemInst>,
}

impl Player {
    pub fn new() -> Self {
        Player {
            x: 0.0,
            y: 0.0,
            facing: vec2(0.0, 1.0),
            kx: 0.0,
            ky: 0.0,
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
            dodge_t: 0.0,
            dodge_cd: 0.0,
            poison_t: 0.0,
            slow_t: 0.0,
            poison_tick: 0.0,
            skill_points: 0,
            skills: Vec::new(),
            inventory: vec![
                ItemStack { inst: ItemInst::plain("bread"), qty: 2 },
                ItemStack { inst: ItemInst::plain("rusty_shiv"), qty: 1 },
            ],
            equipment: HashMap::new(),
        }
    }

    /// Sum of special-affix powers (lifesteal/thorns/echo) across equipped gear.
    pub fn special_sum(&self, c: &Content, key: &str) -> f32 {
        let mut total = 0.0;
        for inst in self.equipment.values() {
            for a in [
                inst.prefix.as_deref().and_then(|p| c.prefix(p)),
                inst.suffix.as_deref().and_then(|s| c.suffix(s)),
            ]
            .into_iter()
            .flatten()
            {
                if a.special == key {
                    total += a.power;
                }
            }
        }
        total
    }

    /// Sum of learned-skill bonuses for a given stat key.
    pub fn skill_sum(&self, c: &Content, stat: &str) -> f32 {
        c.skills
            .iter()
            .filter(|s| s.stat == stat && self.skills.iter().any(|id| id == &s.id))
            .map(|s| s.amount)
            .sum()
    }

    /// (atk, def, maxhp, maxmp, spd) with equipment and skill bonuses applied.
    pub fn totals(&self, c: &Content) -> (i32, i32, i32, i32, f32) {
        let mut atk = self.base_atk;
        let mut def = self.base_def;
        let mut hp = self.base_maxhp;
        let mut mp = self.base_maxmp;
        let mut spd = self.base_spd;
        for inst in self.equipment.values() {
            let (a, d, h, m, s) = inst_bonus(c, inst);
            atk += a;
            def += d;
            hp += h;
            mp += m;
            spd += s as f32;
        }
        atk += self.skill_sum(c, "atk") as i32;
        def += self.skill_sum(c, "def") as i32;
        hp += self.skill_sum(c, "hp") as i32;
        mp += self.skill_sum(c, "mp") as i32;
        spd += self.skill_sum(c, "spd");
        (atk, def, hp, mp, spd)
    }

    pub fn xp_to_next(&self) -> i64 {
        (24.0 * (self.level as f64).powf(1.5)) as i64
    }

    pub fn add_item(&mut self, inst: &ItemInst) -> bool {
        if let Some(st) = self.inventory.iter_mut().find(|s| s.inst == *inst) {
            st.qty += 1;
            return true;
        }
        if self.inventory.len() >= INV_CAP {
            return false;
        }
        self.inventory.push(ItemStack { inst: inst.clone(), qty: 1 });
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
    pub kx: f32,
    pub ky: f32,
    pub hp: i32,
    pub maxhp: i32,
    pub atk: i32,
    pub def: i32,
    pub speed: f32,
    pub attack_cd: f32,
    pub windup: f32,
    pub say: String,
    pub say_t: f32,
    pub say_cd: f32,
    pub aggro: bool,
    pub hurt: f32,
    pub boss: bool,
    pub burn_acc: f32,
}

pub struct Npc {
    pub def_idx: usize,
    pub x: f32,
    pub y: f32,
    pub home_x: f32,
    pub home_y: f32,
    pub wx: f32,
    pub wy: f32,
    pub wander_t: f32,
    pub say: String,
    pub say_t: f32,
}

/// Comrades out in the wild, fighting the good fight against the mobs.
pub struct Fighter {
    pub def_idx: usize,
    pub x: f32,
    pub y: f32,
    pub kx: f32,
    pub ky: f32,
    pub hp: i32,
    pub maxhp: i32,
    pub atk: i32,
    pub attack_cd: f32,
    pub say: String,
    pub say_t: f32,
    pub say_cd: f32,
    pub hurt: f32,
    pub engaged: bool,
    pub recruited: bool,
}

pub struct Chest {
    pub tx: usize,
    pub ty: usize,
    pub opened: bool,
    /// 0 = plain dungeon chest; 1-3 = bronze/silver/gold sponsor crates
    pub tier: u8,
}

/// A thrown stick of dynamite or flask of oil, mid-flight.
pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub fuse: f32,
    pub oil: bool,
}

pub struct OilPatch {
    pub tx: i32,
    pub ty: i32,
    pub lit: bool,
    pub life: f32,
}

pub struct Drop {
    pub item: ItemInst,
    pub x: f32,
    pub y: f32,
    pub t: f32,
}

pub struct Fct {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub t: f32,
    pub big: bool,
    pub color: (u8, u8, u8),
}

pub struct Burst {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub t: f32,
    pub color: String,
}

pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub t: f32,
    pub color: (u8, u8, u8),
}

#[derive(Clone, Copy, PartialEq)]
pub enum LogKind {
    Enemy,
    Comrade,
    System,
    Broadcast,
}

pub struct LogEntry {
    pub who: String,
    pub text: String,
    pub kind: LogKind,
}

pub struct World {
    pub map: Map,
    pub player: Player,
    pub mobs: Vec<Mob>,
    pub npcs: Vec<Npc>,
    pub fighters: Vec<Fighter>,
    pub chests: Vec<Chest>,
    pub drops: Vec<Drop>,
    pub fcts: Vec<Fct>,
    pub bursts: Vec<Burst>,
    pub particles: Vec<Particle>,
    pub events: Vec<GameEvent>,
    pub depth: i32,
    pub stats: HashMap<String, i64>,
    pub unlocked: HashSet<String>,
    pub toasts: VecDeque<(String, f32)>,
    pub log: VecDeque<LogEntry>,
    pub shop_stock: Vec<String>,
    pub mp_regen: f32,
    // game feel
    pub shake: f32,
    pub slowmo: f32,
    // npc ambient chatter
    pub banter_cd: f32,
    pub pending_reply: Option<(usize, String, f32)>,
    // quests: (quest id, stat value when accepted)
    pub quests_active: Vec<(String, i64)>,
    pub quests_done: HashSet<String>,
    // commune upgrades owned this run
    pub commune: Vec<String>,
    pub well_fed_t: f32,
    // daily-run determinism; None for normal runs
    pub fixed_seed: Option<u64>,
    // the broadcast: viewer metrics, System announcements, floor pressure
    pub projectiles: Vec<Projectile>,
    pub oil: Vec<OilPatch>,
    pub viewers: f64,
    pub hype: f32,
    pub last_sponsor: f64,
    pub mail_cd: f32,
    pub floor_time: f32,
    pub floor_warned: bool,
    pub wave_cd: f32,
    pub barefoot_cd: f32,
    pub announce: Option<(String, f32)>,
    pub last_hit_by: String,
    pub obituary: String,
    pub player_burn_acc: f32,
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
            fighters: Vec::new(),
            chests: Vec::new(),
            drops: Vec::new(),
            fcts: Vec::new(),
            bursts: Vec::new(),
            particles: Vec::new(),
            events: Vec::new(),
            depth: 1,
            stats: HashMap::new(),
            unlocked: HashSet::new(),
            toasts: VecDeque::new(),
            log: VecDeque::new(),
            shop_stock: Vec::new(),
            mp_regen: 0.0,
            shake: 0.0,
            slowmo: 0.0,
            banter_cd: 5.0,
            pending_reply: None,
            quests_active: Vec::new(),
            quests_done: HashSet::new(),
            commune: Vec::new(),
            well_fed_t: 0.0,
            fixed_seed: None,
            projectiles: Vec::new(),
            oil: Vec::new(),
            viewers: 42_000.0,
            hype: 0.0,
            last_sponsor: 42_000.0,
            mail_cd: 45.0,
            floor_time: 0.0,
            floor_warned: false,
            wave_cd: 0.0,
            barefoot_cd: 60.0,
            announce: None,
            last_hit_by: String::new(),
            obituary: String::new(),
            player_burn_acc: 0.0,
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
        // daily runs: deterministic layout per depth, same dungeon for everyone
        if let Some(seed) = self.fixed_seed {
            macroquad::rand::srand(seed.wrapping_mul(1000).wrapping_add(self.depth as u64));
        }
        // recruited comrades descend with you
        let mut squad: Vec<Fighter> = Vec::new();
        for f in self.fighters.drain(..) {
            if f.recruited {
                squad.push(f);
            }
        }
        self.map = custom
            .and_then(|c| dungeon::from_custom(c, content.graffiti.len()))
            .unwrap_or_else(|| dungeon::generate(self.depth, content.graffiti.len()));
        self.mobs.clear();
        self.npcs.clear();
        self.fighters.clear();
        self.chests.clear();
        self.drops.clear();
        self.fcts.clear();
        self.bursts.clear();
        self.particles.clear();
        self.pending_reply = None;
        self.projectiles.clear();
        self.oil.clear();
        self.floor_time = 0.0;
        self.floor_warned = false;
        self.wave_cd = 0.0;
        self.populate(content);
        for (i, mut f) in squad.into_iter().enumerate() {
            f.x = self.map.spawn.0 + (i as f32 + 1.0) * 12.0;
            f.y = self.map.spawn.1 + 10.0;
            f.say = String::new();
            f.say_t = 0.0;
            f.kx = 0.0;
            f.ky = 0.0;
            // a breather between floors
            f.hp = (f.hp + f.maxhp / 4).min(f.maxhp);
            self.fighters.push(f);
        }
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
                kx: 0.0,
                ky: 0.0,
                hp: scale_hp(def.hp, self.depth),
                maxhp: scale_hp(def.hp, self.depth),
                atk: scale_atk(def.atk, self.depth),
                def: def.def,
                speed: def.speed,
                attack_cd: 0.0,
                windup: 0.0,
                say: String::new(),
                say_t: 0.0,
                say_cd: gen_range(1.0_f32, 5.0_f32),
                aggro: false,
                hurt: 0.0,
                boss: def.boss,
                burn_acc: 0.0,
            });
        }

        // NPCs: shopkeeper first if this level has a shop, then random comrades
        let mut spots = self.map.npc_spots.clone();
        if self.map.has_shop {
            if let Some(sk) = content.npcs.iter().position(|n| n.shopkeeper) {
                if let Some((tx, ty)) = spots.pop() {
                    self.spawn_npc(sk, tx, ty);
                }
            }
        }
        let civilians: Vec<usize> = content
            .npcs
            .iter()
            .enumerate()
            .filter(|(_, n)| !n.shopkeeper && !n.fighter)
            .map(|(i, _)| i)
            .collect();
        for (tx, ty) in spots {
            if civilians.is_empty() {
                break;
            }
            let def_idx = civilians[gen_range(0, civilians.len() as i32) as usize];
            self.spawn_npc(def_idx, tx, ty);
        }

        // comrade fighters out in the wild, holding the line
        let fighter_defs: Vec<usize> = content
            .npcs
            .iter()
            .enumerate()
            .filter(|(_, n)| n.fighter)
            .map(|(i, _)| i)
            .collect();
        if !fighter_defs.is_empty() && self.map.rooms.len() > 2 {
            let n_fighters = gen_range(1, 4);
            for _ in 0..n_fighters {
                let ri = gen_range(1, self.map.rooms.len() as i32) as usize;
                let r = self.map.rooms[ri];
                let (cx, cy) = r.center();
                let def_idx = fighter_defs[gen_range(0, fighter_defs.len() as i32) as usize];
                let def = &content.npcs[def_idx];
                let hp = scale_hp(def.hp.max(40), self.depth);
                self.fighters.push(Fighter {
                    def_idx,
                    x: (cx as f32 + gen_range(-1.0_f32, 1.0_f32)) * TILE,
                    y: (cy as f32 + gen_range(-1.0_f32, 1.0_f32)) * TILE,
                    kx: 0.0,
                    ky: 0.0,
                    hp,
                    maxhp: hp,
                    atk: scale_atk(def.atk.max(5), self.depth),
                    attack_cd: 0.0,
                    say: String::new(),
                    say_t: 0.0,
                    say_cd: gen_range(2.0_f32, 6.0_f32),
                    hurt: 0.0,
                    engaged: false,
                    recruited: false,
                });
            }
        }

        for &(tx, ty) in &self.map.chest_spots {
            self.chests.push(Chest { tx, ty, opened: false, tier: 0 });
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
            while self.shop_stock.len() < 8.min(pool.len()) && guard < 200 {
                guard += 1;
                let pick = pool[gen_range(0, pool.len() as i32) as usize];
                if !self.shop_stock.iter().any(|s| s == pick) {
                    self.shop_stock.push(pick.to_string());
                }
            }
        }
    }

    fn spawn_npc(&mut self, def_idx: usize, tx: usize, ty: usize) {
        let x = (tx as f32 + 0.5) * TILE;
        let y = (ty as f32 + 0.5) * TILE;
        self.npcs.push(Npc {
            def_idx,
            x,
            y,
            home_x: x,
            home_y: y,
            wx: x,
            wy: y,
            wander_t: gen_range(2.0_f32, 8.0_f32),
            say: String::new(),
            say_t: 0.0,
        });
    }

    pub fn toast(&mut self, text: String) {
        self.log_push(LogKind::System, "", &text);
        self.toasts.push_back((text, 4.0));
        if self.toasts.len() > 4 {
            self.toasts.pop_front();
        }
    }

    pub fn log_push(&mut self, kind: LogKind, who: &str, text: &str) {
        self.log.push_back(LogEntry {
            who: who.to_string(),
            text: text.to_string(),
            kind,
        });
        if self.log.len() > 300 {
            self.log.pop_front();
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
                newly.push(format!("Achievement: {} — {}", a.name, a.desc));
            }
        }
        for t in newly {
            // every achievement ships with a Bronze Box, courtesy of the System
            self.player.gold += 25;
            self.hype += 0.2;
            self.toast(format!("{} [Bronze Box: +25 gold]", t));
        }
    }

    fn fct(&mut self, x: f32, y: f32, text: String, big: bool, color: (u8, u8, u8)) {
        self.fcts.push(Fct { x, y, text, t: 1.0, big, color });
    }

    fn spawn_particles(&mut self, x: f32, y: f32, n: usize, color: (u8, u8, u8)) {
        for _ in 0..n {
            let a = gen_range(0.0_f32, std::f32::consts::TAU);
            let sp = gen_range(30.0_f32, 90.0_f32);
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp - 20.0,
                t: gen_range(0.25_f32, 0.55_f32),
                color,
            });
        }
    }

    // ---------- per-frame simulation ----------

    pub fn update(&mut self, real_dt: f32, content: &Content, move_dir: Vec2, attack: bool, cast: Option<usize>, dodge: bool) {
        // hit-stop: the world runs in slow motion for a few frames after impact
        let dt = if self.slowmo > 0.0 { real_dt * 0.18 } else { real_dt };
        self.slowmo = (self.slowmo - real_dt).max(0.0);
        self.shake = (self.shake - real_dt * 22.0).max(0.0);

        let (atk_total, def_total, maxhp, maxmp, spd) = self.player.totals(content);
        // slow drags your feet; poison ticks
        let spd = if self.player.slow_t > 0.0 { spd * 0.6 } else { spd };
        self.player.slow_t = (self.player.slow_t - dt).max(0.0);
        self.player.poison_t = (self.player.poison_t - dt).max(0.0);
        if self.player.poison_t > 0.0 {
            self.player.poison_tick += dt;
            if self.player.poison_tick >= 0.8 {
                self.player.poison_tick = 0.0;
                self.player.hp -= 2;
                self.last_hit_by = "compound interest (poison)".to_string();
                let (x, y) = (self.player.x, self.player.y);
                self.fct(x, y - 10.0, "-2".to_string(), false, (140, 220, 90));
            }
        }
        self.player.hp = self.player.hp.min(maxhp);
        self.player.mp = self.player.mp.min(maxmp);

        // mp trickle (Deep Breath speeds it up)
        self.mp_regen += dt;
        let regen_interval = 1.6 * (1.0 - self.player.skill_sum(content, "mana_regen") / 100.0).max(0.3);
        if self.mp_regen > regen_interval {
            self.mp_regen = 0.0;
            self.player.mp = (self.player.mp + 1).min(maxmp);
        }

        // movement + knockback, axis-separated so we slide along walls
        let hw = 5.0;
        let mut vx = 0.0;
        let mut vy = 0.0;
        if move_dir.length_squared() > 0.0 {
            let d = move_dir.normalize();
            self.player.facing = d;
            vx = d.x * spd;
            vy = d.y * spd;
        }
        vx += self.player.kx;
        vy += self.player.ky;
        self.player.kx *= 1.0 - (8.0 * dt).min(1.0);
        self.player.ky *= 1.0 - (8.0 * dt).min(1.0);
        let nx = self.player.x + vx * dt;
        if self.map.box_free(nx, self.player.y, hw, hw) {
            self.player.x = nx;
        }
        let ny = self.player.y + vy * dt;
        if self.map.box_free(self.player.x, ny, hw, hw) {
            self.player.y = ny;
        }

        self.player.attack_cd = (self.player.attack_cd - dt).max(0.0);
        self.player.swing = (self.player.swing - dt).max(0.0);
        self.player.hurt = (self.player.hurt - dt).max(0.0);
        self.player.dodge_t = (self.player.dodge_t - dt).max(0.0);
        self.player.dodge_cd = (self.player.dodge_cd - dt).max(0.0);
        self.well_fed_t = (self.well_fed_t - dt).max(0.0);

        // dodge-roll: a burst of speed with invulnerability frames
        if dodge && self.player.dodge_cd <= 0.0 {
            self.player.dodge_cd = 0.75 * (1.0 - self.player.skill_sum(content, "dodge") / 100.0).max(0.3);
            self.player.dodge_t = 0.24;
            let dir = if move_dir.length_squared() > 0.0 {
                move_dir.normalize()
            } else {
                self.player.facing
            };
            self.player.kx += dir.x * 300.0;
            self.player.ky += dir.y * 300.0;
            self.events.push(GameEvent::Dash);
            self.add_stat("dodges", 1, content);
        }
        // ghost trail while rolling
        if self.player.dodge_t > 0.0 {
            let (x, y) = (self.player.x, self.player.y);
            self.particles.push(Particle {
                x,
                y,
                vx: 0.0,
                vy: 0.0,
                t: 0.25,
                color: (140, 180, 255),
            });
        }

        // melee swing: weapon class sets speed, reach, arc width, and knockback
        let wclass = self
            .player
            .equipment
            .get("weapon")
            .and_then(|i| content.item(&i.id))
            .map(|d| d.wclass.clone())
            .unwrap_or_default();
        let (w_cd, w_reach, w_radius, w_kb) = weapon_profile(&wclass);
        if attack && self.player.attack_cd <= 0.0 {
            self.player.attack_cd = w_cd;
            self.player.swing = 0.18;
            self.events.push(GameEvent::Swing);
            // small lunge into the swing gives it body
            let lx = self.player.x + self.player.facing.x * 5.0;
            let ly = self.player.y + self.player.facing.y * 5.0;
            if self.map.box_free(lx, ly, hw, hw) {
                self.player.x = lx;
                self.player.y = ly;
            }
            let reach = self.player.facing * w_reach;
            let px = self.player.x + reach.x;
            let py = self.player.y + reach.y;
            let mut hits: Vec<usize> = Vec::new();
            for (i, m) in self.mobs.iter().enumerate() {
                let dx = m.x - px;
                let dy = m.y - py;
                if dx * dx + dy * dy < w_radius * w_radius {
                    hits.push(i);
                }
            }
            let origin = (self.player.x, self.player.y);
            let crit_chance = 12.0 + self.player.skill_sum(content, "crit");
            let atk_eff = atk_total + if self.well_fed_t > 0.0 { 2 } else { 0 };
            let mut kills = 0;
            for i in hits.into_iter().rev() {
                let crit = gen_range(0.0_f32, 100.0_f32) < crit_chance;
                let mut dmg = (atk_eff - self.mobs[i].def + gen_range(0, 3)).max(1);
                if crit {
                    dmg *= 2;
                }
                let before = self.mobs.len();
                self.damage_mob(i, dmg, origin, crit, w_kb, content);
                if self.mobs.len() < before {
                    kills += 1;
                }
            }
            // Phoenix Picket: kills feed you
            let lifesteal = self.player.special_sum(content, "lifesteal") as i32 * kills;
            if lifesteal > 0 {
                self.player.hp = (self.player.hp + lifesteal).min(maxhp);
                let (x, y) = (self.player.x, self.player.y);
                self.fct(x, y - 14.0, format!("+{}", lifesteal), false, (100, 240, 140));
            }
        }

        // spells
        if let Some(slot) = cast {
            self.cast_spell(slot, content);
        }

        // ---------- fighters attack mobs ----------
        let mut chatter: Vec<(LogKind, String, String)> = Vec::new();
        let mut fighter_attacks: HashMap<usize, i32> = HashMap::new();
        for f in &mut self.fighters {
            let def = &content.npcs[f.def_idx];
            f.attack_cd = (f.attack_cd - dt).max(0.0);
            f.hurt = (f.hurt - dt).max(0.0);
            f.say_t = (f.say_t - dt).max(0.0);
            f.kx *= 1.0 - (8.0 * dt).min(1.0);
            f.ky *= 1.0 - (8.0 * dt).min(1.0);

            let mut nearest: Option<(usize, f32)> = None;
            for (mi, m) in self.mobs.iter().enumerate() {
                let d2 = (m.x - f.x).powi(2) + (m.y - f.y).powi(2);
                if d2 < (TILE * 8.0).powi(2) && nearest.map(|(_, bd)| d2 < bd).unwrap_or(true) {
                    nearest = Some((mi, d2));
                }
            }
            f.engaged = nearest.is_some();
            if let Some((mi, d2)) = nearest {
                // battle cries while fighting (rate-limited)
                f.say_cd -= dt;
                if f.say_cd <= 0.0 && !def.lines.is_empty() {
                    f.say = def.lines[gen_range(0, def.lines.len() as i32) as usize].clone();
                    f.say_t = 3.0;
                    f.say_cd = gen_range(7.0_f32, 15.0_f32);
                    chatter.push((LogKind::Comrade, def.name.clone(), f.say.clone()));
                }
                let d = d2.sqrt().max(0.001);
                let (mx, my) = (self.mobs[mi].x, self.mobs[mi].y);
                if d > 14.0 {
                    let step = 62.0 * dt;
                    let sx = f.x + (mx - f.x) / d * step + f.kx * dt;
                    let sy = f.y + (my - f.y) / d * step + f.ky * dt;
                    if self.map.box_free(sx, f.y, hw, hw) {
                        f.x = sx;
                    }
                    if self.map.box_free(f.x, sy, hw, hw) {
                        f.y = sy;
                    }
                } else if f.attack_cd <= 0.0 {
                    f.attack_cd = 0.7;
                    let dmg = (f.atk + gen_range(0, 3)).max(1);
                    *fighter_attacks.entry(mi).or_insert(0) += dmg;
                }
            } else if f.recruited {
                // no enemies around: fall in behind the player
                let dx = self.player.x - f.x;
                let dy = self.player.y - f.y;
                let d = (dx * dx + dy * dy).sqrt();
                if d > TILE * 1.8 {
                    let step = 78.0 * dt;
                    let sx = f.x + dx / d * step + f.kx * dt;
                    let sy = f.y + dy / d * step + f.ky * dt;
                    if self.map.box_free(sx, f.y, hw, hw) {
                        f.x = sx;
                    }
                    if self.map.box_free(f.x, sy, hw, hw) {
                        f.y = sy;
                    }
                }
            } else {
                // drift while idle
                f.x += f.kx * dt;
                f.y += f.ky * dt;
            }
        }
        // apply fighter damage, highest index first so removals don't shift targets
        let mut fa: Vec<(usize, i32)> = fighter_attacks.into_iter().collect();
        fa.sort_by(|a, b| b.0.cmp(&a.0));
        for (mi, dmg) in fa {
            if mi < self.mobs.len() {
                let origin = (self.mobs[mi].x + gen_range(-8.0_f32, 8.0_f32), self.mobs[mi].y + 8.0);
                self.damage_mob(mi, dmg, origin, false, 1.0, content);
            }
        }

        // ---------- mobs: pick a target (player or nearest fighter) ----------
        let px = self.player.x;
        let py = self.player.y;
        let player_safe = self.map.tile_at_px(px, py).safe();
        let player_dodging = self.player.dodge_t > 0.0;
        let fighter_pos: Vec<(f32, f32)> = self.fighters.iter().map(|f| (f.x, f.y)).collect();
        let mut dmg_to_player = 0;
        let mut hurt_from: Option<(f32, f32)> = None;
        let mut fighter_dmg: HashMap<usize, i32> = HashMap::new();
        let mut attackers: Vec<usize> = Vec::new();
        let mut inflicted: Option<String> = None;
        let mut hurt_name: Option<String> = None;
        let mut boss_engaged = false;

        for (mi, m) in self.mobs.iter_mut().enumerate() {
            let def = &content.mobs[m.def_idx];
            m.attack_cd = (m.attack_cd - dt).max(0.0);
            m.hurt = (m.hurt - dt).max(0.0);
            m.say_t = (m.say_t - dt).max(0.0);

            // knockback decays and slides along walls
            if m.kx.abs() + m.ky.abs() > 1.0 {
                let sx = m.x + m.kx * dt;
                let sy = m.y + m.ky * dt;
                if self.map.box_free(sx, m.y, 5.0, 5.0) {
                    m.x = sx;
                }
                if self.map.box_free(m.x, sy, 5.0, 5.0) {
                    m.y = sy;
                }
            }
            m.kx *= 1.0 - (7.0 * dt).min(1.0);
            m.ky *= 1.0 - (7.0 * dt).min(1.0);

            // nearest legitimate target: the player (outside safe rooms) or a fighter
            let aggro_r = if m.boss { TILE * 12.0 } else { TILE * 7.5 };
            let was_aggro = m.aggro;
            let mut target: Option<(f32, f32, Option<usize>, f32)> = None;
            if !player_safe {
                let d2 = (px - m.x).powi(2) + (py - m.y).powi(2);
                if d2 < aggro_r * aggro_r {
                    target = Some((px, py, None, d2));
                }
            }
            for (fi, &(fx, fy)) in fighter_pos.iter().enumerate() {
                let d2 = (fx - m.x).powi(2) + (fy - m.y).powi(2);
                if d2 < aggro_r * aggro_r && target.map(|(_, _, _, bd)| d2 < bd).unwrap_or(true) {
                    target = Some((fx, fy, Some(fi), d2));
                }
            }
            m.aggro = target.is_some();
            if m.boss && m.aggro && !was_aggro {
                boss_engaged = true;
            }

            if target.is_none() {
                // idle exploiters near enough to watch occasionally emote
                let near2 = (px - m.x).powi(2) + (py - m.y).powi(2);
                m.say_cd -= dt * 0.2;
                if m.say_cd <= 0.0
                    && near2 < (TILE * 14.0).powi(2)
                    && !content.emotes_mob.is_empty()
                {
                    let e = content.emotes_mob[gen_range(0, content.emotes_mob.len() as i32) as usize].clone();
                    m.say = e.clone();
                    m.say_t = 3.0;
                    m.say_cd = gen_range(15.0_f32, 35.0_f32);
                    chatter.push((LogKind::Enemy, def.name.clone(), e));
                }
            }
            if let Some((tx, ty, fighter_idx, dist2)) = target {
                // capitalist propaganda broadcast
                m.say_cd -= dt;
                if m.say_cd <= 0.0 && !def.lines.is_empty() {
                    m.say = def.lines[gen_range(0, def.lines.len() as i32) as usize].clone();
                    m.say_t = 3.0;
                    m.say_cd = gen_range(4.0_f32, 9.0_f32);
                    chatter.push((LogKind::Enemy, def.name.clone(), m.say.clone()));
                }

                let dist = dist2.sqrt().max(0.001);
                let dx = tx - m.x;
                let dy = ty - m.y;
                // no creeping while winding up: they plant their feet to strike
                if dist > 12.0 && m.windup <= 0.0 {
                    // oil underfoot: exploiters slip
                    let mtx = (m.x / TILE) as i32;
                    let mty = (m.y / TILE) as i32;
                    let on_oil = self.oil.iter().any(|o| o.tx == mtx && o.ty == mty);
                    let step = m.speed * dt * if on_oil { 0.5 } else { 1.0 };
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
                // telegraphed attacks: a visible wind-up, then the strike lands
                // only if the target is still in reach — dodge through it!
                if m.windup > 0.0 {
                    m.windup -= dt;
                    if m.windup <= 0.0 {
                        m.attack_cd = 0.9;
                        if dist < 19.0 {
                            match fighter_idx {
                                None if player_dodging => {
                                    // i-frames: the swing whiffs right through you
                                }
                                None => {
                                    let dmg = (m.atk - def_total / 2 + gen_range(0, 2)).max(1);
                                    dmg_to_player += dmg;
                                    hurt_from = Some((m.x, m.y));
                                    hurt_name = Some(def.name.clone());
                                    attackers.push(mi);
                                    if !def.inflicts.is_empty() {
                                        inflicted = Some(def.inflicts.clone());
                                    }
                                }
                                Some(fi) => {
                                    let dmg = (m.atk + gen_range(0, 2)).max(1);
                                    *fighter_dmg.entry(fi).or_insert(0) += dmg;
                                }
                            }
                        }
                    }
                } else if dist < 15.0 && m.attack_cd <= 0.0 {
                    m.windup = 0.35;
                }
            } else {
                m.windup = 0.0;
            }
        }

        if boss_engaged {
            self.system_say(content, "boss_aggro");
            self.hype += 0.3;
        }

        if dmg_to_player > 0 {
            self.player.hp -= dmg_to_player;
            self.player.hurt = 0.25;
            self.shake += 4.0;
            self.slowmo = self.slowmo.max(0.04);
            self.hype += 0.04;
            if let Some(name) = hurt_name.take() {
                self.last_hit_by = name;
            }
            self.events.push(GameEvent::Hurt);
            if let Some((ax, ay)) = hurt_from {
                let d = ((px - ax).powi(2) + (py - ay).powi(2)).sqrt().max(0.001);
                self.player.kx += (px - ax) / d * 120.0;
                self.player.ky += (py - ay) / d * 120.0;
            }
            let (x, y) = (self.player.x, self.player.y);
            self.spawn_particles(x, y, 5, (255, 90, 90));
            self.fct(x, y - 10.0, format!("-{}", dmg_to_player), false, (255, 80, 80));

            // venoms and bureaucratic drag
            if let Some(status) = inflicted.take() {
                match status.as_str() {
                    "poison" => {
                        if self.player.poison_t <= 0.0 {
                            self.toast("Poisoned! The venom of predatory lending courses through you.".to_string());
                        }
                        self.player.poison_t = 3.0;
                    }
                    "slow" => {
                        if self.player.slow_t <= 0.0 {
                            self.toast("Slowed! Wading through terms and conditions.".to_string());
                        }
                        self.player.slow_t = 2.5;
                    }
                    _ => {}
                }
            }

            // Thorned Fences: attackers regret it
            let thorns = self.player.special_sum(content, "thorns") as i32;
            if thorns > 0 {
                let origin = (self.player.x, self.player.y);
                attackers.sort_by(|a, b| b.cmp(a));
                attackers.dedup();
                for mi in attackers {
                    if mi < self.mobs.len() {
                        self.damage_mob(mi, thorns, origin, false, 0.5, content);
                    }
                }
            }
        }

        // wounded comrades
        let mut fallen: Vec<usize> = Vec::new();
        for (fi, dmg) in fighter_dmg {
            if fi < self.fighters.len() {
                self.fighters[fi].hp -= dmg;
                self.fighters[fi].hurt = 0.2;
                let (fx, fy) = (self.fighters[fi].x, self.fighters[fi].y);
                self.fct(fx, fy - 10.0, format!("-{}", dmg), false, (255, 150, 150));
                if self.fighters[fi].hp <= 0 {
                    fallen.push(fi);
                }
            }
        }
        fallen.sort_by(|a, b| b.cmp(a));
        for fi in fallen {
            let f = self.fighters.remove(fi);
            let name = content.npcs[f.def_idx].name.clone();
            self.spawn_particles(f.x, f.y, 14, (220, 60, 60));
            self.toast(format!("{} has fallen. The commune mourns — and remembers.", name));
        }

        // flush speech collected during the entity loops into the chat log
        for (kind, who, text) in chatter.drain(..) {
            self.log_push(kind, &who, &text);
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
            let inst = self.drops[i].item.clone();
            if self.player.add_item(&inst) {
                let name = display_name(content, &inst);
                self.events.push(GameEvent::Pickup);
                if inst.has_affix() {
                    self.toast(format!("Picked up: ✦ {}", name));
                } else {
                    self.toast(format!("Picked up: {}", name));
                }
                self.drops.remove(i);
            }
        }

        // ---------- npc life: wandering + chatting with each other ----------
        self.update_npcs(dt, content);

        // ---------- the broadcast, explosives, and burning oil ----------
        self.update_broadcast(dt, content);
        self.update_projectiles(dt, content);
        self.update_oil(dt, content);

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
        for p in &mut self.particles {
            p.t -= dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy += 160.0 * dt;
        }
        self.particles.retain(|p| p.t > 0.0);
        if let Some(front) = self.toasts.front_mut() {
            front.1 -= real_dt;
            if front.1 <= 0.0 {
                self.toasts.pop_front();
            }
        }

        // level-ups
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
            self.player.skill_points += 1;
            self.events.push(GameEvent::LevelUp);
            let (x, y) = (self.player.x, self.player.y);
            self.spawn_particles(x, y, 16, (255, 220, 90));
            self.toast(format!(
                "Level {}! The movement grows stronger. +1 skill point — press P.",
                self.player.level
            ));
            self.bump_stat_max("level", self.player.level as i64, content);
            for s in &content.spells {
                if s.unlock_level == self.player.level {
                    self.toast(format!("New spell: {} — {}", s.name, s.desc));
                }
            }
        }
        self.bump_stat_max("gold", self.player.gold, content);
        self.check_quests(content);
    }

    fn update_npcs(&mut self, dt: f32, content: &Content) {
        // gentle wandering near home (shopkeepers stay put so you can find them)
        for n in &mut self.npcs {
            n.say_t = (n.say_t - dt).max(0.0);
            let def = &content.npcs[n.def_idx];
            if def.shopkeeper {
                continue;
            }
            n.wander_t -= dt;
            if n.wander_t <= 0.0 {
                n.wander_t = gen_range(3.0_f32, 9.0_f32);
                n.wx = n.home_x + gen_range(-1.6_f32, 1.6_f32) * TILE;
                n.wy = n.home_y + gen_range(-1.6_f32, 1.6_f32) * TILE;
            }
            let dx = n.wx - n.x;
            let dy = n.wy - n.y;
            let d = (dx * dx + dy * dy).sqrt();
            if d > 3.0 {
                let step = 22.0 * dt;
                let sx = n.x + dx / d * step;
                let sy = n.y + dy / d * step;
                // NPCs keep to the safe zone
                if self.map.tile_at_px(sx, n.y).safe() {
                    n.x = sx;
                }
                if self.map.tile_at_px(n.x, sy).safe() {
                    n.y = sy;
                }
            }
        }

        // a queued banter reply lands after a beat
        if let Some((idx, text, delay)) = &mut self.pending_reply {
            *delay -= dt;
            if *delay <= 0.0 {
                let idx = *idx;
                let text = text.clone();
                let mut who = String::new();
                if let Some(n) = self.npcs.get_mut(idx) {
                    n.say = text.clone();
                    n.say_t = 4.5;
                    who = content.npcs[n.def_idx].name.clone();
                }
                if !who.is_empty() {
                    self.log_push(LogKind::Comrade, &who, &text);
                }
                self.pending_reply = None;
            }
        }

        // occasionally, two nearby comrades strike up a conversation
        self.banter_cd -= dt;
        if self.banter_cd <= 0.0 {
            self.banter_cd = gen_range(9.0_f32, 18.0_f32);
            // only bother when the player can see the safe room
            let (px, py) = (self.player.x, self.player.y);
            let near_player: Vec<usize> = self
                .npcs
                .iter()
                .enumerate()
                .filter(|(_, n)| (n.x - px).powi(2) + (n.y - py).powi(2) < (TILE * 16.0).powi(2))
                .filter(|(_, n)| n.say_t <= 0.0)
                .map(|(i, _)| i)
                .collect();
            let mut pair: Option<(usize, usize)> = None;
            'outer: for &a in &near_player {
                for &b in &near_player {
                    if a != b {
                        let (na, nb) = (&self.npcs[a], &self.npcs[b]);
                        if (na.x - nb.x).powi(2) + (na.y - nb.y).powi(2) < (TILE * 4.0).powi(2) {
                            pair = Some((a, b));
                            break 'outer;
                        }
                    }
                }
            }
            if let Some((a, b)) = pair {
                if !content.banter.is_empty() && self.pending_reply.is_none() {
                    let line = &content.banter[gen_range(0, content.banter.len() as i32) as usize];
                    let text = line.a.clone();
                    let reply = line.b.clone();
                    self.npcs[a].say = text.clone();
                    self.npcs[a].say_t = 4.0;
                    let who = content.npcs[self.npcs[a].def_idx].name.clone();
                    self.pending_reply = Some((b, reply, 2.0));
                    self.log_push(LogKind::Comrade, &who, &text);
                }
            } else if let Some(&i) = near_player.first() {
                // no partner around: a line to themselves, or a little emote
                let def = &content.npcs[self.npcs[i].def_idx];
                let roll = gen_range(0, 100);
                let text = if roll < 25 && !content.emotes_npc.is_empty() {
                    Some(content.emotes_npc[gen_range(0, content.emotes_npc.len() as i32) as usize].clone())
                } else if roll < 55 && !def.lines.is_empty() {
                    Some(def.lines[gen_range(0, def.lines.len() as i32) as usize].clone())
                } else {
                    None
                };
                if let Some(text) = text {
                    let who = def.name.clone();
                    self.npcs[i].say = text.clone();
                    self.npcs[i].say_t = 4.0;
                    self.log_push(LogKind::Comrade, &who, &text);
                }
            }
        }
    }

    fn damage_mob(&mut self, i: usize, dmg: i32, origin: (f32, f32), crit: bool, kb_mult: f32, content: &Content) {
        if i >= self.mobs.len() {
            return;
        }
        self.mobs[i].hp -= dmg;
        self.mobs[i].hurt = 0.2;
        // knock the target away from whoever hit it — heavier for crits, bosses resist
        let (mx, my) = (self.mobs[i].x, self.mobs[i].y);
        let d = ((mx - origin.0).powi(2) + (my - origin.1).powi(2)).sqrt().max(0.001);
        let kb = (if self.mobs[i].boss { 40.0 } else if crit { 220.0 } else { 140.0 }) * kb_mult;
        self.mobs[i].kx += (mx - origin.0) / d * kb;
        self.mobs[i].ky += (my - origin.1) / d * kb;
        self.shake += if crit { 4.0 } else { 2.0 };
        self.slowmo = self.slowmo.max(if crit { 0.07 } else { 0.03 });
        self.events.push(GameEvent::Hit);
        self.spawn_particles(mx, my, if crit { 8 } else { 4 }, (255, 230, 120));
        if crit {
            self.fct(mx, my - 12.0, format!("{}!", dmg), true, (255, 120, 60));
        } else {
            self.fct(mx, my - 10.0, format!("{}", dmg), false, (255, 230, 120));
        }
        if self.mobs[i].hp <= 0 {
            self.kill_mob(i, content);
        }
    }

    fn kill_mob(&mut self, i: usize, content: &Content) {
        let m = self.mobs.remove(i);
        let def = content.mobs[m.def_idx].clone();
        self.shake += if def.boss { 14.0 } else { 5.0 };
        self.slowmo = self.slowmo.max(if def.boss { 0.25 } else { 0.09 });
        self.events.push(GameEvent::Kill);
        self.spawn_particles(m.x, m.y, if def.boss { 30 } else { 12 }, (255, 255, 255));
        let base_gold = if def.gold_max > def.gold_min {
            gen_range(def.gold_min, def.gold_max + 1) as i64
        } else {
            def.gold_min as i64
        };
        let gold = (base_gold as f32 * (1.0 + self.player.skill_sum(content, "gold") / 100.0)) as i64;
        self.player.gold += gold;
        self.player.xp +=
            (def.xp as f32 * (1.0 + self.player.skill_sum(content, "xp") / 100.0)) as i64;
        if gold > 0 {
            self.fct(m.x, m.y - 4.0, format!("+{}g", gold), false, (255, 210, 74));
        }
        for d in &def.drops {
            if gen_range(0.0_f32, 1.0_f32) < d.chance {
                let inst = self.make_loot(content, &d.item);
                self.drops.push(Drop {
                    item: inst,
                    x: m.x + gen_range(-6.0_f32, 6.0_f32),
                    y: m.y + gen_range(-6.0_f32, 6.0_f32),
                    t: 0.0,
                });
            }
        }
        self.hype += if def.boss { 0.5 } else { 0.06 };
        self.add_stat("kills", 1, content);
        self.add_stat(&format!("kills_{}", def.id), 1, content);
        if def.boss {
            self.add_stat("bosses", 1, content);
            self.toast(format!("{} liquidated. The dungeon breathes easier.", def.name));
        }
        self.check_quests(content);
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
        let mut spell = content.spells[si].clone();
        // Focused Rage skill + Free Library commune upgrade: cheaper casting
        let mut cost_mult = 1.0 - self.player.skill_sum(content, "focus") / 100.0;
        if self.commune_has("free_library") {
            cost_mult *= 0.9;
        }
        spell.cost = ((spell.cost as f32 * cost_mult).ceil() as i32).max(1);
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
            self.events.push(GameEvent::Cast);
            let (tx, ty) = (self.mobs[ti].x, self.mobs[ti].y);
            self.bursts.push(Burst { x: tx, y: ty, radius: 14.0, t: 0.3, color: spell.color.clone() });
            self.damage_mob(ti, spell.damage, (px, py), false, 1.0, content);
        } else if spell.damage > 0 && spell.radius > 0.0 {
            self.player.mp -= spell.cost;
            self.events.push(GameEvent::Cast);
            let r = spell.radius * TILE;
            self.shake += 6.0;
            self.bursts.push(Burst { x: px, y: py, radius: r, t: 0.45, color: spell.color.clone() });
            let hits: Vec<usize> = self
                .mobs
                .iter()
                .enumerate()
                .filter(|(_, m)| (m.x - px).powi(2) + (m.y - py).powi(2) < r * r)
                .map(|(i, _)| i)
                .collect();
            for i in hits.into_iter().rev() {
                self.damage_mob(i, spell.damage, (px, py), false, 1.0, content);
            }
        } else if spell.heal > 0 {
            self.player.mp -= spell.cost;
            self.events.push(GameEvent::Cast);
        } else {
            return;
        }

        if spell.heal > 0 {
            let (_, _, maxhp, _, _) = self.player.totals(content);
            self.player.hp = (self.player.hp + spell.heal).min(maxhp);
            self.bursts.push(Burst { x: px, y: py, radius: 16.0, t: 0.4, color: "#4fdc7f".to_string() });
            self.fct(px, py - 12.0, format!("+{}", spell.heal), false, (100, 240, 140));
        }

        // Echoing Hall: sometimes the spell pays for itself
        let echo = self.player.special_sum(content, "echo");
        if echo > 0.0 && gen_range(0.0_f32, 100.0_f32) < echo {
            let (_, _, _, maxmp, _) = self.player.totals(content);
            self.player.mp = (self.player.mp + spell.cost).min(maxmp);
            self.fct(px, py - 18.0, "Echo!".to_string(), false, (150, 200, 255));
        }
    }

    /// Roll random affixes for found loot; odds creep up with depth.
    fn roll_affixes(&self, content: &Content) -> (Option<String>, Option<String>) {
        let pchance = 22 + self.depth * 2;
        let schance = 8 + self.depth;
        let prefix = if !content.prefixes.is_empty() && gen_range(0, 100) < pchance {
            Some(content.prefixes[gen_range(0, content.prefixes.len() as i32) as usize].id.clone())
        } else {
            None
        };
        let suffix = if !content.suffixes.is_empty() && gen_range(0, 100) < schance {
            Some(content.suffixes[gen_range(0, content.suffixes.len() as i32) as usize].id.clone())
        } else {
            None
        };
        (prefix, suffix)
    }

    /// Affixes only make sense on gear; potions stay plain.
    fn make_loot(&self, content: &Content, id: &str) -> ItemInst {
        let equippable = content.item(id).map(|d| d.is_equippable()).unwrap_or(false);
        if !equippable {
            return ItemInst::plain(id);
        }
        let (prefix, suffix) = self.roll_affixes(content);
        ItemInst { id: id.to_string(), prefix, suffix }
    }

    /// The System says something. On camera. It always is.
    pub fn system_say(&mut self, content: &Content, cat: &str) {
        if let Some(lines) = content.system.categories.get(cat) {
            if !lines.is_empty() {
                let line = lines[gen_range(0, lines.len() as i32) as usize].clone();
                self.announce = Some((line.clone(), 6.5));
                self.log_push(LogKind::Broadcast, "THE SYSTEM", &line);
            }
        }
    }

    fn spawn_mob_at(&mut self, content: &Content, tier: i32, tx: i32, ty: i32) {
        let pool: Vec<usize> = content
            .mobs
            .iter()
            .enumerate()
            .filter(|(_, m)| m.tier == tier)
            .map(|(i, _)| i)
            .collect();
        if pool.is_empty() {
            return;
        }
        let def_idx = pool[gen_range(0, pool.len() as i32) as usize];
        let def = &content.mobs[def_idx];
        self.mobs.push(Mob {
            def_idx,
            x: (tx as f32 + 0.5) * TILE,
            y: (ty as f32 + 0.5) * TILE,
            kx: 0.0,
            ky: 0.0,
            hp: scale_hp(def.hp, self.depth),
            maxhp: scale_hp(def.hp, self.depth),
            atk: scale_atk(def.atk, self.depth),
            def: def.def,
            speed: def.speed,
            attack_cd: 0.5,
            windup: 0.0,
            say: String::new(),
            say_t: 0.0,
            say_cd: gen_range(1.0_f32, 4.0_f32),
            aggro: false,
            hurt: 0.0,
            boss: def.boss,
            burn_acc: 0.0,
        });
    }

    /// Everything reality-TV: viewer counts, sponsor crates, mail, floor pressure.
    fn update_broadcast(&mut self, dt: f32, content: &Content) {
        // hype decays; viewers chase a baseline scaled by hype
        self.hype = (self.hype - self.hype * 0.12 * dt).clamp(0.0, 2.0);
        let kills = self.stats.get("kills").copied().unwrap_or(0) as f64;
        let baseline = 40_000.0 + self.depth as f64 * 15_000.0 + kills * 400.0;
        let target = baseline * (1.0 + self.hype as f64);
        self.viewers += (target - self.viewers) * (dt as f64 * 0.4);

        if let Some((_, t)) = &mut self.announce {
            *t -= dt;
            if *t <= 0.0 {
                self.announce = None;
            }
        }

        // sponsor crates at viewer milestones
        if self.viewers > self.last_sponsor + 35_000.0 {
            self.last_sponsor = self.viewers;
            let tier: u8 = if self.viewers > 240_000.0 {
                3
            } else if self.viewers > 140_000.0 {
                2
            } else {
                1
            };
            self.spawn_sponsor_crate(tier);
            self.system_say(content, "sponsor_drop");
        }

        // viewer mail
        self.mail_cd -= dt;
        if self.mail_cd <= 0.0 {
            self.mail_cd = gen_range(55.0_f32, 110.0_f32);
            let sys = &content.system;
            if gen_range(0, 100) < 58 && !sys.fan_senders.is_empty() && !sys.fan_lines.is_empty() {
                let s = sys.fan_senders[gen_range(0, sys.fan_senders.len() as i32) as usize].clone();
                let l = sys.fan_lines[gen_range(0, sys.fan_lines.len() as i32) as usize].clone();
                let (_, _, maxhp, _, _) = self.player.totals(content);
                self.player.hp = (self.player.hp + 5).min(maxhp);
                self.hype += 0.1;
                self.toast(format!("FAN MAIL from {}: \"{}\" (+5 HP)", s, l));
            } else if !sys.hate_senders.is_empty() && !sys.hate_lines.is_empty() {
                let s = sys.hate_senders[gen_range(0, sys.hate_senders.len() as i32) as usize].clone();
                let l = sys.hate_lines[gen_range(0, sys.hate_lines.len() as i32) as usize].clone();
                self.hype += 0.05; // hate is still engagement
                self.toast(format!("HATE MAIL from {}: \"{}\"", s, l));
            }
        }

        // floor cost-optimization pressure
        self.floor_time += dt;
        if self.floor_time > 240.0 && !self.floor_warned {
            self.floor_warned = true;
            self.system_say(content, "floor_warning");
        }
        if self.floor_time > 330.0 {
            self.wave_cd -= dt;
            if self.wave_cd <= 0.0 {
                self.wave_cd = 60.0;
                self.system_say(content, "cost_cutting");
                let tier = (1 + (self.depth / 3).min(2)) as i32;
                for _ in 0..2 {
                    for _try in 0..25 {
                        let tx = (self.player.x / TILE) as i32 + gen_range(-8, 9);
                        let ty = (self.player.y / TILE) as i32 + gen_range(-8, 9);
                        let far = (tx as f32 * TILE - self.player.x).abs() + (ty as f32 * TILE - self.player.y).abs() > TILE * 4.0;
                        if far && self.map.tile(tx, ty) == Tile::Floor {
                            self.spawn_mob_at(content, tier, tx, ty);
                            break;
                        }
                    }
                }
            }
        }

        // the System's footwear obsession
        self.barefoot_cd -= dt;
        if self.barefoot_cd <= 0.0 {
            self.barefoot_cd = gen_range(120.0_f32, 240.0_f32);
            if !self.player.equipment.contains_key("boots") {
                self.system_say(content, "barefoot");
            }
        }
    }

    fn spawn_sponsor_crate(&mut self, tier: u8) {
        let ptx = (self.player.x / TILE) as i32;
        let pty = (self.player.y / TILE) as i32;
        for _ in 0..40 {
            let tx = ptx + gen_range(-4, 5);
            let ty = pty + gen_range(-4, 5);
            if self.map.tile(tx, ty) == Tile::Floor
                && !self.chests.iter().any(|c| c.tx == tx as usize && c.ty == ty as usize)
            {
                self.chests.push(Chest { tx: tx as usize, ty: ty as usize, opened: false, tier });
                let name = match tier {
                    3 => "GOLD",
                    2 => "SILVER",
                    _ => "BRONZE",
                };
                self.toast(format!("{} SPONSOR CRATE incoming — check your surroundings.", name));
                return;
            }
        }
    }

    /// Throw the first bomb or oil flask in the backpack toward the facing direction.
    pub fn throw_item(&mut self, content: &Content, kind: &str) -> bool {
        let Some(idx) = self
            .player
            .inventory
            .iter()
            .position(|s| content.item(&s.inst.id).map(|d| d.kind == kind).unwrap_or(false))
        else {
            self.toast(format!(
                "No {} in the backpack. The co-op sometimes stocks them.",
                if kind == "bomb" { "dynamite" } else { "oil" }
            ));
            return false;
        };
        self.player.remove_item(idx);
        let d = self.player.facing;
        self.projectiles.push(Projectile {
            x: self.player.x,
            y: self.player.y - 4.0,
            vx: d.x * 190.0,
            vy: d.y * 190.0,
            fuse: if kind == "bomb" { 1.0 } else { 0.55 },
            oil: kind == "oil",
        });
        self.events.push(GameEvent::Dash);
        if kind == "bomb" {
            self.add_stat("bombs_thrown", 1, content);
        }
        true
    }

    fn update_projectiles(&mut self, dt: f32, content: &Content) {
        let mut boom: Vec<(f32, f32, bool)> = Vec::new();
        for p in &mut self.projectiles {
            let nx = p.x + p.vx * dt;
            let ny = p.y + p.vy * dt;
            if self.map.box_free(nx, p.y, 3.0, 3.0) {
                p.x = nx;
            } else {
                p.vx = 0.0;
            }
            if self.map.box_free(p.x, ny, 3.0, 3.0) {
                p.y = ny;
            } else {
                p.vy = 0.0;
            }
            let damp = 1.0 - (2.6 * dt).min(1.0);
            p.vx *= damp;
            p.vy *= damp;
            p.fuse -= dt;
            if p.fuse <= 0.0 {
                boom.push((p.x, p.y, p.oil));
            }
        }
        self.projectiles.retain(|p| p.fuse > 0.0);
        for (x, y, oil) in boom {
            if oil {
                self.splash_oil(x, y);
            } else {
                self.explode(x, y, content);
            }
        }
    }

    fn splash_oil(&mut self, x: f32, y: f32) {
        let tx = (x / TILE) as i32;
        let ty = (y / TILE) as i32;
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (ox, oy) = (tx + dx, ty + dy);
            if self.map.tile(ox, oy).walkable() && !self.oil.iter().any(|o| o.tx == ox && o.ty == oy) {
                self.oil.push(OilPatch { tx: ox, ty: oy, lit: false, life: 30.0 });
            }
        }
        self.spawn_particles(x, y, 6, (90, 70, 50));
    }

    /// Dynamite: hurts exploiters, comrades, YOU, and load-bearing walls alike.
    fn explode(&mut self, x: f32, y: f32, content: &Content) {
        let r = TILE * 2.3;
        self.shake += 14.0;
        self.slowmo = self.slowmo.max(0.2);
        self.hype += 0.15;
        self.events.push(GameEvent::Boom);
        self.spawn_particles(x, y, 30, (255, 180, 90));
        self.bursts.push(Burst { x, y, radius: r, t: 0.45, color: "#ff9933".to_string() });

        // demolish walls (never the map border)
        let t0x = ((x - r) / TILE) as i32;
        let t1x = ((x + r) / TILE) as i32;
        let t0y = ((y - r) / TILE) as i32;
        let t1y = ((y + r) / TILE) as i32;
        let mut walls = 0;
        for ty in t0y..=t1y {
            for tx in t0x..=t1x {
                let cx = (tx as f32 + 0.5) * TILE;
                let cy = (ty as f32 + 0.5) * TILE;
                if (cx - x).powi(2) + (cy - y).powi(2) < (r * 0.85).powi(2)
                    && tx > 0
                    && ty > 0
                    && (tx as usize) < self.map.w - 1
                    && (ty as usize) < self.map.h - 1
                    && self.map.tile(tx, ty) == Tile::Wall
                {
                    self.map.set(tx, ty, Tile::Floor);
                    self.map.graffiti.retain(|g| !(g.x == tx as usize && g.y == ty as usize));
                    walls += 1;
                }
            }
        }
        if walls > 0 {
            self.add_stat("walls_destroyed", walls, content);
        }

        // ignite nearby oil
        for o in &mut self.oil {
            let ox = (o.tx as f32 + 0.5) * TILE;
            let oy = (o.ty as f32 + 0.5) * TILE;
            if (ox - x).powi(2) + (oy - y).powi(2) < (r + TILE * 1.5).powi(2) {
                o.lit = true;
                o.life = o.life.min(7.0);
            }
        }

        // damage mobs
        let hits: Vec<usize> = self
            .mobs
            .iter()
            .enumerate()
            .filter(|(_, m)| (m.x - x).powi(2) + (m.y - y).powi(2) < r * r)
            .map(|(i, _)| i)
            .collect();
        for i in hits.into_iter().rev() {
            self.damage_mob(i, gen_range(26, 40), (x, y), false, 2.2, content);
        }

        // friendly fire: comrades
        let mut fallen: Vec<usize> = Vec::new();
        for (fi, f) in self.fighters.iter_mut().enumerate() {
            if (f.x - x).powi(2) + (f.y - y).powi(2) < (r * 0.9).powi(2) {
                f.hp -= 14;
                f.hurt = 0.3;
                if f.hp <= 0 {
                    fallen.push(fi);
                }
            }
        }
        for fi in fallen.into_iter().rev() {
            let f = self.fighters.remove(fi);
            let name = content.npcs[f.def_idx].name.clone();
            self.toast(format!("{} was caught in YOUR blast. The commune will remember that.", name));
        }

        // friendly fire: you
        let pd2 = (self.player.x - x).powi(2) + (self.player.y - y).powi(2);
        if pd2 < (r * 0.9).powi(2) && self.player.dodge_t <= 0.0 {
            self.player.hp -= 16;
            self.player.hurt = 0.3;
            self.last_hit_by = "their own dynamite".to_string();
            let d = pd2.sqrt().max(0.001);
            self.player.kx += (self.player.x - x) / d * 260.0;
            self.player.ky += (self.player.y - y) / d * 260.0;
            let (px, py) = (self.player.x, self.player.y);
            self.fct(px, py - 10.0, "-16".to_string(), true, (255, 120, 60));
        }
        self.add_stat("bombs_exploded", 1, content);
    }

    fn update_oil(&mut self, dt: f32, content: &Content) {
        for o in &mut self.oil {
            if o.lit {
                o.life -= dt;
            } else {
                o.life -= dt * 0.2;
            }
        }
        self.oil.retain(|o| o.life > 0.0);

        // burning oil cooks whoever stands in it (mobs and player alike)
        let lit: Vec<(i32, i32)> = self.oil.iter().filter(|o| o.lit).map(|o| (o.tx, o.ty)).collect();
        if lit.is_empty() {
            return;
        }
        let mut cooked: Vec<usize> = Vec::new();
        for (mi, m) in self.mobs.iter_mut().enumerate() {
            let mt = ((m.x / TILE) as i32, (m.y / TILE) as i32);
            if lit.contains(&mt) {
                m.burn_acc += 9.0 * dt;
                if m.burn_acc >= 4.0 {
                    m.burn_acc = 0.0;
                    cooked.push(mi);
                }
            }
        }
        for mi in cooked.into_iter().rev() {
            if mi < self.mobs.len() {
                let origin = (self.mobs[mi].x, self.mobs[mi].y + 6.0);
                self.damage_mob(mi, 4, origin, false, 0.3, content);
            }
        }
        let pt = ((self.player.x / TILE) as i32, (self.player.y / TILE) as i32);
        if lit.contains(&pt) {
            self.player_burn_acc += 9.0 * dt;
            if self.player_burn_acc >= 4.0 {
                self.player_burn_acc = 0.0;
                self.player.hp -= 4;
                self.player.hurt = 0.2;
                self.last_hit_by = "burning oil".to_string();
                let (px, py) = (self.player.x, self.player.y);
                self.fct(px, py - 10.0, "-4".to_string(), false, (255, 160, 60));
            }
        }
    }

    pub fn pick_obituary(&mut self, content: &Content) {
        if self.obituary.is_empty() && !content.system.obituaries.is_empty() {
            self.obituary = content.system.obituaries
                [gen_range(0, content.system.obituaries.len() as i32) as usize]
                .clone();
        }
    }

    /// Complete any active quests whose target stat has advanced far enough.
    pub fn check_quests(&mut self, content: &Content) {
        let mut finished: Vec<usize> = Vec::new();
        for (i, (qid, base)) in self.quests_active.iter().enumerate() {
            if let Some(q) = content.quests.iter().find(|q| &q.id == qid) {
                let now = self.stats.get(&q.stat).copied().unwrap_or(0);
                if now - base >= q.count {
                    finished.push(i);
                }
            }
        }
        for i in finished.into_iter().rev() {
            let (qid, _) = self.quests_active.remove(i);
            let Some(q) = content.quests.iter().find(|q| q.id == qid).cloned() else { continue };
            self.quests_done.insert(qid);
            self.player.gold += q.reward_gold;
            let mut reward = format!("+{} gold", q.reward_gold);
            if !q.reward_item.is_empty() && self.player.add_item(&ItemInst::plain(&q.reward_item)) {
                let name = content
                    .item(&q.reward_item)
                    .map(|d| d.name.clone())
                    .unwrap_or_else(|| q.reward_item.clone());
                reward = format!("{}, {}", reward, name);
            }
            self.events.push(GameEvent::Quest);
            self.toast(format!("QUEST COMPLETE: {} — {}", q.name, reward));
            self.log_push(LogKind::System, "Quest", &q.done_line);
            self.add_stat("quests", 1, content);
        }
    }

    pub fn commune_has(&self, id: &str) -> bool {
        self.commune.iter().any(|c| c == id)
    }

    pub fn buy_commune(&mut self, content: &Content, idx: usize) -> bool {
        let Some(&(id, name, _, cost)) = COMMUNE_UPGRADES.get(idx) else { return false };
        if self.commune_has(id) {
            self.toast("The commune already built that. Onward!".to_string());
            return false;
        }
        if self.player.gold < cost {
            self.toast(format!("The commune needs {} gold for {}.", cost, name));
            return false;
        }
        self.player.gold -= cost;
        self.commune.push(id.to_string());
        if id == "clinic_beds" {
            self.player.base_maxhp += 10;
            self.player.hp += 10;
        }
        self.events.push(GameEvent::Quest);
        self.toast(format!("The commune built: {}! Collective ownership feels good.", name));
        self.add_stat("commune_built", 1, content);
        true
    }

    /// Is the player standing next to the campfire?
    pub fn near_campfire(&self) -> bool {
        let ptx = (self.player.x / TILE) as i32;
        let pty = (self.player.y / TILE) as i32;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if self.map.tile(ptx + dx, pty + dy) == Tile::Campfire {
                    return true;
                }
            }
        }
        false
    }

    /// The forge: melt down one piece of gear and reroll its affixes.
    /// Costs gold, guarantees a prefix, good odds on a suffix.
    pub fn forge_reroll(&mut self, content: &Content, idx: usize) -> bool {
        let Some(stack) = self.player.inventory.get(idx) else { return false };
        let inst = stack.inst.clone();
        let Some(def) = content.item(&inst.id).cloned() else { return false };
        if !def.is_equippable() {
            self.toast("The forge only reworks gear, not lunch.".to_string());
            return false;
        }
        let mut cost = (25 + def.tier * 25) as i64;
        if self.commune_has("communal_forge") {
            cost = (cost as f32 * 0.75) as i64;
        }
        if self.player.gold < cost {
            self.toast(format!("The forge needs {} gold for that.", cost));
            return false;
        }
        if content.prefixes.is_empty() {
            return false;
        }
        self.player.gold -= cost;
        let prefix = Some(
            content.prefixes[gen_range(0, content.prefixes.len() as i32) as usize].id.clone(),
        );
        let suffix = if !content.suffixes.is_empty() && gen_range(0, 100) < 40 {
            Some(content.suffixes[gen_range(0, content.suffixes.len() as i32) as usize].id.clone())
        } else {
            None
        };
        let new_inst = ItemInst { id: inst.id.clone(), prefix, suffix };
        self.player.remove_item(idx);
        if !self.player.add_item(&new_inst) {
            self.player.gold += cost;
            return false;
        }
        self.events.push(GameEvent::Chest);
        let name = display_name(content, &new_inst);
        self.toast(format!("Reforged: ✦ {} (-{} gold)", name, cost));
        true
    }

    pub fn learn_skill(&mut self, content: &Content, id: &str) -> bool {
        let Some(def) = content.skills.iter().find(|s| s.id == id).cloned() else {
            return false;
        };
        if self.player.skill_points <= 0 || self.player.skills.iter().any(|s| s == id) {
            return false;
        }
        if !def.requires.is_empty() && !self.player.skills.iter().any(|s| s == &def.requires) {
            return false;
        }
        self.player.skill_points -= 1;
        self.player.skills.push(id.to_string());
        self.toast(format!("Skill learned: {} — {}", def.name, def.desc));
        self.add_stat("skills_learned", 1, content);
        true
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
                    if self.commune_has("soup_kitchen") {
                        self.well_fed_t = 60.0;
                        self.toast("Well Fed! The soup kitchen provides (+2 ATK, 60s).".to_string());
                    }
                    self.events.push(GameEvent::Rest);
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
            // quest offers come before small talk
            let npc_id = def.id.clone();
            let quest = content
                .quests
                .iter()
                .find(|q| {
                    q.giver == npc_id
                        && !self.quests_done.contains(&q.id)
                        && !self.quests_active.iter().any(|(id, _)| id == &q.id)
                })
                .cloned();
            if let Some(q) = quest {
                if self.quests_active.len() < 3 {
                    let base = self.stats.get(&q.stat).copied().unwrap_or(0);
                    self.quests_active.push((q.id.clone(), base));
                    let def = &content.npcs[self.npcs[i].def_idx];
                    let who = def.name.clone();
                    self.npcs[i].say = q.offer.clone();
                    self.npcs[i].say_t = 4.0;
                    self.events.push(GameEvent::Quest);
                    self.toast(format!("NEW QUEST: {} — {}", q.name, q.desc));
                    self.log_push(LogKind::Comrade, &who, &q.offer);
                    self.add_stat("talks", 1, content);
                    return Interaction::Dialog { who, text: q.offer };
                }
            }
            let line = if def.lines.is_empty() {
                "Solidarity, friend.".to_string()
            } else {
                def.lines[gen_range(0, def.lines.len() as i32) as usize].clone()
            };
            self.npcs[i].say = line.clone();
            self.npcs[i].say_t = 4.0;
            let who = def.name.clone();
            self.log_push(LogKind::Comrade, &who, &line);
            self.add_stat("talks", 1, content);
            return Interaction::Dialog { who, text: line };
        }

        // recruit a fighter comrade
        let mut best_fighter: Option<(usize, f32)> = None;
        for (i, f) in self.fighters.iter().enumerate() {
            if f.recruited {
                continue;
            }
            let d2 = (f.x - px).powi(2) + (f.y - py).powi(2);
            if d2 < (TILE * 1.6).powi(2) && best_fighter.map(|(_, bd)| d2 < bd).unwrap_or(true) {
                best_fighter = Some((i, d2));
            }
        }
        if let Some((i, _)) = best_fighter {
            self.fighters[i].recruited = true;
            let def = &content.npcs[self.fighters[i].def_idx];
            let name = def.name.clone();
            let line = if def.lines.is_empty() {
                "For the commons!".to_string()
            } else {
                def.lines[gen_range(0, def.lines.len() as i32) as usize].clone()
            };
            self.fighters[i].say = line.clone();
            self.fighters[i].say_t = 3.5;
            self.events.push(GameEvent::Recruit);
            self.add_stat("recruits", 1, content);
            self.toast(format!("{} joins you. An injury to one is an injury to all.", name));
            return Interaction::Dialog { who: name, text: line };
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
                self.events.push(GameEvent::Chest);
                let tier = self.chests[ci].tier;
                let gold = (gen_range(8, 20) + self.depth * 4 + tier as i32 * 30) as i64;
                self.player.gold += gold;
                let mut text = format!("+{} gold for the strike fund", gold);
                // sponsor crates guarantee affixed merchandise
                if tier > 0 {
                    let max_tier = (1 + self.depth / 2 + tier as i32 - 1).max(1);
                    let pool: Vec<&str> = content
                        .items
                        .iter()
                        .filter(|i| i.tier <= max_tier && i.is_equippable())
                        .map(|i| i.id.as_str())
                        .collect();
                    let n_items = if tier >= 3 { 2 } else { 1 };
                    for _ in 0..n_items {
                        if pool.is_empty() {
                            break;
                        }
                        let id = pool[gen_range(0, pool.len() as i32) as usize].to_string();
                        let mut inst = self.make_loot(content, &id);
                        if !inst.has_affix() && !content.prefixes.is_empty() {
                            inst.prefix = Some(
                                content.prefixes[gen_range(0, content.prefixes.len() as i32) as usize]
                                    .id
                                    .clone(),
                            );
                        }
                        if self.player.add_item(&inst) {
                            text = format!("{}, ✦ {}", text, display_name(content, &inst));
                        }
                    }
                    self.hype += 0.15;
                    self.add_stat("sponsor_crates", 1, content);
                    return Interaction::ChestLoot { text: format!("SPONSOR CRATE: {}", text) };
                }
                if gen_range(0, 100) < 65 {
                    let max_tier = 1 + self.depth / 2;
                    let pool: Vec<&str> = content
                        .items
                        .iter()
                        .filter(|i| i.tier <= max_tier)
                        .map(|i| i.id.as_str())
                        .collect();
                    if !pool.is_empty() {
                        let id = pool[gen_range(0, pool.len() as i32) as usize].to_string();
                        let inst = self.make_loot(content, &id);
                        if self.player.add_item(&inst) {
                            let name = display_name(content, &inst);
                            let mark = if inst.has_affix() { "✦ " } else { "" };
                            text = format!("{}, and: {}{}", text, mark, name);
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
            self.log_push(LogKind::System, "The wall", &text);
            return Interaction::Dialog { who: "Graffiti on the wall".to_string(), text };
        }

        Interaction::None
    }
}
