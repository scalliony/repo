use super::*;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

type F = f64;

/// sqrt(3)
pub const SQRT3: F = 1.73205077648162841796875;

/// Point in screen space
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Point {
    pub x: F,
    pub y: F,
}
/// Area in screen space
pub type Size = Point;
impl Add for Point {
    type Output = Self;
    #[inline]
    fn add(self, v: Self) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }
}
impl Sub for Point {
    type Output = Self;
    #[inline]
    fn sub(self, v: Self) -> Self {
        Self {
            x: self.x - v.x,
            y: self.y - v.y,
        }
    }
}
impl Mul for Point {
    type Output = Self;
    #[inline]
    fn mul(self, v: Self) -> Self {
        Self {
            x: self.x * v.x,
            y: self.y * v.y,
        }
    }
}
impl Div for Point {
    type Output = Self;
    #[inline]
    fn div(self, v: Self) -> Self {
        Self {
            x: self.x / v.x,
            y: self.y / v.y,
        }
    }
}
impl Mul<F> for Point {
    type Output = Self;
    #[inline]
    fn mul(self, k: F) -> Self {
        Self {
            x: self.x * k,
            y: self.y * k,
        }
    }
}
impl Div<F> for Point {
    type Output = Self;
    #[inline]
    fn div(self, k: F) -> Self {
        Self {
            x: self.x / k,
            y: self.y / k,
        }
    }
}

impl From<(F, F)> for Point {
    #[inline]
    fn from((x, y): (F, F)) -> Self {
        Self { x, y }
    }
}
impl From<FracHex> for Point {
    fn from(h: FracHex) -> Self {
        //NOTE: y sign flipped to put +y up
        Self {
            x: 3. / 2. * h.q(),
            y: -SQRT3 / 2. * h.q() + -SQRT3 * h.r(),
        }
    }
}
impl From<Point> for FracHex {
    fn from(p: Point) -> Self {
        //NOTE: y sign flipped to put +y up
        Self(2. / 3. * p.x, 1. / 3. * p.x + -SQRT3 / 3. * p.y)
    }
}
impl From<Hex> for Point {
    #[inline]
    fn from(h: Hex) -> Self {
        FracHex::from(h).into()
    }
}
impl From<Point> for Hex {
    #[inline]
    fn from(p: Point) -> Self {
        FracHex::from(p).into()
    }
}

/// Hexagon in qrs format with floating point
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct FracHex(F, F);
impl FracHex {
    #[inline]
    pub fn new(q: F, r: F) -> Self {
        Self(q, r)
    }

    #[inline]
    pub fn q(self) -> F {
        self.0
    }
    #[inline]
    pub fn r(self) -> F {
        self.1
    }
    #[inline]
    pub fn s(self) -> F {
        -self.q() - self.r()
    }
}
impl Add for FracHex {
    type Output = Self;
    #[inline]
    fn add(self, v: Self) -> Self {
        Self(self.0 + v.0, self.1 + v.1)
    }
}
impl AddAssign for FracHex {
    #[inline]
    fn add_assign(&mut self, v: Self) {
        *self = *self + v
    }
}
impl Sub for FracHex {
    type Output = Self;
    #[inline]
    fn sub(self, v: Self) -> Self {
        Self(self.0 - v.0, self.1 - v.1)
    }
}
impl SubAssign for FracHex {
    #[inline]
    fn sub_assign(&mut self, v: Self) {
        *self = *self - v
    }
}
impl Mul<F> for FracHex {
    type Output = Self;
    #[inline]
    fn mul(self, k: F) -> Self {
        Self(self.0 * k, self.1 * k)
    }
}
impl Div<F> for FracHex {
    type Output = Self;
    #[inline]
    fn div(self, k: F) -> Self {
        Self(self.0 / k, self.1 / k)
    }
}
impl From<Hex> for FracHex {
    #[inline]
    fn from(h: Hex) -> Self {
        Self(h.q() as F, h.r() as F)
    }
}
impl From<FracHex> for Hex {
    fn from(f: FracHex) -> Hex {
        let _s = f.s();
        let mut q = f.q().round();
        let mut r = f.r().round();
        let s = _s.round();
        let q_diff = (q - f.q()).abs();
        let r_diff = (r - f.r()).abs();
        let s_diff = (s - _s).abs();
        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s;
        } else if r_diff > s_diff {
            r = -q - s;
        }
        Hex::new(q as I, r as I)
    }
}
