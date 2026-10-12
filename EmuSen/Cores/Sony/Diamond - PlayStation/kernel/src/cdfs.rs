//! Files on "cdrom:" from the ISO 9660 filesystem (ECMA-119 2nd edition, §8.4 the primary volume descriptor, §9.1
//! the directory record): paths resolved from the root or the directory `cd` set, names compared without case and,
//! when the pattern gives none, without the version; a direntry carries the identifier whole, its version included; the device functions A(5Fh)-A(64h) and CdGetLbn.
//! Diamond_OpenKernel.md §12.

use crate::cdrom;
use crate::file::{self, dcb, error, fcb};
use crate::mem::{rd, rd8, wr, wr8};
use crate::names;

const SECTOR: u32 = 0x800;
const SIZE: u32 = 0x20;
const START: u32 = 0x24;
const DIRECTORY: u32 = 2;

static mut MOUNTED: bool = false;
static mut ROOT: (u32, u32) = (0, 0);
static mut DISK_ID: u32 = 0;
static mut BUFFER: [u32; 0x200] = [0; 0x200];
static mut CACHED: u32 = u32::MAX;
static mut CWD: [u8; 64] = [0; 64];
static mut PATH: [u8; 96] = [0; 96];
static mut PART: [u8; 32] = [0; 32];
static mut NAME: [u8; 40] = [0; 40];
static mut PATTERN: [u8; 32] = [0; 32];
static mut SEARCH: (u32, u32, u32) = (0, 0, 0);
static mut INFO: [u32; 20] = [0; 20];

fn buffer() -> u32 {
    &raw const BUFFER as u32
}

fn le32(a: u32) -> u32 {
    rd8(a) | rd8(a + 1) << 8 | rd8(a + 2) << 16 | rd8(a + 3) << 24
}

/// A sector into the one-sector buffer, kept while the same sector is asked for again.
fn sector(lba: u32) -> bool {
    if unsafe { CACHED } == lba {
        return true;
    }
    unsafe { CACHED = u32::MAX };
    let ok = cdrom::read(lba, 1, buffer(), 0, SECTOR);
    if ok {
        unsafe { CACHED = lba };
    }
    ok
}

/// The disc forgotten: the next file call reads its volume descriptor again.
pub fn unmount() {
    unsafe {
        MOUNTED = false;
        CACHED = u32::MAX;
    }
}

/// The primary volume descriptor at sector 16: the root's extent, and the disk id, psx-spx's "checksum across path
/// table", here the sum of the path table's first sector's words.
fn mount() -> bool {
    if unsafe { MOUNTED } {
        return true;
    }
    if !sector(16) {
        return false;
    }
    let b = buffer();
    if rd8(b) != 1 || (0..5).any(|i| rd8(b + 1 + i) != b"CD001"[i as usize] as u32) {
        return false;
    }
    let root = (le32(b + 156 + 2), le32(b + 156 + 10));
    let table = le32(b + 140);
    if !sector(table) {
        return false;
    }
    let id = (0..0x200).fold(0u32, |a, i| a.wrapping_add(rd(buffer() + 4 * i)));
    unsafe {
        ROOT = root;
        DISK_ID = id;
        MOUNTED = true;
    }
    true
}

/// The record in directory (lba, size) whose identifier matches `pattern` from record index `from` on: its extent,
/// its size, its flags, and the index after it.
fn scan(dir: (u32, u32), pattern: u32, from: u32, star: bool) -> Option<(u32, u32, u32, u32)> {
    let versioned = names::contains(pattern, b';' as u32);
    let name = &raw mut NAME as u32;
    let mut index = 0;
    for s in 0..dir.1.div_ceil(SECTOR) {
        if !sector(dir.0 + s) {
            return None;
        }
        let mut at = 0;
        while at < SECTOR {
            let r = buffer() + at;
            let len = rd8(r);
            if len == 0 {
                break;
            }
            let n = rd8(r + 32);
            if n > 1 || rd8(r + 33) > 1 {
                if index >= from {
                    let mut k = 0;
                    while k < n.min(38) {
                        wr8(name + k, rd8(r + 33 + k));
                        k += 1;
                    }
                    wr8(name + k, 0);
                    if names::matches(pattern, name, if versioned { 0 } else { b';' as u32 }, star, true) {
                        return Some((le32(r + 2), le32(r + 10), rd8(r + 25), index + 1));
                    }
                }
                index += 1;
            }
            at += len;
        }
    }
    None
}

/// The absolute form of `path`: from the root when it starts with a backslash, else from the directory `cd` set.
fn absolute(path: u32) -> u32 {
    let out = &raw mut PATH as u32;
    let mut n = 0;
    let mut put = |c: u32| {
        if n < 95 {
            wr8(out + n, c);
            n += 1;
        }
    };
    if rd8(path) != b'\\' as u32 {
        let cwd = &raw const CWD as u32;
        let mut i = 0;
        while rd8(cwd + i) != 0 {
            put(rd8(cwd + i));
            i += 1;
        }
        put(b'\\' as u32);
    }
    let mut i = 0;
    while rd8(path + i) != 0 {
        put(rd8(path + i));
        i += 1;
    }
    put(0);
    out
}

/// A path's directory and its last part: the directory's extent, the last part's address in PART.
fn split(path: u32) -> Option<((u32, u32), u32)> {
    if !mount() {
        return None;
    }
    let path = absolute(path);
    let part = &raw mut PART as u32;
    let mut dir = unsafe { ROOT };
    let mut p = path;
    loop {
        while rd8(p) == b'\\' as u32 {
            p += 1;
        }
        let mut n = 0;
        while rd8(p + n) != 0 && rd8(p + n) != b'\\' as u32 {
            if n < 31 {
                wr8(part + n, rd8(p + n));
            }
            n += 1;
        }
        wr8(part + n.min(31), 0);
        p += n;
        if rd8(p) == 0 {
            return Some((dir, part));
        }
        let (lba, size, flags, _) = scan(dir, part, 0, false)?;
        if flags & DIRECTORY == 0 {
            return None;
        }
        dir = (lba, size);
    }
}

/// A file's extent and size by its path.
pub fn lookup(path: u32) -> Option<(u32, u32)> {
    let (dir, part) = split(path)?;
    if rd8(part) == 0 {
        return Some(dir);
    }
    let (lba, size, _, _) = scan(dir, part, 0, false)?;
    Some((lba, size))
}

/// A(5Fh): the file's extent and size in the FCB with the disk id; 0 or an error number.
pub extern "C" fn dev_cd_open(f: u32, path: u32, _mode: u32) -> u32 {
    if cdrom::status().is_some_and(|_| cdrom::shell_was_opened()) {
        unmount();
    }
    if !mount() {
        file::device_error(f, error::GENERAL);
        return error::GENERAL;
    }
    match lookup(path) {
        Some((lba, size)) => {
            wr(f + START, lba);
            wr(f + SIZE, size);
            wr(f + fcb::PORT, unsafe { DISK_ID });
            0
        }
        None => {
            file::device_error(f, error::NOT_FOUND);
            error::NOT_FOUND
        }
    }
}

/// A(60h): from the file position, to the file's end at most; the bytes read, or -1.
pub extern "C" fn dev_cd_read(f: u32, dst: u32, length: u32) -> u32 {
    let (position, size) = (rd(f + fcb::POSITION), rd(f + SIZE));
    if !unsafe { MOUNTED } || rd(f + fcb::PORT) != unsafe { DISK_ID } {
        return file::device_error(f, error::GENERAL);
    }
    if position >= size {
        return file::device_error(f, error::INVALID);
    }
    let n = length.min(size - position);
    let (first, skip) = (rd(f + START) + position / SECTOR, position % SECTOR);
    if cdrom::read(first, (skip + n).div_ceil(SECTOR), dst, skip, n) { n } else { file::device_error(f, error::GENERAL) }
}

/// Writing to the CD returns 0, as psx-spx says.
extern "C" fn dev_cd_write(_f: u32, _src: u32, _length: u32) -> u32 {
    0
}

/// A(61h).
pub extern "C" fn dev_cd_close(_f: u32) -> u32 {
    0
}

fn entry(found: (u32, u32, u32, u32), direntry: u32) -> u32 {
    names::copy(direntry, &raw const NAME as u32, 0x13, 0x14);
    wr(direntry + 0x14, 0);
    wr(direntry + 0x18, found.1);
    wr(direntry + 0x1C, 0);
    wr(direntry + 0x20, found.0);
    wr(direntry + 0x24, 0);
    direntry
}

fn search(direntry: u32) -> u32 {
    let (lba, size, from) = unsafe { SEARCH };
    match scan((lba, size), &raw const PATTERN as u32, from, true) {
        Some(found) => {
            unsafe { SEARCH.2 = found.3 };
            entry(found, direntry)
        }
        None => 0,
    }
}

/// A(62h): the first record of the path's directory matching its last part, "*" and "?" as psx-spx gives.
pub extern "C" fn dev_cd_firstfile(_f: u32, path: u32, direntry: u32) -> u32 {
    let Some((dir, part)) = split(path) else { return 0 };
    names::copy(&raw mut PATTERN as u32, part, 31, 32);
    unsafe { SEARCH = (dir.0, dir.1, 0) };
    search(direntry)
}

/// A(63h).
pub extern "C" fn dev_cd_nextfile(_f: u32, direntry: u32) -> u32 {
    search(direntry)
}

/// A(64h) and B(40h)'s device function: the directory remembered without checking it, the volume read again, as
/// psx-spx says the original reads the path table each time; 1.
pub extern "C" fn dev_cd_chdir(_f: u32, path: u32) -> u32 {
    let full = absolute(path);
    names::copy(&raw mut CWD as u32, full, 63, 64);
    let cwd = &raw mut CWD as u32;
    let mut n = crate::string::length(cwd);
    while n > 0 && rd8(cwd + n - 1) == b'\\' as u32 {
        n -= 1;
        wr8(cwd + n, 0);
    }
    unmount();
    mount();
    1
}

/// A(A4h): the file's first sector, or -1; a device name before the path is passed over.
pub extern "C" fn cd_get_lbn(name: u32) -> u32 {
    let mut path = name;
    let mut i = 0;
    while rd8(name + i) != 0 {
        if rd8(name + i) == b':' as u32 {
            path = name + i + 1;
        }
        i += 1;
    }
    lookup(path).map_or(u32::MAX, |(lba, _)| lba)
}

extern "C" fn test_device(_f: u32, _path: u32) -> u32 {
    0
}

/// The DCB psx-spx gives for "cdrom": flags 14h, 800h-byte sectors, "CD-ROM".
pub fn device() -> u32 {
    let info = &raw mut INFO as u32;
    for o in (0..0x50).step_by(4) {
        wr(info + o, 0);
    }
    wr(info + dcb::NAME, b"cdrom\0".as_ptr() as u32);
    wr(info + dcb::FLAGS, 0x14);
    wr(info + dcb::SECTOR, SECTOR);
    wr(info + dcb::LONG_NAME, b"CD-ROM\0".as_ptr() as u32);
    wr(info + dcb::OPEN, dev_cd_open as *const () as u32);
    wr(info + dcb::CLOSE, dev_cd_close as *const () as u32);
    wr(info + dcb::READ, dev_cd_read as *const () as u32);
    wr(info + dcb::WRITE, dev_cd_write as *const () as u32);
    wr(info + dcb::FIRSTFILE, dev_cd_firstfile as *const () as u32);
    wr(info + dcb::NEXTFILE, dev_cd_nextfile as *const () as u32);
    wr(info + dcb::CD, dev_cd_chdir as *const () as u32);
    wr(info + dcb::TESTDEVICE, test_device as *const () as u32);
    info
}
