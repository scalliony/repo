use super::opts;
use super::util::*;
use bulb::dto::HexRange;
use macroquad::prelude::*;

pub const RATIO: F = 1. / 2.;

pub struct View {
    rad: F,
    center: FracHex,
}
impl Default for View {
    fn default() -> Self {
        Self { rad: 5., center: FracHex::default() }
    }
}
impl View {
    pub fn update(&mut self) -> HexRange {
        let dt = get_frame_time();
        //MAYBE: input config
        let spd = self.zoom(dt) * dt;
        self.move_if(KeyCode::Left, Direction::DownLeft, spd);
        self.move_if(KeyCode::Right, Direction::DownRight, spd);
        self.move_if(KeyCode::Up, Direction::Up, spd);
        self.move_if(KeyCode::Down, Direction::Down, spd);
        self.round()
    }
    #[inline]
    fn zoom(&mut self, dt: F) -> F {
        const MIN: F = 0.1;
        const MAX: F = 100.0;

        if is_key_down(KeyCode::PageUp) {
            self.rad *= 1.0 + dt;
        }
        if is_key_down(KeyCode::PageDown) {
            self.rad *= 1.0 - dt;
        }
        self.rad = self.rad.clamp(MIN, MAX);
        self.rad
    }
    #[inline]
    fn move_if(&mut self, k: KeyCode, d: Direction, spd: F) {
        if is_key_down(k) {
            self.center += FracHex::from(Hex::from(d)) * spd as f64;
        }
    }

    pub fn round(&self) -> HexRange {
        HexRange { center: self.center.into(), rad: self.rad as u8 }
    }
    pub fn iter(&self, center: Pos, range: Size) -> (Size, HexRangeIter, Mapper) {
        let rad = if opts::SLIDE { self.rad } else { self.rad.round() };
        let size = Self::unit_rad(rad, range);
        let center_h: Hex = self.center.into();

        let mapper = Mapper {
            center_p: if opts::SLIDE {
                Point::from(self.center - center_h.into())
            } else {
                Point::default()
            },
            center_h,
            center,
            size,
        };
        (size, center_h.range(self.rad as I), mapper)
    }
    #[inline]
    fn unit_rad(v: F, view: Size) -> Size {
        const SQRT3_F32: F = SQRT3 as F;
        let x = (view.y / (v + 0.5) / SQRT3_F32 / RATIO)
            .min(view.x / (v + (SQRT3_F32 / 2.0)) / (3.0 / 2.0));
        Size::new(x, x * RATIO) / 2.
    }
}
#[derive(Clone)]
pub struct Mapper {
    center_h: Hex,
    center_p: Point,
    size: Size,
    center: Pos,
}
impl Mapper {
    pub fn map(&self, h: Hex) -> Pos {
        to_pos(Point::from(h - self.center_h) - self.center_p) * self.size + self.center
    }
    pub fn map_f(&self, h: FracHex) -> Pos {
        to_pos(Point::from(h - self.center_h.into()) - self.center_p) * self.size + self.center
    }
}

pub fn hash_color<T: std::hash::Hash>(t: &T) -> Color {
    use std::hash::Hasher;
    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    let h = s.finish();
    color_u8!((h & 0xFF0000) >> 16, (h & 0x00FF00) >> 8, h & 0x0000FF, 255)
}
