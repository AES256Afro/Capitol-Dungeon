//! Virtual touch controls: left-half joystick + right-side action buttons.
//! They appear on the first touch, so desktop players never see them.

use macroquad::input::{touches, TouchPhase};
use macroquad::prelude::*;

pub struct TouchInput {
    pub mv: Vec2,
    pub attack: bool,
    pub interact: bool,
    pub inventory: bool,
    pub cast: Option<usize>,
    /// Raw tap positions (screen px) that didn't hit any control — menus use these.
    pub taps: Vec<Vec2>,
}

pub struct TouchUi {
    pub enabled: bool,
    stick_id: Option<u64>,
    stick_origin: Vec2,
    stick_now: Vec2,
}

struct Btn {
    x: f32,
    y: f32,
    r: f32,
    label: &'static str,
    kind: BtnKind,
}

#[derive(Clone, Copy, PartialEq)]
enum BtnKind {
    Attack,
    Interact,
    Inventory,
    Spell(usize),
}

fn buttons(spell_count: usize) -> Vec<Btn> {
    let w = screen_width();
    let h = screen_height();
    let mut b = vec![
        Btn { x: w - 78.0, y: h - 92.0, r: 50.0, label: "ATK", kind: BtnKind::Attack },
        Btn { x: w - 182.0, y: h - 64.0, r: 36.0, label: "E", kind: BtnKind::Interact },
        Btn { x: w - 46.0, y: h - 210.0, r: 28.0, label: "INV", kind: BtnKind::Inventory },
    ];
    for i in 0..spell_count.min(4) {
        b.push(Btn {
            x: w - 250.0 + i as f32 * 58.0,
            y: h - 170.0,
            r: 25.0,
            label: match i {
                0 => "1",
                1 => "2",
                2 => "3",
                _ => "4",
            },
            kind: BtnKind::Spell(i),
        });
    }
    b
}

impl TouchUi {
    pub fn new() -> Self {
        TouchUi {
            enabled: false,
            stick_id: None,
            stick_origin: vec2(0.0, 0.0),
            stick_now: vec2(0.0, 0.0),
        }
    }

    pub fn update(&mut self, spell_count: usize) -> TouchInput {
        let mut out = TouchInput {
            mv: vec2(0.0, 0.0),
            attack: false,
            interact: false,
            inventory: false,
            cast: None,
            taps: Vec::new(),
        };
        let ts = touches();
        if ts.is_empty() && self.stick_id.is_none() {
            return out;
        }
        let btns = buttons(spell_count);

        for t in &ts {
            self.enabled = true;
            match t.phase {
                TouchPhase::Started => {
                    let mut hit = false;
                    for b in &btns {
                        if (t.position.x - b.x).powi(2) + (t.position.y - b.y).powi(2) < b.r * b.r {
                            match b.kind {
                                BtnKind::Attack => out.attack = true,
                                BtnKind::Interact => out.interact = true,
                                BtnKind::Inventory => out.inventory = true,
                                BtnKind::Spell(i) => out.cast = Some(i),
                            }
                            hit = true;
                            break;
                        }
                    }
                    if !hit {
                        if t.position.x < screen_width() * 0.45 && self.stick_id.is_none() {
                            self.stick_id = Some(t.id);
                            self.stick_origin = t.position;
                            self.stick_now = t.position;
                        } else {
                            out.taps.push(t.position);
                        }
                    }
                }
                TouchPhase::Moved | TouchPhase::Stationary => {
                    if Some(t.id) == self.stick_id {
                        self.stick_now = t.position;
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if Some(t.id) == self.stick_id {
                        self.stick_id = None;
                    }
                }
            }
        }
        // if the finger vanished without an Ended event, release the stick
        if self.stick_id.is_some() && !ts.iter().any(|t| Some(t.id) == self.stick_id) {
            self.stick_id = None;
        }

        if self.stick_id.is_some() {
            let d = self.stick_now - self.stick_origin;
            if d.length() > 8.0 {
                out.mv = (d / 48.0).clamp_length_max(1.0);
            }
        }
        out
    }

    /// Draw the overlay (only after the first touch has been seen).
    pub fn draw(&self, spell_count: usize) {
        if !self.enabled {
            return;
        }
        for b in buttons(spell_count) {
            draw_circle(b.x, b.y, b.r, Color::new(1.0, 1.0, 1.0, 0.10));
            draw_circle_lines(b.x, b.y, b.r, 2.0, Color::new(1.0, 1.0, 1.0, 0.35));
            let td = measure_text(b.label, None, 18, 1.0);
            draw_text(
                b.label,
                b.x - td.width / 2.0,
                b.y + 6.0,
                18.0,
                Color::new(1.0, 1.0, 1.0, 0.6),
            );
        }
        if self.stick_id.is_some() {
            draw_circle_lines(
                self.stick_origin.x,
                self.stick_origin.y,
                48.0,
                2.0,
                Color::new(1.0, 1.0, 1.0, 0.3),
            );
            let d = (self.stick_now - self.stick_origin).clamp_length_max(48.0);
            draw_circle(
                self.stick_origin.x + d.x,
                self.stick_origin.y + d.y,
                20.0,
                Color::new(1.0, 1.0, 1.0, 0.25),
            );
        }
    }
}
