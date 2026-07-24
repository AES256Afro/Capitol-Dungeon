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

## v0.3 — "Persistence & Polish"

- [ ] Save/load runs (native: JSON saves; browser: localStorage via quad-storage)
- [ ] Persistent meta-progression: achievements, character look, best depth
      survive page reloads everywhere
- [ ] Music: procedural lo-fi backing loops (same synth pipeline as SFX)
- [ ] Status effects: poison (snakes), slow (slime ads), burn (torch), shield wall
- [ ] Dash/dodge-roll with i-frames; enemy attack telegraphs
- [ ] Ranged enemies (Repo Archer, Subpoena Thrower) and enemy projectiles
- [ ] Minibosses every 2-3 floors with unique mechanics
- [ ] Gamepad support (macroquad reads gamepads on native; browser Gamepad API)
- [ ] Settings screen: volume, screen-shake intensity, text size, key remapping

## v0.4 — "The Commune Grows"

- [ ] Recruitable fighters: rescue a pinned-down comrade and they follow you
- [ ] Quests from NPCs ("clear the co-op's supply route", "find Fern's seeds",
      "escort the medic to floor 3")
- [ ] Safe-room building: spend gold to add clinic beds (faster heal), a library
      (spell discounts), a forge (upgrade gear) — collectively owned, obviously
- [ ] Reputation: helping fighters/NPCs unlocks dialogue, discounts, and allies
      at the boss fight
- [ ] Boss variety: The Landlord King, The Algorithm, The Board (multi-phase)
- [ ] Daily-seed runs with a shareable seed code

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
- [ ] PWA manifest + service worker: installable, offline-capable on iPad/Android
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
