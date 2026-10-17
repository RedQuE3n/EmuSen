//! The committed image against its digest and against a rebuild by the pinned toolchain (Diamond_OpenKernel.md §6).

use diamond_kernel_harness::{image, sha256};

#[test]
fn the_committed_image_is_512_kib_and_matches_its_digest() {
    let bytes = image::committed();
    assert_eq!(bytes.len(), image::SIZE);
    assert_eq!(sha256::hex(&bytes), image::committed_digest());
}

/// Skipped when the pinned toolchain is absent, unless DIAMOND_KERNEL_REBUILD=require (as CI sets it).
#[test]
fn the_image_rebuilds_to_the_committed_bytes() {
    let required = std::env::var("DIAMOND_KERNEL_REBUILD").is_ok_and(|v| v == "require");
    match image::rebuild() {
        None if required => panic!("the pinned toolchain {} is required and absent", image::channel()),
        None => eprintln!("skipped: the pinned toolchain {} is not installed", image::channel()),
        Some(Err(e)) => panic!("the kernel did not build: {e}"),
        Some(Ok(bytes)) => {
            let committed = image::committed();
            let first = bytes.iter().zip(&committed).position(|(a, b)| a != b);
            assert!(
                first.is_none() && bytes.len() == committed.len(),
                "the rebuild differs from diamond-kernel.bin, first at {first:X?}; run `cargo run --release --bin diamond-kernel -- image`"
            );
        }
    }
}

/// Every J and JAL in the kernel's code lands in the part it lies in, the ROM's code or the exception path's in the
/// kernel's RAM: the two lie in different 256 MiB regions, which a jump cannot cross and the linker does not check
/// (Diamond_OpenKernel_Phase2.md §25.3). Skipped like the rebuild.
#[test]
fn every_jump_lands_in_the_part_of_the_kernel_it_lies_in() {
    let required = std::env::var("DIAMOND_KERNEL_REBUILD").is_ok_and(|v| v == "require");
    let elf = match image::rebuild_elf() {
        None if required => panic!("the pinned toolchain {} is required and absent", image::channel()),
        None => return eprintln!("skipped: the pinned toolchain {} is not installed", image::channel()),
        Some(Err(e)) => panic!("the kernel did not build: {e}"),
        Some(Ok(elf)) => elf,
    };
    let at = |name: &str| image::symbol(&elf, name).unwrap_or_else(|| panic!("{name} in the ELF"));
    let parts = [("the ROM", 0x9FC0_0200, at("__text_end")), ("the RAM", at("__ramtext_start"), at("__ramtext_end"))];
    assert!(parts[1].2 > parts[1].1, "the exception path is in the kernel's RAM");
    let mut jumps = 0;
    for (part, from, to) in parts {
        let words = image::words(&elf, from, to).unwrap_or_else(|| panic!("{part}'s code in one section"));
        for (i, w) in words.iter().enumerate() {
            if matches!(w >> 26, 2 | 3) {
                let pc = from + 4 * i as u32;
                let target = (pc + 4) & 0xF000_0000 | (w & 0x03FF_FFFF) << 2;
                assert!((from..to).contains(&target), "{pc:08X} in {part} jumps to {target:08X}, outside it");
                jumps += 1;
            }
        }
    }
    assert!(jumps > 500, "{jumps} jumps checked");
}

/// The shell's committed PS-EXE matches its digest and a rebuild by the pinned toolchain (Diamond_Shell.md §1.3, P4).
#[test]
fn the_shell_rebuilds_to_its_committed_bytes() {
    assert_eq!(sha256::hex(&image::committed_shell()), image::committed_shell_digest());
    let required = std::env::var("DIAMOND_KERNEL_REBUILD").is_ok_and(|v| v == "require");
    match image::rebuild_shell() {
        None if required => panic!("the pinned toolchain {} is required and absent", image::channel()),
        None => eprintln!("skipped: the pinned toolchain {} is not installed", image::channel()),
        Some(Err(e)) => panic!("the shell did not build: {e}"),
        Some(Ok(bytes)) => assert!(bytes == image::committed_shell(), "the rebuild differs from diamond-shell.exe; run `cargo run --release --bin diamond-kernel -- image`"),
    }
}
