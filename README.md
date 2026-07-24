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
- **Combat with weight** — hit-stop, knockback, screen shake, critical hits,
  impact particles, swing arcs, and synthesized chiptune SFX (no asset files)
- **Playable on iPad & phones** — virtual joystick + touch buttons appear on the
  first touch; menus are fully tappable; runs in any mobile browser
- **Character editor** — hairstyles and hair/skin/shirt/pants palettes, with a
  live preview; your look persists on desktop
- **Living NPCs** — comrades wander the safe rooms and talk *to each other*
  (call-and-response banter, rate-limited so nobody spams), each with a deep
  pool of dialogue
- **Rebel fighters in the wild** — Red Brigadier Kass and Shieldmate Bruna roam
  the dungeon fighting monsters on their own, shouting battle cries; enemies
  choose between you and them, and comrades can fall
- **13 enemy types** — rats, mites, pythons, skeletons, goblins, ghouls, trolls,
  slimes, wraiths, leeches, specters, parachutists, and the Vulture boss —
  all broadcasting pro-profit propaganda
- **Safe rooms & rest areas** — campfires heal for free (no copay, no deductible),
  NPCs hang out, mobs cannot enter
- **The Co-op shop** — at-cost gear, buy and sell, every other floor
- **Real-time combat + spells** — melee plus four unlockable spells
  (Spark of Rage, Mutual Aid, General Strike, Eat the Rich)
- **Leveling system** — XP, level-ups, growing stats
- **Inventory + paper doll** — 7 equipment slots, 24-slot backpack, **58 items**
  across weapons, shields, armor, rings, and community-kitchen consumables
- **16 achievements** — from *Seize the Means* to *Hostile Takeover Averted*
- **Revolutionary graffiti** — readable wall scrawls scattered through the dungeon
- **Level creator** — full in-game tile editor (F9), save/load custom levels,
  test-play instantly; levels are human-editable JSON text art
- **Fully data-driven / customizable** — mobs, items, spells, NPCs, dialogue,
  banter, achievements, and graffiti are plain JSON files with in-file pixel art
- **Dodge-roll with i-frames** — Shift/K or the DASH button; ghost-trail included
- **Recruitable comrades** — ask a fighter in the wild to join you (E); they
  follow you between floors under a little red banner
- **Saves that stick** — autosave at campfires, on descent, and on exit;
  localStorage in the browser, JSON files on desktop; achievements and your
  character's look persist across sessions
- **Procedural lo-fi soundtrack** — an Am-F-C-G chiptune loop composed in code
  at startup, plus full SFX
- **Installable on iPad/Android** — PWA manifest + offline service worker; host
  over HTTPS, "Add to Home Screen", play fullscreen offline
- **Sitcom-grade satire** — middle-management mummies, unyielding knights in
  absurd denial, gaslighting gremlins, a propane-calm handyman, a portal-weary
  cellar scientist, and the Mud Collective's thirty-seven-year-old delegate
- **The Solidarity Tree** — a 3-branch, 15-node skill tree (Picket Line /
  Mutual Aid / Class Consciousness): crit builds, dodge builds, cheap-spell
  builds; one point per level, data-driven in `data/skills.json`
- **Dungeon chat log** — a live feed plus a full scrollable transcript (T) of
  every mob taunt, comrade conversation, emote, graffito, and item pickup
- **Emotes** — idle enemies *laminate non-compete agreements* and *do trust
  falls, alone*; comrades *stir the soup approvingly* (`data/emotes.json`)
- **Rich loot** — every enemy has a 4-6 item drop table across **65 items**
- **Weapon classes** — daggers strike fast, hammers hit slow with huge
  knockback, spears out-range everything, scythes sweep a wide arc, swords
  balance it all; 18 weapons across 5 classes (`wclass` in items.json)
- **Random item bonuses** — found gear can roll prefixes (Sturdy, Vicious,
  Nimble…) and suffixes (of the Commune, of the Long March, of Deep Roots…)
  with stat bonuses and boosted sell value; affixed loot glints on the
  ground and shows gold in your pack (`data/affixes.json`)
- **Runs everywhere** — native macOS/Linux/Windows, and in any browser via WASM

See [ROADMAP.md](ROADMAP.md) for where this is going (saves, music, quests,
co-op multiplayer, Discord Activities, and more).

## Controls

| Key | Action |
|---|---|
| WASD / arrows | Move |
| Space / J | Melee attack |
| 1–4 | Cast spells |
| Shift / K | Dodge-roll (i-frames) |
| E | Interact (talk, shop, chest, graffiti, rest, descend, **recruit fighters**) |
| I | Inventory & paper doll |
| P | Skill tree (the Solidarity Tree) |
| T | Chat log (everything mobs & comrades have said) |
| V | Achievements |
| C | Character editor (from menu) |
| F9 | Level creator |
| Esc | Menu / close |

**Touch (iPad / phone):** drag on the left half of the screen for the joystick;
ATK / E / spell / INV buttons sit on the right. Tap menu rows to select, tap
again to activate; tap outside a panel to close it.

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
