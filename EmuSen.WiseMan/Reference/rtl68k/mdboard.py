#!/usr/bin/env python3
"""Programs for Nuked-MD's board run as a black box (tb_md), and what their bus activity says: the Z80's window
onto the 68000's bus, and BUSREQ's latency. Nephrite_Native.md §10 is the method.

  mdboard.py window [cycles]   Z80 loops reading banked cartridge ROM, or its own RAM; the 68000 counts in its RAM
  mdboard.py busreq [cycles]   the 68000 requests the bus, polls until granted, releases, marking each in RAM

Times are in MCLK2 cycles, two to a master clock; the bench prints cartridge reads (c) and 68000 RAM writes (w).
"""
import os, subprocess, sys

WORK = os.path.expanduser("~/.cache/emusen/probe/nukedmd-board")
LIB = os.path.expanduser("~/.cache/emusen/toolchains/verilator/usr/lib64")

def w(*words):
    return b"".join(x.to_bytes(2, "big") for x in words)

def rom(z80, ramloop):
    """A cartridge: vectors, then code that loads `z80` into the Z80's RAM and starts it, then copies `ramloop` to
    $FF0000 and runs it there, so that the 68000 leaves the cartridge bus to the Z80."""
    r = bytearray(b"\xff" * 0x1000)
    r[0:8] = w(0x00FF, 0xFE00, 0x0000, 0x0200)
    r[0x100:0x110] = b"SEGA MEGA DRIVE "
    code = w(0x46FC, 0x2700,                          # move.w #$2700,sr
             0x33FC, 0x0100, 0x00A1, 0x1100,          # move.w #$100,$A11100: BUSREQ
             0x33FC, 0x0100, 0x00A1, 0x1200,          # move.w #$100,$A11200: RESET released
             0x0839, 0x0000, 0x00A1, 0x1100,          # btst #0,$A11100
             0x66F6,                                  # bne.s the btst
             0x41FA, 0x00E0,                          # lea z80(pc),a0: $300
             0x43F9, 0x00A0, 0x0000,                  # lea $A00000,a1
             0x323C, len(z80) - 1,                    # move.w #n-1,d1
             0x12D8,                                  # move.b (a0)+,(a1)+
             0x51C9, 0xFFFC,                          # dbra d1,the move
             0x33FC, 0x0000, 0x00A1, 0x1200,          # RESET held
             0x33FC, 0x0000, 0x00A1, 0x1100,          # BUSREQ released
             0x33FC, 0x0100, 0x00A1, 0x1200,          # RESET released: the Z80 runs
             0x41FA, 0x0018,                          # lea ramloop(pc),a0
             0x43F9, 0x00FF, 0x0000,                  # lea $FF0000,a1
             0x323C, 0x001F,                          # move.w #31,d1
             0x32D8,                                  # move.w (a0)+,(a1)+
             0x51C9, 0xFFFC,                          # dbra d1,the move
             0x4EF9, 0x00FF, 0x0000)                  # jmp $FF0000
    assert len(code) == 0x64
    r[0x200:0x200 + len(code)] = code
    assert len(ramloop) <= 0x40
    r[0x264:0x264 + len(ramloop)] = ramloop
    r[0x300:0x300 + len(z80)] = z80
    return bytes(r)

# The 68000 in its RAM: d0 stored at $FF1000 and counted, 8 + 8 + 10 clocks a round (move.l, addq.l, bra.s).
COUNT = w(0x41F9, 0x00FF, 0x1000, 0x7000, 0x2080, 0x5280, 0x60FA)
# The Z80: interrupts off, the bank at 0, then ld a,(nn) and jr back, 13 + 12 T-states a round.
def z80_loop(address):
    return bytes([0xF3, 0x21, 0x00, 0x60, 0xAF] + [0x77] * 9 + [0x3A, address & 0xFF, address >> 8, 0x18, 0xFB])

def run(image, cycles):
    path = os.path.join(WORK, "program.bin")
    open(path, "wb").write(image)
    out = subprocess.run([os.path.join(WORK, "tb_md"), path, str(cycles)], capture_output=True, text=True, cwd=WORK,
                         env=dict(os.environ, LD_LIBRARY_PATH=LIB), check=True).stdout
    events = []
    for line in out.splitlines():
        k, t, a, *v = line.split()
        events.append((k, int(t), int(a, 16), int(v[0], 16) if v else None))
    return events

def rate(times):
    gaps = [b - a for a, b in zip(times, times[1:])]
    return gaps, (sum(gaps[len(gaps) // 2:]) / max(1, len(gaps) - len(gaps) // 2)) if gaps else None

def window(cycles):
    for name, z80 in (("window reads", z80_loop(0x8000)), ("Z80 RAM reads", z80_loop(0x1000))):
        ev = run(rom(z80, COUNT), cycles)
        z = [t for k, t, a, _ in ev if k == "c" and a < 0x8000 and t > cycles // 4]
        m = [t for k, t, a, _ in ev if k == "w" and a == 0x1000 and t > cycles // 4]
        zg, zr = rate(z)
        mg, mr = rate(m)
        print(f"{name}: {len(z)} window reads, a round every {zr} MCLK2 ({sorted(set(zg))[:6]}); "
              f"68000 rounds every {mr} MCLK2 ({sorted(set(mg))[:6]})")

def busreq(cycles):
    loop = w(0x41F9, 0x00FF, 0x1000,                  # lea $FF1000,a0
             0x33FC, 0x0100, 0x00A1, 0x1100,          # move.w #$100,$A11100
             0x3080,                                  # move.w d0,(a0): marker A
             0x0839, 0x0000, 0x00A1, 0x1100,          # btst #0,$A11100
             0x66F6,                                  # bne.s the btst
             0x3140, 0x0002,                          # move.w d0,2(a0): marker B
             0x33FC, 0x0000, 0x00A1, 0x1100,          # move.w #0,$A11100
             0x5240,                                  # addq.w #1,d0
             0x60DC)                                  # bra.s the request
    ev = run(rom(z80_loop(0x8000), loop), cycles)
    marks = [(t, a) for k, t, a, _ in ev if k == "w" and a in (0x1000, 0x1002) and t > cycles // 4]
    lat = [b[0] - a[0] for a, b in zip(marks, marks[1:]) if a[1] == 0x1000 and b[1] == 0x1002]
    print(f"BUSREQ: {len(lat)} requests; marker A to marker B in MCLK2: {sorted(set(lat))}")

if __name__ == "__main__":
    what, cycles = sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 400000
    {"window": window, "busreq": busreq}[what](cycles)
