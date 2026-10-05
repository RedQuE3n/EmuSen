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
def z80_loop(address, op=0x3A):
    """Bank 0, then ld a,(address) or, with op $32, ld (address),a, and jr back: 13 + 12 T a round."""
    return bytes([0xF3, 0x21, 0x00, 0x60, 0xAF] + [0x77] * 9 + [op, address & 0xFF, address >> 8, 0x18, 0xFB])

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

# Where the RAM's pins put $FF1000 and $FF1002: the board orders the RAM's address lines { VA14, IA14, VA12-VA0 }.
MARK, MARK2 = 0x5000, 0x5002

def rate(times):
    gaps = [b - a for a, b in zip(times, times[1:])]
    return gaps, (sum(gaps[len(gaps) // 2:]) / max(1, len(gaps) - len(gaps) // 2)) if gaps else None

def start(ev):
    """When the 68000 has left the cartridge: its last read of it, the jump to its RAM."""
    return max(t for k, t, a, _ in ev if k == "c" and 0x200 <= a < 0x300)

def accesses(times):
    """A window access shows as a pair of cartridge strobes about 100 MCLK2 apart: the first of each."""
    out = []
    for t in times:
        if not out or t - out[-1] > 300:
            out.append(t)
    return out

# MCLK2 cycles a Z80 T-state and a 68000 clock (MCLK2 runs at twice the master clock), and the loops' nominal lengths.
Z80_T, M68K = 30, 14
Z80_ROUND = 25

def window(cycles, op=0x3A):
    kind = "c" if op == 0x3A else "x"
    print("Z80 reads:" if op == 0x3A else "Z80 writes:")
    runs = {}
    for name, z80 in (("window", z80_loop(0x8000, op)), ("control", z80_loop(0x1000, op))):
        ev = run(rom(z80, COUNT), cycles)
        t0 = start(ev) + 20000
        z = accesses([t for k, t, a, _ in ev if k == kind and a == 0 and t > t0])
        m = [t for k, t, a, _ in ev if k == "w" and a == MARK and t > t0]
        runs[name] = (z, m)
    z, m = runs["window"]
    _, mc = runs["control"]
    base = (mc[-1] - mc[0]) / (len(mc) - 1)
    period = (z[-1] - z[0]) / (len(z) - 1)
    span = (m[0], m[-1])
    inside = [t for t in z if span[0] <= t < span[1]]
    lost = ((span[1] - span[0]) - (len(m) - 1) * base) / len(inside)
    print(f"68000 round without the window {base:.1f} MCLK2 ({base / M68K:.2f} clocks) over {len(mc) - 1} rounds")
    print(f"Z80 round with a window access {period:.1f} MCLK2 = {period / Z80_T:.2f} T over {len(z) - 1} rounds: "
          f"the window access waits {period / Z80_T - Z80_ROUND:.2f} T")
    print(f"68000: {len(m) - 1} rounds beside {len(inside)} window accesses lose {lost:.1f} MCLK2 = {lost / M68K:.2f} "
          f"68000 clocks an access")

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
    marks = [(t, a) for k, t, a, _ in ev if k == "w" and a in (MARK, MARK2) and t > start(ev)]
    lat = [b[0] - a[0] for a, b in zip(marks, marks[1:]) if a[1] == MARK and b[1] == MARK2]
    print(f"BUSREQ: {len(lat)} requests; marker A to marker B in MCLK2: {sorted(set(lat))}")

def busreq_tight(cycles, z80_address=0x8000):
    """The marker before the request and the poll right after it: a poll that finds the bus not yet granted adds
    one round of btst and bne.s, 30 68000 clocks."""
    loop = w(0x41F9, 0x00FF, 0x1000,                  # lea $FF1000,a0
             0x3080,                                  # move.w d0,(a0): marker A
             0x33FC, 0x0100, 0x00A1, 0x1100,          # move.w #$100,$A11100
             0x0839, 0x0000, 0x00A1, 0x1100,          # btst #0,$A11100
             0x66F6,                                  # bne.s the btst
             0x3140, 0x0002,                          # move.w d0,2(a0): marker B
             0x33FC, 0x0000, 0x00A1, 0x1100,          # move.w #0,$A11100
             0x5240,                                  # addq.w #1,d0
             0x60DC)                                  # bra.s marker A
    ev = run(rom(z80_loop(z80_address), loop), cycles)
    marks = [(t, a) for k, t, a, _ in ev if k == "w" and a in (MARK, MARK2) and t > start(ev)]
    lat = [b[0] - a[0] for a, b in zip(marks, marks[1:]) if a[1] == MARK and b[1] == MARK2]
    print(f"the Z80 reading {'its window' if z80_address >= 0x8000 else 'its RAM'}:", end=" ")
    polls = [round((x - min(lat)) / (30 * M68K)) for x in lat]
    print(f"BUSREQ, polled at once: {len(lat)} requests; marker A to B in MCLK2 from {min(lat)} to {max(lat)}; "
          f"extra polls {sorted(set(polls))}, counts {[polls.count(p) for p in sorted(set(polls))]}")

def rom_loop(cycles):
    """The 68000 alone, the Z80 held in reset: add.w (a0)+,d1; cmp.l a0,d0; bcc.s, 8 + 6 + 10 clocks by the 68000's
    manual, summing words of the cartridge or of the 68000's RAM; a round is timed by its fetch of the bcc.s."""
    import collections
    for name, base in (("cartridge", 0x00001000), ("RAM", 0x00FF0000)):
        r = bytearray(b"\xff" * 0x1000)
        r[0:8] = w(0x00FF, 0xFE00, 0x0000, 0x0200)
        r[0x100:0x110] = b"SEGA MEGA DRIVE "
        r[0x200:0x216] = w(0x46FC, 0x2700,                          # move.w #$2700,sr
                           0x207C, base >> 16, base & 0xFFFF,       # movea.l #base,a0
                           0x203C, (base + 0xFFFE) >> 16, (base + 0xFFFE) & 0xFFFF,  # move.l #end,d0
                           0x7200,                                  # moveq #0,d1
                           0xD258,                                  # add.w (a0)+,d1
                           0xB088,                                  # cmp.l a0,d0
                           0x64FA)                                  # bcc.s the add
        ev = run(bytes(r), cycles)
        t = [t for k, t, a, _ in ev if k == "c" and a == 0x212]
        g = [b - a for a, b in zip(t, t[1:])][10:]
        mean = sum(g) / len(g)
        print(f"{name}: {len(g)} rounds, {mean:.2f} MCLK2 = {mean / M68K:.3f} clocks a round (24 by the manual); "
              f"rounds in clocks {sorted((k / M68K, n) for k, n in collections.Counter(g).items())}")

if __name__ == "__main__":
    what, cycles = sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 400000
    if what == "busreq-tight":
        busreq_tight(cycles, 0x8000)
        busreq_tight(cycles, 0x1000)
    else:
        {"window": window, "window-write": lambda c: window(c, 0x32), "busreq": busreq, "rom-loop": rom_loop}[what](cycles)
