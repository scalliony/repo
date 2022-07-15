mod client;
mod game;
mod gui;
mod logger;
mod opts;
mod util;
mod view;
use macroquad::prelude::*;
use util::*;

#[macroquad::main(window)]
async fn main() {
    gui::cfg(|ctx| {
        ctx.set_visuals(egui::Visuals { window_rounding: 0.0.into(), ..Default::default() })
    });

    let mut client = game::Client::new_local();

    let mut view = view::View::default();
    let mut view_tracker = game::ViewTracker::new();

    let mut code = Code::Binary(
        include_bytes!("../../target/wasm32-unknown-unknown/release/explorer.wasm").to_vec(),
    );
    let mut programs = game::Programs::new();
    let mut program: usize = 0;
    let mut spawn: Hex = Hex::default();

    let mut state = game::AnimatedState::default();

    let mut tick_duration: F = 1.;
    let mut tick_lerp: F = 0.;

    loop {
        if let Some(cr) = view_tracker.track(&mut client, view.update()) {
            state.apply_one(bulb::dto::Event::Cells(cr));
        }
        client.update();
        programs.update();

        if state.apply(&mut client) {
            tick_lerp = 0.;
        } else {
            tick_lerp = (tick_lerp + get_frame_time() / tick_duration).min(1.);
        }

        gui::ui(|ui| {
            use egui::*;
            Window::new("Program Editor").show(ui, |ui| {
                ui.code_editor(&mut code);
                if ui.add_enabled(!code.as_bytes().is_empty(), Button::new("Compile")).clicked() {
                    programs.compile(&mut client, std::mem::take(&mut code).into_bytes().into())
                }
            });

            if !programs.as_ref().is_empty() {
                Window::new("Spawn").show(ui, |ui| {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("At:");
                        let mut q = spawn.q();
                        let mut r = spawn.r();
                        ui.add(egui::DragValue::new(&mut q));
                        ui.add(egui::DragValue::new(&mut r));
                        spawn = Hex::new(q, r);
                    });
                    ComboBox::from_label("Program").show_index(
                        ui,
                        &mut program,
                        programs.as_ref().len(),
                        |i| usize::from(programs.as_ref()[i]).to_string(),
                    );
                    if ui.add_enabled(!programs.as_ref().is_empty(), Button::new("Spawn")).clicked()
                    {
                        game::spawn(&mut client, programs.as_ref()[program], spawn)
                    }
                });
            }
        });

        draw(&view, &state, tick_lerp);
        next_frame().await
    }
}
fn window() -> Conf {
    logger::init();
    Conf {
        window_title: concat!("Scalliony ", env!("CARGO_PKG_VERSION")).to_string(),
        sample_count: 4,
        icon: None,
        ..Conf::default()
    }
}

#[inline]
fn draw(view: &view::View, state: &game::AnimatedState, lerp: F) {
    clear_background(Color::new(0.1, 0.1, 0.1, 1.0));
    let mut bots = std::collections::BTreeSet::<bulb::dto::BotId>::new();
    let hex_area = Size::new(screen_width() * 0.75, screen_height());
    let center = Pos::new(screen_width() / 2.0, screen_height() / 2.0);
    let (rad, range, mapper) = view.iter(center, hex_area);
    for h in range {
        use bulb::dto::Cell;
        let pos = mapper.map(h);
        let (from, to) = state.at(h);
        if let Some(Cell::Bot(id)) = from {
            bots.insert(*id);
        }
        //TODO: animate cell change
        if let Some(cell) = to {
            match cell {
                Cell::Ground => draw_cell(pos, rad, LIGHTGRAY),
                Cell::Bot(id) => {
                    bots.insert(*id);
                    draw_cell(pos, rad, LIGHTGRAY);
                }
                Cell::Wall => draw_cell(pos, rad, DARKGRAY),
            }
        }
        draw_border(pos, rad);
    }
    for id in bots {
        let (from, to) = state.bot(id);
        if let Some(bot) = to {
            let prev_at = from.map_or(bot.at, |prev| prev.at);
            let center = match bot.collide {
                Some(collide) => {
                    if lerp < 1. / 2. {
                        lerp_hex(prev_at, collide, lerp)
                    } else {
                        lerp_hex(collide, bot.at, lerp)
                    }
                }
                None => lerp_hex(prev_at, bot.at, lerp),
            };
            let rot = match (from.and_then(|bot| bot.dir), bot.dir) {
                (_, None) => None,
                (None, Some(to)) => Some((to - Direction::Up).degre() as F),
                //TODO: lerp_degre
                (Some(from), Some(to)) => Some(lerp_f(
                    (from - Direction::Up).degre() as F,
                    (to - Direction::Up).degre() as F,
                    lerp,
                )),
            };
            //TODO: rot
            draw_cell(mapper.map_f(center), rad * 0.8, PURPLE);
        } else {
            //TODO: animate death
        }
    }

    gui::draw();
    draw_text(&format!("FPS: {}", get_fps()), 20.0, 20.0, 30.0, BLUE);
}

enum Code {
    Text(String),
    Binary(Vec<u8>),
}
impl Code {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Code::Text(s) => s.as_bytes(),
            Code::Binary(v) => v.as_slice(),
        }
    }
    fn into_bytes(self) -> Vec<u8> {
        match self {
            Code::Text(s) => s.into_bytes(),
            Code::Binary(v) => v,
        }
    }
}
impl Default for Code {
    fn default() -> Self {
        Code::Text(Default::default())
    }
}
impl AsRef<str> for Code {
    fn as_ref(&self) -> &str {
        match self {
            Code::Text(s) => &s,
            Code::Binary(_) => "<binary>",
        }
    }
}
impl egui::TextBuffer for Code {
    fn is_mutable(&self) -> bool {
        matches!(self, Code::Text(_))
    }
    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        match self {
            Code::Text(s) => s.insert_text(text, char_index),
            Code::Binary(_) => 0,
        }
    }
    fn delete_char_range(&mut self, char_range: std::ops::Range<usize>) {
        if let Code::Text(s) = self {
            s.delete_char_range(char_range);
        }
    }
}
