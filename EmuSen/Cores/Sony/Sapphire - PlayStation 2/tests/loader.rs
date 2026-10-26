//! The loader for IOP modules (Sapphire_Native.md §40): its call laid out as D-480 measured, and its program an EE
//! program the media reader takes. It runs only on the player's BIOS, so its runs are §40's, not CI's.

mod irx;

use sapphire::media::{Media, program};

/// D-480: the path from byte 8 of a block of 512 zero bytes, NUL-terminated, the 512-byte reply asked back.
#[test]
fn the_module_load_puts_the_path_at_byte_8() {
    let c = irx::module_load("cdrom0:\\TEST.IRX;1");
    assert_eq!((c.function, c.args.len(), c.recv), (irx::LOAD_MODULE, 512, 512));
    assert!(c.args[..8].iter().all(|&b| b == 0), "words 0 and 1 zero");
    assert_eq!(&c.args[8..26], b"cdrom0:\\TEST.IRX;1");
    assert!(c.args[26..].iter().all(|&b| b == 0), "the path ends in a NUL and the rest is zero");
}

/// The loader is an EE program entered at its origin, its argument blocks the image's last words, each in place.
#[test]
fn the_loader_is_an_ee_program_carrying_its_argument_blocks() {
    let calls = [irx::module_load("cdrom0:\\A.IRX;1"), irx::module_load("cdrom0:\\B.IRX;1")];
    let words = irx::image(&calls);
    let image = program(&words, irx::ORIGIN);
    let Ok(Media::Elf(elf)) = Media::read(&image) else { panic!("not an EE program") };
    assert_eq!(elf.entry, irx::ORIGIN);
    let tail: Vec<u8> = words[words.len() - 256..].iter().flat_map(|w| w.to_le_bytes()).collect();
    assert_eq!(&tail[..512], &calls[0].args[..]);
    assert_eq!(&tail[512..], &calls[1].args[..]);
}
