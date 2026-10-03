//! What each image needs, as `firmware_for` answers the host: `firmware_for <rom>...` prints a line an image.
use emusen_native::core::Core;

fn main() {
    for path in std::env::args().skip(1) {
        let image = std::fs::read(&path).unwrap();
        let list: Vec<String> = <venusrt::machine::Machine as Core>::firmware_for(&image).iter().map(|f| format!("{}={} ({} bytes)", f.which, f.name, f.size)).collect();
        println!("{path}\t{}", list.join(", "));
    }
}
