//! The test programs for the referees (Endou_Native.md §9.11, §11): writes each program's image into a folder for the
//! reference probe to run, a Master System's as `.sms`, an SG-1000's as `.sg` and a Game Gear's as `.gg`, and prints what Endou leaves in RAM after ten frames on each model, one line a run:
//! `<program> <model> <the first 16 bytes of RAM>`. `cargo run --release --example referee -- <folder>`.

#[path = "../programs/images.rs"]
mod images;

use emusen_native::core::{Core, Create, Settings, sys};
use endou::machine::Machine;

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).expect("a folder"));
    std::fs::create_dir_all(&dir).expect("the folder");
    for p in images::PROGRAMS.into_iter().chain(images::GG_PROGRAMS) {
        let image = images::image(p);
        let (extension, models): (&str, &[&str]) = if p.gg { ("gg", &["gg"]) } else if p.header.is_some() { ("sms", &["mark3", "sms1", "sms2"]) } else { ("sg", &["sg1000", "mark3"]) };
        std::fs::write(dir.join(format!("{}.{extension}", p.name)), &image).expect("the image written");
        for &model in models {
            let mut settings = vec![("model".to_owned(), model.to_owned())];
            if model == "mark3" {
                settings.push(("region".to_owned(), "sms.jp".to_owned()));
            }
            let request = Create { image: &image, settings: Settings::from_pairs(settings), files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
            let mut m = if p.gg { Machine::create_as("gg", &request) } else { Machine::create(&request) }.expect("the machine");
            for _ in 0..10 {
                Core::advance(&mut m, &mut 0).unwrap();
            }
            let ram: Vec<String> = m.bus.ram[..16].iter().map(|b| format!("{b:02X}")).collect();
            println!("{} {} {}", p.name, model, ram.join(""));
        }
    }
}
