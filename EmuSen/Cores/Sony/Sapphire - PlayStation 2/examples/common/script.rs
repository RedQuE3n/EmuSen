//! The pad's input as a script of lines, shared by the game runner (`s28.rs`) and the bench (`bench.rs`), so that a
//! run reaching a game's play and the bench's windows inside it hold the same buttons and sticks at the same frames
//! (Sapphire_Speed.md §51). Frames are the machine's own count, the frame being advanced to.
//!
//! - `press <frame> <bits> [frames]`: the core's button bits (`sio2::pad::bit`, `+`-joined) held from that frame for six
//!   frames or the count given; presses that overlap are held together.
//! - `stick <frame> <axis> <value> [frames]`: an axis of port 1 (0 left X, 1 left Y, 2 right X, 3 right Y; -1.0 left or
//!   up to 1.0 right or down) held from that frame for six frames or the count given; where two overlap on one axis the
//!   later begun holds, and an axis no line holds is centred.
//!
//! Every other line is left to the caller.

#[derive(Clone, Debug, Default)]
pub struct Script {
    presses: Vec<(i64, u32, i64)>,
    sticks: Vec<(i64, u32, f64, i64)>,
}

impl Script {
    pub fn parse(text: &str) -> Script {
        let mut s = Script::default();
        for line in text.lines() {
            let w: Vec<&str> = line.split_whitespace().collect();
            match w.as_slice() {
                ["press", f, bits, rest @ ..] => {
                    let mask = bits.split('+').filter_map(|b| b.parse::<u32>().ok()).fold(0u32, |m, b| m | 1 << b);
                    let held = rest.first().and_then(|d| d.parse().ok()).unwrap_or(6);
                    s.presses.extend(f.parse().ok().map(|f| (f, mask, held)));
                }
                ["stick", f, axis, value, rest @ ..] => {
                    let held = rest.first().and_then(|d| d.parse().ok()).unwrap_or(6);
                    if let (Ok(f), Ok(a), Ok(v)) = (f.parse(), axis.parse::<u32>(), value.parse()) {
                        s.sticks.extend((a < 4).then_some((f, a, v, held)));
                    }
                }
                _ => {}
            }
        }
        s.sticks.sort_by_key(|k| k.0);
        s
    }

    /// The buttons held while the machine advances to `frame`.
    pub fn buttons(&self, frame: i64) -> u32 {
        self.presses.iter().filter(|p| (p.0..p.0 + p.2).contains(&frame)).fold(0, |m, p| m | p.1)
    }

    /// Axis `axis`'s value while the machine advances to `frame`.
    pub fn axis(&self, frame: i64, axis: u32) -> f64 {
        self.sticks.iter().rev().find(|s| s.1 == axis && (s.0..s.0 + s.3).contains(&frame)).map_or(0.0, |s| s.2)
    }

    /// Whether any line holds a stick, so that a caller with none leaves the axes as they are.
    pub fn has_sticks(&self) -> bool {
        !self.sticks.is_empty()
    }
}
