//! The S-DSP: eight voices of BRR samples with their envelopes, noise, pitch modulation and the echo with its FIR
//! filter, run one of its 32 steps per SPC700 cycle as anomie's S-DSP document places each access, into a 32 kHz
//! stereo sample (anomie, romhacking.net 191; fullsnes, "SNES APU DSP"). See VenusRT_Native.md §22.

/// The 512 Gaussian interpolation coefficients, as fullsnes and anomie both list them.
const GAUSS: [i32; 512] = [
    0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000,
    0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x002, 0x002, 0x002, 0x002, 0x002,
    0x002, 0x002, 0x003, 0x003, 0x003, 0x003, 0x003, 0x004, 0x004, 0x004, 0x004, 0x004, 0x005, 0x005, 0x005, 0x005,
    0x006, 0x006, 0x006, 0x006, 0x007, 0x007, 0x007, 0x008, 0x008, 0x008, 0x009, 0x009, 0x009, 0x00A, 0x00A, 0x00A,
    0x00B, 0x00B, 0x00B, 0x00C, 0x00C, 0x00D, 0x00D, 0x00E, 0x00E, 0x00F, 0x00F, 0x00F, 0x010, 0x010, 0x011, 0x011,
    0x012, 0x013, 0x013, 0x014, 0x014, 0x015, 0x015, 0x016, 0x017, 0x017, 0x018, 0x018, 0x019, 0x01A, 0x01B, 0x01B,
    0x01C, 0x01D, 0x01D, 0x01E, 0x01F, 0x020, 0x020, 0x021, 0x022, 0x023, 0x024, 0x024, 0x025, 0x026, 0x027, 0x028,
    0x029, 0x02A, 0x02B, 0x02C, 0x02D, 0x02E, 0x02F, 0x030, 0x031, 0x032, 0x033, 0x034, 0x035, 0x036, 0x037, 0x038,
    0x03A, 0x03B, 0x03C, 0x03D, 0x03E, 0x040, 0x041, 0x042, 0x043, 0x045, 0x046, 0x047, 0x049, 0x04A, 0x04C, 0x04D,
    0x04E, 0x050, 0x051, 0x053, 0x054, 0x056, 0x057, 0x059, 0x05A, 0x05C, 0x05E, 0x05F, 0x061, 0x063, 0x064, 0x066,
    0x068, 0x06A, 0x06B, 0x06D, 0x06F, 0x071, 0x073, 0x075, 0x076, 0x078, 0x07A, 0x07C, 0x07E, 0x080, 0x082, 0x084,
    0x086, 0x089, 0x08B, 0x08D, 0x08F, 0x091, 0x093, 0x096, 0x098, 0x09A, 0x09C, 0x09F, 0x0A1, 0x0A3, 0x0A6, 0x0A8,
    0x0AB, 0x0AD, 0x0AF, 0x0B2, 0x0B4, 0x0B7, 0x0BA, 0x0BC, 0x0BF, 0x0C1, 0x0C4, 0x0C7, 0x0C9, 0x0CC, 0x0CF, 0x0D2,
    0x0D4, 0x0D7, 0x0DA, 0x0DD, 0x0E0, 0x0E3, 0x0E6, 0x0E9, 0x0EC, 0x0EF, 0x0F2, 0x0F5, 0x0F8, 0x0FB, 0x0FE, 0x101,
    0x104, 0x107, 0x10B, 0x10E, 0x111, 0x114, 0x118, 0x11B, 0x11E, 0x122, 0x125, 0x129, 0x12C, 0x130, 0x133, 0x137,
    0x13A, 0x13E, 0x141, 0x145, 0x148, 0x14C, 0x150, 0x153, 0x157, 0x15B, 0x15F, 0x162, 0x166, 0x16A, 0x16E, 0x172,
    0x176, 0x17A, 0x17D, 0x181, 0x185, 0x189, 0x18D, 0x191, 0x195, 0x19A, 0x19E, 0x1A2, 0x1A6, 0x1AA, 0x1AE, 0x1B2,
    0x1B7, 0x1BB, 0x1BF, 0x1C3, 0x1C8, 0x1CC, 0x1D0, 0x1D5, 0x1D9, 0x1DD, 0x1E2, 0x1E6, 0x1EB, 0x1EF, 0x1F3, 0x1F8,
    0x1FC, 0x201, 0x205, 0x20A, 0x20F, 0x213, 0x218, 0x21C, 0x221, 0x226, 0x22A, 0x22F, 0x233, 0x238, 0x23D, 0x241,
    0x246, 0x24B, 0x250, 0x254, 0x259, 0x25E, 0x263, 0x267, 0x26C, 0x271, 0x276, 0x27B, 0x280, 0x284, 0x289, 0x28E,
    0x293, 0x298, 0x29D, 0x2A2, 0x2A6, 0x2AB, 0x2B0, 0x2B5, 0x2BA, 0x2BF, 0x2C4, 0x2C9, 0x2CE, 0x2D3, 0x2D8, 0x2DC,
    0x2E1, 0x2E6, 0x2EB, 0x2F0, 0x2F5, 0x2FA, 0x2FF, 0x304, 0x309, 0x30E, 0x313, 0x318, 0x31D, 0x322, 0x326, 0x32B,
    0x330, 0x335, 0x33A, 0x33F, 0x344, 0x349, 0x34E, 0x353, 0x357, 0x35C, 0x361, 0x366, 0x36B, 0x370, 0x374, 0x379,
    0x37E, 0x383, 0x388, 0x38C, 0x391, 0x396, 0x39B, 0x39F, 0x3A4, 0x3A9, 0x3AD, 0x3B2, 0x3B7, 0x3BB, 0x3C0, 0x3C5,
    0x3C9, 0x3CE, 0x3D2, 0x3D7, 0x3DC, 0x3E0, 0x3E5, 0x3E9, 0x3ED, 0x3F2, 0x3F6, 0x3FB, 0x3FF, 0x403, 0x408, 0x40C,
    0x410, 0x415, 0x419, 0x41D, 0x421, 0x425, 0x42A, 0x42E, 0x432, 0x436, 0x43A, 0x43E, 0x442, 0x446, 0x44A, 0x44E,
    0x452, 0x455, 0x459, 0x45D, 0x461, 0x465, 0x468, 0x46C, 0x470, 0x473, 0x477, 0x47A, 0x47E, 0x481, 0x485, 0x488,
    0x48C, 0x48F, 0x492, 0x496, 0x499, 0x49C, 0x49F, 0x4A2, 0x4A6, 0x4A9, 0x4AC, 0x4AF, 0x4B2, 0x4B5, 0x4B7, 0x4BA,
    0x4BD, 0x4C0, 0x4C3, 0x4C5, 0x4C8, 0x4CB, 0x4CD, 0x4D0, 0x4D2, 0x4D5, 0x4D7, 0x4D9, 0x4DC, 0x4DE, 0x4E0, 0x4E3,
    0x4E5, 0x4E7, 0x4E9, 0x4EB, 0x4ED, 0x4EF, 0x4F1, 0x4F3, 0x4F5, 0x4F6, 0x4F8, 0x4FA, 0x4FB, 0x4FD, 0x4FF, 0x500,
    0x502, 0x503, 0x504, 0x506, 0x507, 0x508, 0x50A, 0x50B, 0x50C, 0x50D, 0x50E, 0x50F, 0x510, 0x511, 0x511, 0x512,
    0x513, 0x514, 0x514, 0x515, 0x516, 0x516, 0x517, 0x517, 0x517, 0x518, 0x518, 0x518, 0x518, 0x518, 0x519, 0x519,
];

/// Samples between counter events for each 5-bit rate, 0 meaning never (anomie, "COUNTERS").
const RATES: [u32; 32] = [
    0, 2048, 1536, 1280, 1024, 768, 640, 512, 384, 320, 256, 192, 160, 128, 96, 80, 64, 48, 40, 32, 24, 20, 16, 12, 10,
    8, 6, 5, 4, 3, 2, 1,
];

/// Each rate's offset from the counter's zero.
const OFFSETS: [u32; 32] = [
    0, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536,
    0, 1040, 536, 0, 1040, 0, 0,
];

mod reg {
    pub const MVOLL: usize = 0x0C;
    pub const MVOLR: usize = 0x1C;
    pub const EVOLL: usize = 0x2C;
    pub const EVOLR: usize = 0x3C;
    pub const KON: usize = 0x4C;
    pub const KOFF: usize = 0x5C;
    pub const FLG: usize = 0x6C;
    pub const ENDX: usize = 0x7C;
    pub const EFB: usize = 0x0D;
    pub const PMON: usize = 0x2D;
    pub const NON: usize = 0x3D;
    pub const EON: usize = 0x4D;
    pub const DIR: usize = 0x5D;
    pub const ESA: usize = 0x6D;
    pub const EDL: usize = 0x7D;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    #[default]
    Release = 0,
    Attack = 1,
    Decay = 2,
    Sustain = 3,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Voice {
    /// The 12-sample ring of decoded BRR samples, three groups of four; `write` is the oldest group's start.
    pub buf: [i16; 12],
    pub write: u8,
    /// The interpolation position: bits 12-14 the sample within the two groups ahead of `write`, bits 4-11 the fraction.
    pub pos: u16,
    pub brr_addr: u16,
    /// The next data byte's offset within the 9-byte block, 1 to 7.
    pub brr_offset: u8,
    pub loop_pending: bool,
    /// Samples left of the key-on start-up (anomie's #1 to #5), and what it was at this sample's S3c.
    pub kon_delay: u8,
    pub prev_delay: u8,
    pub keyed: bool,
    pub looped: bool,
    pub mode: Mode,
    pub env: i32,
    /// The last new envelope value before clamping, for the bent increase.
    pub hidden_env: i32,
    pub out: i32,
    /// ENVX as this sample shows it: the envelope applied to the sample, before this sample's update.
    pub envx: u8,
    pub srcn: u8,
    pub next_addr: u16,
    pub pitch: i32,
    pub adsr1: u8,
    pub header: u8,
    pub data: u8,
}

#[derive(Clone, Debug)]
pub struct Dsp {
    pub regs: [u8; 128],
    pub voices: [Voice; 8],
    /// The step of the 32 within the sample, the next to run.
    pub step: u8,
    pub every_other: bool,
    /// The global counter, from 0x77FF down to 0, and the noise generator's 15 bits.
    pub counter: u16,
    pub noise: u16,
    /// KON as last written, cleared where it took effect; and the KON and KOFF the voices see for two samples.
    pub new_kon: u8,
    pub t_kon: u8,
    pub t_koff: u8,
    pub t_pmon: u8,
    pub t_non: u8,
    pub t_eon: u8,
    pub t_dir: u8,
    pub t_esa: u8,
    pub t_echo_off: bool,
    pub t_ffc: [i8; 8],
    /// The last voice's output after its envelope, the next voice's pitch modulator.
    pub last_out: i32,
    pub endx_buf: u8,
    pub outx_buf: u8,
    pub envx_buf: u8,
    pub main: [i32; 2],
    pub echo: [i32; 2],
    pub fir_out: [i32; 2],
    pub fir: [[i16; 8]; 2],
    pub fir_pos: u8,
    pub echo_ptr: u16,
    pub echo_offset: u16,
    pub echo_length: u16,
}

impl Default for Dsp {
    fn default() -> Self {
        let mut regs = [0u8; 128];
        // anomie: FLG acts as $E0 at power-on and reset; ENDX is clear.
        regs[reg::FLG] = 0xE0;
        Dsp {
            regs,
            voices: Default::default(),
            step: 0,
            every_other: false,
            counter: 0,
            noise: 0x4000,
            new_kon: 0,
            t_kon: 0,
            t_koff: 0,
            t_pmon: 0,
            t_non: 0,
            t_eon: 0,
            t_dir: 0,
            t_esa: 0,
            t_echo_off: true,
            t_ffc: [0; 8],
            last_out: 0,
            endx_buf: 0,
            outx_buf: 0,
            envx_buf: 0,
            main: [0; 2],
            echo: [0; 2],
            fir_out: [0; 2],
            fir: [[0; 8]; 2],
            fir_pos: 0,
            echo_ptr: 0,
            echo_offset: 0,
            echo_length: 0,
        }
    }
}

/// The DSP's state beyond its registers, packed for the machine's state in a fixed size.
pub const PACKED_BYTES: usize = 8 * VOICE_BYTES + 96;
const VOICE_BYTES: usize = 24 + 2 + 2 + 2 + 8 + 4 + 4 + 4 + 1 + 4 + 4;

struct Packer<'a>(&'a mut [u8], usize);

impl Packer<'_> {
    fn put(&mut self, b: &[u8]) {
        self.0[self.1..self.1 + b.len()].copy_from_slice(b);
        self.1 += b.len();
    }
}

struct Unpacker<'a>(&'a [u8], usize);

impl Unpacker<'_> {
    fn take<const N: usize>(&mut self) -> [u8; N] {
        let b = self.0[self.1..self.1 + N].try_into().expect("a packed DSP state");
        self.1 += N;
        b
    }
    fn u8(&mut self) -> u8 {
        self.take::<1>()[0]
    }
    fn u16(&mut self) -> u16 {
        u16::from_le_bytes(self.take())
    }
    fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.take())
    }
}

fn clamp16(v: i32) -> i32 {
    v.clamp(-0x8000, 0x7FFF)
}

/// Sign-extends a value's low 15 bits.
fn clip15(v: i32) -> i32 {
    ((v as i16) << 1 >> 1) as i32
}

impl Dsp {
    pub fn pack(&self, out: &mut [u8]) {
        let mut p = Packer(out, 0);
        for x in &self.voices {
            for b in x.buf {
                p.put(&b.to_le_bytes());
            }
            p.put(&x.pos.to_le_bytes());
            p.put(&x.brr_addr.to_le_bytes());
            p.put(&x.next_addr.to_le_bytes());
            p.put(&[x.write, x.brr_offset, x.loop_pending as u8, x.kon_delay, x.prev_delay, x.keyed as u8, x.looped as u8, x.mode as u8]);
            p.put(&x.env.to_le_bytes());
            p.put(&x.hidden_env.to_le_bytes());
            p.put(&x.out.to_le_bytes());
            p.put(&[x.srcn]);
            p.put(&x.pitch.to_le_bytes());
            p.put(&[x.adsr1, x.header, x.data, x.envx]);
        }
        p.put(&[self.step, self.every_other as u8, self.new_kon, self.t_kon, self.t_koff, self.t_pmon, self.t_non, self.t_eon]);
        p.put(&[self.t_dir, self.t_esa, self.t_echo_off as u8, self.endx_buf, self.outx_buf, self.envx_buf, self.fir_pos, 0]);
        p.put(&self.counter.to_le_bytes());
        p.put(&self.noise.to_le_bytes());
        p.put(&self.echo_ptr.to_le_bytes());
        p.put(&self.echo_offset.to_le_bytes());
        p.put(&self.echo_length.to_le_bytes());
        p.put(&self.t_ffc.map(|c| c as u8));
        p.put(&self.last_out.to_le_bytes());
        for v in [self.main, self.echo, self.fir_out].concat() {
            p.put(&(v as i16).to_le_bytes());
        }
        for side in &self.fir {
            for v in side {
                p.put(&v.to_le_bytes());
            }
        }
        let used = p.1;
        p.0[used..].fill(0);
    }

    pub fn unpack(&mut self, data: &[u8]) {
        let mut u = Unpacker(data, 0);
        for x in &mut self.voices {
            for b in &mut x.buf {
                *b = u.u16() as i16;
            }
            x.pos = u.u16();
            x.brr_addr = u.u16();
            x.next_addr = u.u16();
            let f: [u8; 8] = u.take();
            x.write = f[0] % 12;
            x.brr_offset = f[1];
            x.loop_pending = f[2] != 0;
            x.kon_delay = f[3];
            x.prev_delay = f[4];
            x.keyed = f[5] != 0;
            x.looped = f[6] != 0;
            x.mode = [Mode::Release, Mode::Attack, Mode::Decay, Mode::Sustain][f[7] as usize & 3];
            x.env = u.i32();
            x.hidden_env = u.i32();
            x.out = u.i32();
            x.srcn = u.u8();
            x.pitch = u.i32();
            let g: [u8; 4] = u.take();
            (x.adsr1, x.header, x.data, x.envx) = (g[0], g[1], g[2], g[3]);
        }
        let g: [u8; 8] = u.take();
        (self.step, self.every_other, self.new_kon, self.t_kon, self.t_koff, self.t_pmon, self.t_non, self.t_eon) = (g[0] & 31, g[1] != 0, g[2], g[3], g[4], g[5], g[6], g[7]);
        let g: [u8; 8] = u.take();
        (self.t_dir, self.t_esa, self.t_echo_off, self.endx_buf, self.outx_buf, self.envx_buf, self.fir_pos) = (g[0], g[1], g[2] != 0, g[3], g[4], g[5], g[6] & 7);
        self.counter = u.u16();
        self.noise = u.u16();
        self.echo_ptr = u.u16();
        self.echo_offset = u.u16();
        self.echo_length = u.u16();
        self.t_ffc = u.take::<8>().map(|c| c as i8);
        self.last_out = u.i32();
        for i in 0..6 {
            let v = u.u16() as i16 as i32;
            [&mut self.main, &mut self.echo, &mut self.fir_out][i / 2][i % 2] = v;
        }
        for side in &mut self.fir {
            for v in side {
                *v = u.u16() as i16;
            }
        }
    }

    pub fn read(&self, address: u8) -> u8 {
        self.regs[(address & 0x7F) as usize]
    }

    /// The SPC700's write through $F3; $80-$FF are read-only mirrors.
    pub fn write(&mut self, address: u8, value: u8) {
        let a = address as usize;
        if a >= 0x80 {
            return;
        }
        match a {
            reg::ENDX => self.regs[a] = 0,
            reg::KON => {
                self.new_kon = value;
                self.regs[a] = value;
            }
            _ => self.regs[a] = value,
        }
    }

    /// Whether the counter's event for `rate` falls on this sample.
    fn fires(&self, rate: u8) -> bool {
        let r = rate as usize & 0x1F;
        r != 0 && (self.counter as u32 + OFFSETS[r]) % RATES[r] == 0
    }

    fn vreg(&self, v: usize, r: usize) -> u8 {
        self.regs[v << 4 | r]
    }

    /// One DSP step, in the SPC700 cycle it shares; `writable` is TEST bit 1, `out` takes the stereo sample.
    pub fn step(&mut self, ram: &mut [u8], writable: bool, out: &mut Vec<i16>) {
        let s = self.step;
        self.step = (s + 1) & 31;
        match s {
            0 => {
                self.s5(0);
                self.s2(1, ram);
            }
            1 => {
                self.s6(0);
                self.s3(1, ram);
            }
            2 => {
                self.s7(0);
                self.s4(1, ram);
                self.s1(3);
            }
            3 => {
                self.s8(0);
                self.s5(1);
                self.s2(2, ram);
            }
            4 => {
                self.s9(0);
                self.s6(1);
                self.s3(2, ram);
            }
            5 => {
                self.s7(1);
                self.s4(2, ram);
                self.s1(4);
            }
            6 => {
                self.s8(1);
                self.s5(2);
                self.s2(3, ram);
            }
            7 => {
                self.s9(1);
                self.s6(2);
                self.s3(3, ram);
            }
            8 => {
                self.s7(2);
                self.s4(3, ram);
                self.s1(5);
            }
            9 => {
                self.s8(2);
                self.s5(3);
                self.s2(4, ram);
            }
            10 => {
                self.s9(2);
                self.s6(3);
                self.s3(4, ram);
            }
            11 => {
                self.s7(3);
                self.s4(4, ram);
                self.s1(6);
            }
            12 => {
                self.s8(3);
                self.s5(4);
                self.s2(5, ram);
            }
            13 => {
                self.s9(3);
                self.s6(4);
                self.s3(5, ram);
            }
            14 => {
                self.s7(4);
                self.s4(5, ram);
                self.s1(7);
            }
            15 => {
                self.s8(4);
                self.s5(5);
                self.s2(6, ram);
            }
            16 => {
                self.s9(4);
                self.s6(5);
                self.s3(6, ram);
            }
            17 => {
                self.s1(0);
                self.s7(5);
                self.s4(6, ram);
            }
            18 => {
                self.s8(5);
                self.s5(6);
                self.s2(7, ram);
            }
            19 => {
                self.s9(5);
                self.s6(6);
                self.s3(7, ram);
            }
            20 => {
                self.s1(1);
                self.s7(6);
                self.s4(7, ram);
            }
            21 => {
                self.s2(0, ram);
                self.s8(6);
                self.s5(7);
            }
            22 => {
                self.s3a(0);
                self.s9(6);
                self.s6(7);
                self.echo_ptr = ((self.t_esa as u16) << 8).wrapping_add(self.echo_offset);
                self.echo_read(0, ram);
                self.t_ffc[0] = self.regs[0x0F] as i8;
            }
            23 => {
                self.s7(7);
                self.echo_read(1, ram);
                self.t_ffc[1] = self.regs[0x1F] as i8;
                self.t_ffc[2] = self.regs[0x2F] as i8;
            }
            24 => {
                self.s8(7);
                for i in 3..6 {
                    self.t_ffc[i] = self.regs[i << 4 | 0x0F] as i8;
                }
            }
            25 => {
                self.s3b(0, ram);
                self.s9(7);
                self.t_ffc[6] = self.regs[0x6F] as i8;
                self.t_ffc[7] = self.regs[0x7F] as i8;
                self.fir_pos = (self.fir_pos + 1) & 7;
            }
            26 => self.output(0, out),
            27 => {
                self.output(1, out);
                self.t_pmon = self.regs[reg::PMON] & 0xFE;
            }
            28 => {
                self.t_non = self.regs[reg::NON];
                self.t_eon = self.regs[reg::EON];
                self.t_dir = self.regs[reg::DIR];
                self.t_echo_off = self.regs[reg::FLG] & 0x20 != 0;
            }
            29 => {
                self.every_other = !self.every_other;
                if self.every_other {
                    self.new_kon &= !self.t_kon;
                }
                self.counter = if self.counter == 0 { 0x77FF } else { self.counter - 1 };
                if self.echo_offset == 0 {
                    self.echo_length = ((self.regs[reg::EDL] & 0x0F) as u16) << 11;
                }
                self.t_esa = self.regs[reg::ESA];
                self.echo_write(0, ram, writable);
                self.t_echo_off = self.regs[reg::FLG] & 0x20 != 0;
            }
            30 => {
                if self.every_other {
                    self.t_koff = self.regs[reg::KOFF];
                    self.t_kon = self.new_kon;
                }
                self.s3c(0);
                self.echo_write(1, ram, writable);
                self.echo_offset = self.echo_offset.wrapping_add(4);
                if self.echo_offset >= self.echo_length {
                    self.echo_offset = 0;
                }
                if self.fires(self.regs[reg::FLG]) {
                    let n = self.noise;
                    self.noise = (n >> 1) | (((n << 14) ^ (n << 13)) & 0x4000);
                }
            }
            _ => {
                self.s4(0, ram);
                self.s1(2);
            }
        }
    }

    fn s1(&mut self, v: usize) {
        self.voices[v].srcn = self.vreg(v, 4);
    }

    fn s2(&mut self, v: usize, ram: &[u8]) {
        let entry = ((self.t_dir as u16) << 8).wrapping_add((self.voices[v].srcn as u16) << 2);
        let pitch_lo = self.vreg(v, 2);
        let adsr1 = self.vreg(v, 5);
        let x = &mut self.voices[v];
        let at = if x.kon_delay == 5 { entry } else { entry.wrapping_add(2) };
        x.next_addr = u16::from_le_bytes([ram[at as usize], ram[at.wrapping_add(1) as usize]]);
        if x.kon_delay == 5 {
            x.brr_addr = x.next_addr;
        } else if x.loop_pending {
            x.brr_addr = x.next_addr;
            x.loop_pending = false;
        }
        x.pitch = pitch_lo as i32;
        x.adsr1 = adsr1;
    }

    fn s3(&mut self, v: usize, ram: &[u8]) {
        self.s3a(v);
        self.s3b(v, ram);
        self.s3c(v);
    }

    fn s3a(&mut self, v: usize) {
        let hi = (self.vreg(v, 3) & 0x3F) as i32;
        let mut p = hi << 8 | self.voices[v].pitch;
        if self.t_pmon & (1 << v) != 0 {
            p += ((self.last_out >> 4) * p) >> 10;
        }
        self.voices[v].pitch = p;
    }

    fn s3b(&mut self, v: usize, ram: &[u8]) {
        let x = &mut self.voices[v];
        x.header = ram[x.brr_addr as usize];
        x.data = ram[x.brr_addr.wrapping_add(x.brr_offset as u16) as usize];
    }

    fn s3c(&mut self, v: usize) {
        let bit = 1u8 << v;
        let flg = self.regs[reg::FLG];
        let gain_reg = self.vreg(v, 6);
        let gain = self.vreg(v, 7);
        let noise = clip15(self.noise as i32);
        let (koff, kon, non) = (self.t_koff & bit != 0, self.t_kon & bit != 0, self.t_non & bit != 0);
        let counter = self.counter;
        let even = self.every_other;
        let x = &mut self.voices[v];
        let d = x.kon_delay;
        x.kon_delay = d.saturating_sub(1);
        x.prev_delay = d;
        x.keyed = false;
        if d == 5 {
            x.pos = 0;
            x.write = 0;
            x.brr_offset = 1;
            x.loop_pending = false;
        }
        let sample = if non { noise } else { interpolate(x) };
        x.out = (sample * x.env) >> 11;
        x.envx = (x.env >> 4) as u8;
        self.last_out = x.out;
        if d != 5 && x.header & 3 == 1 {
            x.mode = Mode::Release;
            x.env = 0;
        }
        // D-29: KON and KOFF act on the samples the poll loads them, every other one.
        if flg & 0x80 != 0 || (koff && even) {
            x.mode = Mode::Release;
            if flg & 0x80 != 0 {
                x.env = 0;
            }
        }
        if kon && even {
            x.kon_delay = 5;
            x.keyed = true;
            x.mode = Mode::Attack;
            x.env = 0;
            x.hidden_env = 0;
            x.looped = false;
        }
        let setting = if x.adsr1 & 0x80 != 0 { gain_reg } else { gain };
        if x.kon_delay == 0 && d <= 1 {
            run_envelope(x, setting, |r| r != 0 && (counter as u32 + OFFSETS[r as usize]) % RATES[r as usize] == 0);
        }
    }

    fn s4(&mut self, v: usize, ram: &[u8]) {
        let vol = self.vreg(v, 0) as i8 as i32;
        let eon = self.t_eon & (1 << v) != 0;
        let x = &mut self.voices[v];
        let side = clamp16((x.out * vol) >> 6);
        self.main[0] = clamp16(self.main[0] + side);
        if eon {
            self.echo[0] = clamp16(self.echo[0] + side);
        }
        match x.prev_delay {
            2..=4 => decode(x, ram),
            0 => {
                if x.pos >= 0x4000 {
                    decode(x, ram);
                    x.pos -= 0x4000;
                }
                x.pos = (x.pos as i32 + x.pitch).clamp(0, 0x7FFF) as u16;
            }
            _ => {}
        }
    }

    fn s5(&mut self, v: usize) {
        let vol = self.vreg(v, 1) as i8 as i32;
        let eon = self.t_eon & (1 << v) != 0;
        let bit = 1u8 << v;
        let x = &mut self.voices[v];
        let side = clamp16((x.out * vol) >> 6);
        self.main[1] = clamp16(self.main[1] + side);
        if eon {
            self.echo[1] = clamp16(self.echo[1] + side);
        }
        let mut endx = self.regs[reg::ENDX];
        if x.looped {
            endx |= bit;
            x.looped = false;
        }
        if x.keyed {
            endx &= !bit;
        }
        self.endx_buf = endx;
    }

    fn s6(&mut self, v: usize) {
        self.outx_buf = (self.voices[v].out >> 7) as u8;
    }

    fn s7(&mut self, v: usize) {
        self.regs[reg::ENDX] = self.endx_buf;
        self.envx_buf = self.voices[v].envx;
    }

    fn s8(&mut self, v: usize) {
        self.regs[v << 4 | 9] = self.outx_buf;
    }

    fn s9(&mut self, v: usize) {
        self.regs[v << 4 | 8] = self.envx_buf;
    }

    fn echo_read(&mut self, side: usize, ram: &[u8]) {
        let a = self.echo_ptr.wrapping_add(side as u16 * 2);
        let s = i16::from_le_bytes([ram[a as usize], ram[a.wrapping_add(1) as usize]]);
        let next = (self.fir_pos + 1) & 7;
        self.fir[side][next as usize] = s >> 1;
    }

    /// The FIR over the eight latest echo samples of one side, the newest at `fir_pos`.
    fn fir(&self, side: usize) -> i32 {
        let b = &self.fir[side];
        let mut sum = 0i32;
        for k in 0..7 {
            sum += (b[(self.fir_pos as usize + 1 + k) & 7] as i32 * self.t_ffc[k] as i32) >> 6;
        }
        sum = sum as i16 as i32;
        sum = clamp16(sum + ((b[self.fir_pos as usize] as i32 * self.t_ffc[7] as i32) >> 6));
        sum & !1
    }

    fn output(&mut self, side: usize, out: &mut Vec<i16>) {
        let fir = self.fir(side);
        self.fir_out[side] = fir;
        let (mvol, evol) = if side == 0 { (reg::MVOLL, reg::EVOLL) } else { (reg::MVOLR, reg::EVOLR) };
        let main = ((self.main[side] * self.regs[mvol] as i8 as i32) >> 7) as i16 as i32;
        let echo = ((fir * self.regs[evol] as i8 as i32) >> 7) as i16 as i32;
        let mut sample = clamp16(main + echo);
        if self.regs[reg::FLG] & 0x40 != 0 {
            sample = 0;
        }
        out.push(sample as i16);
        self.main[side] = 0;
        let feedback = ((fir * self.regs[reg::EFB] as i8 as i32) >> 7) as i16 as i32;
        self.echo[side] = clamp16(self.echo[side] + feedback);
    }

    fn echo_write(&mut self, side: usize, ram: &mut [u8], writable: bool) {
        if !self.t_echo_off && writable {
            let a = self.echo_ptr.wrapping_add(side as u16 * 2);
            let [lo, hi] = ((self.echo[side] & !1) as i16).to_le_bytes();
            ram[a as usize] = lo;
            ram[a.wrapping_add(1) as usize] = hi;
        }
        self.echo[side] = 0;
    }
}

/// fullsnes's interpolation (D-28): three products by the table shifted by 10 and summed in 16 bits with no overflow
/// handling, the fourth added with saturation, the result halved to 15 bits.
fn interpolate(x: &Voice) -> i32 {
    let i = ((x.pos >> 4) & 0xFF) as usize;
    let base = x.write as usize + (x.pos >> 12) as usize;
    let s = |k: usize| x.buf[(base + k) % 12] as i32;
    let mut out = (GAUSS[0xFF - i] * s(0)) >> 10;
    out += (GAUSS[0x1FF - i] * s(1)) >> 10;
    out += (GAUSS[0x100 + i] * s(2)) >> 10;
    out = out as i16 as i32;
    out = clamp16(out + ((GAUSS[i] * s(3)) >> 10));
    out >> 1
}

/// Four samples of BRR from the block's next two data bytes, the first loaded at S3b (fullsnes and anomie, "BRR").
fn decode(x: &mut Voice, ram: &[u8]) {
    let second = ram[x.brr_addr.wrapping_add(x.brr_offset as u16 + 1) as usize];
    let nibbles = [x.data >> 4, x.data & 15, second >> 4, second & 15];
    let shift = x.header >> 4;
    let filter = (x.header >> 2) & 3;
    for (k, &n) in nibbles.iter().enumerate() {
        let n = ((n as i8) << 4 >> 4) as i32;
        let mut s = if shift <= 12 { (n << shift) >> 1 } else if n < 0 { -2048 } else { 0 };
        let at = x.write as usize + k;
        let p1 = x.buf[(at + 11) % 12] as i32;
        let p2 = x.buf[(at + 10) % 12] as i32;
        s += match filter {
            0 => 0,
            1 => p1 + ((-p1) >> 4),
            2 => (p1 << 1) + ((-((p1 << 1) + p1)) >> 5) - p2 + (p2 >> 4),
            _ => (p1 << 1) + ((-(p1 + (p1 << 2) + (p1 << 3))) >> 6) - p2 + (((p2 << 1) + p2) >> 4),
        };
        x.buf[at % 12] = clip15(clamp16(s)) as i16;
    }
    x.write = (x.write + 4) % 12;
    x.brr_offset += 2;
    if x.brr_offset >= 9 {
        x.brr_offset = 1;
        if x.header & 1 != 0 {
            x.loop_pending = true;
            x.looped = true;
        } else {
            x.brr_addr = x.brr_addr.wrapping_add(9);
        }
    }
}

/// anomie's envelope: a new value every sample from the mode and the setting, applied when the counter fires for its
/// rate; the phase changes and the bent increase's memory follow the new value whether applied or not.
fn run_envelope(x: &mut Voice, setting: u8, fires: impl Fn(u8) -> bool) {
    if x.mode == Mode::Release {
        x.env = (x.env - 8).max(0);
        return;
    }
    let e = x.env;
    let exp = |e: i32| e - (((e - 1) >> 8) + 1);
    let (new, rate) = if x.adsr1 & 0x80 != 0 {
        match x.mode {
            Mode::Attack => {
                let a = x.adsr1 & 0x0F;
                if a == 15 { (e + 1024, 31) } else { (e + 32, a * 2 + 1) }
            }
            Mode::Decay => (exp(e), ((x.adsr1 >> 4) & 7) * 2 + 16),
            _ => (exp(e), setting & 0x1F),
        }
    } else if setting & 0x80 == 0 {
        (((setting & 0x7F) as i32) << 4, 31)
    } else {
        let rate = setting & 0x1F;
        match (setting >> 5) & 3 {
            0 => (e - 32, rate),
            1 => (exp(e), rate),
            2 => (e + 32, rate),
            _ => (e + if (x.hidden_env as u32) < 0x600 { 32 } else { 8 }, rate),
        }
    };
    if x.mode == Mode::Decay && (new >> 8) & 7 == (setting >> 5) as i32 {
        x.mode = Mode::Sustain;
    }
    x.hidden_env = new;
    if !(0..=0x7FF).contains(&new) && x.mode == Mode::Attack {
        x.mode = Mode::Decay;
    }
    if fires(rate) {
        x.env = new.clamp(0, 0x7FF);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // fullsnes and anomie: with shift 12 and filter 0, a nibble n decodes to (n << 12) >> 1.
    #[test]
    fn a_brr_group_decodes_by_its_shift() {
        let mut ram = vec![0u8; 0x10000];
        ram[0x200..0x203].copy_from_slice(&[0xC0, 0x7F, 0x80]);
        let mut x = Voice { brr_addr: 0x200, brr_offset: 1, ..Voice::default() };
        x.header = ram[0x200];
        x.data = ram[0x201];
        decode(&mut x, &ram);
        assert_eq!(&x.buf[..4], &[0x3800, -0x800, -0x4000, 0]);
        assert_eq!((x.write, x.brr_offset), (4, 3));
    }

    // A voice keyed on with direct gain sounds after its five silent samples, and an end-and-loop block sets ENDX.
    #[test]
    fn a_keyed_voice_sounds_and_ends_its_block() {
        let mut ram = vec![0u8; 0x10000];
        ram[0x100..0x104].copy_from_slice(&[0x00, 0x02, 0x00, 0x02]);
        ram[0x200] = 0xC3;
        ram[0x201..0x209].fill(0x77);
        let mut d = Dsp::default();
        for (r, v) in [(reg::DIR, 1), (0x00, 0x7F), (0x01, 0x7F), (0x03, 0x10), (0x07, 0x7F), (reg::MVOLL, 0x7F), (reg::MVOLR, 0x7F), (reg::FLG, 0x20)] {
            d.write(r as u8, v);
        }
        d.write(reg::KON as u8, 1);
        let mut out = Vec::new();
        for _ in 0..32 * 40 {
            d.step(&mut ram, true, &mut out);
        }
        assert!(out.iter().any(|&s| s != 0), "silent");
        assert_eq!(d.read(reg::ENDX as u8) & 1, 1);
        assert_eq!(d.read(0x08), 0x7F);
    }
}
