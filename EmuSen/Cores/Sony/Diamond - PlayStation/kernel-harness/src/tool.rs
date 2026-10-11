//! `diamond-kernel image` rebuilds the kernel with the pinned toolchain and writes diamond-kernel.bin and its SHA-256;
//! `diamond-kernel calls` prints the call table for Diamond_OpenKernel.md §5; `diamond-kernel digest <elf>` prints
//! the SHA-256 of an ELF's flattened image, to compare a build made elsewhere, and `diamond-kernel flatten <elf> <out>`
//! writes that image, to run the suite against it through DIAMOND_KERNEL_IMAGE.

use diamond_kernel_harness::{calls, image, sha256};

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("image") => {
            let bytes = match image::rebuild() {
                None => {
                    eprintln!("the pinned toolchain ({}) is not installed", image::channel());
                    std::process::exit(2);
                }
                Some(Err(e)) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
                Some(Ok(b)) => b,
            };
            let dir = image::kernel_dir();
            std::fs::write(dir.join("diamond-kernel.bin"), &bytes).unwrap();
            let digest = sha256::hex(&bytes);
            std::fs::write(dir.join("diamond-kernel.bin.sha256"), format!("{digest}  diamond-kernel.bin\n")).unwrap();
            println!("{digest}  diamond-kernel.bin ({} bytes)", bytes.len());
        }
        Some("digest") => {
            let path = std::env::args().nth(2).expect("an ELF");
            let image = image::flatten(&std::fs::read(path).expect("readable")).unwrap_or_else(|e| panic!("{e}"));
            println!("{}", sha256::hex(&image));
        }
        Some("flatten") => {
            let (from, to) = (std::env::args().nth(2).expect("an ELF"), std::env::args().nth(3).expect("an output"));
            let image = image::flatten(&std::fs::read(from).expect("readable")).unwrap_or_else(|e| panic!("{e}"));
            std::fs::write(to, image).expect("writable");
        }
        Some("calls") => print!("{}", calls::markdown(&calls::parse(&calls::source()))),
        _ => {
            eprintln!("usage: diamond-kernel image | calls | digest <elf> | flatten <elf> <out>");
            std::process::exit(2);
        }
    }
}
