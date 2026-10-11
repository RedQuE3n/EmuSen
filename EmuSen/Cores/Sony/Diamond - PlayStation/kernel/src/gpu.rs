//! psx-spx's "BIOS GPU Functions" A(46h)-A(4Eh), as the port writes it gives; A(48h) is assembly so that v0 is left
//! as it was. The TTY messages send_gpu_linked_list prints are not reproduced. Diamond_OpenKernel.md §4.12.

use crate::layout::io;
use crate::mem::{rd, wr};

const TIMEOUT: u32 = 0x10_0000;

fn wait(mask: u32, limited: bool) -> bool {
    let mut n = 0;
    while rd(io::GP1) & mask == 0 {
        n += 1;
        if limited && n >= TIMEOUT {
            return false;
        }
    }
    true
}

/// A(4Eh): 0, or -1 on a timeout.
pub extern "C" fn gpu_sync() -> u32 {
    if rd(io::GP1) & 0x6000_0000 == 0 {
        return if wait(1 << 28, true) { 0 } else { u32::MAX };
    }
    let mut n = 0;
    while rd(io::D2_CHCR) & 1 << 24 != 0 {
        n += 1;
        if n >= TIMEOUT {
            return u32::MAX;
        }
    }
    wait(1 << 28, false);
    wr(io::GP1, 0x0400_0000);
    0
}

/// A(49h).
pub extern "C" fn gpu_cw(command: u32) -> u32 {
    let r = gpu_sync();
    wr(io::GP0, command);
    r
}

/// A(4Ah).
pub extern "C" fn gpu_cwp(src: u32, count: u32) -> u32 {
    gpu_sync();
    for i in 0..count {
        wr(io::GP0, rd(src + 4 * i));
    }
    0
}

/// A(4Bh).
pub extern "C" fn send_gpu_linked_list(src: u32) -> u32 {
    gpu_sync();
    wr(io::GP1, 0x0400_0002);
    wr(io::DICR, 0);
    wr(io::DPCR, rd(io::DPCR) | 0x800);
    wr(io::D2_MADR, src);
    wr(io::D2_BCR, 0);
    wr(io::D2_CHCR, 0x0100_0401);
    0
}

/// A(4Ch): the I/O address returned, as documented.
pub extern "C" fn gpu_abort_dma() -> u32 {
    wr(io::D2_CHCR, 0x401);
    wr(io::GP1, 0x0400_0000);
    wr(io::GP1, 0x0200_0000);
    wr(io::GP1, 0x0100_0000);
    0x1F80_1814
}

/// A(4Dh).
pub extern "C" fn get_gpu_status() -> u32 {
    rd(io::GP1)
}

fn position(x: u32, y: u32) -> u32 {
    y << 16 | x & 0xFFFF
}

/// A(46h): by the CPU once GPUSTAT bit 26 is set; Xsiz*Ysiz/2 words, so an odd count leaves the GPU waiting.
pub extern "C" fn gpu_dw(x: u32, y: u32, w: u32, h: u32, src: u32) -> u32 {
    wait(1 << 26, false);
    wr(io::GP0, 0xA000_0000);
    wr(io::GP0, position(x, y));
    wr(io::GP0, position(w, h));
    let n = w.wrapping_mul(h) / 2;
    for i in 0..n {
        wr(io::GP0, rd(src + 4 * i));
    }
    src + 4 * n
}

/// A(47h): by DMA in blocks of 10h words.
pub extern "C" fn gpu_send_dma(_x: u32, _y: u32, w: u32, h: u32, src: u32) -> u32 {
    gpu_sync();
    wr(io::GP0, 0xA000_0000);
    wr(io::GP1, 0x0400_0002);
    wr(io::DPCR, rd(io::DPCR) | 0x800);
    wr(io::D2_MADR, src);
    wr(io::D2_BCR, (w.wrapping_mul(h) / 32) << 16 | 0x10);
    wr(io::D2_CHCR, 0x0100_0201);
    0x1F80_1810
}
