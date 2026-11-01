//! The core's memory as EmuSen's spaces: from its memory maps by `libretro.h`'s mapping rule, else from
//! `retro_get_memory_data`; read and written between frames, where EmuSen's cheat engine and debugger reach it
//! (EmuSen_LibretroHost.md §4.9).

use emusen_libretro_sys as sys;

/// One descriptor of a space, as `libretro.h` gives it.
#[derive(Clone, Copy, Debug)]
pub struct Region {
    pub ptr: usize,
    pub offset: usize,
    pub start: u64,
    pub select: u64,
    pub disconnect: u64,
    pub len: u64,
    pub read_only: bool,
}

impl Region {
    /// The byte of the host's memory for console address `a`, if this region claims it.
    pub fn host(&self, a: u64) -> Option<usize> {
        if self.ptr == 0 {
            return None;
        }
        let rel = if self.select == 0 {
            if a < self.start || a - self.start >= self.len {
                return None;
            }
            a - self.start
        } else {
            if a & self.select != self.start & self.select {
                return None;
            }
            let mut rel = (a & !self.disconnect).wrapping_sub(self.start & !self.disconnect);
            while self.len != 0 && rel >= self.len {
                rel &= !(1u64 << (63 - rel.leading_zeros()));
            }
            rel
        };
        Some(self.ptr + self.offset + rel as usize)
    }

    /// One past the last console address the region can claim, within 32 bits.
    fn end(&self) -> u64 {
        let span = if self.select == 0 { self.len } else { (!self.select & 0xFFFF_FFFF) + 1 };
        (self.start & 0xFFFF_FFFF).saturating_add(span).min(1 << 32)
    }
}

#[derive(Clone, Debug)]
pub struct Space {
    pub name: String,
    pub size: u64,
    pub big_endian: bool,
    pub regions: Vec<Region>,
}

impl Space {
    fn byte_at(&self, a: u64) -> Option<(usize, bool)> {
        self.regions.iter().find_map(|r| r.host(a).map(|h| (h, r.read_only)))
    }

    /// Bytes from `address`; past every region's end, zero.
    pub fn read(&self, address: u32, out: &mut [u8]) {
        for (i, b) in out.iter_mut().enumerate() {
            *b = self.byte_at(address as u64 + i as u64).map_or(0, |(h, _)| unsafe { *(h as *const u8) });
        }
    }

    /// Bytes to `address`; false when a byte falls in a read-only region, which is left as it was.
    pub fn write(&self, address: u32, data: &[u8]) -> bool {
        let mut all = true;
        for (i, &b) in data.iter().enumerate() {
            match self.byte_at(address as u64 + i as u64) {
                Some((h, false)) => unsafe { *(h as *mut u8) = b },
                Some((_, true)) => all = false,
                None => {}
            }
        }
        all
    }
}

/// Spaces from `SET_MEMORY_MAPS`, one per address-space name in the order first named, a blank name as `BUS`.
pub unsafe fn from_maps(map: *const sys::retro_memory_map) -> Vec<Space> {
    let mut spaces: Vec<Space> = Vec::new();
    let Some(map) = (unsafe { map.as_ref() }) else { return spaces };
    for i in 0..map.num_descriptors as usize {
        let d = unsafe { &*map.descriptors.add(i) };
        let name = crate::options::text(d.addrspace);
        let name = if name.is_empty() { "BUS".to_owned() } else { name };
        let region = Region {
            ptr: d.ptr as usize,
            offset: d.offset,
            start: d.start as u64,
            select: d.select as u64,
            disconnect: d.disconnect as u64,
            len: d.len as u64,
            read_only: d.flags & sys::MEMDESC_CONST != 0,
        };
        match spaces.iter_mut().find(|s| s.name == name) {
            Some(s) => {
                s.size = s.size.max(region.end());
                s.regions.push(region);
            }
            None => spaces.push(Space { name, size: region.end(), big_endian: d.flags & sys::MEMDESC_BIGENDIAN != 0, regions: vec![region] }),
        }
    }
    spaces
}

/// A space over one block `retro_get_memory_data` gives.
pub fn block(name: &str, ptr: *mut u8, len: usize) -> Option<Space> {
    (!ptr.is_null() && len > 0).then(|| Space {
        name: name.to_owned(),
        size: len as u64,
        big_endian: false,
        regions: vec![Region { ptr: ptr as usize, offset: 0, start: 0, select: 0, disconnect: 0, len: len as u64, read_only: false }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_region_maps_each_byte_once() {
        let mut ram = [0u8; 16];
        let r = Region { ptr: ram.as_mut_ptr() as usize, offset: 0, start: 0x100, select: 0, disconnect: 0, len: 16, read_only: false };
        assert_eq!(r.host(0x100), Some(ram.as_ptr() as usize));
        assert_eq!(r.host(0x10F), Some(ram.as_ptr() as usize + 15));
        assert_eq!(r.host(0x110), None);
        assert_eq!(r.host(0xFF), None);
    }

    #[test]
    fn a_selected_region_mirrors_and_drops_high_bits_past_its_length() {
        let mut ram = [0u8; 0x2000];
        let base = ram.as_mut_ptr() as usize;
        let r = Region { ptr: base, offset: 0, start: 0x7E_0000, select: 0xFF_E000, disconnect: 0, len: 0x2000, read_only: false };
        assert_eq!(r.host(0x7E_0010), Some(base + 0x10));
        assert_eq!(r.host(0x7F_0010), None);
        let mirrored = Region { ptr: base, offset: 0, start: 0, select: 0xE000, disconnect: 0, len: 0x800, read_only: false };
        assert_eq!(mirrored.host(0x0801), Some(base + 1));
        assert_eq!(mirrored.host(0x1FFF), Some(base + 0x7FF));
    }

    #[test]
    fn writes_skip_read_only_bytes_and_say_so() {
        let mut rom = [7u8; 4];
        let mut ram = [0u8; 4];
        let s = Space {
            name: "BUS".into(),
            size: 8,
            big_endian: false,
            regions: vec![
                Region { ptr: rom.as_mut_ptr() as usize, offset: 0, start: 0, select: 0, disconnect: 0, len: 4, read_only: true },
                Region { ptr: ram.as_mut_ptr() as usize, offset: 0, start: 4, select: 0, disconnect: 0, len: 4, read_only: false },
            ],
        };
        assert!(!s.write(2, &[1, 2, 3]));
        assert_eq!((rom, ram), ([7, 7, 7, 7], [3, 0, 0, 0]));
        let mut out = [0u8; 10];
        s.read(0, &mut out);
        assert_eq!(out, [7, 7, 7, 7, 3, 0, 0, 0, 0, 0]);
    }
}
