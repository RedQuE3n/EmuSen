//! How the EE core reaches memory (EE Core User's Manual chapters 5 and 6): the operating mode's segments, the TLB,
//! the scratchpad, the data cache with write-back and write-allocate, the instruction cache, the uncached accelerated
//! buffer, the bus with its errors, and the CACHE instruction. Each access charges its wait to the instruction's
//! timing and counts its events. Sapphire_Native.md §17.3 and §17.5 have the readings.

use super::cache::{DCache, ICache};
use super::cop0::{self, config, event};
use super::timing::{REFILL, UCAB_REFILL, UNCACHED, WRITEBACK};
use super::tlb::Target;
use super::{Bus, Ee, Exception, status};

/// What an access does with the caches, from the page's C field or Config.K0 (§5.2.6; §3.2, "Config Register").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Uncached,
    /// Cached, write-back with write allocate (C or K0 3).
    Cached,
    /// K0 0, "cached without writeback and write allocate": stores write through and do not allocate.
    Through,
    Accelerated,
}

/// Where a virtual address lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Scratchpad(u32),
    Physical(u32, Mode),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Access {
    Fetch,
    Load,
    Store,
}

fn tlb_mode(c: u32) -> Mode {
    match c {
        3 => Mode::Cached,
        7 => Mode::Accelerated,
        _ => Mode::Uncached,
    }
}

fn k0_mode(k0: u32) -> Mode {
    match k0 {
        0 => Mode::Through,
        3 => Mode::Cached,
        7 => Mode::Accelerated,
        _ => Mode::Uncached,
    }
}

fn bytes_of(line: &[u8], at: usize, n: usize) -> u128 {
    let mut v = [0u8; 16];
    v[..n].copy_from_slice(&line[at..at + n]);
    u128::from_le_bytes(v)
}

fn put_bytes(line: &mut [u8], at: usize, n: usize, value: u128) {
    line[at..at + n].copy_from_slice(&value.to_le_bytes()[..n]);
}

impl Ee {
    /// The address error a mode's access raises (§4.2.9), kseg0's and kseg1's fixed windows, kuseg unmapped at level 2,
    /// and the TLB for the rest (§5.1.3 to §5.2.3).
    pub(crate) fn translate(&mut self, vaddr: u32, access: Access) -> Result<Place, Exception> {
        let store = access == Access::Store;
        let allowed = self.kernel() || vaddr < 0x8000_0000 || (self.supervisor() && (0xC000_0000..0xE000_0000).contains(&vaddr));
        if !allowed {
            return Err(if store { Exception::AddressStore(vaddr) } else { Exception::AddressLoad(vaddr) });
        }
        match vaddr >> 29 {
            4 => return Ok(Place::Physical(vaddr - 0x8000_0000, k0_mode(self.cop0[cop0::CONFIG] & config::K0))),
            5 => return Ok(Place::Physical(vaddr - 0xA000_0000, Mode::Uncached)),
            0..=3 if self.status() & status::ERL != 0 => return Ok(Place::Physical(vaddr, Mode::Uncached)),
            _ => {}
        }
        let Some(hit) = self.tlb.lookup(vaddr, self.cop0[cop0::ENTRYHI] & 0xFF) else {
            self.event(event::TLB_MISS, 1);
            return Err(Exception::TlbRefill { vaddr, store });
        };
        if !hit.valid {
            return Err(Exception::TlbInvalid { vaddr, store });
        }
        if store && !hit.dirty {
            return Err(Exception::TlbModified(vaddr));
        }
        Ok(match hit.target {
            Target::Scratchpad(o) => Place::Scratchpad(o),
            Target::Physical { paddr, c } => Place::Physical(paddr, tlb_mode(c)),
        })
    }

    /// Config.DCE clear makes a cached page's data accesses uncached (§3.2).
    pub(super) fn data_mode(&self, m: Mode) -> Mode {
        match m {
            Mode::Cached | Mode::Through if self.cop0[cop0::CONFIG] & config::DCE == 0 => Mode::Uncached,
            m => m,
        }
    }

    /// A bus error at `paddr` (EE User's Manual §2.1.2): BadPAddr and BEM set and the exception, unless BEM masks it,
    /// when the access goes on unseen (true).
    fn bus_error(&mut self, bus: &mut dyn Bus, paddr: u32, fetch: bool) -> Result<bool, Exception> {
        if !bus.error(paddr) {
            return Ok(false);
        }
        if self.status() & status::BEM != 0 {
            return Ok(true);
        }
        self.cop0[cop0::BADPADDR] = paddr & !0xF;
        self.cop0[cop0::STATUS] |= status::BEM;
        Err(if fetch { Exception::BusInstruction } else { Exception::BusData })
    }

    /// An instruction fetch (§6.3.3): the instruction cache for a cached page with Config.ICE, the bus otherwise.
    pub(crate) fn fetch(&mut self, bus: &mut dyn Bus, vaddr: u32) -> Result<u32, Exception> {
        match self.translate(vaddr, Access::Fetch)? {
            Place::Scratchpad(o) => Ok(bytes_of(&self.spram[..], o as usize, 4) as u32),
            Place::Physical(paddr, Mode::Cached | Mode::Through) if self.cop0[cop0::CONFIG] & config::ICE != 0 => {
                let set = ICache::set(vaddr);
                let way = match self.icache.hit(set, paddr) {
                    Some(w) => w,
                    None => {
                        self.bus_error(bus, paddr & !63, true)?;
                        self.event(event::ICACHE_MISS, 1);
                        self.stall_fetch += REFILL;
                        self.ifill(bus, set, self.icache.victim(set), paddr)
                    }
                };
                Ok(self.icache.data[set][way][(paddr >> 2) as usize & 15])
            }
            Place::Physical(paddr, _) => {
                self.stall_fetch += UNCACHED;
                if self.bus_error(bus, paddr, true)? { Ok(0) } else { Ok(bus.read(paddr, 4) as u32) }
            }
        }
    }

    /// The word a fetch at `vaddr` would read, with no effect on any state; none where the fetch would fault.
    pub(crate) fn peek_fetch(&self, bus: &mut dyn Bus, vaddr: u32) -> Option<u32> {
        let paddr = match vaddr >> 29 {
            4 => vaddr - 0x8000_0000,
            5 => vaddr - 0xA000_0000,
            _ => match self.tlb.lookup(vaddr, self.cop0[cop0::ENTRYHI] & 0xFF)?.target {
                Target::Scratchpad(o) => return Some(bytes_of(&self.spram[..], o as usize, 4) as u32),
                Target::Physical { paddr, .. } => paddr,
            },
        };
        let set = ICache::set(vaddr);
        Some(match self.icache.hit(set, paddr) {
            Some(w) => self.icache.data[set][w][(paddr >> 2) as usize & 15],
            None => bus.read(paddr, 4) as u32,
        })
    }

    /// Refills an instruction cache line into `way`, flipping its LRF bit (Table 6-4).
    fn ifill(&mut self, bus: &mut dyn Bus, set: usize, way: usize, paddr: u32) -> usize {
        let base = paddr & !63;
        for q in 0..4u32 {
            let v = bus.read(base + 16 * q, 16);
            for k in 0..4 {
                self.icache.data[set][way][(4 * q + k) as usize] = (v >> (32 * k)) as u32;
            }
        }
        self.icache.side[set][way] = [0; 16];
        self.fills += 1;
        let t = &mut self.icache.tags[set][way];
        *t = super::cache::ITag { pfn: paddr >> 12, valid: true, lrf: !t.lrf };
        way
    }

    /// A data cache miss (§6.3.3, §6.3.4): the victim written back if dirty, then the line refilled; its way.
    fn dmiss(&mut self, bus: &mut dyn Bus, set: usize, paddr: u32, load: bool) -> Result<usize, Exception> {
        self.bus_error(bus, paddr & !63, false)?;
        let way = self.dcache.victim(set);
        self.write_back(bus, set, way);
        let base = paddr & !63;
        for q in 0..4usize {
            let v = bus.read(base + 16 * q as u32, 16);
            put_bytes(&mut self.dcache.data[set][way], 16 * q, 16, v);
        }
        let t = &mut self.dcache.tags[set][way];
        *t = super::cache::DTag { pfn: paddr >> 12, dirty: false, valid: true, lrf: !t.lrf, lock: false };
        self.stall_data += REFILL;
        self.fills += 1;
        self.event(event::DCACHE_MISS, 1);
        if load {
            self.event(event::NONBLOCKING_LOAD, 1);
        }
        Ok(way)
    }

    /// Writes a valid dirty line back to memory and marks it clean.
    fn write_back(&mut self, bus: &mut dyn Bus, set: usize, way: usize) {
        let t = self.dcache.tags[set][way];
        if t.valid && t.dirty {
            let base = self.dcache.line_address(set, way);
            for q in 0..4usize {
                bus.write(base + 16 * q as u32, 16, bytes_of(&self.dcache.data[set][way], 16 * q, 16));
            }
            self.dcache.tags[set][way].dirty = false;
            self.stall_data += WRITEBACK;
        }
    }

    /// A naturally aligned load of `bytes` at `vaddr`.
    pub(crate) fn read(&mut self, bus: &mut dyn Bus, vaddr: u32, bytes: usize) -> Result<u128, Exception> {
        let paddr = match self.translate(vaddr, Access::Load)? {
            Place::Scratchpad(o) => return Ok(bytes_of(&self.spram[..], o as usize, bytes)),
            Place::Physical(p, m) => match self.data_mode(m) {
                Mode::Cached | Mode::Through => {
                    self.event(event::DTLB_ACCESS, 1);
                    let set = DCache::set(vaddr);
                    let way = match self.dcache.hit(set, p) {
                        Some(w) => w,
                        None => self.dmiss(bus, set, p, true)?,
                    };
                    return Ok(bytes_of(&self.dcache.data[set][way], (p & 63) as usize, bytes));
                }
                Mode::Accelerated => {
                    if !(self.ucab.valid && self.ucab.tag == p >> 7) {
                        self.bus_error(bus, p & !127, false)?;
                        let base = p & !127;
                        for q in 0..8usize {
                            let v = bus.read(base + 16 * q as u32, 16);
                            put_bytes(&mut self.ucab.data, 16 * q, 16, v);
                        }
                        self.ucab.valid = true;
                        self.ucab.tag = p >> 7;
                        self.stall_data += UCAB_REFILL;
                        self.fills += 1;
                        self.event(event::DCACHE_MISS, 1);
                        self.event(event::NONBLOCKING_LOAD, 1);
                    }
                    return Ok(bytes_of(&self.ucab.data, (p & 127) as usize, bytes));
                }
                Mode::Uncached => p,
            },
        };
        self.stall_data += UNCACHED;
        self.event(event::DCACHE_MISS, 1);
        self.event(event::NONBLOCKING_LOAD, 1);
        if self.bus_error(bus, paddr, false)? { Ok(0) } else { Ok(bus.read(paddr, bytes)) }
    }

    /// A naturally aligned store; any store disables the uncached accelerated buffer (§6.5.1).
    pub(crate) fn write(&mut self, bus: &mut dyn Bus, vaddr: u32, bytes: usize, value: u128) -> Result<(), Exception> {
        self.write_merge(bus, vaddr, bytes, value, 0)
    }

    /// A store of `value` into the aligned unit of `bytes` at `vaddr`, keeping the memory's bits under `keep`
    /// (SWL, SWR, SDL, SDR); write-allocate and write-back on a cached page (§5.2.6), a locked line kept clean (§6.3.7).
    pub(crate) fn write_merge(&mut self, bus: &mut dyn Bus, vaddr: u32, bytes: usize, value: u128, keep: u128) -> Result<(), Exception> {
        let place = self.translate(vaddr, Access::Store)?;
        self.ucab.valid = false;
        let merge = |old: u128| old & keep | value & !keep;
        match place {
            Place::Scratchpad(o) => {
                let o = o as usize;
                let v = merge(bytes_of(&self.spram[..], o, bytes));
                put_bytes(&mut self.spram[..], o, bytes, v);
            }
            Place::Physical(p, m) => match self.data_mode(m) {
                Mode::Cached => {
                    self.event(event::DTLB_ACCESS, 1);
                    let set = DCache::set(vaddr);
                    let way = match self.dcache.hit(set, p) {
                        Some(w) => w,
                        None => self.dmiss(bus, set, p, false)?,
                    };
                    let at = (p & 63) as usize;
                    let v = merge(bytes_of(&self.dcache.data[set][way], at, bytes));
                    put_bytes(&mut self.dcache.data[set][way], at, bytes, v);
                    let t = &mut self.dcache.tags[set][way];
                    if !t.lock {
                        t.dirty = true;
                    }
                }
                Mode::Through => {
                    self.event(event::DTLB_ACCESS, 1);
                    let set = DCache::set(vaddr);
                    if self.bus_error(bus, p, false)? {
                        return Ok(());
                    }
                    let v = merge(match self.dcache.hit(set, p) {
                        Some(w) => bytes_of(&self.dcache.data[set][w], (p & 63) as usize, bytes),
                        None => bus.read(p, bytes),
                    });
                    if let Some(w) = self.dcache.hit(set, p) {
                        put_bytes(&mut self.dcache.data[set][w], (p & 63) as usize, bytes, v);
                    }
                    bus.write(p, bytes, v);
                }
                Mode::Uncached | Mode::Accelerated => {
                    if self.bus_error(bus, p, false)? {
                        return Ok(());
                    }
                    let v = if keep == 0 { value } else { merge(bus.read(p, bytes)) };
                    bus.write(p, bytes, v);
                }
            },
        }
        Ok(())
    }

    /// Every dirty data cache line written back, then both caches, the UCAB and the BTAC emptied, as a program's loader
    /// leaves them before it writes code into memory past the caches (the fast boot, Sapphire_Native.md §36.2).
    pub fn flush_caches(&mut self, bus: &mut dyn Bus) {
        let saved = self.stall_data;
        for set in 0..self.dcache.tags.len() {
            for way in 0..2 {
                self.write_back(bus, set, way);
                self.invalidate_d(set, way);
            }
        }
        self.stall_data = saved;
        for set in self.icache.tags.iter_mut() {
            for t in set.iter_mut() {
                t.valid = false;
            }
        }
        self.ucab.valid = false;
        self.btac.flush();
    }

    /// PREF: the line read into the data cache where the page is cached and the address translates; no exception
    /// and no wait (EE Core Instruction Set Manual, PREF).
    pub(crate) fn prefetch(&mut self, bus: &mut dyn Bus, vaddr: u32) {
        let saved = self.stall_data;
        if let Ok(Place::Physical(p, m)) = self.translate(vaddr, Access::Load)
            && self.data_mode(m) == Mode::Cached
            && !bus.error(p)
        {
            let set = DCache::set(vaddr);
            if self.dcache.hit(set, p).is_none() {
                let _ = self.dmiss(bus, set, p, false);
            }
        }
        self.stall_data = saved;
    }

    /// The bytes at `paddr` as the core sees them: a valid data cache line over memory (for tests and the debugger).
    pub fn peek(&self, bus: &mut dyn Bus, paddr: u32, bytes: usize) -> u128 {
        let set = DCache::set(paddr);
        match self.dcache.hit(set, paddr) {
            Some(w) => bytes_of(&self.dcache.data[set][w], (paddr & 63) as usize, bytes),
            None => bus.read(paddr, bytes),
        }
    }

    /// The physical address a hit operation names; the scratchpad and the unmapped windows give theirs.
    fn hit_address(&mut self, vaddr: u32) -> Result<Option<u32>, Exception> {
        Ok(match self.translate(vaddr, Access::Load)? {
            Place::Physical(p, _) => Some(p),
            Place::Scratchpad(_) => None,
        })
    }

    /// The CACHE instruction's operations by the rt field (EE Core Instruction Set Manual, CACHE, pages 292 to 311).
    pub(crate) fn cache_op(&mut self, bus: &mut dyn Bus, op: u32, vaddr: u32) -> Result<(), Exception> {
        let way = (vaddr & 1) as usize;
        let iset = ICache::set(vaddr);
        let dset = DCache::set(vaddr);
        let word = (vaddr >> 2) as usize & 15;
        let tag_lo = self.cop0[cop0::TAGLO];
        let tag_hi = self.cop0[cop0::TAGHI];
        let bit = |n: u32| tag_lo >> n & 1 == 1;
        match op {
            0x00 => {
                let t = self.icache.tags[iset][way];
                self.cop0[cop0::TAGLO] = t.pfn << 12 | u32::from(t.valid) << 5 | u32::from(t.lrf) << 4;
            }
            0x01 => {
                self.cop0[cop0::TAGLO] = self.icache.data[iset][way][word];
                self.cop0[cop0::TAGHI] = u32::from(self.icache.side[iset][way][word]);
            }
            0x02 => {
                let (lo, hi) = self.btac.entries[(vaddr & 63) as usize];
                self.cop0[cop0::TAGLO] = lo;
                self.cop0[cop0::TAGHI] = hi;
            }
            0x04 => {
                self.icache.tags[iset][way] = super::cache::ITag { pfn: tag_lo >> 12, valid: bit(5), lrf: bit(4) };
            }
            0x05 => {
                self.icache.data[iset][way][word] = tag_lo;
                self.icache.side[iset][way][word] = (tag_hi & 0x3F) as u8;
                self.btac.flush();
            }
            0x06 => self.btac.entries[(vaddr & 63) as usize] = (tag_lo & !6, tag_hi & !3),
            0x07 => self.icache.tags[iset][way].valid = false,
            0x0A => self.btac.invalidate(vaddr, 3),
            0x0B => {
                if let Some(p) = self.hit_address(vaddr)?
                    && let Some(w) = self.icache.hit(iset, p)
                {
                    self.icache.tags[iset][w].valid = false;
                }
                self.btac.invalidate(vaddr, 6);
            }
            0x0C => self.btac.flush(),
            0x0E => {
                if let Some(p) = self.hit_address(vaddr)? {
                    let t = self.icache.tags[iset];
                    let w = super::cache::lrf_way(t[0].lrf, t[1].lrf);
                    self.stall_data += REFILL;
                    self.ifill(bus, iset, w, p);
                }
            }
            0x10 => {
                let t = self.dcache.tags[dset][way];
                self.cop0[cop0::TAGLO] = t.pfn << 12
                    | u32::from(t.dirty) << 6
                    | u32::from(t.valid) << 5
                    | u32::from(t.lrf) << 4
                    | u32::from(t.lock) << 3;
            }
            0x11 => self.cop0[cop0::TAGLO] = bytes_of(&self.dcache.data[dset][way], word * 4, 4) as u32,
            0x12 => {
                self.hit_address(vaddr)?;
                self.dcache.tags[dset][way] =
                    super::cache::DTag { pfn: tag_lo >> 12, dirty: bit(6) && bit(5), valid: bit(5), lrf: bit(4), lock: bit(3) };
            }
            0x13 => put_bytes(&mut self.dcache.data[dset][way], word * 4, 4, u128::from(tag_lo)),
            0x14 => {
                self.write_back(bus, dset, way);
                self.invalidate_d(dset, way);
            }
            0x16 => self.invalidate_d(dset, way),
            0x18 | 0x1A | 0x1C => {
                let hit = match self.hit_address(vaddr)? {
                    Some(p) => self.dcache.hit(dset, p),
                    None => None,
                };
                if op != 0x1C {
                    let s = &mut self.cop0[cop0::STATUS];
                    *s = if hit.is_some() { *s | status::CH } else { *s & !status::CH };
                }
                if let Some(w) = hit {
                    if op != 0x1A {
                        self.write_back(bus, dset, w);
                    }
                    if op != 0x1C {
                        self.invalidate_d(dset, w);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Clears a data cache line's V, D and L, keeping its LRF bit.
    fn invalidate_d(&mut self, set: usize, way: usize) {
        let t = &mut self.dcache.tags[set][way];
        t.valid = false;
        t.dirty = false;
        t.lock = false;
    }
}

#[cfg(test)]
mod tests {
    use super::super::bus::Ram;
    use super::super::cache::LINE;
    use super::*;

    #[test]
    fn a_store_stays_in_the_data_cache_until_its_line_is_written_back() {
        let mut ram = Ram::new();
        let mut e = Ee::new(0x0010_0000);
        e.write(&mut ram, 0x0004_0000, 4, 0x1234_5678).unwrap();
        assert_eq!(ram.word(0x0004_0000), 0, "memory unchanged");
        assert_eq!(e.peek(&mut ram, 0x0004_0000, 4), 0x1234_5678);
        e.cache_op(&mut ram, 0x18, 0x0004_0000).unwrap();
        assert_eq!(ram.word(0x0004_0000), 0x1234_5678);
        assert_eq!(LINE, 64);
    }
}
