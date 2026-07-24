# Capitol Dungeon

A lo-fi pixel, top-down dungeon crawler written entirely in **Rust**.

You are dropped into a massive procedurally generated dungeon that vulture
capital has been strip-mining for profit. Every enemy — Rat Racers, Debt
Mites, Payday Pythons, Skeleton Shareholders, Goblin Landlords, Crypto
Ghouls, Troll C.E.O.s, and the dreaded **Vulture Capitalist** boss — will
cheerfully explain to you why profit matters more than life. Every friendly
face — the medics, teachers, bards, veterans, and co-op quartermasters in
the safe rooms — is organizing for community, free education, healthcare
for all, and no more wars. The graffiti is on the wall. Literally.

The fire is free. The clinic is free. The lessons are free.
Everything else, the goblins financialized. Go fix that.

![Rust](https://img.shields.io/badge/rust-stable-orange) ![Platform](https://img.shields.io/badge/platform-native%20%2B%20browser%20(wasm)-blue)

## Features

- **Procedural dungeon levels** — endless descent, difficulty scales with depth,
  themed floors, a Vulture Capitalist boss guarding the stairs every 5th floor
- **Safe rooms & rest areas** — campfires heal for free (no copay, no deductible),
  NPCs hang out, mobs cannot enter
- **The Co-op shop** — at-cost gear, buy and sell, every other floor
- **Real-time combat** — melee swings, four unlockable spells
  (Spark of Rage, Mutual Aid, General Strike, Eat the Rich)
- **Leveling system** — XP, level-ups, growing stats
- **Inventory + paper doll** — 7 equipment slots (weapon, offhand, head, chest,
  legs, boots, ring), 24-slot backpack, usable potions, loot drops, treasure chests
- **16 achievements** — from *Seize the Means* to *Hostile Takeover Averted*
- **Talking enemies & comrades** — enemies broadcast capitalist propaganda in
  speech bubbles; NPCs answer with solidarity
- **Revolutionary graffiti** — readable wall scrawls scattered through the dungeon
- **Level creator** — full in-game tile editor (F9), save/load custom levels,
  test-play instantly; levels are human-editable JSON text art
- **Fully data-driven / customizable** — mobs, items, spells, NPCs, dialogue,
  achievements, and graffiti are plain JSON files with in-file pixel art
- **Runs everywhere** — native macOS/Linux/Windows, and in any browser via WASM

## Controls

| Key | Action |
|---|---|
| WASD / arrows | Move |
| Space / J | Melee attack |
| 1–4 | Cast spells |
| E | Interact (talk, shop, chest, graffiti, rest, descend) |
| I | Inventory & paper doll |
| V | Achievements |
| F9 | Level creator |
| Esc | Menu / close |

## Build & run (native)

Install Rust via [rustup](https://rustup.rs), then:

```bash
cargo run --release
```

## Build & run (browser)

```bash
rustup target add wasm32-unknown-unknown
./build-web.sh
python3 -m http.server 8080 --directory web
```

Then open http://localhost:8080. The `web/` directory is fully static — deploy
it to GitHub Pages, itch.io, Netlify, or any static host to publish the game.

## Customizing everything

All game content lives in `data/*.json`. Edit the files and restart — no
recompiling needed (the native build reads them from disk; the web build
fetches them over HTTP; compiled-in copies are used as fallback).

- `data/mobs.json` — enemies: stats, loot tables, **dialogue lines**, and
  pixel-art sprites (rows of characters + a palette map of char → hex color)
- `data/items.json` — gear and potions: slot (`weapon`, `offhand`, `head`,
  `chest`, `legs`, `boots`, `ring`, `potion`), stat bonuses, prices, sprites
- `data/npcs.json` — friendly NPCs and their lines; set `"shopkeeper": true`
  to make one run the co-op
- `data/spells.json` — cost, damage, radius, heal, unlock level, color
- `data/achievements.json` — stat-based triggers (`kills`, `depth`, `gold`,
  `talks`, `graffiti`, `chests`, `rests`, `bosses`, `buys`, `level`, `deaths`)
- `data/graffiti.json` — the writing on the walls

To add a new mob, copy an existing entry, change the `id`, stats, lines, and
draw a new sprite. `.` is transparent; any other character is looked up in
that entry's `palette`.

## Level creator

Press **F9**. Paint with the mouse (LMB paint, RMB erase), pick brushes with
`1`–`0` or `[`/`]`: floors, walls, safe floors, campfires, stairs, chests,
graffiti, mob spawns by tier, boss spawns, and NPC spawns. `P` sets the player
spawn, `G` generates a procedural level to remix, `F5` saves to
`data/custom_level.json`, `T` test-plays your level immediately.

Custom levels are plain text-art JSON — you can write them by hand:

```
#  wall        .  floor       s  safe floor   c  campfire
>  stairs      C  chest       g  graffiti     n  NPC
1-4 mob tier   @  player spawn
```

## Multiplayer & Discord (roadmap)

The game is architected so a networked build is a natural next step:

- **Multiplayer**: the simulation (`world.rs`) is already separated from
  rendering (`ui.rs`) and input (`main.rs`). A co-op mode would add a small
  WebSocket relay (e.g. `tokio` + `tungstenite` server, `quad-net` on the
  wasm client) syncing player positions/actions, with the host authoritative
  over mob AI and loot.
- **Discord**: Discord **Activities** embed any HTTPS-hosted web app in a
  voice channel via iframe. Deploy `web/` to a static host, register an
  Activity in the Discord developer portal pointing at that URL, and the
  browser build runs inside Discord as-is. (The Embedded App SDK can then be
  layered in for identity/invites.)

## Project layout

```
src/main.rs      game states + input + main loop
src/world.rs     simulation: player, mobs, combat, loot, spells, achievements
src/dungeon.rs   procedural generation + custom-level format
src/content.rs   JSON content loading (moddable defs)
src/ui.rs        all rendering: world, HUD, minimap, menus, shop, paper doll
src/editor.rs    the level creator
src/sprites.rs   char-grid → pixel texture builder
data/*.json      ALL game content (edit me!)
web/             browser build (static, deployable anywhere)
```

## A note on the villains

The enemies are cartoons of vulture capitalism — they monologue about rent,
APRs, layoffs, and quarterly growth while trying to eat you. The game keeps
its satire aimed at systems of exploitation and greed; the heroes answer with
solidarity, mutual aid, free healthcare, free education, and soup.
