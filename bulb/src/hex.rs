mod frac;
use super::zorder::{Excess, Z2};
pub use frac::*;
use std::cmp::{max, min, Ordering};
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

pub type I = i32;

/// Hexagon in qrs format
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Hex(I, I);
impl Hex {
    #[inline]
    pub fn new(q: I, r: I) -> Self {
        Self(q, r)
    }

    #[inline]
    pub fn q(self) -> I {
        self.0
    }
    #[inline]
    pub fn r(self) -> I {
        self.1
    }
    #[inline]
    pub fn s(self) -> I {
        -self.q() - self.r()
    }
}
impl Add for Hex {
    type Output = Self;
    #[inline]
    fn add(self, v: Self) -> Self {
        Self(self.0 + v.0, self.1 + v.1)
    }
}
impl AddAssign for Hex {
    #[inline]
    fn add_assign(&mut self, v: Self) {
        *self = *self + v
    }
}
impl Sub for Hex {
    type Output = Self;
    #[inline]
    fn sub(self, v: Self) -> Self {
        Self(self.0 - v.0, self.1 - v.1)
    }
}
impl SubAssign for Hex {
    #[inline]
    fn sub_assign(&mut self, v: Self) {
        *self = *self - v
    }
}
impl Mul<I> for Hex {
    type Output = Self;
    #[inline]
    fn mul(self, k: I) -> Self {
        Self(self.0 * k, self.1 * k)
    }
}
impl Hex {
    pub fn length(self) -> I {
        (self.q().abs() + self.r().abs() + self.s().abs()) / 2
    }
    pub fn dist(self, other: Self) -> I {
        (self - other).length()
    }
}
impl Hex {
    fn to_z(self) -> u64 {
        Z2::to(Excess::to(self.q()), Excess::to(self.r()))
    }
}
impl Ord for Hex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_z().cmp(&other.to_z())
    }
}
impl PartialOrd for Hex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hex {
    #[inline]
    pub fn range(self, rad: I) -> HexRangeIter {
        HexRangeIter::new(self, rad)
    }
}
pub struct HexRangeIter {
    center: Hex,
    rad: I,
    cur: Hex,
    len: usize,
}
impl HexRangeIter {
    pub fn new(center: Hex, rad: I) -> Self {
        assert!(rad >= 0);
        Self { center, cur: Hex(-rad, 0), len: 3 * rad as usize * (rad as usize + 1) + 1, rad }
    }
}
impl Iterator for HexRangeIter {
    type Item = Hex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur.q() > self.rad {
            debug_assert!(self.len == 0);
            return None;
        }

        let v = self.center + self.cur;
        self.len -= 1;
        if self.cur.r() < min(self.rad, -self.cur.q() + self.rad) {
            self.cur.1 += 1;
        } else {
            self.cur.0 += 1;
            self.cur.1 = max(-self.rad, -self.cur.q() - self.rad);
        }
        Some(v)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
    fn count(self) -> usize {
        self.len
    }
    fn last(self) -> Option<Self::Item> {
        if self.len > 0 {
            Some(self.center + Hex(self.rad, self.rad))
        } else {
            None
        }
    }
}
impl ExactSizeIterator for HexRangeIter {
    fn len(&self) -> usize {
        self.len
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Direction {
    Up = 0,
    UpRight,
    DownRight,
    Down,
    DownLeft,
    UpLeft,
}
impl Direction {
    const ALL: [Self; 6] =
        [Self::Up, Self::UpRight, Self::DownRight, Self::Down, Self::DownLeft, Self::UpLeft];
    pub fn all() -> &'static [Self; 6] {
        &Self::ALL
    }
    fn new(x: u8) -> Self {
        unsafe { std::mem::transmute(x % 6) }
    }
}
impl Hex {
    const DIRECTIONS: [Self; 6] =
        [Self(0, 1), Self(1, 0), Self(1, -1), Self(0, -1), Self(-1, 0), Self(-1, 1)];
    /// Unit vector for each direction
    /// * `r` in Up *(like y)*
    /// * `q` in UpRight *(like x + y/2)*
    pub fn directions() -> &'static [Self; 6] {
        &Self::DIRECTIONS
    }
}
impl From<Direction> for Hex {
    #[inline]
    fn from(d: Direction) -> Self {
        Self::DIRECTIONS[d as usize]
    }
}
impl Hex {
    #[inline]
    pub fn neighbor(self, d: Direction) -> Self {
        self + d.into()
    }
}
impl std::ops::Neg for Direction {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self + Angle::Back
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Angle {
    Front = 0,
    Right,
    RightBack,
    Back,
    LeftBack,
    Left,
}
impl Angle {
    fn new(x: u8) -> Self {
        unsafe { std::mem::transmute(x % 6) }
    }
    #[inline]
    pub fn degre(&self) -> u16 {
        *self as u16 * 60
    }
}
impl Add for Angle {
    type Output = Self;
    #[inline]
    fn add(self, v: Self) -> Self {
        Self::new(self as u8 + v as u8)
    }
}
impl Add<Angle> for Direction {
    type Output = Self;
    #[inline]
    fn add(self, v: Angle) -> Self {
        Self::new(self as u8 + v as u8)
    }
}
impl AddAssign<Angle> for Direction {
    #[inline]
    fn add_assign(&mut self, rhs: Angle) {
        *self = *self + rhs;
    }
}
impl Sub for Angle {
    type Output = Self;
    #[inline]
    fn sub(self, v: Self) -> Self {
        Self::new(self as u8 + 6 - v as u8)
    }
}
impl Sub<Angle> for Direction {
    type Output = Self;
    #[inline]
    fn sub(self, v: Angle) -> Self {
        Self::new(self as u8 + 6 - v as u8)
    }
}
impl Sub for Direction {
    type Output = Angle;
    #[inline]
    fn sub(self, v: Direction) -> Angle {
        Angle::new((self - Angle::new(v as u8)) as u8)
    }
}
impl Mul<I> for Angle {
    type Output = Self;
    #[inline]
    fn mul(self, k: I) -> Self {
        Self::new(((self as I * k) % 6 + 6) as u8)
    }
}
