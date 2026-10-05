#!/usr/bin/env python3
"""Programs for Nuked-MD's board run as a black box (tb_md), and what their bus activity and video pins say: the
Z80's window onto the 68000's bus, BUSREQ's latency, the V counter, mode 4's ports and the VDP's pictures.
Nephrite_Native.md §10 and §15.3 are the method.

  mdboard.py window [cycles]   Z80 loops reading banked cartridge ROM, or its own RAM; the 68000 counts in its RAM
  mdboard.py busreq [cycles]   the 68000 requests the bus, polls until granted, releases, marking each in RAM
  mdboard.py vcounter|interlace [cycles]   the HV counter's changes in V28 and V30, or in interlace modes 1 and 2
  mdboard.py mode4-ports [cycles]   mode 4's writes, read back through mode 5
  mdboard.py picture <name> [frames] [dir]   one of PICTURES' programs, its ROM and the board's frames as PNGs in dir
  mdboard.py vram128 [cycles]   the 128 KiB mode's writes, read back with it clear

Times are in MCLK2 cycles, two to a master clock; the bench prints cartridge reads (c) and 68000 RAM writes (w).

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

def hv(cycles, v30=False, h40=True, lsm=0):
    """The HV counter read in a loop and stored to RAM: each change of the V counter, with the H counter beside it.
    TB_PAL in the environment makes the board PAL."""
    loop = w(0x33FC, 0x810C if v30 else 0x8104, 0x00C0, 0x0004,   # move.w #$8104/$810C,$C00004: mode 5, V28/V30
             0x33FC, (0x8C81 if h40 else 0x8C00) | lsm << 1, 0x00C0, 0x0004,   # register 12: H40/H32, interlace
             0x41F9, 0x00FF, 0x1000,                              # lea $FF1000,a0
             0x30B9, 0x00C0, 0x0008,                              # move.w $C00008,(a0)
             0x60F8)                                              # bra.s the move
    ev = run(rom(bytes([0x18, 0xFE]), loop), cycles)
    samples = [(t, d) for k, t, a, d in ev if k == "w" and a == MARK and t > start(ev)]
    seq, last = [], None
    for t, d in samples:
        v = d >> 8
        if v != last:
            seq.append((t, v, d & 0xFF))
            last = v
    return seq

def vcounter(cycles):
    import os
    for v30 in (False, True):
        seq = hv(cycles, v30)
        vs = [v for _, v, _ in seq]
        jumps = [(vs[i], vs[i + 1]) for i in range(len(vs) - 1) if vs[i + 1] != (vs[i] + 1) & 0xFF]
        lines = [b[0] - a[0] for a, b in zip(seq, seq[1:])]
        print(f"{'PAL' if os.environ.get('TB_PAL') else 'NTSC'} {'V30' if v30 else 'V28'}: {len(seq)} changes; jumps {jumps}; "
              f"MCLK2 between changes {sorted(set(lines))[:6]}")

def interlace(cycles):
    """Interlace modes 1 and 2: the V counter's values over two fields, the line count of each field, and the odd flag."""
    for lsm in (1, 3):
        seq = hv(cycles, False, True, lsm)
        vs = [v for _, v, _ in seq]
        jumps = [(i, vs[i], vs[i + 1]) for i in range(len(vs) - 1) if vs[i + 1] != (vs[i] + 1) & 0xFF and vs[i + 1] != (vs[i] + 2) & 0xFF]
        starts = [i + 1 for i, a, b in jumps if b < a and b < 8]
        fields = [b - a for a, b in zip(starts, starts[1:])]
        print(f"interlace mode {'2' if lsm == 3 else '1'}: jumps {[(a, b) for _, a, b in jumps][:8]}; changes a field {fields}; "
              f"first values of a field {vs[starts[0]:starts[0] + 6] if starts else []}")

# The picture bench: a program that sets registers and loads VRAM, CRAM and VSRAM through the data port, then runs a
# tail (by default a branch to itself); TB_PICTURE captures the video pins and frames() cuts them into pictures.
def vram(a): return (0x4000 | (a & 0x3FFF)) << 16 | a >> 14
def cram(a): return (0xC000 | (a & 0x3FFF)) << 16 | a >> 14
def vsram(a): return (0x4000 | (a & 0x3FFF)) << 16 | 0x10 | a >> 14

def program(regs, blocks, tail=w(0x60FE)):
    r = bytearray(b"\xff" * 0x8000)
    r[0:8] = w(0x00FF, 0xFE00, 0x0000, 0x0200)
    r[0x100:0x110] = b"SEGA MEGA DRIVE "
    code = bytearray(w(0x46FC, 0x2700, 0x43F9, 0x00C0, 0x0004, 0x41F9, 0x00C0, 0x0000))  # sr; lea ctrl,a1; lea data,a0
    for v in regs:
        code += w(0x32BC, v)                                    # move.w #reg,(a1)
    data, at = bytearray(), 0x2000
    for cmd, words in blocks:
        op = 0x200 + len(code)
        code += w(0x22BC, cmd >> 16, cmd & 0xFFFF)              # move.l #cmd,(a1)
        code += w(0x45FA, (at + len(data) - (op + 8)) & 0xFFFF)  # lea data(pc),a2
        code += w(0x303C, len(words) - 1, 0x309A, 0x51C8, 0xFFFC)  # move.w #n-1,d0; move.w (a2)+,(a0); dbra d0
        data += w(*words)
    code += tail
    r[0x200:0x200 + len(code)] = code
    r[at:at + len(data)] = data
    return bytes(r)

def frames(path):
    """The capture's frames: each run of lines with both display enables, sampled mid-pixel on the pixel clock."""
    d = open(path, "rb").read()
    f = d[0::4]
    lines, start = [], None
    for i, fl in enumerate(f):
        on = fl & 3 == 3
        if on and start is None: start = i
        if not on and start is not None: lines.append((start, i, f[start] >> 4 & 1)); start = None
    out, g = [], []
    for ln in lines:
        if g and ln[0] - g[-1][0] > 6840 * 4: out.append(g); g = []
        g.append(ln)
    if g: out.append(g)
    pics = []
    for g in out:
        s0 = g[0][0]
        rises = [j for j in range(s0 + 1, s0 + 400) if f[j] & 32 and not f[j - 1] & 32]
        cpp = round((rises[-1] - rises[0]) / (len(rises) - 1))
        width = (g[0][1] - g[0][0]) // cpp
        rows = [bytes(b for x in range(width) for b in d[4 * (s + x * cpp + cpp // 2) + 1:4 * (s + x * cpp + cpp // 2) + 4]) for s, _, _ in g]
        pics.append((width, rows, g[0][2]))
    return pics

def picture(name, nframes=4, outdir=None):
    from PIL import Image
    image = PICTURES[name]()
    outdir = outdir or os.path.join(WORK, "pictures")
    os.makedirs(outdir, exist_ok=True)
    open(os.path.join(outdir, name + ".bin"), "wb").write(image)
    path = os.path.join(WORK, "program.bin")
    open(path, "wb").write(image)
    cycles = 2 * (1070460 if name in PAL_PICTURES else 896040) * (nframes + 1)
    pic = os.path.join(outdir, name + ".pic")
    env = dict(os.environ, LD_LIBRARY_PATH=LIB, TB_PICTURE=f"{pic}:0:{cycles}", **({"TB_PAL": "1"} if name in PAL_PICTURES else {}))
    # The 68000's RAM writes are kept beside the frames, for the programs that store there.
    with open(os.path.join(outdir, name + ".ram"), "w") as log:
        p = subprocess.Popen([os.path.join(WORK, "tb_md"), path, str(cycles)], stdout=subprocess.PIPE, text=True, cwd=WORK, env=env)
        for line in p.stdout:
            if line.startswith("w "):
                log.write(line)
        assert p.wait() == 0
    for k, (width, rows, field) in enumerate(frames(pic)):
        if len(rows) < 100: continue
        im = Image.frombytes("RGB", (width, len(rows)), b"".join(rows))
        im.save(os.path.join(outdir, f"{name}_{k}.png"))
        print(f"{name} frame {k}: {width}x{len(rows)}, field pin {field}")
    os.remove(pic)

def mode4_ports(cycles=0):
    """Mode 4's data port as a Master System program uses it: bytes written after one-word commands, a marker at each
    address bit, one even and one odd address alone, two words, and CRAM's 32 bytes; then mode 5 reads all of VRAM
    and CRAM back into one RAM word, which the bench prints in order."""
    code = bytearray(w(0x46FC, 0x2700, 0x43F9, 0x00C0, 0x0004, 0x41F9, 0x00C0, 0x0000, 0x47F9, 0x00FF, 0x1000))
    for v in (0x8004, 0x8100, 0x8F01):
        code += w(0x32BC, v)
    marks = [(1 << b, 0xC0 + b) for b in range(14)] + [(0x0030, 0x51), (0x0043, 0x52)]
    for a, v in marks:
        code += w(0x32BC, 0x4000 | a, 0x10BC, v)                                # cmd; move.b #v,(a0)
    for a, v in ((0x3000, 0x1234), (0x3011, 0x5678)):
        code += w(0x32BC, 0x4000 | a, 0x30BC, v)                                # cmd; move.w #v,(a0)
    code += w(0x32BC, 0x8F02, 0x32BC, 0x4100)                                   # register 15 at 2; address $100
    for v in (0x61, 0x62, 0x63, 0x64):
        code += w(0x10BC, v)                                                    # four bytes, no command between
    code += w(0x32BC, 0x8F01)
    code += w(0x32BC, 0xC000, 0x7000, 0x1080, 0x5240, 0x0C40, 0x0020, 0x66F6)  # CRAM: bytes 0 to 31, each its index
    code += w(0x32BC, 0x8104, 0x32BC, 0x8F02, 0x22BC, 0x0000, 0x0000)          # mode 5; VRAM read from 0
    code += w(0x303C, 0x7FFF, 0x3690, 0x51C8, 0xFFFC)                          # 32768 words: move.w (a0),(a3)
    code += w(0x22BC, 0x0000, 0x0020, 0x303C, 0x003F, 0x3690, 0x51C8, 0xFFFC)  # CRAM read: 64 words
    code += w(0x36BC, 0x5A5A, 0x60FE)                                          # the end marker
    r = bytearray(b"\xff" * 0x1000)
    r[0:8] = w(0x00FF, 0xFE00, 0x0000, 0x0200)
    r[0x100:0x110] = b"SEGA MEGA DRIVE "
    r[0x200:0x200 + len(code)] = code
    ev = run(bytes(r), cycles or 16000000)
    words = [d for k, t, a, d in ev if k == "w" and a == MARK]
    if words and words[-1] == 0x5A5A: words = words[:-1]
    print(f"{len(words)} words read back")
    vr, cr = words[:32768], words[32768:]
    b = bytearray()
    for x in vr: b += x.to_bytes(2, "big")
    where = {}
    for i, v in enumerate(b):
        if v: where.setdefault(v, []).append(i)
    print("mode 4 VRAM address -> mode 5 byte addresses:")
    for a, v in marks + [(0x3000, 0x12), (0x3000, 0x34), (0x3011, 0x56), (0x3011, 0x78), (0x100, 0x61), (0x101, 0x62), (0x102, 0x63), (0x103, 0x64)]:
        print(f"  {a:#06x} (value {v:#04x}) -> {[hex(x) for x in where.get(v, [])][:8]}")
    print("CRAM words:", [hex(x) for x in cr])
    open(os.path.join(WORK, "mode4-vram.bin"), "wb").write(bytes(b))

def vram128(cycles=0):
    """Register 1's bit 7, the 128 KiB VRAM mode, on a board with 64: words written with it set to an address at each
    address bit, an odd address and a run of eight, then read back with it clear, and read with it set."""
    code = bytearray(w(0x46FC, 0x2700, 0x43F9, 0x00C0, 0x0004, 0x41F9, 0x00C0, 0x0000, 0x47F9, 0x00FF, 0x1000))
    code += w(0x32BC, 0x8004, 0x32BC, 0x8184, 0x32BC, 0x8F02)              # mode 5, display off, 128 KiB mode
    marks = [(1 << b, 0xC000 | b << 8 | b) for b in range(1, 16)] + [(0x0000, 0xA5A5), (0x0101, 0x1234)]
    for a, v in marks:
        cmd = vram(a)
        code += w(0x22BC, cmd >> 16, cmd & 0xFFFF, 0x30BC, v)               # move.l #cmd,(a1); move.w #v,(a0)
    cmd = vram(0x2200)
    code += w(0x22BC, cmd >> 16, cmd & 0xFFFF)
    for k in range(8):
        code += w(0x30BC, 0x5100 + k * 0x11)                                 # eight words in a run
    code += w(0x32BC, 0x8104, 0x22BC, 0x0000, 0x0000)                       # 64 KiB mode; VRAM read from 0
    code += w(0x303C, 0x7FFF, 0x3690, 0x51C8, 0xFFFC)                       # 32768 words to RAM
    code += w(0x32BC, 0x8184, 0x22BC, 0x0000, 0x0000)                       # 128 KiB mode; read the first 64 words
    code += w(0x303C, 0x003F, 0x3690, 0x51C8, 0xFFFC)
    code += w(0x36BC, 0x5A5A, 0x60FE)
    r = bytearray(b"\xff" * 0x1000)
    r[0:8] = w(0x00FF, 0xFE00, 0x0000, 0x0200)
    r[0x100:0x110] = b"SEGA MEGA DRIVE "
    r[0x200:0x200 + len(code)] = code
    ev = run(bytes(r), cycles or 18000000)
    words = [d for k, t, a, d in ev if k == "w" and a == MARK]
    if words and words[-1] == 0x5A5A: words = words[:-1]
    print(f"{len(words)} words read back")
    b = bytearray()
    for x in words[:32768]: b += x.to_bytes(2, "big")
    for a, v in marks + [(0x2200 + 2 * k, 0x5100 + k * 0x11) for k in range(8)]:
        hi, lo = v >> 8, v & 0xFF
        at = [i for i in range(len(b) - 1) if b[i] in (hi, lo) and (b[i] == hi or b[i] == lo) and (b[i:i+1] == bytes([hi]) or b[i:i+1] == bytes([lo]))]
        print(f"  {a:#07x} {v:#06x}: high byte at {[hex(i) for i in range(len(b)) if b[i] == hi][:6]}, low at {[hex(i) for i in range(len(b)) if b[i] == lo][:6]}")
    print("read in 128 KiB mode from 0:", [hex(x) for x in words[32768:32768 + 64]])
    open(os.path.join(WORK, "vram128.bin"), "wb").write(bytes(b))

PICTURES = {}

# Mode 5's registers for the pictures: H40, plane A at $C000, B at $E000, the sprites at $F000, the scroll at $FC00.
MODE5 = [0x8004, 0x8144, 0x8230, 0x8407, 0x8578, 0x8700, 0x8C81, 0x8D3F, 0x8F02, 0x9001]
def colours(n):
    """n CRAM words whose components step differently, so that every bit of each is seen in some colour."""
    return [(k & 7) << 1 | (k >> 3 & 7) << 5 | ((k * 5) >> 2 & 7) << 9 for k in range(n)]
def solid_tiles():
    """Tiles 0 to 15, each one colour index."""
    return [(i | i << 4) * 0x101 for i in range(16) for _ in range(16)]

def palette(lsb):
    """Every colour of the four palettes in a band of tiles, with register 0's bit 2 set (full colour) or clear."""
    names = [((r % 4) << 13) | (c % 16) for r in range(32) for c in range(64)]
    return program([0x8000 | (4 if lsb == 0 else 0)] + MODE5[1:], [(cram(0), colours(64)), (vram(0), solid_tiles()), (vram(0xC000), names)])
PICTURES["palette64"] = lambda: palette(0)
PICTURES["palette8"] = lambda: palette(1)

def cram_dots():
    """The display all backdrop, and the 68000 writing CRAM entry 2, which nothing shows, as fast as it can."""
    tail = w(0x22BC, cram(4) >> 16, cram(4) & 0xFFFF,   # move.l #cram(4),(a1)
             0x3081, 0x5241, 0x60FA)                    # move.w d1,(a0); addq.w #1,d1; bra.s the move
    return program(MODE5[:-2] + [0x8F00, 0x9001], [(cram(0), [0x0222, 0, 0, 0])], tail)
PICTURES["cram-dots"] = cram_dots

def mode4():
    """Mode 4 set up from mode 5: patterns, a name table, sprites and CRAM written as words, then registers 0 and 1
    switched to mode 4 with the display on, register 2 at $3800, the sprites at $3F00 and a horizontal scroll of 5."""
    def planar(fn):
        out = []
        for y in range(8):
            px = [fn(x, y) for x in range(8)]
            out += [sum(((px[x] >> b) & 1) << (7 - x) for x in range(8)) for b in range(4)]
        return out
    tiles = [0] * 32
    for n in range(1, 5):
        tiles += planar(lambda x, y, n=n: (x + 2 * y + n) & 15)
    tiles += planar(lambda x, y: 15 if x in (0, 7) or y in (0, 7) else (9 if x == y else 0))
    names = []
    for r in range(28):
        for c in range(32):
            e = 1 + (r + c) % 4 | (0x200 if r & 1 else 0) | (0x800 if r % 4 == 3 else 0) | (0x400 if r % 8 == 5 else 0)
            names += [e & 0xFF, e >> 8]
    sat = [0] * 256
    for i in range(4):
        sat[i] = 50 + 3 * i
        sat[0x80 + 2 * i], sat[0x81 + 2 * i] = 60 + 20 * i, 5
    sat[4] = 0xD0
    as_words = lambda b: [b[i] << 8 | b[i + 1] for i in range(0, len(b), 2)]
    tail = w(0x32BC, 0x8004, 0x32BC, 0x8140, 0x32BC, 0x82FF, 0x32BC, 0x85FF, 0x32BC, 0x86FB, 0x32BC, 0x8705,
             0x32BC, 0x8805, 0x32BC, 0x8900, 0x60FE)
    return program([0x8004, 0x8104, 0x8F02], [(cram(0), colours(32)), (vram(0), as_words(tiles)),
                                              (vram(0x3800), as_words(names)), (vram(0x3F00), as_words(sat))], tail)
PICTURES["mode4"] = mode4

def cram_dots_h32():
    tail = w(0x22BC, cram(4) >> 16, cram(4) & 0xFFFF, 0x3081, 0x5241, 0x60FA)
    regs = [r for r in MODE5 if r >> 8 not in (0x8C, 0x8F, 0x90)] + [0x8C00, 0x8F00, 0x9000]
    return program(regs, [(cram(0), [0x0222, 0, 0, 0])], tail)
PICTURES["cram-dots-h32"] = cram_dots_h32

def mode4_bytes():
    """Mode 4 from the first write, as a Master System program has it: one-word commands, and VRAM and CRAM
    written a byte at a time; CRAM in the Master System's format, --BBGGRR."""
    def planar(fn):
        out = []
        for y in range(8):
            px = [fn(x, y) for x in range(8)]
            out += [sum(((px[x] >> b) & 1) << (7 - x) for x in range(8)) for b in range(4)]
        return out
    tiles = [0] * 32
    for n in range(1, 5):
        tiles += planar(lambda x, y, n=n: (x + 2 * y + n) & 15)
    tiles += planar(lambda x, y: 15 if x in (0, 7) or y in (0, 7) else (9 if x == y else 0))
    names = []
    for r in range(28):
        for c in range(32):
            e = 1 + (r + c) % 4 | (0x200 if r & 1 else 0) | (0x800 if r % 4 == 3 else 0) | (0x400 if r % 8 == 5 else 0)
            names += [e & 0xFF, e >> 8]
    sat = [0] * 256
    for i in range(4):
        sat[i] = 50 + 3 * i
        sat[0x80 + 2 * i], sat[0x81 + 2 * i] = 60 + 20 * i, 5
    sat[4] = 0xD0
    pal = [(k * 13) & 63 for k in range(32)]
    r = bytearray(b"\xff" * 0x8000)
    r[0:8] = w(0x00FF, 0xFE00, 0x0000, 0x0200)
    r[0x100:0x110] = b"SEGA MEGA DRIVE "
    code = bytearray(w(0x46FC, 0x2700, 0x43F9, 0x00C0, 0x0004, 0x41F9, 0x00C0, 0x0000))
    for v in (0x8004, 0x8100, 0x82FF, 0x85FF, 0x86FB, 0x8700, 0x8805, 0x8900, 0x8F01):
        code += w(0x32BC, v)
    data, at = bytearray(), 0x2000
    for cmd, block in ((0xC000, pal), (0x4000, tiles), (0x7800, names), (0x7F00, sat)):
        op = 0x200 + len(code)
        code += w(0x32BC, cmd)                                    # move.w #cmd,(a1): one word
        code += w(0x45FA, (at + len(data) - (op + 6)) & 0xFFFF)   # lea data(pc),a2
        code += w(0x303C, len(block) - 1, 0x109A, 0x51C8, 0xFFFC)  # move.w #n-1,d0; move.b (a2)+,(a0); dbra d0
        data += bytes(block)
    code += w(0x32BC, 0x8140, 0x60FE)                              # the display on; bra.s itself
    r[0x200:0x200 + len(code)] = code
    r[at:at + len(data)] = data
    return bytes(r)
PICTURES["mode4-bytes"] = mode4_bytes

def cram_dma():
    """40 columns of solid tiles showing 40 CRAM entries, and transfers into CRAM through the shown lines, whose writes
    land slot by slot whatever the 68000's timing."""
    names = [(c % 16) | ((c // 16) % 4) << 13 for r in range(32) for c in range(64)]
    regs = [0x8014, 0x8174, 0x8230, 0x8407, 0x8578, 0x8700, 0x8A63, 0x8C81, 0x8D3F, 0x8F02, 0x9001]
    values = [(k * 0x2B5 + (k >> 3) * 0x13) & 0xEEE for k in range(1024)]
    return dma_into(0xC000_0000, 99, 1024, values, regs, [(cram(0), colours(64)), (vram(0), solid_tiles()), (vram(0xC000), names)])
PICTURES["cram-dma"] = cram_dma

def cram_dma_one():
    """Every pixel CRAM entry 1, and transfers into that entry alone (register 15 at 0) through the shown lines: each
    write's dot and the colour after it, along the line."""
    regs = [0x8014, 0x8174, 0x8230, 0x8407, 0x8578, 0x8700, 0x8A63, 0x8C81, 0x8D3F, 0x8F02, 0x9001]
    values = [(k * 0x2B5 + (k >> 3) * 0x13 + 0x222) & 0xEEE for k in range(1024)]
    return dma_into(0xC002_0000, 99, 1024, values, regs, [(cram(0), colours(16)), (vram(0x20), [0x1111] * 16),
                    (vram(0xC000), [1] * 2048)], step=0)
PICTURES["cram-dma-one"] = cram_dma_one

def dma_start(nops):
    """When a transfer's first word lands: 64 words into CRAM entry 1 from the line interrupt after line 150, its
    registers set first and then `nops` NOPs before the command, so that the command's time steps by 28 master
    clocks; the first change along line 150 is the first write (Nephrite_Disputes.md D-11)."""
    words, ram = 64, 0xFF0000
    regs = [0x8014, 0x8174, 0x8230, 0x8407, 0x8578, 0x8700, 0x8A96, 0x8C81, 0x8D3F, 0x8F02, 0x9001]
    values = [(j * 0x2B5 + (j >> 3) * 0x13 + 0x222) & 0xEEE for j in range(words)]
    r = bytearray(dma_into(0xC002_0000, 150, words, values, regs, [(cram(0), colours(16)), (vram(0x20), [0x1111] * 16),
                                                              (vram(0xC000), [1] * 2048)], step=0))
    setregs = w(0x32BC, 0x8F00, 0x32BC, 0x9300 | words, 0x32BC, 0x9400, 0x32BC, 0x9500 | (ram >> 1) & 0xFF,
                0x32BC, 0x9600 | (ram >> 9) & 0xFF, 0x32BC, 0x9700 | (ram >> 17) & 0x7F)
    hint = setregs + w(*([0x4E71] * nops)) + w(0x22BC, 0xC002, 0x0080, 0x4E73)
    r[0x1000:0x1100] = b"\xff" * 0x100
    r[0x1000:0x1000 + len(hint)] = hint
    return bytes(r)
for _n in (0, 2, 4, 6, 8, 10):
    PICTURES[f"dma-start-{_n}"] = lambda n=_n: dma_start(n)

def register_loop(first, second, regs, blocks):
    """The 68000 reading the HV counter and writing a register in a loop, alternately `first` and `second`, each read
    stored to RAM from $FF0000: the picture shows where each write takes effect, the RAM where it was made (D-9)."""
    tail = w(0x303C, first, 0x323C, second, 0x47F9, 0x00FF, 0x0000, 0x3639, 0x00C0, 0x0008, 0x3280, 0x36C3,
             0x3639, 0x00C0, 0x0008, 0x3281, 0x36C3, 0x60EA)
    return program(regs, blocks, tail)
_REG_CRAM = [0x000, 0x00E, 0x0E0, 0xE00] + [0] * 60
PICTURES["reg-backdrop"] = lambda: register_loop(0x8701, 0x8702, MODE5, [(cram(0), _REG_CRAM)])
PICTURES["reg-backdrop-h32"] = lambda: register_loop(0x8701, 0x8702, [r if r >> 8 != 0x8C else 0x8C00 for r in MODE5], [(cram(0), _REG_CRAM)])
PICTURES["reg-display"] = lambda: register_loop(0x8174, 0x8134, [0x8004, 0x8174, 0x8230, 0x8407, 0x8578, 0x8702, 0x8C81, 0x8D3F, 0x8F02, 0x9001],
                                                [(cram(0), _REG_CRAM), (vram(0x20), [0x1111] * 16), (vram(0xC000), [1] * 2048)])

def dma_into(target, start, words, values, regs, blocks, flip=False, step=2):
    """The cartridge for a transfer into `target` (a command's two words) started by every line interrupt, after lines
    `start` and 2 x `start` + 1 (both shown): the values copied to RAM first; with `flip` the frame interrupt
    inverts them, so that every frame's transfers change what they write."""
    tail = w(0x45F9, 0x0000, 0x4000, 0x47F9, 0x00FF, 0x0000, 0x303C, words - 1, 0x36DA, 0x51C8, 0xFFFC,
             0x46FC, 0x2000, 0x60FE)
    r = bytearray(program(regs, blocks, tail))
    for k, v in enumerate(values):
        r[0x4000 + 2 * k:0x4002 + 2 * k] = v.to_bytes(2, "big")
    ram = 0xFF0000
    cmd = target | 0x80
    hint = w(0x32BC, 0x8F00 | step, 0x32BC, 0x9300 | words & 0xFF, 0x32BC, 0x9400 | words >> 8, 0x32BC, 0x9500 | (ram >> 1) & 0xFF,
             0x32BC, 0x9600 | (ram >> 9) & 0xFF, 0x32BC, 0x9700 | (ram >> 17) & 0x7F,
             0x22BC, cmd >> 16, cmd & 0xFFFF, 0x4E73)
    vint = w(0x4E73)
    if flip:
        # The frame interrupt inverts the source first, so that every frame's transfer changes what it writes.
        vint = w(0x49F9, 0x00FF, 0x0000, 0x3E3C, words - 1, 0x0A5C, 0xFFFF, 0x51CF, 0xFFFA) + vint
    r[0x1000:0x1000 + len(hint)] = hint
    r[0x1100:0x1100 + len(vint)] = vint
    r[0x70:0x74] = (0x1000).to_bytes(4, "big")
    r[0x78:0x7C] = (0x1100).to_bytes(4, "big")
    return bytes(r)

def vsram_dma():
    """Plane A in bands, a colour a cell row, scrolled per 2-cell column, and a transfer into VSRAM through the
    shown lines: where each column takes a write shows when the VDP reads that column's scroll."""
    names = [(r % 15) + 1 for r in range(32) for c in range(64)]
    regs = [0x8014, 0x8174, 0x8230, 0x8407, 0x8578, 0x8700, 0x8A00 | 99, 0x8B04, 0x8C81, 0x8D3F, 0x8F02, 0x9001]
    values = [(k * 5 + (k >> 4)) & 0x3FF for k in range(1024)]
    return dma_into(0x4000_0010, 99, 1024, values, regs, [(cram(0), colours(16)), (vram(0), solid_tiles()), (vram(0xC000), names)])
PICTURES["vsram-dma"] = vsram_dma

def vram_dma():
    """Plane A's name table rewritten by a transfer through the shown lines, its entries naming solid tiles: where a
    column takes its new name shows when the VDP reads the name table."""
    names = [(c % 15) + 1 for r in range(32) for c in range(64)]
    regs = [0x8014, 0x8174, 0x8230, 0x8407, 0x8578, 0x8700, 0x8A00 | 99, 0x8C81, 0x8D3F, 0x8F02, 0x9001]
    values = [((k * 7) % 15) + 1 for k in range(1024)]
    return dma_into(0x4000_0000 | (0xC000 & 0x3FFF) << 16 | 0xC000 >> 14, 99, 1024, values, regs,
                    [(cram(0), colours(16)), (vram(0), solid_tiles()), (vram(0xC000), names)])
PICTURES["vram-dma"] = vram_dma

def pattern_dma():
    """Plane A's columns naming tiles 1 to 15, every line showing a row of each, and a transfer rewriting those
    tiles' patterns through the shown lines: a write to the row being shown changes it from where the VDP next
    fetches that tile, which places the pattern fetches against the beam."""
    names = [(c % 15) + 1 for r in range(32) for c in range(64)]
    regs = [0x8014, 0x8174, 0x8230, 0x8407, 0x8578, 0x8700, 0x8A00 | 99, 0x8C81, 0x8D3F, 0x8F02, 0x9001]
    values = [((k * 0x3A7) ^ (k >> 2) * 0x1111) & 0xFFFF for k in range(240)]
    return dma_into(0x4020_0000, 99, 240, values, regs, [(cram(0), colours(16)), (vram(0), solid_tiles()), (vram(0xC000), names)], flip=True)
PICTURES["pattern-dma"] = pattern_dma

def mode4_colours(first):
    """Mode 4 from the first write: VRAM in words (a byte pair, the even address's byte low), a command for each, then
    CRAM's 32 entries a byte at a time, the Master System colours first to first + 31; tiles 0 to 15 each
    one colour, rows 0 to 11 in the first palette and 12 to 23 in the second."""
    tiles = []
    for c in range(16):
        for y in range(8):
            tiles += [0xFF if c >> b & 1 else 0 for b in range(4)]
    names = []
    for r in range(28):
        for col in range(32):
            e = (col // 2) % 16 | (0x800 if r >= 12 else 0)
            names += [e & 0xFF, e >> 8]
    pair = lambda b: [b[i + 1] << 8 | b[i] for i in range(0, len(b), 2)]
    r = bytearray(b"\xff" * 0x8000)
    r[0:8] = w(0x00FF, 0xFE00, 0x0000, 0x0200)
    r[0x100:0x110] = b"SEGA MEGA DRIVE "
    code = bytearray(w(0x46FC, 0x2700, 0x43F9, 0x00C0, 0x0004, 0x41F9, 0x00C0, 0x0000))
    for v in (0x8004, 0x8100, 0x82FF, 0x85FF, 0x86FB, 0x8700, 0x8800, 0x8900, 0x8F02):
        code += w(0x32BC, v)
    data, at = bytearray(), 0x2000
    for cmd, words in ((0x4000, pair(tiles)), (0x7800, pair(names))):
        op = 0x200 + len(code)
        # lea data(pc),a2; move.w #cmd,d3; move.w #n-1,d0; then a command a word: move.w d3,(a1); move.w (a2)+,(a0);
        # addq.w #2,d3; dbra d0
        code += w(0x45FA, (at + len(data) - (op + 2)) & 0xFFFF, 0x363C, cmd, 0x303C, len(words) - 1,
                  0x3283, 0x309A, 0x5443, 0x51C8, 0xFFF8)
        data += w(*words)
    code += w(0x32BC, 0x8F01)
    op = 0x200 + len(code)
    code += w(0x32BC, 0xC000, 0x45FA, (at + len(data) - (op + 6)) & 0xFFFF, 0x303C, 31, 0x109A, 0x51C8, 0xFFFC)
    data += bytes(range(first, first + 32))
    code += w(0x32BC, 0x8140, 0x60FE)
    r[0x200:0x200 + len(code)] = code
    r[at:at + len(data)] = data
    return bytes(r)
PICTURES["mode4-colours-0"] = lambda: mode4_colours(0)
PICTURES["mode4-colours-32"] = lambda: mode4_colours(32)

def interlace(lsm):
    """Double resolution or normal interlace: tiles whose rows are each a different colour, so that a field's rows show."""
    rows16 = [((y % 15) + 1) * 0x1111 for y in range(16) for _ in range(2)]
    names = [(c + r) % 4 for r in range(32) for c in range(64)]
    tiles = rows16 * 4
    return program([0x8004, 0x8144, 0x8230, 0x8407, 0x8578, 0x8700, 0x8C81 | lsm << 1, 0x8D3F, 0x8F02, 0x9001],
                   [(cram(0), colours(16)), (vram(0), tiles), (vram(0xC000), names)])
PICTURES["interlace2"] = lambda: interlace(3)
PICTURES["interlace1"] = lambda: interlace(1)

def field_colour():
    """Double resolution, every pixel colour 1, which the 68000 sets in vertical blanking from the status register's
    odd-field bit: red for an odd field, green for an even one, so that each field's rows show the bit it read."""
    tail = w(0x3011, 0x0240, 0x0010, 0xB240, 0x67F6, 0x3200, 0x22BC, 0xC002, 0x0000, 0x4A40, 0x6706,
             0x30BC, 0x000E, 0x60E4, 0x30BC, 0x00E0, 0x60DE)
    return program([0x8004, 0x8144, 0x8230, 0x8407, 0x8578, 0x8700, 0x8C87, 0x8D3F, 0x8F02, 0x9001],
                   [(vram(0), [0x1111] * 32)], tail)
PICTURES["field-colour"] = field_colour

def pal_v30():
    """PAL's 30 rows: each row of cells one colour, its index the row's plus one; the header says Europe."""
    names = [(r % 15) + 1 for r in range(32) for c in range(64)]
    r = bytearray(program([0x8004, 0x814C] + MODE5[2:], [(cram(0), colours(16)), (vram(0), solid_tiles()), (vram(0xC000), names)]))
    r[0x1F0:0x1F3] = b"E  "
    return bytes(r)
PICTURES["pal-v30"] = pal_v30

def ntsc_v30():
    """The same rows with V30 on an NTSC board, which the documents call misbehaving."""
    names = [(r % 15) + 1 for r in range(32) for c in range(64)]
    return program([0x8004, 0x814C] + MODE5[2:], [(cram(0), colours(16)), (vram(0), solid_tiles()), (vram(0xC000), names)])
PICTURES["ntsc-v30"] = ntsc_v30
PAL_PICTURES = {"pal-v30"}

if __name__ == "__main__":
    what = sys.argv[1]
    if what == "picture":
        picture(sys.argv[2], int(sys.argv[3]) if len(sys.argv) > 3 else 4, sys.argv[4] if len(sys.argv) > 4 else None)
        sys.exit()
    cycles = int(sys.argv[2]) if len(sys.argv) > 2 else 400000
    if what == "busreq-tight":
        busreq_tight(cycles, 0x8000)
        busreq_tight(cycles, 0x1000)
    else:
        {"window": window, "window-write": lambda c: window(c, 0x32), "busreq": busreq, "rom-loop": rom_loop, "vcounter": vcounter, "interlace": interlace, "mode4-ports": mode4_ports, "vram128": vram128}[what](cycles)
