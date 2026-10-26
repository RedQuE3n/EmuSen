#!/usr/bin/env python3
"""ps2discs.py OUT: synthetic discs for the referees (Sapphire_Native.md §25.3), sparse where large: an ISO 9660 volume whose root holds SYSTEM.CNF, as a
single-layer DVD, a dual-layer DVD (layer 0's descriptor naming its own sectors, a second at layer 1's sector 16) and
a CD as a raw mode 2 track with its sheet. Never real game data.

ps2discs.py files OUT.iso NAME=FILE ...: a DVD image of 2,048-byte sectors whose root holds SYSTEM.CNF naming the first
file as BOOT2, then each file given under its name, with both path tables (Sapphire_Native.md §40, §40.14): the discs that carry
the project's loader and an IOP module for the IOP to open from cdrom0:. A DVD's path tables are at 257 and 259 and its
root at 261, where the PlayStation 2's DVDs place them and where the player's BIOS reads a DVD's path table whatever its
descriptor names (Sapphire_Disputes.md D-483); OUT.bin writes the volume as a CD's raw data track of mode 2 form 1
sectors, its path tables at 18 and 19 and its root at 20, where the BIOS follows the descriptor."""
import os, struct, sys

U = 2048
CNF = b"BOOT2 = cdrom0:\\EMUS_000.02;1\r\nVER = 1.00\r\nVMODE = NTSC\r\n"

def both32(v): return struct.pack("<I", v) + struct.pack(">I", v)
def both16(v): return struct.pack("<H", v) + struct.pack(">H", v)

def record(extent, size, name, flags):
    r = bytearray(33 + len(name) + (1 - len(name) % 2))
    r[0] = len(r); r[2:10] = both32(extent); r[10:18] = both32(size); r[25] = flags; r[28:32] = both16(1)
    r[32] = len(name); r[33:33 + len(name)] = name
    return bytes(r)

# The L and M path tables' sectors and the root's: a CD's after its descriptors, a DVD's past sector 256 as on the discs (D-483).
CD_LAYOUT = (18, 19, 20)
DVD_LAYOUT = (257, 259, 261)

def pvd(blocks, root=20, tables=None):
    d = bytearray(U)
    d[0] = 1; d[1:6] = b"CD001"; d[6] = 1
    d[8:40] = b"PLAYSTATION".ljust(32); d[40:72] = b"EMUSEN_SYNTH".ljust(32)
    d[80:88] = both32(blocks); d[120:124] = both16(1); d[124:128] = both16(1); d[128:132] = both16(U)
    if tables:
        d[132:140] = both32(10); d[140:144] = struct.pack("<I", tables[0]); d[148:152] = struct.pack(">I", tables[1])
    d[156:156 + 34] = record(root, U, b"\x00", 2)
    d[881] = 1
    return bytes(d)

def volume(sectors, blocks):
    head = bytearray(22 * U)
    head[16 * U:17 * U] = pvd(blocks)
    head[17 * U] = 255; head[17 * U + 1:17 * U + 6] = b"CD001"; head[17 * U + 6] = 1
    root = record(20, U, b"\x00", 2) + record(20, U, b"\x01", 2) + record(21, len(CNF), b"SYSTEM.CNF;1", 0)
    head[20 * U:20 * U + len(root)] = root
    head[21 * U:21 * U + len(CNF)] = CNF
    return head

def files_volume(files, layout=CD_LAYOUT):
    """A volume of the root alone: the descriptors, the L and M path tables and the root at `layout`'s sectors,
    SYSTEM.CNF naming the first file as BOOT2, and each file in sectors of its own after the root (ECMA-119 §8.4, §9.1,
    §9.4)."""
    lpt, mpt, at_root = layout
    cnf = f"BOOT2 = cdrom0:\\{files[0][0]};1\r\nVER = 1.00\r\nVMODE = NTSC\r\n".encode()
    entries, extent = [], at_root + 1
    for name, data in [("SYSTEM.CNF", cnf)] + files:
        entries.append((name.encode() + b";1", extent, data))
        extent += max(1, -(-len(data) // U))
    image = bytearray(extent * U)
    image[16 * U:17 * U] = pvd(extent, at_root, (lpt, mpt))
    image[17 * U] = 255; image[17 * U + 1:17 * U + 6] = b"CD001"; image[17 * U + 6] = 1
    image[lpt * U:lpt * U + 10] = bytes([1, 0]) + struct.pack("<I", at_root) + struct.pack("<H", 1) + bytes(2)
    image[mpt * U:mpt * U + 10] = bytes([1, 0]) + struct.pack(">I", at_root) + struct.pack(">H", 1) + bytes(2)
    root = record(at_root, U, b"\x00", 2) + record(at_root, U, b"\x01", 2)
    for name, at, data in sorted(entries):
        root += record(at, len(data), name, 0)
        image[at * U:at * U + len(data)] = data
    image[at_root * U:at_root * U + len(root)] = root
    return bytes(image)

def bcd(v): return (v // 10) << 4 | v % 10

def raw_track(vol):
    """A volume as a CD's data track of mode 2 form 1 sectors (2,352 bytes), its header's time from 00:02:00; the
    EDC and ECC are left zero, as the referees and Sapphire read the sectors' data alone."""
    sync = bytes([0] + [255] * 10 + [0])
    raw = bytearray()
    for n in range(len(vol) // U):
        a = n + 150
        s = bytearray(2352); s[:12] = sync; s[12:15] = bytes([bcd(a // 4500), bcd(a // 75 % 60), bcd(a % 75)]); s[15] = 2
        s[18] = s[22] = 8
        s[24:24 + U] = vol[n * U:(n + 1) * U]
        raw += s
    return bytes(raw)

def write_sparse(path, chunks, total):
    with open(path, "wb") as f:
        for at, data in chunks:
            f.seek(at); f.write(data)
        f.truncate(total)

def main(argv):
    if argv[:1] == ["files"]:
        files = [(n.upper(), open(f, "rb").read()) for n, f in (a.split("=", 1) for a in argv[2:])]
        cd = argv[1].endswith(".bin")
        vol = files_volume(files, CD_LAYOUT if cd else DVD_LAYOUT)
        open(argv[1], "wb").write(raw_track(vol) if cd else vol)
        return
    out = argv[0]
    write_sparse(f"{out}/dvd.iso", [(0, volume(1_000_000, 1_000_000))], 1_000_000 * U)
    l0, total = 600_000, 1_100_000
    write_sparse(f"{out}/dual.iso", [(0, volume(total, l0)), ((l0 + 16) * U, pvd(total - l0)), ((l0 + 17) * U, b"\xffCD001\x01")], total * U)
    sync = bytes([0] + [255] * 10 + [0])
    raw = bytearray()
    vol = volume(30000, 30000)
    for n in range(30000):
        a = n + 150
        s = bytearray(2352); s[:12] = sync; s[12:15] = bytes([bcd(a // 4500), bcd(a // 75 % 60), bcd(a % 75)]); s[15] = 2
        s[18] = s[22] = 8
        if n < 22: s[24:24 + U] = vol[n * U:(n + 1) * U]
        raw += s
    open(f"{out}/cd.bin", "wb").write(raw)
    open(f"{out}/cd.cue", "w").write('FILE "cd.bin" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n')


if __name__ == "__main__":
    main(sys.argv[1:])
