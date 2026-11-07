//! .NET's `System.Random` given a seed, its subtractive generator (Knuth's, as .NET Framework and .NET's seeded
//! compatibility path implement it), so a program generated from a seed is the bytes the C# fixtures generated.

const MBIG: i32 = i32::MAX;
const MSEED: i32 = 161_803_398;

/// The generator for one seed.
pub struct DotNetRandom {
    seeds: [i32; 56],
    next: usize,
    nextp: usize,
}

impl DotNetRandom {
    pub fn new(seed: i32) -> DotNetRandom {
        let mut seeds = [0i32; 56];
        let subtraction = if seed == i32::MIN { i32::MAX } else { seed.abs() };
        let mut mj = MSEED - subtraction;
        seeds[55] = mj;
        let mut mk = 1i32;
        let mut ii = 0usize;
        for _ in 1..55 {
            ii += 21;
            if ii >= 55 {
                ii -= 55;
            }
            seeds[ii] = mk;
            mk = mj.wrapping_sub(mk);
            if mk < 0 {
                mk = mk.wrapping_add(MBIG);
            }
            mj = seeds[ii];
        }
        for _ in 1..5 {
            for i in 1..56 {
                let mut n = i + 30;
                if n >= 55 {
                    n -= 55;
                }
                seeds[i] = seeds[i].wrapping_sub(seeds[1 + n]);
                if seeds[i] < 0 {
                    seeds[i] = seeds[i].wrapping_add(MBIG);
                }
            }
        }
        DotNetRandom { seeds, next: 0, nextp: 21 }
    }

    fn sample(&mut self) -> i32 {
        let mut a = self.next + 1;
        if a >= 56 {
            a = 1;
        }
        let mut b = self.nextp + 1;
        if b >= 56 {
            b = 1;
        }
        let mut r = self.seeds[a].wrapping_sub(self.seeds[b]);
        if r == MBIG {
            r -= 1;
        }
        if r < 0 {
            r = r.wrapping_add(MBIG);
        }
        self.seeds[a] = r;
        self.next = a;
        self.nextp = b;
        r
    }

    /// `NextBytes`: each byte the low eight bits of one sample.
    pub fn next_bytes(&mut self, into: &mut [u8]) {
        for b in into {
            *b = self.sample() as u8;
        }
    }
}
