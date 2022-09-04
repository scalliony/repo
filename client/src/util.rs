pub use bulb::hex::*;
use macroquad::models::Vertex;
use macroquad::prelude::*;
pub use macroquad::prelude::{debug, error, info, trace, warn};

pub type F = f32;
/// 2*pi
pub const TAU: F = std::f32::consts::PI * 2.;

pub fn draw_cell(center: Pos, rad: Size, color: Color) {
    let mut vertices = Vec::<Vertex>::with_capacity(8);
    vertices.push(Vertex { position: (center, 0.).into(), uv: (0., 0.).into(), color });
    for i in 0..=6 {
        let r = i as F * TAU / 6.;
        let s = Pos::from((r.cos(), r.sin()));
        let p = center + rad * s;
        vertices.push(Vertex { position: (p, 0.).into(), uv: s, color });
    }

    let mut indices = Vec::<u16>::with_capacity(18);
    for i in 0..6 {
        indices.extend_from_slice(&[0, i as u16 + 1, i as u16 + 2]);
    }

    draw_mesh(&Mesh { vertices, indices, texture: None });
}
pub fn draw_border(center: Pos, rad: Size, color: Color) {
    if super::opts::GRID {
        for i in 0..6 {
            let r = i as F * TAU / 6.;
            let a = center + rad * Pos::from((r.cos(), r.sin()));
            let r = (i + 1) as F * TAU / 6.;
            let b = center + rad * Pos::from((r.cos(), r.sin()));
            draw_line(a.x, a.y, b.x, b.y, 1., color);
        }
    }
}

/// Pos in screen space
pub type Pos = glam::Vec2;
/// Area in screen space
pub type Size = Pos;
#[inline]
pub fn to_point(pos: Pos) -> Point {
    Point { x: pos.x as f64, y: pos.y as f64 }
}
#[inline]
pub fn to_pos(point: Point) -> Pos {
    Pos::new(point.x as F, point.y as F)
}

#[inline]
pub fn lerp_f(a: F, b: F, t: F) -> F {
    a * (1. - t) + b * t
}
#[inline]
pub fn lerp_hex(a: Hex, b: Hex, t: F) -> FracHex {
    FracHex::new(lerp_f(a.q() as F, b.q() as F, t) as f64, lerp_f(a.r() as F, b.r() as F, t) as f64)
}
