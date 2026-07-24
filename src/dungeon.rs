use macroquad::rand::gen_range;
use serde::{Deserialize, Serialize};

pub const TILE: f32 = 16.0;
pub const MAP_W: usize = 72;
pub const MAP_H: usize = 54;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Floor,
    Safe,     // mobs will not enter
    Campfire, // rest spot (also safe)
    Stairs,   // descend to next depth
}

impl Tile {
    pub fn walkable(self) -> bool {
        !matches!(self, Tile::Wall)
    }
    pub fn safe(self) -> bool {
        matches!(self, Tile::Safe | Tile::Campfire)
    }
}

#[derive(Clone, Copy)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Room {
    pub fn center(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }
    fn intersects(&self, o: &Room) -> bool {
        self.x - 1 < o.x + o.w && self.x + self.w + 1 > o.x && self.y - 1 < o.y + o.h
            && self.y + self.h + 1 > o.y
    }
}

pub struct Graffiti {
    pub x: usize,
    pub y: usize,
    pub text_idx: usize,
}

pub struct Map {
    pub w: usize,
    pub h: usize,
    pub tiles: Vec<Tile>,
    pub rooms: Vec<Room>,
    pub spawn: (f32, f32), // player spawn, world px
    pub graffiti: Vec<Graffiti>,
    pub chest_spots: Vec<(usize, usize)>,
    pub mob_spots: Vec<(usize, usize, i32)>, // x, y, tier
    pub npc_spots: Vec<(usize, usize)>,
    pub has_shop: bool,
}

impl Map {
    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            Tile::Wall
        } else {
            self.tiles[y as usize * self.w + x as usize]
        }
    }
    pub fn set(&mut self, x: i32, y: i32, t: Tile) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.tiles[y as usize * self.w + x as usize] = t;
        }
    }
    /// AABB vs wall tiles, for an entity hitbox in world pixels.
    pub fn box_free(&self, x: f32, y: f32, hw: f32, hh: f32) -> bool {
        let x0 = ((x - hw) / TILE).floor() as i32;
        let x1 = ((x + hw - 0.01) / TILE).floor() as i32;
        let y0 = ((y - hh) / TILE).floor() as i32;
        let y1 = ((y + hh - 0.01) / TILE).floor() as i32;
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if !self.tile(tx, ty).walkable() {
                    return false;
                }
            }
        }
        true
    }
    pub fn tile_at_px(&self, x: f32, y: f32) -> Tile {
        self.tile((x / TILE).floor() as i32, (y / TILE).floor() as i32)
    }
}

fn carve_room(tiles: &mut [Tile], r: &Room) {
    for y in r.y..r.y + r.h {
        for x in r.x..r.x + r.w {
            tiles[y as usize * MAP_W + x as usize] = Tile::Floor;
        }
    }
}

fn carve_corridor(tiles: &mut [Tile], a: (i32, i32), b: (i32, i32)) {
    let (mut x, mut y) = a;
    // L-shaped, 2 wide so it doesn't feel like a crawlspace
    while x != b.0 {
        x += (b.0 - x).signum();
        for dy in 0..2 {
            let yy = (y + dy).clamp(1, MAP_H as i32 - 2);
            tiles[yy as usize * MAP_W + x as usize] = Tile::Floor;
        }
    }
    while y != b.1 {
        y += (b.1 - y).signum();
        for dx in 0..2 {
            let xx = (x + dx).clamp(1, MAP_W as i32 - 2);
            tiles[y as usize * MAP_W + xx as usize] = Tile::Floor;
        }
    }
}

/// Procedural level. Room 0 is always the safe room (campfire, NPCs, shop on
/// every other depth); the room farthest from it holds the stairs down.
pub fn generate(depth: i32, graffiti_count: usize) -> Map {
    let mut tiles = vec![Tile::Wall; MAP_W * MAP_H];
    let mut rooms: Vec<Room> = Vec::new();
    let target = 14 + (depth.min(10)) as usize;
    let mut attempts = 0;
    while rooms.len() < target && attempts < 400 {
        attempts += 1;
        let w = gen_range(5, 12);
        let h = gen_range(4, 9);
        let x = gen_range(1, MAP_W as i32 - w - 1);
        let y = gen_range(1, MAP_H as i32 - h - 1);
        let r = Room { x, y, w, h };
        if rooms.iter().any(|o| r.intersects(o)) {
            continue;
        }
        rooms.push(r);
    }
    for r in &rooms {
        carve_room(&mut tiles, r);
    }
    for i in 1..rooms.len() {
        carve_corridor(&mut tiles, rooms[i - 1].center(), rooms[i].center());
    }

    // safe room = room 0
    let safe = rooms[0];
    for y in safe.y..safe.y + safe.h {
        for x in safe.x..safe.x + safe.w {
            tiles[y as usize * MAP_W + x as usize] = Tile::Safe;
        }
    }
    let (cx, cy) = safe.center();
    tiles[cy as usize * MAP_W + cx as usize] = Tile::Campfire;

    // stairs in the room farthest from the safe room
    let mut far_i = rooms.len() - 1;
    let mut far_d = -1i32;
    for (i, r) in rooms.iter().enumerate().skip(1) {
        let (rx, ry) = r.center();
        let d = (rx - cx).abs() + (ry - cy).abs();
        if d > far_d {
            far_d = d;
            far_i = i;
        }
    }
    let (sx, sy) = rooms[far_i].center();
    tiles[sy as usize * MAP_W + sx as usize] = Tile::Stairs;

    let mut map = Map {
        w: MAP_W,
        h: MAP_H,
        tiles,
        spawn: (
            (cx as f32 + 0.5) * TILE,
            (cy as f32 + 1.8) * TILE,
        ),
        graffiti: Vec::new(),
        chest_spots: Vec::new(),
        mob_spots: Vec::new(),
        npc_spots: Vec::new(),
        has_shop: depth % 2 == 1,
        rooms,
    };

    // NPCs cluster in the safe room
    let npc_n = gen_range(2, 4);
    for _ in 0..npc_n {
        let nx = gen_range(safe.x + 1, safe.x + safe.w - 1);
        let ny = gen_range(safe.y + 1, safe.y + safe.h - 1);
        map.npc_spots.push((nx as usize, ny as usize));
    }

    // chests + mobs in later rooms
    for (i, r) in map.rooms.clone().iter().enumerate().skip(1) {
        if gen_range(0, 100) < 40 {
            let gx = gen_range(r.x + 1, r.x + r.w - 1) as usize;
            let gy = gen_range(r.y + 1, r.y + r.h - 1) as usize;
            if map.tiles[gy * MAP_W + gx] == Tile::Floor {
                map.chest_spots.push((gx, gy));
            }
        }
        let mob_n = gen_range(1, 4) + (depth / 3).min(3);
        for _ in 0..mob_n {
            let gx = gen_range(r.x, r.x + r.w) as usize;
            let gy = gen_range(r.y, r.y + r.h) as usize;
            if map.tiles[gy * MAP_W + gx] == Tile::Floor {
                let tier = pick_tier(depth);
                map.mob_spots.push((gx, gy, tier));
            }
        }
        // boss floor: vulture guards the stairs
        if depth % 5 == 0 && i == far_i {
            let (bx, by) = r.center();
            map.mob_spots.push((bx as usize, by as usize, 4));
        }
        // miniboss floors: sometimes a mid-tier menace claims a room
        if depth % 5 != 0 && depth >= 2 && i == far_i && gen_range(0, 100) < 35 {
            let (bx, by) = r.center();
            map.mob_spots.push((bx.max(1) as usize, (by - 1).max(1) as usize, 5));
        }
    }

    // graffiti on walls that touch floor
    let mut placed = 0;
    let mut tries = 0;
    while placed < graffiti_count.min(6) && tries < 500 {
        tries += 1;
        let x = gen_range(1, MAP_W as i32 - 1);
        let y = gen_range(1, MAP_H as i32 - 1);
        if map.tile(x, y) == Tile::Wall && map.tile(x, y + 1).walkable() {
            if map.graffiti.iter().any(|g| (g.x as i32 - x).abs() < 4 && (g.y as i32 - y).abs() < 4) {
                continue;
            }
            map.graffiti.push(Graffiti {
                x: x as usize,
                y: y as usize,
                text_idx: gen_range(0, graffiti_count.max(1) as i32) as usize,
            });
            placed += 1;
        }
    }

    map
}

fn pick_tier(depth: i32) -> i32 {
    let roll = gen_range(0, 100);
    match depth {
        1 => 1,
        2 => if roll < 75 { 1 } else { 2 },
        3..=4 => if roll < 40 { 1 } else if roll < 85 { 2 } else { 3 },
        5..=7 => if roll < 20 { 1 } else if roll < 60 { 2 } else { 3 },
        _ => if roll < 35 { 2 } else { 3 },
    }
}

// ---------- custom levels (level creator output / hand-editable) ----------

#[derive(Serialize, Deserialize)]
pub struct CustomLevel {
    pub name: String,
    pub rows: Vec<String>,
}

/// Legend: `#` wall, `.` floor, `s` safe floor, `c` campfire, `>` stairs,
/// `C` chest, `g` graffiti wall, `1`/`2`/`3`/`4` mob spawn by tier,
/// `n` NPC spawn, `@` player spawn.
pub fn from_custom(level: &CustomLevel, graffiti_count: usize) -> Option<Map> {
    let h = level.rows.len();
    let w = level.rows.iter().map(|r| r.chars().count()).max()?;
    if h < 3 || w < 3 {
        return None;
    }
    let mut map = Map {
        w,
        h,
        tiles: vec![Tile::Wall; w * h],
        rooms: Vec::new(),
        spawn: (TILE * 2.0, TILE * 2.0),
        graffiti: Vec::new(),
        chest_spots: Vec::new(),
        mob_spots: Vec::new(),
        npc_spots: Vec::new(),
        has_shop: true,
    };
    for (y, row) in level.rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let t = match ch {
                '#' => Tile::Wall,
                '.' => Tile::Floor,
                's' => Tile::Safe,
                'c' => Tile::Campfire,
                '>' => Tile::Stairs,
                'C' => {
                    map.chest_spots.push((x, y));
                    Tile::Floor
                }
                'g' => {
                    map.graffiti.push(Graffiti {
                        x,
                        y,
                        text_idx: (x * 7 + y * 13) % graffiti_count.max(1),
                    });
                    Tile::Wall
                }
                '1' | '2' | '3' | '4' | '5' => {
                    map.mob_spots
                        .push((x, y, ch.to_digit(10).unwrap_or(1) as i32));
                    Tile::Floor
                }
                'n' => {
                    map.npc_spots.push((x, y));
                    Tile::Safe
                }
                '@' => {
                    map.spawn = ((x as f32 + 0.5) * TILE, (y as f32 + 0.5) * TILE);
                    Tile::Floor
                }
                _ => Tile::Wall,
            };
            map.tiles[y * w + x] = t;
        }
    }
    Some(map)
}

pub fn to_custom(map: &Map, name: &str) -> CustomLevel {
    let mut rows = Vec::with_capacity(map.h);
    for y in 0..map.h {
        let mut row = String::with_capacity(map.w);
        for x in 0..map.w {
            let mut ch = match map.tiles[y * map.w + x] {
                Tile::Wall => '#',
                Tile::Floor => '.',
                Tile::Safe => 's',
                Tile::Campfire => 'c',
                Tile::Stairs => '>',
            };
            if map.graffiti.iter().any(|g| g.x == x && g.y == y) {
                ch = 'g';
            }
            if map.chest_spots.iter().any(|&(cx, cy)| cx == x && cy == y) {
                ch = 'C';
            }
            if let Some(&(_, _, t)) = map.mob_spots.iter().find(|&&(mx, my, _)| mx == x && my == y) {
                ch = char::from_digit(t.clamp(1, 5) as u32, 10).unwrap_or('1');
            }
            if map.npc_spots.iter().any(|&(nx, ny)| nx == x && ny == y) {
                ch = 'n';
            }
            let (sx, sy) = map.spawn;
            if (sx / TILE) as usize == x && (sy / TILE) as usize == y {
                ch = '@';
            }
            row.push(ch);
        }
        rows.push(row);
    }
    CustomLevel {
        name: name.to_string(),
        rows,
    }
}
