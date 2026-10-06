//! The YM2612 alone, given a bus log: its reset line asserted and released, each write at its master clock, each
//! read answered, then run on; every sample's six channel outputs written as little-endian i16 and the reads'
//! values printed, for comparison with the board's pins and bus where nothing of the 68000's timing is wanted
//! (Nephrite_Native.md §24). `chip <events> <samples> <out>`; the events are lines of `a <clock>` (the line
//! asserted), `z <clock>` (released), `w <clock> <port> <value>` and `r <clock> <port>`, in time order.

use nephrite::ym2612::{SAMPLE, Ym2612};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let events = std::fs::read_to_string(&args[1]).expect("the events");
    let samples: u64 = args[2].parse().expect("a sample count");
    let mut y = Ym2612::new(false);
    y.trace = Some(Vec::new());
    let mut last = 0;
    let mut reads = Vec::new();
    for line in events.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let t: u64 = f[1].parse().expect("a clock");
        y.run(t, |_, _| {});
        match f[0] {
            "a" => y.reset_line(true, t),
            "z" => y.reset_line(false, t),
            "w" => y.write(f[2].parse().expect("a port"), u8::from_str_radix(f[3], 16).expect("a value"), t),
            "r" => reads.push(format!("{:02x}", y.read(f[2].parse().expect("a port"), t))),
            other => panic!("no event {other}"),
        }
        last = t;
    }
    y.run(last + samples * SAMPLE, |_, _| {});
    let bytes: Vec<u8> = y.trace.take().unwrap().iter().flat_map(|c| c.iter().flat_map(|&v| (v as i16).to_le_bytes())).collect();
    std::fs::write(&args[3], bytes).expect("the trace");
    println!("{}", reads.join(" "));
}
