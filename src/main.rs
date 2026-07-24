mod content;
mod dungeon;
mod editor;
mod sprites;
mod ui;
mod world;

use content::Content;
use dungeon::CustomLevel;
use macroquad::prelude::*;
use world::{Interaction, World};

enum Screen {
    Menu,
    Play,
    Inventory { sel: usize, doll: bool, doll_sel: usize },
    Shop { sel: usize, selling: bool },
    Dialog { who: String, text: String },
    Achievements,
    Dead,
    Editor,
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
    let id = stack.id.clone();
    let Some(def) = content.item(&id).cloned() else { return };
    if def.is_equippable() {
        w.player.remove_item(idx);
        if let Some(prev) = w.player.equipment.insert(def.kind.clone(), id) {
            w.player.add_item(&prev);
        }
        w.toast(format!("Equipped: {}", def.name));
        w.add_stat("equips", 1, content);
    } else if def.is_usable() {
        let (_, _, maxhp, maxmp, _) = w.player.totals(content);
        w.player.hp = (w.player.hp + def.heal).min(maxhp);
        w.player.mp = (w.player.mp + def.mana).min(maxmp);
        w.player.remove_item(idx);
        w.toast(format!("Used: {}", def.name));
    }
}

fn unequip(w: &mut World, content: &Content, slot_idx: usize) {
    let slot = world::EQUIP_SLOTS[slot_idx.min(world::EQUIP_SLOTS.len() - 1)];
    if let Some(id) = w.player.equipment.get(slot).cloned() {
        if w.player.add_item(&id) {
            w.player.equipment.remove(slot);
            let name = content.item(&id).map(|d| d.name.clone()).unwrap_or(id);
            w.toast(format!("Unequipped: {}", name));
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

#[macroquad::main(conf)]
async fn main() {
    macroquad::rand::srand(macroquad::miniquad::date::now() as u64);
    let content = content::load_content().await;
    let textures = ui::build_textures(&content);
    let mut custom: Option<CustomLevel> = load_custom_level().await;

    let mut world = World::new(&content);
    let mut started = false;
    let mut screen = Screen::Menu;
    let mut editor = editor::Editor::new();

    loop {
        let dt = get_frame_time().min(0.05);

        match &mut screen {
            Screen::Menu => {
                ui::draw_menu(custom.is_some());
                if started {
                    let msg = "[Enter] Continue the descent   [N] New run";
                    let td = measure_text(msg, None, 18, 1.0);
                    draw_text(msg, (screen_width() - td.width) / 2.0, 200.0, 18.0, ui_gold());
                }
                if is_key_pressed(KeyCode::Enter) {
                    if !started {
                        world = World::new(&content);
                        started = true;
                    }
                    screen = Screen::Play;
                }
                if is_key_pressed(KeyCode::N) {
                    world = World::new(&content);
                    started = true;
                    screen = Screen::Play;
                }
                if is_key_pressed(KeyCode::L) && custom.is_some() {
                    world = World::new(&content);
                    world.load_level(&content, custom.as_ref());
                    world.toast("Playing your custom level. Nice work, architect.".to_string());
                    started = true;
                    screen = Screen::Play;
                }
                if is_key_pressed(KeyCode::F9) {
                    screen = Screen::Editor;
                }
            }

            Screen::Play => {
                let mv = movement_input();
                let attack = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::J);
                let cast = cast_input();
                world.update(dt, &content, mv, attack, cast);

                if world.player.hp <= 0 {
                    world.add_stat("deaths", 1, &content);
                    screen = Screen::Dead;
                } else if is_key_pressed(KeyCode::E) {
                    match world.interact(&content) {
                        Interaction::Dialog { who, text } => {
                            screen = Screen::Dialog { who, text };
                        }
                        Interaction::Shop => {
                            screen = Screen::Shop { sel: 0, selling: false };
                        }
                        Interaction::Descend => {
                            world.descend(&content, None);
                        }
                        Interaction::Rested => {
                            world.toast("You rest by the fire. Fully restored — no copay, no deductible.".to_string());
                        }
                        Interaction::ChestLoot { text } => {
                            world.toast(format!("Chest: {}", text));
                        }
                        Interaction::None => {}
                    }
                } else if is_key_pressed(KeyCode::I) {
                    screen = Screen::Inventory { sel: 0, doll: false, doll_sel: 0 };
                } else if is_key_pressed(KeyCode::V) {
                    screen = Screen::Achievements;
                } else if is_key_pressed(KeyCode::F9) {
                    screen = Screen::Editor;
                } else if is_key_pressed(KeyCode::Escape) {
                    screen = Screen::Menu;
                }

                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                ui::draw_hud(&world, &content);
            }

            Screen::Inventory { sel, doll, doll_sel } => {
                let cols = 6;
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
                    }
                    if is_key_pressed(KeyCode::X) {
                        if let Some(st) = world.player.inventory.get(*sel) {
                            let name = content
                                .item(&st.id)
                                .map(|d| d.name.clone())
                                .unwrap_or_else(|| st.id.clone());
                            world.player.remove_item(*sel);
                            world.toast(format!("Dropped: {} (a comrade will find it)", name));
                        }
                    }
                }
                let close = is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::I);

                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                let (s, d, ds) = (*sel, *doll, *doll_sel);
                dim();
                ui::draw_inventory(&world, &content, &textures, s, d, ds);
                if close {
                    screen = Screen::Play;
                }
            }

            Screen::Shop { sel, selling } => {
                if is_key_pressed(KeyCode::Tab) {
                    *selling = !*selling;
                    *sel = 0;
                }
                let list_len = if *selling {
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
                    let i = (*sel).min(list_len - 1);
                    if *selling {
                        if let Some(st) = world.player.inventory.get(i) {
                            let id = st.id.clone();
                            if let Some(d) = content.item(&id).cloned() {
                                world.player.remove_item(i);
                                world.player.gold += (d.value / 2) as i64;
                                world.toast(format!("Sold {} for {} gold.", d.name, d.value / 2));
                            }
                        }
                    } else {
                        let id = world.shop_stock[i].clone();
                        if let Some(d) = content.item(&id).cloned() {
                            if world.player.gold >= d.value as i64 {
                                if world.player.add_item(&id) {
                                    world.player.gold -= d.value as i64;
                                    world.add_stat("buys", 1, &content);
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
                let close = is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::E);

                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                let (s, sl) = (*sel, *selling);
                dim();
                ui::draw_shop(&world, &content, &textures, s, sl);
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
                if is_key_pressed(KeyCode::E)
                    || is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                {
                    screen = Screen::Play;
                }
            }

            Screen::Achievements => {
                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                dim();
                ui::draw_achievements(&world, &content);
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::V) {
                    screen = Screen::Play;
                }
            }

            Screen::Dead => {
                clear_background(BLACK);
                ui::draw_world(&world, &content, &textures);
                ui::draw_dead(&world);
                if is_key_pressed(KeyCode::R) {
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
