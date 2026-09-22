//! The passes over a pixel: the anti-aliasing pull, undoing dither, divot, the mix and gamma. See Mars_VideoFilter.md and Mars_VideoPasses.md.

/// C#'s `Vi.Pixel`: eight bits a channel as `int`, and the coverage read beside them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pixel {
    pub red: i32,
    pub green: i32,
    pub blue: i32,
    pub coverage: i32,
}

/// The two runners-up bracket the pixel, which moves towards them by the coverage it lacks (Mars_VideoFilter.md §2.3).
#[inline]
pub fn pull(values: &[i32], centre: i32, missing: i32) -> i32 {
    let (low, high) = runners(values);
    (((low.wrapping_add(high).wrapping_sub(centre << 1)).wrapping_mul(missing).wrapping_add(4)) >> 3).wrapping_add(centre)
}

/// The runner-up at each end; a leader never displaced stands as its own (Mars_VideoFilter.md §2.2).
#[inline]
pub fn runners(values: &[i32]) -> (i32, i32) {
    let (mut lowest, mut highest) = (0, 0);
    let (mut low, mut high) = (values[0], values[0]);
    for i in 1..values.len() {
        if values[i] > values[highest] {
            high = values[highest];
            highest = i;
        }
        if values[i] < values[lowest] {
            low = values[lowest];
            lowest = i;
        }
    }
    for &v in &values[highest + 1..] {
        if v > high {
            high = v;
        }
    }
    for &v in &values[lowest + 1..] {
        if v < low {
            low = v;
        }
    }
    (low, high)
}

/// The dither filter's step: the top five bits compared (Mars_VideoPasses.md §1).
#[inline]
pub fn step(centre: i32, neighbour: i32) -> i32 {
    ((neighbour & 0xF8) - (centre & 0xF8)).signum()
}

/// Divot: the median of three across the row, unless all three are whole (Mars_VideoPasses.md §2).
#[inline]
pub fn divot(centre: Pixel, left: Pixel, right: Pixel) -> Pixel {
    if centre.coverage & left.coverage & right.coverage == 7 {
        return centre;
    }
    Pixel {
        red: median(centre.red, left.red, right.red),
        green: median(centre.green, left.green, right.green),
        blue: median(centre.blue, left.blue, right.blue),
        coverage: centre.coverage,
    }
}

#[inline]
pub fn median(centre: i32, left: i32, right: i32) -> i32 {
    if (left >= centre && right >= left) || (left >= right && centre >= left) {
        return left;
    }
    if (right >= centre && left >= right) || (right >= left && centre >= right) {
        return right;
    }
    centre
}

/// Mixing moves the colour and keeps the near pixel's coverage (Mars_Video.md §2.6).
#[inline]
pub fn mix(near: Pixel, far: Pixel, fraction: i32) -> Pixel {
    if fraction == 0 {
        return near;
    }
    Pixel {
        red: between(near.red, far.red, fraction),
        green: between(near.green, far.green, fraction),
        blue: between(near.blue, far.blue, fraction),
        coverage: near.coverage,
    }
}

#[inline]
fn between(near: i32, far: i32, fraction: i32) -> i32 {
    ((((far.wrapping_sub(near)).wrapping_mul(fraction).wrapping_add(16)) >> 5).wrapping_add(near)) & 0xFF
}

const GAMMA_ENTRIES: usize = 0x4000;

/// `Vi.GammaTable`: twice the integer square root, indexed by the channel with six dither bits below it (Mars_VideoPasses.md §3).
static GAMMA: [u8; GAMMA_ENTRIES] = build_gamma();

const fn build_gamma() -> [u8; GAMMA_ENTRIES] {
    let mut table = [0u8; GAMMA_ENTRIES];
    let mut i = 0;
    while i < GAMMA_ENTRIES {
        table[i] = (root(i as i32) << 1) as u8;
        i += 1;
    }
    table
}

/// The reference's square root, two bits of the argument a step.
pub const fn root(value: i32) -> i32 {
    let (mut rest, mut result, mut one) = (value, 0, 1 << 30);
    while one > rest {
        one >>= 2;
    }
    while one != 0 {
        if rest >= result + one {
            rest -= result + one;
            result += one << 1;
        }
        result >>= 1;
        one >>= 2;
    }
    result
}

/// A channel through gamma; C# throws outside the table, which only a hidden byte above 3 can reach, and this reads zero there.
#[inline]
fn gamma_of(channel: i32) -> i32 {
    GAMMA.get(channel.wrapping_shl(6) as u32 as usize).map_or(0, |&v| v as i32)
}

#[inline]
pub fn gamma(pixel: Pixel) -> Pixel {
    Pixel { red: gamma_of(pixel.red), green: gamma_of(pixel.green), blue: gamma_of(pixel.blue), coverage: pixel.coverage }
}
