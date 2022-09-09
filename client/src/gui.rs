use egui_miniquad::EguiMq;
use macroquad::input::utils::*;
use macroquad::prelude::*;
use miniquad as mq;

pub use egui;

struct State {
    mq: EguiMq,
    input: usize,
}
impl State {
    fn new() -> Self {
        let gl = unsafe { get_internal_gl() };
        Self { mq: EguiMq::new(gl.quad_context), input: register_input_subscriber() }
    }

    fn ui<F: FnOnce(&egui::Context)>(&mut self, f: F) {
        let gl = unsafe { get_internal_gl() };
        repeat_all_miniquad_input(self, self.input);
        self.mq.run(gl.quad_context, |_, egui| f(egui));
    }
    fn draw(&mut self) {
        let mut gl = unsafe { get_internal_gl() };
        // Ensure that macroquad's shapes are not goint to be lost, and draw them now
        gl.flush();
        self.mq.draw(&mut gl.quad_context);
    }
}

// Global variable and global functions because it's more like macroquad way
static mut STATE: Option<State> = None;
fn get_state() -> &'static mut State {
    unsafe {
        if let Some(state) = &mut STATE {
            state
        } else {
            STATE = Some(State::new());
            STATE.as_mut().unwrap()
        }
    }
}

/// Calculates egui ui. Must be called once per frame.
pub fn ui<F: FnOnce(&egui::Context)>(f: F) {
    get_state().ui(f)
}
/// Configure egui without beginning or ending a frame.
pub fn cfg<F: FnOnce(&egui::Context)>(f: F) {
    f(get_state().mq.egui_ctx());
}
/// Draw egui ui. Must be called after `ui` and once per frame.
pub fn draw() {
    get_state().draw()
}

impl mq::EventHandler for State {
    fn update(&mut self, _ctx: &mut mq::Context) {}

    fn draw(&mut self, _ctx: &mut mq::Context) {}

    fn mouse_motion_event(&mut self, _: &mut mq::Context, x: f32, y: f32) {
        self.mq.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, _: &mut mq::Context, dx: f32, dy: f32) {
        self.mq.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.mq.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.mq.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(
        &mut self,
        _ctx: &mut mq::Context,
        character: char,
        _keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.mq.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.mq.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(&mut self, _ctx: &mut mq::Context, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.mq.key_up_event(keycode, keymods);
    }
}
