mod audio;
mod charedit;
mod content;
mod dungeon;
mod editor;
mod save;
mod settings;
mod sprites;
mod touch;
mod ui;
mod world;

use content::Content;
use dungeon::CustomLevel;
use macroquad::prelude::*;
use world::{GameEvent, Interaction, World};

enum Screen {
    Menu,
    Play,
    Inventory { sel: usize, doll: bool, doll_sel: usize },
    Shop { sel: usize, mode: usize }, // 0 buy · 1 sell · 2 forge
    Dialog { who: String, text: String },
    Achievements,
    Dead,
    Editor,
    CharEditor { sel: usize },
    Log { scroll: usize },
    Skills { branch: usize, tier: usize },
    Commune,
    Settings { sel: usize },
}

/// Run saves are skipped during daily runs (they're their own thing).
fn autosave(world: &World) {
    if world.fixed_seed.is_none() {
        save::write(&save::snapshot(world, true));
    }
}

fn conf() -> Conf {
    Conf {
        window_title: "Capitol Dungeon".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

async fn load_custom_level() -> Option<CustomLevel> {
    let s = macroquad::file::load_string("data/custom_level.json").await.ok()?;
    serde_json::from_str(&s).ok()
}

fn use_or_equip(w: &mut World, content: &Content, idx: usize) {
    let Some(stack) = w.player.inventory.get(idx) else { return };
    let inst = stack.inst.clone();
    let Some(def) = content.item(&inst.id).cloned() else { return };
    if def.is_equippable() {
        w.player.remove_item(idx);
        let name = world::display_name(content, &inst);
        if let Some(prev) = w.player.equipment.insert(def.kind.clone(), inst) {
            w.player.add_item(&prev);
        }
        w.toast(format!("Equipped: {}", name));
        w.add_stat("equips", 1, content);
    } else if def.is_usable() {
        let (_, _, maxhp, maxmp, _) = w.player.totals(content);
        w.player.hp = (w.player.hp + def.heal).min(maxhp);
        w.player.mp = (w.player.mp + def.mana).min(maxmp);
        w.player.remove_item(idx);
        w.toast(format!("Used: {}", def.name));
    } else if def.is_throwable() {
        let key = if def.kind == "bomb" { "G" } else { "F" };
        w.toast(format!("{} is thrown with [{}] during play. Aim away from comrades.", def.name, key));
    }
}

fn unequip(w: &mut World, content: &Content, slot_idx: usize) {
    let slot = world::EQUIP_SLOTS[slot_idx.min(world::EQUIP_SLOTS.len() - 1)];
    if let Some(inst) = w.player.equipment.get(slot).cloned() {
        if w.player.add_item(&inst) {
            w.player.equipment.remove(slot);
            w.toast(format!("Unequipped: {}", world::display_name(content, &inst)));
        } else {
            w.toast("Backpack is full.".to_string());
        }
    }
}

fn movement_input() -> Vec2 {
    let mut mv = vec2(0.0, 0.0);
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        mv.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        mv.y += 1.0;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        mv.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        mv.x += 1.0;
    }
    mv
}

fn cast_input() -> Option<usize> {
    if is_key_pressed(KeyCode::Key1) {
        Some(0)
    } else if is_key_pressed(KeyCode::Key2) {
        Some(1)
    } else if is_key_pressed(KeyCode::Key3) {
        Some(2)
    } else if is_key_pressed(KeyCode::Key4) {
        Some(3)
    } else {
        None
    }
}

fn play_events(w: &mut World, sfx: &audio::Sfx) {
    for ev in w.events.drain(..) {
        match ev {
            GameEvent::Swing => sfx.swing(),
            GameEvent::Hit => sfx.hit(),
            GameEvent::Kill => sfx.kill(),
            GameEvent::Hurt => sfx.hurt(),
            GameEvent::Pickup => sfx.pickup(),
            GameEvent::LevelUp => sfx.levelup(),
            GameEvent::Cast => sfx.cast(),
            GameEvent::Chest => sfx.chest(),
            GameEvent::Rest => sfx.rest(),
            GameEvent::Dash => sfx.dash(),
            GameEvent::Recruit => sfx.recruit(),
            GameEvent::Quest => sfx.recruit(),
            GameEvent::Boom => sfx.boom(),
        }
    }
}

#[macroquad::main(conf)]
async fn main() {
    macroquad::rand::srand(macroquad::miniquad::date::now() as u64);
    settings::load();
    let content = content::load_content().await;
    let mut look = charedit::load().await;
    let mut textures = ui::build_textures(&content);
    textures.set("player", look.build_texture());
    let sfx = audio::Sfx::load_all().await;
    let music = audio::build_music().await;
    let mut music_started = false;
    let mut custom: Option<CustomLevel> = load_custom_level().await;

    let mut world = World::new(&content);
    let mut started = false;
    // resume where we left off: profile always, run if one was saved
    if let Some(sd) = save::read() {
        if save::apply(&mut world, &content, &sd) {
            started = true;
            world.toast(format!(
                "Welcome back, comrade. Depth {} remembers you.",
                world.depth
            ));
        }
    }
    let mut screen = Screen::Menu;
    let mut editor = editor::Editor::new();
    let mut touch_ui = touch::TouchUi::new();

    loop {
        let dt = get_frame_time().min(0.05);
        let known_spells = content
            .spells
            .iter()
            .filter(|s| s.unlock_level <= world.player.level)
            .count();
        let tin = touch_ui.update(known_spells);

        match &mut screen {
            Screen::Menu => {
                ui::draw_menu(custom.is_some());
                if started {
                    let msg = "[Enter] Continue the descent   [N] New run";
                    let td = measure_text(msg, None, 18, 1.0);
                    draw_text(msg, (screen_width() - td.width) / 2.0, 200.0, 18.0, ui_gold());
                }
                let tap_action = tin.taps.first().and_then(|t| ui::menu_hit(*t, custom.is_some()));
                let want_start = is_key_pressed(KeyCode::Enter) || matches!(tap_action, Some(ui::MenuAction::Start));
                let want_custom = (is_key_pressed(KeyCode::L) || matches!(tap_action, Some(ui::MenuAction::Custom))) && custom.is_some();
                let want_char = is_key_pressed(KeyCode::C) || matches!(tap_action, Some(ui::MenuAction::CharEditor));
                let want_editor = is_key_pressed(KeyCode::F9) || matches!(tap_action, Some(ui::MenuAction::Editor));
                let want_daily = is_key_pressed(KeyCode::D) || matches!(tap_action, Some(ui::MenuAction::Daily));
                let want_settings = is_key_pressed(KeyCode::O) || matches!(tap_action, Some(ui::MenuAction::Settings));

                if want_daily {
                    let day = (macroquad::miniquad::date::now() / 86400.0) as u64;
                    macroquad::rand::srand(day.wrapping_mul(7919));
                    world = World::new(&content);
                    world.fixed_seed = Some(day);
                    world.load_level(&content, None);
                    world.toast(format!(
                        "DAILY RUN #{} — everyone gets this dungeon today. No saves; go far.",
                        day % 10000
                    ));
                    started = true;
                    sfx.ui();
                    screen = Screen::Play;
                } else if want_settings {
                    sfx.ui();
                    screen = Screen::Settings { sel: 0 };
                } else if want_start {
                    if !started {
                        world = World::new(&content);
                        started = true;
                    }
                    sfx.ui();
                    screen = Screen::Play;
                } else if is_key_pressed(KeyCode::N) {
                    let stats = world.stats.clone();
                    let unlocked = world.unlocked.clone();
                    world = World::new(&content);
                    world.stats = stats;
                    world.unlocked = unlocked;
                    save::write(&save::snapshot(&world, false));
                    started = true;
                    sfx.ui();
                    screen = Screen::Play;
                } else if want_custom {
                    world = World::new(&content);
                    world.load_level(&content, custom.as_ref());
                    world.toast("Playing your custom level. Nice work, architect.".to_string());
                    started = true;
                    screen = Screen::Play;
                } else if want_char {
                    sfx.ui();
                    screen = Screen::CharEditor { sel: 0 };
                } else if want_editor {
                    screen = Screen::Editor;
                }
                touch_ui.draw(0);
            }

            Screen::Play => {
                if !music_started {
                    audio::start_music(&music);
                    music_started = true;
                }
                let mv = movement_input() + tin.mv;
                let attack = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::J) || tin.attack;
                let cast = cast_input().or(tin.cast);
                let dodge = is_key_pressed(KeyCode::LeftShift)
                    || is_key_pressed(KeyCode::RightShift)
                    || is_key_pressed(KeyCode::K)
                    || tin.dash;
                world.update(dt, &content, mv, attack, cast, dodge);
                play_events(&mut world, &sfx);

                if is_key_pressed(KeyCode::G) {
                    world.throw_item(&content, "bomb");
                }
                if is_key_pressed(KeyCode::F) {
                    world.throw_item(&content, "oil");
                }
                let interact = is_key_pressed(KeyCode::E) || tin.interact;
                if world.player.hp <= 0 {
                    world.add_stat("deaths", 1, &content);
                    world.pick_obituary(&content);
                    // the run ends; the movement's memory does not
                    save::write(&save::snapshot(&world, false));
                    screen = Screen::Dead;
                } else if interact {
                    match world.interact(&content) {
                        Interaction::Dialog { who, text } => {
                            screen = Screen::Dialog { who, text };
                        }
                        Interaction::Shop => {
                            sfx.ui();
                            screen = Screen::Shop { sel: 0, mode: 0 };
                        }
                        Interaction::Descend => {
                            world.descend(&content, None);
                            autosave(&world);
                        }
                        Interaction::Rested => {
                            world.toast("You rest by the fire. Fully restored — no copay, no deductible. (Saved.)".to_string());
                            autosave(&world);
                        }
                        Interaction::ChestLoot { text } => {
                            world.toast(format!("Chest: {}", text));
                        }
                        Interaction::None => {}
                    }
                } else if is_key_pressed(KeyCode::B) && world.near_campfire() {
                    sfx.ui();
                    screen = Screen::Commune;
                } else if is_key_pressed(KeyCode::I) || tin.inventory {
                    sfx.ui();
                    screen = Screen::Inventory { sel: 0, doll: false, doll_sel: 0 };
                } else if is_key_pressed(KeyCode::V) {
                    screen = Screen::Achievements;
                } else if is_key_pressed(KeyCode::T) {
                    screen = Screen::Log { scroll: 0 };
                } else if is_key_pressed(KeyCode::P) {
                    sfx.ui();
                    screen = Screen::Skills { branch: 0, tier: 1 };
                } else if is_key_pressed(KeyCode::F9) {
                    screen = Screen::Editor;
                } else if is_key_pressed(KeyCode::Escape) {
                    autosave(&world);
                    screen = Screen::Menu;
                }

                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                ui::draw_hud(&world, &content);
                touch_ui.draw(known_spells);
            }

            Screen::Inventory { sel, doll, doll_sel } => {
                let cols = 6;
                let mut close = is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::I) || tin.inventory;
                if is_key_pressed(KeyCode::Tab) {
                    *doll = !*doll;
                }
                if *doll {
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                        *doll_sel = doll_sel.saturating_sub(1);
                    }
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                        *doll_sel = (*doll_sel + 1).min(world::EQUIP_SLOTS.len() - 1);
                    }
                    if is_key_pressed(KeyCode::Enter) {
                        unequip(&mut world, &content, *doll_sel);
                        sfx.ui();
                    }
                } else {
                    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                        *sel = sel.saturating_sub(1);
                    }
                    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                        *sel = (*sel + 1).min(world::INV_CAP - 1);
                    }
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                        *sel = sel.saturating_sub(cols);
                    }
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                        *sel = (*sel + cols).min(world::INV_CAP - 1);
                    }
                    if is_key_pressed(KeyCode::Enter) {
                        use_or_equip(&mut world, &content, *sel);
                        sfx.ui();
                    }
                    if is_key_pressed(KeyCode::X) {
                        if let Some(st) = world.player.inventory.get(*sel) {
                            let name = world::display_name(&content, &st.inst);
                            world.player.remove_item(*sel);
                            world.toast(format!("Dropped: {} (a comrade will find it)", name));
                        }
                    }
                }
                // touch: tap cell to select, tap again to use/equip; tap outside closes
                for t in &tin.taps {
                    match ui::inventory_hit(*t) {
                        ui::InvHit::Cell(i) => {
                            if !*doll && *sel == i {
                                use_or_equip(&mut world, &content, i);
                                sfx.ui();
                            } else {
                                *doll = false;
                                *sel = i;
                            }
                        }
                        ui::InvHit::Doll(i) => {
                            if *doll && *doll_sel == i {
                                unequip(&mut world, &content, i);
                                sfx.ui();
                            } else {
                                *doll = true;
                                *doll_sel = i;
                            }
                        }
                        ui::InvHit::Outside => close = true,
                        ui::InvHit::Inside => {}
                    }
                }

                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                let (s, d, ds) = (*sel, *doll, *doll_sel);
                dim();
                ui::draw_inventory(&world, &content, &textures, s, d, ds);
                touch_ui.draw(0);
                if close {
                    screen = Screen::Play;
                }
            }

            Screen::Shop { sel, mode } => {
                let mut close = is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::E) || tin.interact;
                let mut do_transact: Option<usize> = None;
                if is_key_pressed(KeyCode::Tab) {
                    *mode = (*mode + 1) % 3;
                    *sel = 0;
                }
                let list_len = if *mode > 0 {
                    world.player.inventory.len()
                } else {
                    world.shop_stock.len()
                };
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    *sel = sel.saturating_sub(1);
                }
                if (is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S)) && list_len > 0 {
                    *sel = (*sel + 1).min(list_len - 1);
                }
                if is_key_pressed(KeyCode::Enter) && list_len > 0 {
                    do_transact = Some((*sel).min(list_len - 1));
                }
                for t in &tin.taps {
                    match ui::shop_hit(*t, *mode) {
                        ui::ShopHit::Row(i) => {
                            if i < list_len {
                                if *sel == i {
                                    do_transact = Some(i);
                                } else {
                                    *sel = i;
                                }
                            }
                        }
                        ui::ShopHit::ToggleTab => {
                            *mode = (*mode + 1) % 3;
                            *sel = 0;
                        }
                        ui::ShopHit::Outside => close = true,
                        ui::ShopHit::Inside => {}
                    }
                }
                if let Some(i) = do_transact {
                    if *mode == 2 {
                        if world.forge_reroll(&content, i) {
                            sfx.levelup();
                            autosave(&world);
                        }
                    } else if *mode == 1 {
                        if let Some(st) = world.player.inventory.get(i) {
                            let inst = st.inst.clone();
                            let price = (world::inst_value(&content, &inst) / 2).max(1) as i64;
                            let name = world::display_name(&content, &inst);
                            world.player.remove_item(i);
                            world.player.gold += price;
                            sfx.pickup();
                            world.toast(format!("Sold {} for {} gold.", name, price));
                        }
                    } else if i < world.shop_stock.len() {
                        let id = world.shop_stock[i].clone();
                        if let Some(d) = content.item(&id).cloned() {
                            if world.player.gold >= d.value as i64 {
                                if world.player.add_item(&world::ItemInst::plain(&id)) {
                                    world.player.gold -= d.value as i64;
                                    world.add_stat("buys", 1, &content);
                                    sfx.chest();
                                    world.toast(format!("Bought {} — receipt goes to the clinic fund.", d.name));
                                } else {
                                    world.toast("Backpack is full.".to_string());
                                }
                            } else {
                                world.toast("Not enough gold. The co-op can't run on vibes alone.".to_string());
                            }
                        }
                    }
                }

                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                let (s, md) = (*sel, *mode);
                dim();
                ui::draw_shop(&world, &content, &textures, s, md);
                touch_ui.draw(0);
                if close {
                    screen = Screen::Play;
                }
            }

            Screen::Dialog { who, text } => {
                let (w2, t2) = (who.clone(), text.clone());
                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                ui::draw_hud(&world, &content);
                ui::draw_dialog(&w2, &t2);
                touch_ui.draw(0);
                if is_key_pressed(KeyCode::E)
                    || is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                    || !tin.taps.is_empty()
                    || tin.interact
                    || tin.attack
                {
                    screen = Screen::Play;
                }
            }

            Screen::Achievements => {
                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                dim();
                ui::draw_achievements(&world, &content);
                touch_ui.draw(0);
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::V) || !tin.taps.is_empty() {
                    screen = Screen::Play;
                }
            }

            Screen::Dead => {
                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                ui::draw_dead(&world);
                touch_ui.draw(0);
                if is_key_pressed(KeyCode::R) || !tin.taps.is_empty() {
                    let stats = world.stats.clone();
                    let unlocked = world.unlocked.clone();
                    world = World::new(&content);
                    world.stats = stats;
                    world.unlocked = unlocked;
                    world.toast("Back on your feet. The commons remembers.".to_string());
                    screen = Screen::Play;
                }
                if is_key_pressed(KeyCode::Escape) {
                    started = false;
                    screen = Screen::Menu;
                }
            }

            Screen::Editor => {
                if let Some(level) = editor.update(content.graffiti.len()) {
                    world = World::new(&content);
                    world.load_level(&content, Some(&level));
                    world.toast("Test-playing your level. F9 to hop back into the editor.".to_string());
                    custom = Some(level);
                    started = true;
                    screen = Screen::Play;
                } else {
                    editor.draw();
                    if is_key_pressed(KeyCode::Escape) {
                        screen = Screen::Menu;
                    }
                }
            }

            Screen::Log { scroll } => {
                let step = 3;
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    *scroll = (*scroll + step).min(world.log.len().saturating_sub(1));
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    *scroll = scroll.saturating_sub(step);
                }
                let (_, wheel) = mouse_wheel();
                if wheel > 0.0 {
                    *scroll = (*scroll + step).min(world.log.len().saturating_sub(1));
                } else if wheel < 0.0 {
                    *scroll = scroll.saturating_sub(step);
                }
                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                let s = *scroll;
                dim();
                ui::draw_log(&world, s);
                touch_ui.draw(0);
                if is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::T)
                    || !tin.taps.is_empty()
                {
                    screen = Screen::Play;
                }
            }

            Screen::Skills { branch, tier } => {
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    *branch = branch.saturating_sub(1);
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    *branch = (*branch + 1).min(2);
                }
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    *tier = tier.saturating_sub(1).max(1);
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    *tier = (*tier + 1).min(5);
                }
                let mut learn_now = is_key_pressed(KeyCode::Enter);
                for t in &tin.taps {
                    if let Some((b, ti)) = ui::skill_hit(0.0, 0.0, &content, *t) {
                        if *branch == b && *tier == ti {
                            learn_now = true;
                        } else {
                            *branch = b;
                            *tier = ti;
                        }
                    }
                }
                if learn_now {
                    let id = content
                        .skills
                        .iter()
                        .find(|s| s.branch == *branch && s.tier == *tier as i32)
                        .map(|s| s.id.clone());
                    if let Some(id) = id {
                        if world.learn_skill(&content, &id) {
                            sfx.levelup();
                            autosave(&world);
                        } else {
                            sfx.ui();
                        }
                    }
                }
                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                let (b, t) = (*branch, *tier);
                dim();
                ui::draw_skills(&world, &content, b, t);
                touch_ui.draw(0);
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
                    screen = Screen::Play;
                }
            }

            Screen::Commune => {
                let mut build: Option<usize> = None;
                if is_key_pressed(KeyCode::Key1) { build = Some(0); }
                if is_key_pressed(KeyCode::Key2) { build = Some(1); }
                if is_key_pressed(KeyCode::Key3) { build = Some(2); }
                if is_key_pressed(KeyCode::Key4) { build = Some(3); }
                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                dim();
                let rects = ui::draw_commune(&world);
                for t in &tin.taps {
                    let mut hit_any = false;
                    for (i, (rx, ry, rw, rh)) in rects.iter().enumerate() {
                        if t.x >= *rx && t.x <= rx + rw && t.y >= *ry && t.y <= ry + rh {
                            build = Some(i);
                            hit_any = true;
                        }
                    }
                    if !hit_any {
                        screen = Screen::Play;
                    }
                }
                if let Some(i) = build {
                    if world.buy_commune(&content, i) {
                        sfx.chest();
                        autosave(&world);
                    } else {
                        sfx.ui();
                    }
                }
                play_events(&mut world, &sfx);
                touch_ui.draw(0);
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::B) {
                    screen = Screen::Play;
                }
            }

            Screen::Settings { sel } => {
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    *sel = sel.saturating_sub(1);
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    *sel = (*sel + 1).min(2);
                }
                let mut adjust: Option<(usize, bool)> = None;
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    adjust = Some((*sel, false));
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Enter) {
                    adjust = Some((*sel, true));
                }
                for t in &tin.taps {
                    if let Some((row, right)) = ui::settings_row_hit(*t) {
                        *sel = row;
                        adjust = Some((row, right));
                    }
                }
                if let Some((row, up)) = adjust {
                    match row {
                        0 => {
                            let v = settings::volume_pct();
                            settings::set_volume_pct(if up { (v + 10).min(200) } else { v.saturating_sub(10) });
                            sfx.ui();
                        }
                        1 => {
                            let v = settings::shake_pct();
                            settings::set_shake_pct(if up { (v + 10).min(200) } else { v.saturating_sub(10) });
                            sfx.hit();
                        }
                        _ => {
                            let on = !settings::music_on();
                            settings::set_music_on(on);
                            if on {
                                audio::start_music(&music);
                                music_started = true;
                            } else {
                                audio::stop_music(&music);
                            }
                        }
                    }
                    settings::save();
                }
                clear_background(sprites::hex("#14101c"));
                ui::draw_settings(*sel);
                touch_ui.draw(0);
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::O) {
                    settings::save();
                    screen = Screen::Menu;
                }
            }

            Screen::CharEditor { sel } => {
                if charedit::update(&mut look, sel, &tin.taps) {
                    textures.set("player", look.build_texture());
                    sfx.ui();
                }
                let done = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape);
                let player_tex = textures.get("player").cloned();
                if let Some(t) = player_tex {
                    charedit::draw(&look, *sel, &t);
                }
                touch_ui.draw(0);
                if done {
                    charedit::save(&look);
                    screen = Screen::Menu;
                }
            }
        }

        next_frame().await
    }
}

fn dim() {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
}

fn ui_gold() -> Color {
    sprites::hex("#ffd24a")
}
