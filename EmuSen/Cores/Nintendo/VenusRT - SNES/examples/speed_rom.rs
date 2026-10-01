//! Writes one of the speed test's synthetic ROMs: cargo run --release --example speed_rom <variant> <out>.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    std::fs::write(&a[2], venusrt::speedtest::image(venusrt::speedtest::VARIANTS[a[1].parse::<usize>().unwrap()])).unwrap();
}
