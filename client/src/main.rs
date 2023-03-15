mod client;
mod game;
mod logger;
mod opts;
mod util;
mod view;
use chrono::{TimeZone, Utc};
use egui_macroquad::egui;
use macroquad::prelude::*;
use util::*;

#[macroquad::main(window)]
async fn main() {
    egui_macroquad::cfg(|ctx| {
        ctx.set_visuals(egui::Visuals {
            window_rounding: 0.0.into(),
            ..Default::default()
        })
    });

    let mut client = game::Client::new(if game::Client::HAS_ONLINE {
        Some(("127.0.0.1:3000", false))
    } else {
        None
    })
    .unwrap();
    //FIXME: GameState and GUI
    while !client.connected() {
        next_frame().await
    }

    let mut view = view::View::default();
    let mut view_tracker = game::ViewTracker::new();

    let mut code = Code::default();
    let mut program: usize = 0;
    let mut spawn: Hex = Hex::default();

    let mut event_buf = game::EventBuffer::default();
    let mut state = game::AnimatedState::default();

    let mut tick_duration: F = 1.;
    let mut tick_lerp: F = 0.;

    loop {
        view_tracker.track(&mut client, view.update());
        client.update();

        if state.apply(&mut client) {
            tick_lerp = 0.;
        } else {
            tick_lerp = (tick_lerp + get_frame_time() / tick_duration).min(1.);
        }

        egui_macroquad::ui(|ctx| {
            use egui::*;
            Window::new("Program Editor").show(ctx, |ui| {
                if let Some(drop) = dropped_bytes() {
                    code = String::from_utf8(drop)
                        .map_or_else(|err| Code::Binary(err.into_bytes()), Code::Text)
                }
                ui.code_editor(&mut code);
                if ui
                    .add_enabled(!code.as_bytes().is_empty(), Button::new("Compile"))
                    .clicked()
                {
                    game::compile(&mut client, std::mem::take(&mut code).into_bytes().into())
                }
            });

            if !state.programs().is_empty() {
                Window::new("Spawn").show(ctx, |ui| {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("At:");
                        let mut q = spawn.q();
                        let mut r = spawn.r();
                        ui.add(DragValue::new(&mut q));
                        ui.add(DragValue::new(&mut r));
                        spawn = Hex::new(q, r);
                    });
                    ComboBox::from_label("Program").show_index(
                        ui,
                        &mut program,
                        state.programs().len(),
                        |i| usize::from(state.programs()[i]).to_string(),
                    );
                    if ui
                        .add_enabled(!state.programs().is_empty(), Button::new("Spawn"))
                        .clicked()
                    {
                        game::spawn(&mut client, state.programs()[program], spawn)
                    }
                });
            }

            Window::new("Tracker")
                .title_bar(false)
                .anchor(Align2::RIGHT_TOP, (-5., 5.))
                .auto_sized()
                .show(ctx, |ui| {
                    ui.small(format!(
                        "FPS: {} - {:.3}ms",
                        get_fps(),
                        get_frame_time() * 1000.
                    ));
                    if let Some((id, ts)) = state.tick() {
                        let ts = Utc.timestamp_millis(ts.into());
                        ui.small(format!("{}\n{}\n{}", id, ts.date(), ts.time()));
                    }
                });
        });

        draw(&view, &state, tick_lerp);
        next_frame().await
    }
}
fn window() -> Conf {
    logger::init();
    macro_rules! title {
        () => {
            concat!("Scalliony ", env!("CARGO_PKG_VERSION"))
        };
    }
    info!(title!());
    Conf {
        window_title: title!().to_string(),
        sample_count: 4,
        icon: None,
        ..Conf::default()
    }
}

#[inline]
fn draw(view: &view::View, state: &game::AnimatedState, lerp: F) {
    clear_background(BLACK);
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
        draw_border(pos, rad, Color::new(0.1, 0.1, 0.1, 1.0));
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

    egui_macroquad::draw();
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
    fn as_str(&self) -> &str {
        self.as_ref()
    }
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
