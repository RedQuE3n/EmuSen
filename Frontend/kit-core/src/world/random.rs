/// Mulberry32, Tommy Ettinger's 32-bit generator: a splash's stars and orbits and a bell's detune are the same on every showing (LunaP §219.3).
#[derive(Clone, Debug)]
pub struct SeededRandom {
    state: u32,
}

impl SeededRandom {
    pub const fn new(seed: u32) -> SeededRandom {
        SeededRandom { state: seed }
    }

    /// The next number of the sequence, in [0, 1).
    pub fn draw(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x6D2B_79F5);
        let mut t = self.state;
        t = (t ^ (t >> 15)).wrapping_mul(1 | t);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        (t ^ (t >> 14)) as f64 / 4_294_967_296.0
    }

    /// The first numbers of a seed's sequence.
    pub fn first(seed: u32, count: usize) -> Vec<f64> {
        let mut random = SeededRandom::new(seed);
        (0..count).map(|_| random.draw()).collect()
    }
}
