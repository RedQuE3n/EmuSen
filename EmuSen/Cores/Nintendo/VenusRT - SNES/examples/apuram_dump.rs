#[path = "common/load.rs"]
mod load;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = load::machine(&a[1]);
    let f: i64 = a[2].parse().unwrap();
    while m.total_frames() < f { m.run_frame(); }
    std::fs::write(&a[3], &m.sys.apu.ram[..]).unwrap();
}
