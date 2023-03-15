use std::collections::BTreeMap;
use super::noise;
use bulb::{hex::Hex, dto::Cell};

pub struct Grid(BTreeMap<Hex, Cell>, Generator);
impl Grid {
    pub fn new(seed: u32) -> Self {
        Self(BTreeMap::new(), Generator::new(seed))
    }
    fn set(&mut self, h: Hex, v: Cell) {
        self.grid.insert(h, v);
    }
    fn drain_unchanged(&mut self) {
        self.grid.retain(|h, v| *v != self.gen.get(*h));
    }
}
impl CellGrid for Grid {
    fn get(&self, h: Hex) -> Cell {
        if let Some(v) = self.grid.get(&h) {
            *v
        } else {
            self.gen.get(h)
        }
    }
}

struct Generator(noise::Fbm<noise::OpenSimplex>);
impl Generator {
    fn new(seed: u32) -> Self {
        use noise::Seedable;
        let mut noise = noise::Fbm::new_seed(seed);
        noise.frequency = 1. / 256.;
        Self(noise)
    }
    fn get(&self, h: Hex) -> Cell {
        use noise::NoiseFn;
        let p = bulb::hex::Point::from(h);
        let height = self.0.get([p.x, p.y]);
        if height < 0. {
            Cell::Ground
        } else {
            Cell::Wall
        }
    }
}
