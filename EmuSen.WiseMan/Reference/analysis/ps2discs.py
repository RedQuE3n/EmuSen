#!/usr/bin/env python3
"""ps2discs.py OUT: synthetic discs for the referees (Sapphire_Native.md §25.3), sparse where large: an ISO 9660 volume whose root holds SYSTEM.CNF, as a
single-layer DVD, a dual-layer DVD (layer 0's descriptor naming its own sectors, a second at layer 1's sector 16) and
a CD as a raw mode 2 track with its sheet. Never real game data."""
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

def pvd(blocks, root=20):
    d = bytearray(U)
    d[0] = 1; d[1:6] = b"CD001"; d[6] = 1
    d[8:40] = b"PLAYSTATION".ljust(32); d[40:72] = b"EMUSEN_SYNTH".ljust(32)
    d[80:88] = both32(blocks); d[120:124] = both16(1); d[124:128] = both16(1); d[128:132] = both16(U)
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

def write_sparse(path, chunks, total):
    with open(path, "wb") as f:
        for at, data in chunks:
            f.seek(at); f.write(data)
        f.truncate(total)

out = sys.argv[1]
write_sparse(f"{out}/dvd.iso", [(0, volume(1_000_000, 1_000_000))], 1_000_000 * U)
l0, total = 600_000, 1_100_000
write_sparse(f"{out}/dual.iso", [(0, volume(total, l0)), ((l0 + 16) * U, pvd(total - l0)), ((l0 + 17) * U, b"\xffCD001\x01")], total * U)
sync = bytes([0] + [255] * 10 + [0])
def bcd(v): return (v // 10) << 4 | v % 10
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
