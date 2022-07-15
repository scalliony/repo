use std::ops::{Add, Mul, Sub};
#[inline(always)]
pub(crate) fn from_fn<F, T, const N: usize>(mut cb: F) -> [T; N]
where
    F: FnMut(usize) -> T,
{
    let mut idx = 0;
    [(); N].map(|_| {
        let res = cb(idx);
        idx += 1;
        res
    })
}

pub type F = f64;
pub type P<const D: usize> = [F; D];

#[inline(always)]
pub(crate) fn zip<const D: usize, C: Fn(F, F) -> F>(a: P<D>, b: P<D>, f: C) -> P<D> {
    from_fn(|i| f(a[i], b[i]))
}
#[inline(always)]
pub(crate) fn fold<const D: usize, C: Fn(F, F) -> F>(p: P<D>, f: C) -> F {
    p.into_iter().reduce(f).unwrap_or_default()
}
#[inline(always)]
pub(crate) fn map<const D: usize, C: Fn(F) -> F>(p: P<D>, f: C) -> P<D> {
    from_fn(|i| f(p[i]))
}

#[inline(always)]
pub(crate) fn to_isize<const D: usize>(point: P<D>) -> [isize; D] {
    from_fn(|i| point[i] as isize)
}

#[inline(always)]
pub(crate) fn add<const D: usize>(a: P<D>, b: P<D>) -> P<D> {
    zip(a, b, Add::add)
}
#[inline(always)]
pub(crate) fn sub<const D: usize>(a: P<D>, b: P<D>) -> P<D> {
    zip(a, b, Sub::sub)
}
#[inline(always)]
pub(crate) fn mul<const D: usize>(point: P<D>, k: F) -> P<D> {
    from_fn(|i| point[i] * k)
}
#[inline(always)]
pub(crate) fn dot<const D: usize>(a: P<D>, b: P<D>) -> F {
    fold(zip(a, b, Mul::mul), Add::add)
}
