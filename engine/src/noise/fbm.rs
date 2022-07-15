use super::{
    math::{from_fn, mul, F, P},
    NoiseFn, OpenSimplex, Seedable,
};

/// Fractal noise
#[derive(Clone, Debug)]
pub struct Fbm<Noise: Seedable = OpenSimplex, const OCTAVES: usize = 6> {
    /// The number of cycles per unit length that the noise function outputs.
    pub frequency: F,

    /// A multiplier that determines how quickly the frequency increases for
    /// each successive octave in the noise function.
    ///
    /// The frequency of each successive octave is equal to the product of the
    /// previous octave's frequency and the lacunarity value.
    pub lacunarity: F,

    /// A multiplier that determines how quickly the amplitudes diminish for
    /// each successive octave in the noise function.
    ///
    /// The amplitude of each successive octave is equal to the product of the
    /// previous octave's amplitude and the persistence value. Increasing the
    /// persistence produces "rougher" noise.
    pub persistence: F,

    seed: u32,
    sources: [Noise; OCTAVES],
}
impl<Noise: Seedable, const OCTAVES: usize> Fbm<Noise, OCTAVES> {
    fn build_sources(seed: u32) -> [Noise; OCTAVES] {
        from_fn(|i| Noise::new_seed(seed + i as u32))
    }
}
impl Seedable for Fbm {
    fn new_seed(seed: u32) -> Self {
        Self {
            frequency: 1.0,
            lacunarity: std::f64::consts::PI * 2.0 / 3.0,
            persistence: 0.5,
            seed,
            sources: Self::build_sources(seed),
        }
    }
    fn seed(&self) -> u32 {
        self.seed
    }
}
impl<const D: usize, Noise: NoiseFn<D> + Seedable, const OCTAVES: usize> NoiseFn<D>
    for Fbm<Noise, OCTAVES>
{
    fn get(&self, mut point: P<D>) -> F {
        let mut result = 0.0;
        point = mul(point, self.frequency);

        for x in 0..OCTAVES {
            let mut signal = self.sources[x].get(point);
            signal *= self.persistence.powi(x as i32);
            result += signal;
            point = mul(point, self.lacunarity);
        }

        let scale = 2.0 - self.persistence.powi(OCTAVES as i32 - 1);
        result / scale
    }
}
