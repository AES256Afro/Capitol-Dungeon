# Capitol Dungeon — Roadmap

The plan from here to v1.0. Checked items are already in the game.

## v0.2 — "Weight & Welcome" (SHIPPED)

- [x] Combat feel: hit-stop (slow-mo frames on impact), knockback, screen shake,
      critical hits, impact particles, swing arcs, lunge on attack
- [x] Synthesized chiptune SFX (no asset files) — hits, kills, spells, pickups,
      level-ups, chests, resting
- [x] Touch controls: virtual joystick + attack/interact/spell/inventory buttons —
      playable on iPad & phones in the browser; menus are tappable
- [x] Character editor: hairstyles + hair/skin/shirt/pants palettes, saved on desktop
- [x] Living NPCs: wandering in safe rooms, NPC-to-NPC call-and-response banter
      (rate-limited, no spam), big per-NPC dialogue pools
- [x] Rebel fighters in the wild (Red Brigadier Kass, Shieldmate Bruna) who hunt
      mobs, take damage, shout battle cries, and can fall in battle
- [x] Mob targeting: enemies choose between the player and nearby fighters
- [x] 5 new enemies: Ad Slime, HR Wraith, Lobbyist Leech, Pinkerton Specter,
      Golden Parachutist (13 total incl. the Vulture Capitalist boss)
- [x] ~35 new items (58 total): weapon tiers up to the Guillotine Edge, shields,
      armor sets, rings, street-medic kits, strike coffee, community stew…

## v0.3 — "Persistence & Polish" (MOSTLY SHIPPED)

- [x] Save/load runs — desktop: JSON files; browser: localStorage via a tiny
      JS bridge plugin; autosaves at campfires, on descent, and on exit
- [x] Persistent meta-progression: achievements, stats, and character look
      survive page reloads everywhere
- [x] Music: procedural lo-fi backing loop (same synth pipeline as SFX)
- [x] Dodge-roll with i-frames (Shift/K or the DASH touch button)
- [x] Humor pass: Office-style middle management, Python-esque absurdism,
      propane-grade calm, portal-scientist nihilism, and gremlin schemes —
      3 new NPCs, 3 new enemies, 13 new banter pairs
- [ ] Status effects: poison (snakes), slow (slime ads), burn (torch), shield wall
- [ ] Enemy attack telegraphs
- [ ] Ranged enemies (Repo Archer, Subpoena Thrower) and enemy projectiles
- [ ] Minibosses every 2-3 floors with unique mechanics
- [ ] Gamepad support (macroquad reads gamepads on native; browser Gamepad API)
- [ ] Settings screen: volume, screen-shake intensity, text size, key remapping

## v0.4 — "The Commune Grows" (SHIPPED in v0.7)

- [x] Recruitable fighters: walk up to a comrade in the wild, ask them to join
      (E), and they follow you between floors — banner overhead, achievements
- [x] Quests from NPCs: 9 data-driven quests (quests.json), accepted by
      talking, tracked live under the minimap, gold + item rewards
- [x] Safe-room building: [B] at any campfire — Clinic Beds, Free Library,
      Communal Forge, Soup Kitchen (Well Fed buff); persists in saves
- [x] Minibosses: The Landlord King and The Algorithm claim rooms on
      non-boss floors (35% from depth 2); The Algorithm inflicts slow
- [x] Daily runs: [D] on the menu — date-seeded dungeon, same for every
      player, fresh character, no saves, bragging rights only
- [x] Settings: [O] — volume, screen-shake intensity, music toggle; persisted
- [ ] Deeper recruits: loyalty, gear-sharing, revival at campfires
- [ ] Reputation: helping fighters/NPCs unlocks dialogue and discounts
- [ ] The Board: multi-phase final boss

## v0.5 — "Together" (multiplayer)

- [ ] Co-op netcode: small `tokio`+`tungstenite` WebSocket relay server; host is
      authoritative over mobs/loot, clients send input (the sim in `world.rs` is
      already isolated from rendering/input for exactly this)
- [ ] 2-4 player online co-op, drop-in at the campfire
- [ ] Shared loot rules: it's a commune — need before greed, automatically
- [ ] Server browser + friend invites via room codes

## v0.6 — "Everywhere"

- [ ] Discord Activity: register the hosted web build as an Embedded App;
      identity + invites through the Discord SDK, co-op in voice channels
- [ ] itch.io + GitHub Pages auto-deploy on push (CI workflow)
- [x] PWA manifest + service worker: installable, offline-capable on iPad/Android
      (needs HTTPS hosting, e.g. GitHub Pages, to install)
- [ ] Native packages: .app / .exe / .AppImage via CI; Steam later if wanted

## v1.0 — "The General Strike"

- [ ] Campaign arc: 25 floors, 5 acts, final confrontation with The Board
- [ ] Ending cinematics (pixel-art vignettes) + epilogue reflecting your choices
- [ ] Full mod loader UI: pick content packs (data/ folders) from inside the game
- [ ] Level-sharing: publish custom levels as JSON gists, play by URL
- [ ] Localization (the revolution is multilingual)
- [ ] Accessibility pass: colorblind-safe palettes, reduced-motion mode,
      screen-reader-friendly menus on web

## Always

- Keep everything data-driven and moddable
- Keep the browser build a single static folder
- Keep the satire punching up


## The Broadcast Arc (from the spiritual-successor design doc)

### v0.8 — "You're Live" (SHIPPED)

- [x] Sentient System AI announcer: categorized, data-driven snark
      (`data/system.json`) with an on-screen banner and Broadcast log channel
- [x] Live viewer count + hype trend on the HUD; kills, crits, bosses, and
      explosions spike engagement
- [x] Sponsor crates: viewer milestones airdrop Bronze/Silver/Gold crates
      with guaranteed-affix loot
- [x] Achievement boxes: every unlock ships with a Bronze Box (gold + hype)
- [x] Fan mail & hate mail from named alien viewers (buffs included;
      hate is still engagement)
- [x] Floor cost-optimization timers: linger too long and the System
      deploys "efficiency waves"
- [x] Death broadcasts: cause-of-death + a System obituary on the death screen
- [x] The System's footwear obsession (go bootless, get commentary)
- [x] Explosive Engineering: Solidarity Dynamite (G) — friendly fire,
      wall demolition, knockback, oil ignition
- [x] Trap & Kiting: Slick Oil Flasks (F) — enemies slip; fire spreads

### v0.9 — "Sweeps Week" (next)

- [ ] Viewer micro-polls: periodic audience votes apply floor mutators
      (double loot, elite spawns, low gravity knockback, fog)
- [ ] Post-floor summary card: top play, damage dealt/taken, viewer rating
- [ ] Leaderboards (local): best daily-run depth, fastest floor, kill records
- [ ] Loot boxes as items: Bronze/Silver/Gold Weapon Boxes in drop tables
- [ ] Appraisal skill: scan enemies for stats, weaknesses, and System snark
- [ ] Trap rooms: pressure plates, dart walls — disarm and re-deploy on mobs
- [ ] Corpse logistics: bodies persist, block corridors, attract scavenger
      grubs that level up if left unchecked
- [ ] Bounty events: high hype paints a bounty on you — hunter mini-elites

### v1.0+ — "Season Finale" ideas from the doc

- [ ] Race/class milestone picks at level thresholds
- [ ] Companion leveling for recruited fighters (gear, stats, popularity)
- [ ] Descent slots + stairwell guardians (boss gate every floor)
- [ ] Sponsors offering pacts (stat boosts for gear ad placement)
- [ ] Crawler-union events: coordinate with fighters, trigger System
      union-busting waves
- [ ] Secret glitched sub-floors behind demolished walls
- [ ] Psychological decay meter cleared by campfires and companions

Deliberate adaptations: our satire stays anti-vulture-capital (the System
IS the corporate broadcaster); item and character names are original to
Capitol Dungeon rather than borrowed from any book.
