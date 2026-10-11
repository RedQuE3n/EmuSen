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
