//! The CPU's trace in the reference probe's record, to be compared with Mesen's instruction by instruction.
//! cargo run --release --example cpu_trace <rom> <frames> <out>
#[path = "common/load.rs"]
mod load;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = load::machine(&a[1]);
    m.trace = Some(Vec::new());
    while m.total_frames() < a[2].parse::<i64>().unwrap() {
        m.run_frame();
    }
    let mut out = b"ESCT\x02\x00\x00\x00".to_vec();
    out.extend(m.trace.take().unwrap());
    std::fs::write(&a[3], out).unwrap();
}
