mod fbm;
mod gradient;
mod math;
mod opensimplex;
mod permutationtable;

pub trait NoiseFn<const D: usize> {
    fn get(&self, point: P<D>) -> f64;
}
pub trait Seedable {
    fn new_seed(seed: u32) -> Self;
    fn seed(&self) -> u32;
}

pub use fbm::Fbm;
pub use math::*;
pub use opensimplex::OpenSimplex;
use permutationtable::PermutationTable;
