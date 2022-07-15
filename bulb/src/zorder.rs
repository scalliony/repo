use std::ops::Deref;

/// Excess-2^32 signed integer
/// dist(i) = dist(Excess::from(i))
///
/// * 000..00 -> i32::MIN
/// * 011..11 -> -1
/// * 100..00 -> 0
/// * 100..01 -> 1
/// * 111..11 -> i32::MAX
#[repr(transparent)]
pub struct Excess(u32);
impl Excess {
    const HIGH_BIT: u32 = 1 << (u32::BITS - 1);
    #[inline]
    pub fn to(v: i32) -> u32 {
        *Self::from(v)
    }
    #[inline]
    pub fn of(x: u32) -> i32 {
        Self(x).into()
    }
}
impl From<i32> for Excess {
    #[inline]
    fn from(v: i32) -> Excess {
        Self(v as u32 ^ Excess::HIGH_BIT)
    }
}
impl From<Excess> for i32 {
    #[inline]
    fn from(v: Excess) -> i32 {
        (*v ^ Excess::HIGH_BIT) as i32
    }
}
impl Deref for Excess {
    type Target = u32;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 2D Z-order or Morton order
#[repr(transparent)]
pub struct Z2(u64);
impl Z2 {
    #[inline]
    pub fn to(x: u32, y: u32) -> u64 {
        *Self::from((x, y))
    }
    #[inline]
    pub fn of(z: u64) -> (u32, u32) {
        Self(z).into()
    }
}
impl From<(u32, u32)> for Z2 {
    fn from(v: (u32, u32)) -> Z2 {
        let x = v.0 as u64;
        let y = v.1 as u64;
        let mut z = 0u64;
        for i in 0..u32::BITS {
            z |= (x & 1u64 << i) << i | (y & 1u64 << i) << (i + 1);
        }
        Self(z)
    }
}
impl From<Z2> for (u32, u32) {
    fn from(z: Z2) -> (u32, u32) {
        let mut x = 0u32;
        let mut y = 0u32;
        for i in 0..u32::BITS {
            x |= ((*z & 1u64 << i) >> i) as u32;
            y |= ((*z & 1u64 << (i + 1)) >> (i + 1)) as u32;
        }
        (x, y)
    }
}
impl Deref for Z2 {
    type Target = u64;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
