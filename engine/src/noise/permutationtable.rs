use core::num::Wrapping as w;

struct XorShiftRng {
    x: w<u32>,
    y: w<u32>,
    z: w<u32>,
    w: w<u32>,
}
impl XorShiftRng {
    fn from_seed(mut seed: [u32; 4]) -> Self {
        // Xorshift cannot be seeded with 0 and we cannot return an Error, but
        // also do not wish to panic (because a random seed can legitimately be
        // 0); our only option is therefore to use a preset value.
        if seed.iter().all(|&x| x == 0) {
            seed = [0xBAD_5EED, 0xBAD_5EED, 0xBAD_5EED, 0xBAD_5EED];
        }
        XorShiftRng { x: w(seed[0]), y: w(seed[1]), z: w(seed[2]), w: w(seed[3]) }
    }

    #[inline]
    fn rand(&mut self) -> u32 {
        let x = self.x;
        let t = x ^ (x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        let w_ = self.w;
        self.w = w_ ^ (w_ >> 19) ^ (t ^ (t >> 8));
        self.w.0
    }
    /// _ >= low && _ < high
    #[inline]
    fn gen_range(&mut self, low: u32, high: u32) -> u32 {
        let r = self.rand() as f32 / std::u32::MAX as f32;
        let r = low as f32 + (high as f32 - low as f32) * r;
        r as u32
    }
}

use std::fmt;

const TABLE_SIZE: usize = 256;

/// A seed table, required by all noise functions.
///
/// Table creation is expensive, so in most circumstances you'll only want to
/// create one of these per generator.
#[derive(Copy, Clone)]
pub(crate) struct PermutationTable {
    values: [u8; TABLE_SIZE],
}

impl PermutationTable {
    /// Deterministically generates a new permutation table based on a `u32` seed value.
    ///
    /// Using `XorShiftRng` and Fisher-Yates shuffle
    pub fn new(seed: u32) -> Self {
        let mut rng = XorShiftRng::from_seed([seed, seed, seed, seed]);
        let mut values = super::math::from_fn(|i| i as u8);
        for i in (1..TABLE_SIZE).rev() {
            values.swap(i, rng.gen_range(0, (i + 1) as u32) as usize);
        }
        Self { values }
    }

    #[inline]
    pub fn get1(&self, x: isize) -> usize {
        let x = (x & 0xff) as usize;
        self.values[x] as usize
    }

    #[inline]
    pub fn get2(&self, pos: [isize; 2]) -> usize {
        let y = (pos[1] & 0xff) as usize;
        self.values[self.get1(pos[0]) ^ y] as usize
    }

    #[inline]
    pub fn get3(&self, pos: [isize; 3]) -> usize {
        let z = (pos[2] & 0xff) as usize;
        self.values[self.get2([pos[0], pos[1]]) ^ z] as usize
    }

    #[inline]
    pub fn get4(&self, pos: [isize; 4]) -> usize {
        let _w = (pos[3] & 0xff) as usize;
        self.values[self.get3([pos[0], pos[1], pos[2]]) ^ _w] as usize
    }
}

impl fmt::Debug for PermutationTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PermutationTable {{ .. }}")
    }
}
