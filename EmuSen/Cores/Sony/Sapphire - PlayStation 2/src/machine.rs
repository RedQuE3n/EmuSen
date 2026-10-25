//! The stage S1 machine (Sapphire_Native.md §2): the memories of Sapphire_Plan.md §4.3 allocated and zeroed, a frame
//! counter, a black picture and silence at the output rate. Nothing runs.

use crate::media::Media;

/// A memory the machine holds, by its space id and name (Sapphire_Plan.md §4.3).
pub struct Memory {
    pub id: u32,
    pub name: &'static str,
    pub bytes: Vec<u8>,
    pub read_only: bool,
}

use crate::sio2::card::MemoryCard;

pub const RAM_ID: u32 = 1;
pub const BIOS_ID: u32 = 10;

/// Each memory by id, name, size and whether it is read-only [R ps2tek "Memory Map", "VU", "GS", "SPU2", by the plan's
/// citations]: the EE's RDRAM, its scratchpad, the GS's eDRAM, the VUs' micro and data memories, the IOP's RAM, the
/// SPU2's sound RAM, and the boot ROM, the player's image when given.
pub const MEMORIES: [(u32, &str, usize, bool); 10] = [
    (RAM_ID, "RAM", 32 << 20, false),
    (2, "SCRATCH", 16 << 10, false),
    (3, "GSMEM", 4 << 20, false),
    (4, "VU0MICRO", 4 << 10, false),
    (5, "VU0DATA", 4 << 10, false),
    (6, "VU1MICRO", 16 << 10, false),
    (7, "VU1DATA", 16 << 10, false),
    (8, "IOPRAM", 2 << 20, false),
    (9, "SPU2RAM", 2 << 20, false),
    (BIOS_ID, "BIOS", BIOS_BYTES, true),
];

/// The boot ROM's size, every BIOS image's (Sapphire_Plan.md §5.9).
pub const BIOS_BYTES: usize = 4 << 20;

/// The output rate, the SPU2's 48,000 Hz (Sapphire_Plan.md §5.5).
pub const AUDIO_RATE: i32 = 48_000;

/// The standards' field rates, provisional until D-0 and the PCRTC's timing at S20 (Sapphire_Native.md §2.3).
pub const NTSC_FRAME: (u64, u64) = (60_000, 1_001);
pub const PAL_FRAME: (u64, u64) = (50, 1);

/// The stub's picture, 640 by 448 NTSC and 512 PAL, black, until the PCRTC decides it at S20.
pub const WIDTH: u32 = 640;
pub const NTSC_HEIGHT: u32 = 448;
pub const PAL_HEIGHT: u32 = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Market {
    Japan,
    Americas,
    Europe,
}

pub struct Machine {
    pub media: Media,
    pub market: Market,
    pub memories: Vec<Memory>,
    pub frames: i64,
    pub picture: Vec<u8>,
    pub skip: bool,
    pub pads: [u32; 2],
    pub ports: [Option<&'static str>; 2],
    /// Firmware files given, by number.
    pub firmware: Vec<u32>,
    /// Silent stereo frames made and not yet drained, and the bound on them in samples.
    pub silence: usize,
    pub audio_limit: usize,
    /// The memory cards in the two slots, the medium `ps2.card`: the player's files 0 and 1, or erased cards (S25).
    pub cards: [MemoryCard; 2],
    /// The sticks the frontend set, by port, in the pad's order: right X, right Y, left X, left Y (S25).
    pub sticks: [[u8; 4]; 2],
}

impl Machine {
    pub fn new(media: Media, market: Market, ports: [Option<&'static str>; 2]) -> Machine {
        let memories = MEMORIES.iter().map(|&(id, name, size, read_only)| Memory { id, name, bytes: vec![0; size], read_only }).collect();
        let height = if market == Market::Europe { PAL_HEIGHT } else { NTSC_HEIGHT };
        Machine {
            media,
            market,
            memories,
            frames: 0,
            picture: [0, 0, 0, 255].repeat((WIDTH * height) as usize),
            skip: false,
            pads: [0; 2],
            ports,
            firmware: Vec::new(),
            silence: 0,
            audio_limit: usize::MAX,
            cards: [MemoryCard::erased(), MemoryCard::erased()],
            sticks: [[0x80; 4]; 2],
        }
    }

    pub fn pal(&self) -> bool {
        self.market == Market::Europe
    }

    pub fn height(&self) -> u32 {
        if self.pal() { PAL_HEIGHT } else { NTSC_HEIGHT }
    }

    pub fn frame_rate(&self) -> (u64, u64) {
        if self.pal() { PAL_FRAME } else { NTSC_FRAME }
    }

    pub fn memory(&self, id: u32) -> Option<&Memory> {
        self.memories.iter().find(|m| m.id == id)
    }

    pub fn memory_mut(&mut self, id: u32) -> Option<&mut Memory> {
        self.memories.iter_mut().find(|m| m.id == id)
    }

    /// The stereo frames of silence frame `n` makes, so that every frame's count sums to the rate exactly.
    pub fn samples_in(&self, n: i64) -> usize {
        let (num, den) = self.frame_rate();
        let upto = |k: i64| (AUDIO_RATE as u128 * k as u128 * den as u128 / num as u128) as usize;
        upto(n + 1) - upto(n)
    }

    /// One frame: the count moves and the frame's silence is queued; nothing else does.
    pub fn advance(&mut self) {
        self.silence = (self.silence + self.samples_in(self.frames)).min(self.audio_limit / 2);
        self.frames += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::sample::elf;

    fn machine(market: Market) -> Machine {
        Machine::new(Media::read(&elf()).unwrap(), market, [Some("ps2.dualshock2"), None])
    }

    #[test]
    fn a_seconds_frames_make_a_seconds_silence_exactly() {
        for market in [Market::Americas, Market::Europe] {
            let mut m = machine(market);
            let (num, den) = m.frame_rate();
            let frames = (num * 10 / den) as i64;
            let made: usize = (0..frames).map(|n| m.samples_in(n)).sum();
            assert_eq!(made as u128, AUDIO_RATE as u128 * frames as u128 * den as u128 / num as u128);
            assert!((800..=960).contains(&m.samples_in(0)), "{}", m.samples_in(0));
            m.audio_limit = 1000;
            m.advance();
            m.advance();
            assert_eq!(m.silence, 500, "the bound drops the oldest");
        }
    }

    #[test]
    fn the_memories_are_the_plans_and_the_picture_is_black() {
        let m = machine(Market::Japan);
        assert_eq!(m.memories.iter().map(|m| (m.name, m.bytes.len())).collect::<Vec<_>>()[..3], [("RAM", 32 << 20), ("SCRATCH", 16 << 10), ("GSMEM", 4 << 20)]);
        assert_eq!(m.memories.iter().filter(|m| m.read_only).map(|m| m.name).collect::<Vec<_>>(), ["BIOS"]);
        assert!(m.picture.chunks(4).all(|p| p == [0, 0, 0, 255]) && m.picture.len() == 640 * 448 * 4);
        assert_eq!(machine(Market::Europe).picture.len(), 640 * 512 * 4);
    }
}
