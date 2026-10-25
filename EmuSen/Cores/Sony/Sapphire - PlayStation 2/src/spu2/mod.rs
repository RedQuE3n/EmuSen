//! The SPU2 (stage S26, Sapphire_Native.md §32): two cores of 24 voices each in 2 MiB of sound RAM, every voice of both
//! cores each sample from `blackmoon-spu`, each core's mix and effects area, AutoDMA's PCM input, core 0's output into
//! core 1, the IRQ addresses, IOP DMA channels 4 and 7, and the output at 48,000 Hz exactly, one sample every 768 IOP
//! cycles. It is brought to the present at each access, each DMA word and the bus's events, sample by sample.

pub mod adma;
pub mod cores;
pub mod regs;

use emusen_native::samples::SampleQueue;

use crate::iop::dma;

use cores::Core;

/// The registers' window: 1F900000h to 1F9007FFh [R ps2tek "Memory Map", "SPU2 registers", and S21's decode].
pub const BASE: u32 = 0x1F90_0000;
pub const END: u32 = 0x1F90_0800;
/// Sound RAM [R ps2tek "Memory Map": "2 MB SPU2 work RAM"].
pub const RAM_BYTES: usize = 2 << 20;
/// The halfword index of 7C2h, which shows each core's IRQ flag: core 0's in bit 2, core 1's in bit 3 (D-265).
const IRQ_INFO: usize = 0x7C2 / 2;
/// IOP cycles a sample: 36.864 MHz over 48,000 Hz [A, Sapphire_Disputes.md D-0's arithmetic].
pub const CYCLES_PER_SAMPLE: u64 = 768;

/// What an access to sound RAM meets: both cores' IRQ addresses, as halfword addresses, armed or not, hit or not; an
/// access by either core is compared with both (Sapphire_Disputes.md D-265).
#[derive(Clone, Copy, Debug, Default)]
pub struct Watch {
    pub at: [u32; 2],
    pub armed: [bool; 2],
    pub hit: [bool; 2],
}

impl blackmoon_spu::Touch for Watch {
    #[inline]
    fn touch(&mut self, address: u32) {
        let hw = address >> 1;
        for c in 0..2 {
            if self.armed[c] && hw == self.at[c] {
                self.hit[c] = true;
            }
        }
    }
}

#[derive(Clone)]
pub struct Spu2 {
    pub ram: Vec<u8>,
    pub cores: [Core; 2],
    /// The halfwords outside the cores' blocks and volumes (770h-7FFh's rest), as written.
    pub regs: [u16; 0x400],
    /// The IOP cycle at which the next sample is made.
    pub next_tick: u64,
    /// The samples made and not yet taken by the host; not machine state.
    pub out: SampleQueue,
    /// AutoDMA freed a buffer, so a channel held for want of room should look again.
    wake: bool,
    /// The output areas' next halfword, 0..1FFh.
    pub area: u16,
}

impl Default for Spu2 {
    fn default() -> Spu2 {
        Spu2::new()
    }
}

/// Where a register offset lands.
enum Place {
    Core(usize, u32),
    Volume(usize, usize),
    Other(usize),
}

fn place(offset: u32) -> Place {
    let o = offset & 0x7FE;
    if (regs::VOLUMES..regs::VOLUMES_END).contains(&o) {
        let k = o - regs::VOLUMES;
        return Place::Volume((k / regs::VOLUMES_STRIDE) as usize, (k % regs::VOLUMES_STRIDE / 2) as usize);
    }
    let (c, r) = ((o / regs::CORE_STRIDE) as usize, o % regs::CORE_STRIDE);
    if r < 0x346 { Place::Core(c, r) } else { Place::Other(o as usize / 2) }
}

impl Spu2 {
    pub fn new() -> Spu2 {
        Spu2 {
            ram: vec![0; RAM_BYTES],
            cores: [Core::new(0), Core::new(1)],
            regs: [0; 0x400],
            next_tick: CYCLES_PER_SAMPLE,
            out: SampleQueue::with_capacity(emusen_native::samples::DEFAULT_LIMIT),
            wake: false,
            area: 0,
        }
    }

    fn watch(&self) -> Watch {
        Watch { at: [self.cores[0].irq_address(), self.cores[1].irq_address()], armed: [self.cores[0].armed(), self.cores[1].armed()], hit: [false; 2] }
    }

    /// Sets the flags of the cores whose address was hit while armed; whether an IRQ rose.
    fn hit(&mut self, w: Watch) -> bool {
        let mut rose = false;
        for c in 0..2 {
            if w.hit[c] && self.cores[c].armed() {
                self.cores[c].irq = true;
                rose = true;
            }
        }
        rose
    }

    /// A halfword register's value; the SPU2 is brought to the present first by the bus.
    pub fn read16(&self, offset: u32) -> u16 {
        match place(offset) {
            Place::Core(c, r) => self.cores[c].read(r),
            Place::Volume(c, i) => self.cores[c].vol[i],
            Place::Other(IRQ_INFO) => (self.cores[0].irq as u16) << 2 | (self.cores[1].irq as u16) << 3,
            Place::Other(i) => self.regs[i],
        }
    }

    /// A halfword register written; whether an IRQ rose (a transfer's access may hit an IRQ address).
    pub fn write16(&mut self, offset: u32, value: u16) -> bool {
        match place(offset) {
            Place::Core(c, regs::DATA) => {
                let mut w = self.watch();
                self.store(c, value, &mut w);
                return self.hit(w);
            }
            Place::Core(c, r) => self.cores[c].write(r, value),
            Place::Volume(c, i) => {
                if i != regs::vol::MVOLX && i != regs::vol::MVOLX + 1 {
                    self.cores[c].vol[i] = value;
                }
            }
            Place::Other(i) => self.regs[i] = value,
        }
        false
    }

    /// A halfword of the transfer at core `c`'s address, which then advances [C: no FIFO is modelled, D-263].
    fn store(&mut self, c: usize, value: u16, w: &mut Watch) {
        let a = (self.cores[c].tsa as usize * 2) & (RAM_BYTES - 2);
        blackmoon_spu::Touch::touch(w, a as u32);
        self.ram[a..a + 2].copy_from_slice(&value.to_le_bytes());
        self.cores[c].tsa = (self.cores[c].tsa + 1) & cores::ADDRESS_MASK;
    }

    fn load(&mut self, c: usize, w: &mut Watch) -> u16 {
        let a = (self.cores[c].tsa as usize * 2) & (RAM_BYTES - 2);
        blackmoon_spu::Touch::touch(w, a as u32);
        self.cores[c].tsa = (self.cores[c].tsa + 1) & cores::ADDRESS_MASK;
        u16::from_le_bytes([self.ram[a], self.ram[a + 1]])
    }

    // --- the DMA channels: core 0 on 4, core 1 on 7 (ps2tek "IOP DMA") ---

    /// The words core `c` takes or gives now: in AutoDMA the rest of a free buffer's block, else any [C].
    pub fn dma_room(&self, c: usize, want: u32, to_memory: bool) -> u32 {
        if !to_memory && self.cores[c].adma_on() { want.min(self.cores[c].adma.room()) } else { want }
    }

    /// A word from IOP memory, low halfword first: into MEMIN in AutoDMA, else at the transfer address; whether an IRQ
    /// rose.
    pub fn dma_put(&mut self, c: usize, word: u32) -> bool {
        let mut w = self.watch();
        if self.cores[c].adma_on() {
            let at = self.cores[c].adma.next_word(c) as usize * 2;
            for (k, h) in [word as u16, (word >> 16) as u16].into_iter().enumerate() {
                blackmoon_spu::Touch::touch(&mut w, (at + 2 * k) as u32);
                self.ram[at + 2 * k..at + 2 * k + 2].copy_from_slice(&h.to_le_bytes());
            }
        } else {
            self.store(c, word as u16, &mut w);
            self.store(c, (word >> 16) as u16, &mut w);
        }
        self.hit(w)
    }

    /// A word for IOP memory from the transfer address, low halfword first.
    pub fn dma_get(&mut self, c: usize) -> (u32, bool) {
        let mut w = self.watch();
        let lo = self.load(c, &mut w) as u32;
        let hi = self.load(c, &mut w) as u32;
        (lo | hi << 16, self.hit(w))
    }

    // --- time ---

    /// The next IOP cycle at which the SPU2 must run unasked: each sample while a core's IRQ is armed or its AutoDMA
    /// may free a buffer, else never.
    pub fn next_event(&self) -> u64 {
        if self.cores.iter().any(|c| c.armed() || c.adma_on()) { self.next_tick } else { u64::MAX }
    }

    /// Brings the SPU2 to IOP cycle `now`, sample by sample; whether an IRQ rose.
    pub fn run(&mut self, now: u64) -> bool {
        let mut rose = false;
        while self.next_tick <= now {
            rose |= self.tick();
            self.next_tick += CYCLES_PER_SAMPLE;
        }
        rose
    }

    /// One sample: core 0, then core 1 with core 0's output as its external input; core 1's output is the SPU2's.
    fn tick(&mut self) -> bool {
        let mut w = self.watch();
        let Spu2 { ram, cores, .. } = self;
        let [c0, c1] = cores;
        let first = c0.tick(ram, &mut w, [0, 0]);
        let out = c1.tick(ram, &mut w, first);
        // The output areas (D-261), each write watched for the IRQ addresses (D-265).
        let i = self.area as usize;
        let mut put = |at: usize, v: i32| {
            ram[2 * (at + i)..2 * (at + i) + 2].copy_from_slice(&(v as i16).to_le_bytes());
            blackmoon_spu::Touch::touch(&mut w, 2 * (at + i) as u32);
        };
        for (c, core) in [&*c0, &*c1].into_iter().enumerate() {
            put(0x400 + 0x800 * c, core.voices[1].out as i32);
            put(0x600 + 0x800 * c, core.voices[3].out as i32);
            for (k, &x) in core.sums.iter().enumerate() {
                put(0x1000 + 0x800 * c + 0x200 * k, x);
            }
        }
        put(0x800, first[0]);
        put(0xA00, first[1]);
        self.area = (self.area + 1) & 0x1FF;
        self.out.push_pair(out[0] as i16, out[1] as i16);
        self.wake |= self.cores.iter().any(|c| c.adma_on() && c.adma.play.is_multiple_of(adma::BUFFER));
        self.hit(w)
    }

    /// Whether AutoDMA freed a buffer since the last call.
    pub fn take_wake(&mut self) -> bool {
        core::mem::take(&mut self.wake)
    }
}

/// The IOP DMAC's peripherals: the SPU2's cores on channels 4 and 7, the others through `others`; `irq` records
/// whether a transfer's access raised the SPU2's IRQ.
pub struct Ports<'a, P: dma::Peripherals> {
    pub others: &'a mut P,
    pub spu2: &'a mut Spu2,
    pub irq: bool,
}

fn core_of(ch: usize) -> Option<usize> {
    match ch {
        dma::SPU => Some(0),
        dma::SPU2 => Some(1),
        _ => None,
    }
}

impl<P: dma::Peripherals> dma::Peripherals for Ports<'_, P> {
    fn request(&mut self, ch: usize, want: u32, to_memory: bool) -> u32 {
        match core_of(ch) {
            Some(c) => self.spu2.dma_room(c, want, to_memory),
            None => self.others.request(ch, want, to_memory),
        }
    }

    fn put(&mut self, ch: usize, w: u32) {
        match core_of(ch) {
            Some(c) => self.irq |= self.spu2.dma_put(c, w),
            None => self.others.put(ch, w),
        }
    }

    fn get(&mut self, ch: usize) -> u32 {
        match core_of(ch) {
            Some(c) => {
                let (w, rose) = self.spu2.dma_get(c);
                self.irq |= rose;
                w
            }
            None => self.others.get(ch),
        }
    }
}
