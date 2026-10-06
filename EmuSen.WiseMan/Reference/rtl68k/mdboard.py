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
  mdboard.py sound <name> <out.bin>   one of SOUNDS' programs, written out for a reference or the bench
  mdboard.py pins <name> <out.log> [cycles]   one of SOUNDS' programs on the board, its sound pins logged
  mdboard.py fm <pins.log> <trace> [channel]   a channel on the board's pins against Nephrite's fmtrace output
  mdboard.py board-fm <logs>   Nephrite's board_fm.rs from the board's logs in a directory, made if missing
  mdboard.py board-bus <logs>   Nephrite's board_bus.rs, the RAM loops' cartridge reads, likewise
  mdboard.py read-after-write [nops] [dac] [timers]   a status read after a data write, at 49 places in the sample
  mdboard.py reset-replay [rounds]   one voice replayed after each pulse of the Z80's reset line
  mdboard.py status-ports   the YM2612's four ports read under the busy flag and after, the board's status input each way
  mdboard.py lfo <logs> [first last]   the vibrato's offset for every frequency number, PMS and step (numpy), runs made if missing

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

def dma_start(nops, ram=0xFF0000):
    """When a transfer's first word lands: 64 words from `ram` into CRAM entry 1 from the line interrupt after line 150,
    its registers set first and then `nops` NOPs before the command, so that the command's time steps by 28 master
    clocks; the first change along line 150 is the first write (Nephrite_Disputes.md D-11)."""
    words = 64
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
    PICTURES[f"dma-start-rom-{_n}"] = lambda n=_n: dma_start(n, 0x4000)   # the same from the cartridge's copy

def write_landing(nops, count=1, display=True, h40=True):
    """When a word written through the data port lands: the line interrupt after line 150 sets CRAM's address to entry
    1 (register 15 at 0), waits `nops` NOPs and writes `count` words, a new colour each time; every pixel shows
    entry 1 (or, the display off, the backdrop), so the changes along the line are the writes (D-11)."""
    regs = [0x8014, 0x8174 if display else 0x8134, 0x8230, 0x8407, 0x8578, 0x8700, 0x8A96, 0x8C81 if h40 else 0x8C00, 0x8D3F, 0x8F02, 0x9001]
    r = bytearray(program(regs, [(cram(0), colours(16)), (vram(0x20), [0x1111] * 16), (vram(0xC000), [1] * 2048)],
                          w(0x46FC, 0x2000, 0x60FE)))
    # With the display off every pixel is the backdrop, entry 0, and every slot is the 68000's.
    entry = cram(2) if display else cram(0)
    hint = w(0x32BC, 0x8F00, 0x22BC, entry >> 16, entry & 0xFFFF, *([0x4E71] * nops))
    hint += w(0x0645, 0x0246)                                              # addi.w #$246,d5: a new colour a frame
    for k in range(count):
        hint += w(0x3085, 0x5445)                                          # move.w d5,(a0); addq.w #2,d5
    hint += w(0x4E73)
    r[0x1000:0x1000 + len(hint)] = hint
    r[0x1100:0x1102] = w(0x4E73)
    r[0x70:0x74] = (0x1000).to_bytes(4, "big")
    r[0x78:0x7C] = (0x1100).to_bytes(4, "big")
    return bytes(r)
for _n in range(10, 26, 2):
    PICTURES[f"write-landing-{_n}"] = lambda n=_n: write_landing(n)
    PICTURES[f"write-landing-off-{_n}"] = lambda n=_n: write_landing(n, display=False)
    PICTURES[f"write-landing-h32-{_n}"] = lambda n=_n: write_landing(n, h40=False)
    PICTURES[f"write-landing-off-h32-{_n}"] = lambda n=_n: write_landing(n, display=False, h40=False)

def status_hv(h40):
    """The status register and the HV counter read as one long and stored to one RAM long in a loop, a NOP every other
    round so that the reads drift across the line; the picture's RAM log is the samples (Nephrite_Disputes.md D-12)."""
    tail = w(0x41F9, 0x00C0, 0x0006, 0x47F9, 0x00FF, 0x1000, 0x2690, 0x2690, 0x4E71, 0x60F8)
    return program([0x8004, 0x8144, 0x8230, 0x8407, 0x8578, 0x8700, 0x8C81 if h40 else 0x8C00, 0x8D3F, 0x8F02, 0x9001], [], tail)
PICTURES["status-hv-h40"] = lambda: status_hv(True)
PICTURES["status-hv-h32"] = lambda: status_hv(False)

def status_boot():
    """The status register at power-on, again, after mode 4 with the display off, after mode 5, and again, about four
    frames apart, to RAM from $FF1000 (D-12)."""
    delay = lambda: w(0x203C, 0x0002, 0x0000, 0x5380, 0x66FC)          # move.l #$20000,d0; subq.l #1,d0; bne
    code = w(0x46FC, 0x2700, 0x43F9, 0x00C0, 0x0004, 0x47F9, 0x00FF, 0x1000)
    code += delay() + w(0x36D1) + delay() + w(0x36D1)                   # move.w (a1),(a3)+
    code += w(0x32BC, 0x8004) + delay() + w(0x36D1)
    code += w(0x32BC, 0x8104) + delay() + w(0x36D1)
    code += delay() + w(0x36D1) + w(0x60FE)
    r = bytearray(b"\xff" * 0x1000)
    r[0:8] = w(0x00FF, 0xFE00, 0x0000, 0x0200)
    r[0x100:0x110] = b"SEGA MEGA DRIVE "
    r[0x200:0x200 + len(code)] = code
    return bytes(r)
PICTURES["status-boot"] = status_boot

def fifo_wait_states():
    """VDPFIFOTesting's tenth FIFO Wait States part as its source gives it, 64 times at phases a loop apart: three
    words queued to VRAM, DMA enabled, the status read, a three-word transfer from the cartridge started, then sixteen
    status reads, each stored to RAM from $FF1000 (D-11)."""
    C4, C0 = (0x00C0, 0x0004), (0x00C0, 0x0000)
    loop = w(0x3E06, 0x51CF, 0xFFFE,                                  # move.w d6,d7; dbra d7,itself: the phase
             0x33FC, 0x8144, *C4, 0x23FC, 0x4000, 0x0002, *C4,        # DMA off; VRAM write at $8000
             *([0x33FC, 0xFFFF, *C0] * 3), 0x33FC, 0x8154, *C4,       # three words queued; DMA on
             0x3A39, *C4, 0x36C5,                                     # move.w status,d5; move.w d5,(a3)+
             0x33FC, 0x9303, *C4, 0x33FC, 0x9500, *C4,
             0x28BC, 0x4000, 0x0082, 0x23D4, *C4)                     # move.l #cmd,(a4); move.l (a4),$C00004
    loop += w(*([0x36F9, *C4] * 16))
    loop += w(0x51CE, (-(len(loop) + 2)) & 0xFFFF)                    # dbra d6,the loop
    tail = w(0x47F9, 0x00FF, 0x1000, 0x49F9, 0x00FF, 0x0000, 0x3C3C, 63,
             0x33FC, 0x9400, *C4, 0x33FC, 0x9600, *C4) + loop + w(0x60FE)
    return program([0x8004, 0x8144, 0x8230, 0x8407, 0x8578, 0x8700, 0x8C81, 0x8D3F, 0x8F02, 0x9001], [], tail)
PICTURES["fifo-wait-states"] = fifo_wait_states

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

# ---- sound programs: the board's audio pins or a reference's output measure them (Nephrite_Native.md §18).

def z80_bus():
    """RESET released and the Z80's bus requested and granted, so that the 68000 reaches the YM2612 at $A04000."""
    return w(0x33FC, 0x0100, 0x00A1, 0x1200, 0x33FC, 0x0100, 0x00A1, 0x1100,
             0x0839, 0x0000, 0x00A1, 0x1100, 0x66F6)                       # btst #0,$A11100; bne.s the btst

def dac_square(half=700):
    """Channel 6's DAC from $FF to $00 and back, `half` dbra rounds (ten clocks each) at each level: a square of
    the DAC's whole swing."""
    code = z80_bus() + w(0x45F9, 0x00A0, 0x4000, 0x14BC, 0x002B, 0x157C, 0x0080, 0x0001)   # lea; $2B = $80
    loop = w(0x14BC, 0x002A, 0x157C, 0x00FF, 0x0001, 0x303C, half, 0x51C8, 0xFFFE,
             0x14BC, 0x002A, 0x157C, 0x0000, 0x0001, 0x303C, half, 0x51C8, 0xFFFE)
    loop += w(0x6000, (-(len(loop) + 2)) & 0xFFFF)                                             # bra.w loop
    return program([0x8004, 0x8104, 0x8F02], [], code + loop)

def psg_tone():
    """PSG tone 0 at half-period $0FE (440 Hz on NTSC) at full volume, written through $C00011."""
    return program([0x8004, 0x8104, 0x8F02], [], w(0x47F9, 0x00C0, 0x0011, 0x16BC, 0x008E, 0x16BC, 0x000F,
                                                   0x16BC, 0x0090, 0x60FE))

def dac_ramp():
    """Channel 6's DAC through $00-$FF, each value held about four samples (a dbra of 60 rounds), then again."""
    code = z80_bus() + w(0x45F9, 0x00A0, 0x4000, 0x14BC, 0x002B, 0x157C, 0x0080, 0x0001, 0x7000)          # d0 = 0
    loop = w(0x14BC, 0x002A, 0x1540, 0x0001, 0x323C, 60, 0x51C9, 0xFFFE, 0x5200)                            # $2A = d0; wait; addq.b
    loop += w(0x6000, (-(len(loop) + 2)) & 0xFFFF)
    return program([0x8004, 0x8104, 0x8F02], [], code + loop)

def ym_writes(pairs, part=0):
    """68000 code writing each (register, value) to the YM2612 through $A04000 (a2), part 0 or 1: the busy flag waited
    out first, then the address and the data, then four NOPs. On the board a status read whose bus cycle follows a
    data write's at once leaves 0 in the register at two of every six places in a slot (Nephrite_Disputes.md D-15),
    so the next poll of the busy flag comes no sooner than 16 clocks after."""
    code = bytearray()
    for r, v in pairs:
        code += w(0x4A12, 0x6BFC)                                                   # tst.b (a2); bmi.s the tst
        code += w(0x14BC, r) if part == 0 else w(0x157C, r, 0x0002)
        code += w(0x157C, v, 0x0001 if part == 0 else 0x0003)
        code += w(0x4E71, 0x4E71, 0x4E71, 0x4E71)
    return bytes(code)

def op_alone(test21, test2c=0x10, reads=12000):
    """Channel 1's operator S4 alone at TL 0, its phase one step a sample (block 1, fnum 1,024, multiple 1), the
    other three at TL $7F; then the test registers `$21` and `$2C` set and the status read `reads` times into RAM from
    $FF0000, an unrolled loop of reads 20 clocks apart, which drift across the sample's 24 slots (Nephrite_Disputes.md D-15)."""
    setup = [(0x22, 0), (0x27, 0), (0x28, 0), (0x2B, 0), (0xB0, 0x07), (0xB4, 0xC0)]
    for slot, tl in ((0x00, 0x7F), (0x04, 0x7F), (0x08, 0x7F), (0x0C, 0x00)):
        setup += [(0x30 + slot, 0x01), (0x40 + slot, tl), (0x50 + slot, 0x1F), (0x60 + slot, 0), (0x70 + slot, 0), (0x80 + slot, 0x0F), (0x90 + slot, 0)]
    setup += [(0xA4, 0x0C), (0xA0, 0x00), (0x28, 0xF0), (0x21, test21), (0x2C, test2c)]
    code = z80_bus() + w(0x45F9, 0x00A0, 0x4000, 0x47F9, 0x00FF, 0x0000) + ym_writes(setup)
    code += w(0x303C, reads // 8 - 1)                                               # d0 = rounds
    loop = w(*([0x16D2, 0x4E71, 0x4E71] * 8))                                       # move.b (a2),(a3)+; two nops
    loop += w(0x51C8, (-(len(loop) + 2)) & 0xFFFF)
    return program([0x8004, 0x8104, 0x8F02], [], code + loop + w(0x60FE))

def status_loop(writes, reads=1536, nops=0):
    """The YM2612 given `writes` (register, value), the last of them `nops` NOPs after its wait for the busy flag,
    then its status read `reads` times into RAM from $FF0000, a read every 23 clocks, which drift across the sample:
    what the timers' flags and the busy flag read, and when (Nephrite_Native.md section 24)."""
    last = ym_writes(writes[-1:])
    code = z80_bus() + w(0x45F9, 0x00A0, 0x4000, 0x47F9, 0x00FF, 0x0000) + ym_writes(writes[:-1]) + last[:4] + w(*[0x4E71] * nops) + last[4:]
    code += w(0x303C, reads - 1, 0x16D2, 0x51C8, 0xFFFC)                             # move.b (a2),(a3)+; dbra d0
    return program([0x8004, 0x8104, 0x8F02], [], code + w(0x60FE))

def fm_voice(ops, alg=7, fb=0, fnum=1024, block=1, channel=0, keys=0xF0, extra=(), lfo=0, b4=0xC0, pre=(), ssg=(0, 0, 0, 0), mode=0, pre_nops=0, key_nops=0):
    """One FM channel's voice written through $A04000, keyed on, then the 68000 idles: `ops` gives S1-S4 as
    (dt_mul, tl, ks_ar, am_dr, sr, sl_rr); the channel's algorithm, feedback, frequency, the LFO's register `$22` and
    the channel's `$B4` (panning, AMS, PMS), S1-S4's SSG-EG registers `ssg`, `$27` as `mode`, any `pre` writes (part 0)
    before the key-on, and any `extra` writes after the key-on, each (register, value), (register, value, thousands
    of dbra rounds of 10 clocks to wait first) or the same with a count of NOPs before the write
    (Nephrite_Native.md §19, §23, §24)."""
    part, i = channel // 3, channel % 3
    pairs = [(0x22, lfo), (0x27, mode), (0x28, 0), (0x2B, 0)]
    regs = [(0xB0 + i, fb << 3 | alg), (0xB4 + i, b4)]
    for slot, op, eg in zip((0x00, 0x08, 0x04, 0x0C), ops, ssg):
        for base, v in zip((0x30, 0x40, 0x50, 0x60, 0x70, 0x80), op):
            regs.append((base + slot + i, v))
        regs.append((0x90 + slot + i, eg))
    regs += [(0xA4 + i, block << 3 | fnum >> 8), (0xA0 + i, fnum & 0xFF)]
    code = z80_bus() + w(0x45F9, 0x00A0, 0x4000) + ym_writes(pairs) + ym_writes(regs, part) + ym_writes(list(pre)[:-1])
    # `pre_nops` NOPs before the last of the `pre` writes and `key_nops` before the key-on, each after the write's wait
    # for the busy flag, move it across the sample.
    late = lambda pairs, nops: b"".join(ym_writes([pr])[:4] + w(*[0x4E71] * nops) + ym_writes([pr])[4:] for pr in pairs)
    code += late(list(pre)[-1:], pre_nops) + late([(0x28, keys | (part << 2 | i))], key_nops)
    for e in extra:
        if len(e) >= 3:
            code += w(0x303C, e[2] * 1000) + w(0x51C8, 0xFFFE)     # about e[2] * 1,000 rounds of 10 clocks
        # A fourth entry is NOPs after the write's wait for the busy flag, which move it across the sample.
        one = ym_writes([e[:2]], part if e[0] >= 0x30 else 0)
        code += one[:4] + w(*[0x4E71] * (e[3] if len(e) > 3 else 0)) + one[4:]
    return program([0x8004, 0x8104, 0x8F02], [], code + w(0x60FE))

def z80_writer(writes, wait=40, then=None):
    """A program in which the Z80 writes each (part, register, value) to the YM2612 at $4000-$4003, with a `wait`
    djnz loop after each, then halts; the 68000 loads it into the Z80's RAM, releases the Z80 and runs `then`
    (an idle loop by default). The Z80's reset line resets the YM2612 too, so a pulse of it replays the program on a
    chip at rest (Nephrite_Disputes.md D-19)."""
    z = bytearray()
    for part, r, v in writes:
        z += bytes([0x3E, r, 0x32, 0x00 | 2 * part, 0x40, 0x3E, v, 0x32, 0x01 | 2 * part, 0x40])   # ld a,r; ld ($4000+2p),a ...
        z += bytes([0x06, wait, 0x10, 0xFE])                                                     # ld b,wait; djnz $
    z += bytes([0x76])                                                                           # halt
    code = w(0x33FC, 0x0100, 0x00A1, 0x1100, 0x33FC, 0x0100, 0x00A1, 0x1200,                    # BUSREQ, RESET released
             0x0839, 0x0000, 0x00A1, 0x1100, 0x66F6, 0x41FA, 0x0000)                             # wait; lea z(pc),a0
    lea_at = len(code) - 2
    code += w(0x43F9, 0x00A0, 0x0000, 0x303C, len(z) - 1, 0x12D8, 0x51C8, 0xFFFC)               # copy to $A00000
    code += w(0x33FC, 0x0000, 0x00A1, 0x1200, 0x33FC, 0x0000, 0x00A1, 0x1100,                   # RESET, BUSREQ off
              0x33FC, 0x0100, 0x00A1, 0x1200) + (then if then is not None else w(0x60FE))         # RESET released
    code = bytearray(code)
    z_at = len(code)
    code[lea_at:lea_at + 2] = (z_at - lea_at).to_bytes(2, "big")
    return program([0x8004, 0x8104, 0x8F02], [], bytes(code + z))

def fm_writes(ops, alg=7, fb=0, fnum=1024, block=1, channel=0, keys=0xF0, extra=()):
    """`fm_voice`'s writes as (part, register, value), for the Z80 writer."""
    part, i = channel // 3, channel % 3
    out = [(0, 0x22, 0), (0, 0x27, 0), (0, 0x2B, 0), (part, 0xB0 + i, fb << 3 | alg), (part, 0xB4 + i, 0xC0)]
    for slot, op in zip((0x00, 0x08, 0x04, 0x0C), ops):
        for base, v in zip((0x30, 0x40, 0x50, 0x60, 0x70, 0x80), op):
            out.append((part, base + slot + i, v))
        out.append((part, 0x90 + slot + i, 0))
    out += [(part, 0xA4 + i, block << 3 | fnum >> 8), (part, 0xA0 + i, fnum & 0xFF), (0, 0x28, keys | (part << 2 | i))]
    return out + [(part if r >= 0x30 else 0, r, v) for r, v in extra]

SINE = (0x01, 0x00, 0x1F, 0x00, 0x00, 0x0F)
QUIET = (0x01, 0x7F, 0x1F, 0x00, 0x00, 0x0F)

def rate_voice(rate, attack=False):
    """S4 alone with key scaling 3 at block 4, where the envelope's rate is twice the register plus the key code (18
    at frequency 1,081, 19 at 1,152): its decay at `rate` from an instant attack, or its attack at `rate` to a held
    level (Nephrite_Disputes.md D-18)."""
    r, fnum = (rate - 18) // 2, 1152 if rate & 1 else 1081
    op = (0x01, 0x00, 0xC0 | r, 0x00, 0x00, 0x0F) if attack else (0x01, 0x00, 0xDF, r, 0x00, 0xFF)
    return fm_voice((QUIET, QUIET, QUIET, op), fnum=fnum, block=4)

# (S3's multiple, its TL and decay rate, S4's multiple, frequency, block): S3 at or near full level, decaying, into S4.
ROM_VOICES = [(1, 0, 16, 1, 1081, 4), (1, 0, 16, 2, 1081, 4), (2, 0, 17, 1, 1081, 4), (1, 1, 16, 3, 1081, 4), (3, 0, 15, 0, 1081, 4),
              (1, 0, 16, 1, 1151, 3), (1, 0, 17, 2, 777, 4), (2, 2, 16, 1, 1999, 2), (1, 3, 15, 5, 613, 5), (1, 0, 18, 7, 1333, 3)]

def rom_voice(i):
    """Algorithm 4's second pair, a decaying S3 modulating S4: S3's every output bit moves S4's phase, which is what
    lets the nine-bit pins say what the two ROMs hold (Nephrite_Disputes.md D-16)."""
    mm, tl, dr, cm, fnum, block = ROM_VOICES[i]
    return fm_voice((QUIET, QUIET, (mm, tl, 0x1F, dr, 0x00, 0xFF), (cm, 0x00, 0x1F, 0, 0, 0x0F)), alg=4, fnum=fnum, block=block)

TRIALS, TRIAL_MARK = 49, 0x7000

def read_after_write(nops=0, dac=False, timers=False):
    """A Z80 program sets a voice on channel 1 (or, with `dac`, the 68000 enables the DAC); then the 68000, 49
    times: `p` NOPs to move along the sample, a mark in RAM ($7000 + p at $FF0006), a register's address and data
    written, `nops` NOPs, the status read, and a wait. The register is S4's TL, $10 and $18 in turn, or the DAC's
    data, $90 + p; with `timers` both timers have overflowed first, so that the status reads $03
    (Nephrite_Disputes.md D-15)."""
    tail = w(0x303C, 8000, 0x51C8, 0xFFFE)                                                # the Z80's writes finish
    tail += w(0x33FC, 0x0100, 0x00A1, 0x1100, 0x0839, 0x0000, 0x00A1, 0x1100, 0x66F6, 0x45F9, 0x00A0, 0x4000)
    if dac:
        tail += ym_writes([(0x2B, 0x80), (0x2A, 0x80)])
    if timers:
        tail += ym_writes([(0x24, 0xFF), (0x25, 0x03), (0x26, 0xFF), (0x27, 0x0F)]) + w(0x303C, 3000, 0x51C8, 0xFFFE)
    for p in range(TRIALS):
        r, v = (0x2A, 0x90 + p) if dac else (0x4C, 0x18 if p & 1 else 0x10)
        tail += w(*([0x4E71] * p)) + w(0x33FC, TRIAL_MARK + p, 0x00FF, 0x0006)
        tail += w(0x14BC, r, 0x157C, v, 0x0001) + w(*([0x4E71] * nops)) + w(0x4A12)       # address; data; tst.b (a2)
        tail += w(0x303C, 800, 0x51C8, 0xFFFE)
    tail += w(0x33FC, TRIAL_MARK + TRIALS, 0x00FF, 0x0006, 0x60FE)
    voice = (QUIET, QUIET, QUIET, QUIET if dac else SINE)
    return z80_writer(fm_writes(voice, keys=0x00 if dac else 0x80, fnum=1081, block=6), then=tail)

def reset_replay(rounds=200, gaps=(20000, 20555)):
    """A Z80 program plays S4's attack at AR 12; after each of `gaps` (dbra rounds of ten clocks) the 68000 holds the
    Z80's reset for `rounds` more, or for 40 NOPs if that is 0, and releases it, so that the Z80 writes the voice
    again (Nephrite_Disputes.md D-19)."""
    hold = w(0x323C, rounds, 0x51C9, 0xFFFE) if rounds else w(*([0x4E71] * 40))
    pulse = w(0x33FC, 0x0000, 0x00A1, 0x1200) + hold + w(0x33FC, 0x0100, 0x00A1, 0x1200)
    tail = b"".join(w(0x303C, g, 0x51C8, 0xFFFE) + pulse for g in gaps) + w(0x60FE)
    return z80_writer(fm_writes((QUIET, QUIET, QUIET, (0x01, 0x00, 0x0C, 0, 0, 0x0F)), **A4), then=tail)

def status_ports():
    """Both timers overflowed, so that the status reads $03; then a data write and, 8 clocks on, ports 0 to 3 read
    into $FF0000-$FF0006 while the busy flag stands; then, long after, the four read again into $FF0008-$FF000E
    (Nephrite_Disputes.md D-13)."""
    code = z80_bus() + w(0x45F9, 0x00A0, 0x4000) + ym_writes([(0x24, 0xFF), (0x25, 0x03), (0x26, 0xFF), (0x27, 0x0F)])
    code += w(0x303C, 3000, 0x51C8, 0xFFFE, 0x4A12, 0x6BFC, 0x14BC, 0x002A, 0x157C, 0x0080, 0x0001, 0x4E71, 0x4E71)
    reads = lambda at: w(0x13D2, 0x00FF, at) + b"".join(w(0x13EA, port, 0x00FF, at + 2 * port) for port in (1, 2, 3))
    return program([0x8004, 0x8104, 0x8F02], [], code + reads(0x0000) + w(0x303C, 300, 0x51C8, 0xFFFE) + reads(0x0008) + w(0x60FE))

def reset_sweep(first=60, step=4, count=9, gap=12000):
    """A Z80 program plays S4's decay at rate 39 from an instant attack; the 68000 then pulses the Z80's reset `count`
    times, `gap` dbra rounds apart, holding it for `first`, `first + step`, ... NOPs, so that each release, and the
    key-on 339 master clocks further round the sample, falls somewhere else in the chip's cycle (Nephrite_Disputes.md
    D-19)."""
    tail = b""
    for k in range(count):
        tail += w(0x303C, gap, 0x51C8, 0xFFFE, 0x33FC, 0x0000, 0x00A1, 0x1200) + w(*([0x4E71] * (first + k * step)))
        tail += w(0x33FC, 0x0100, 0x00A1, 0x1200)
    return z80_writer(fm_writes((QUIET, QUIET, QUIET, (0x01, 0x00, 0xDF, 10, 0x00, 0xFF)), fnum=1152, block=4), then=tail + w(0x60FE))

# ---- the 68000's bus against the refreshes (Nephrite_Disputes.md D-2): unrolled loops over the main RAM at $FF0100.
RAM_LOOPS = {"ram-reads": ([0x3013], 400), "ram-longs": ([0x2013], 300), "ram-reads-3": ([0x3013, 0x4E71, 0x4E71, 0x4E71], 400),
             "ram-reads-6": ([0x3013] + [0x4E71] * 6, 228)}

def ram_loop(name):
    """`move.w (a3),d0` or `move.l (a3),d0` unrolled, some with NOPs between, then an idle loop."""
    words, count = RAM_LOOPS[name]
    return program([0x8004, 0x8104, 0x8F02], [], w(0x47F9, 0x00FF, 0x0100) + w(*(words * count)) + w(0x60FE))

def fetches(lines):
    """The cartridge reads' starts in master clocks from the 68000's first, each read once (a repeated strobe within
    35 master clocks is the same read): [(time, address)]."""
    ev = [(int(l[1]), int(l[2], 16)) for l in lines if l[0] == "c"]
    start = min(t for t, a in ev if t > 1000000)
    out = []
    for t, a in [((t - start) / 2, a) for t, a in ev if t >= start]:
        if out and out[-1][1] == a and t - out[-1][0] <= 35:
            out[-1] = (t, a)
        else:
            out.append((t, a))
    return out

def board_bus(logs):
    """Nephrite's `board_bus.rs`: for each of `RAM_LOOPS`, its program's hash and the starts of its loop's cartridge
    reads on the board, from the logs `<name>.lines` in the directory `logs` (made if missing)."""
    at = lambda name: os.path.join(os.path.abspath(logs), name)
    out = [BOARD_BUS_HEAD]
    for name in RAM_LOOPS:
        lines = logged(at(name + ".lines"), lambda: [l for l in bench(ram_loop(name), 1600000) if l[0] == "c"])
        body = [int(t) for t, a in fetches(lines) if 0x222 <= a < 0x222 + 2 * len(RAM_LOOPS[name][0]) * RAM_LOOPS[name][1]]
        rows = "\n".join("        " + " ".join(f"{t}," for t in body[i:i + 14]) for i in range(0, len(body), 14))
        out.append(f'    BoardLoop {{ name: "{name}", image: 0x{fnv(ram_loop(name)):08x}, fetches: &[\n{rows}\n    ] }},')
    return "\n".join(out) + "\n];\n"

BOARD_BUS_HEAD = '''//! The board's cartridge reads in `mdboard.py`'s RAM loops, written by `mdboard.py board-bus`: the start of each read
//! of the loop, in master clocks from the 68000's first bus cycle (Nephrite_Disputes.md D-2, Nephrite_Native.md §22).

pub struct BoardLoop {
    pub name: &'static str,
    pub image: u32,
    pub fetches: &'static [u32],
}

pub const LOOPS: &[BoardLoop] = &['''

# ---- the board's sound pins read back (Nephrite_Disputes.md D-16): an FM sample is 2,016 MCLK2 cycles of 24 slots.
SAMPLE, SLOT = 2016, 84

def bench(image, cycles, audio=None, zbus=False, pins=False, mol=None, status_enable=None):
    """The board run on `image`, with its sound pins logged to `audio`, both forms of the FM output's left side to
    `mol`, the Z80 bus printed (b lines), the Z80's reset and bus acknowledge printed (z lines) and the board's
    `ym2612_status_enable` input set, as asked; its output's lines, split."""
    path = os.path.join(WORK, "program.bin")
    open(path, "wb").write(image)
    env = dict(os.environ, LD_LIBRARY_PATH=LIB)
    if audio: env["TB_AUDIO"] = audio
    if zbus: env["TB_ZBUS"] = f"0:{cycles}"
    if pins: env["TB_PINS"] = "1"
    if mol: env["TB_MOL"] = mol
    if status_enable is not None: env["TB_YMSTATUS"] = str(status_enable)
    out = subprocess.run([os.path.join(WORK, "tb_md"), path, str(cycles)], capture_output=True, text=True, cwd=WORK, env=env, check=True).stdout
    return [line.split() for line in out.splitlines()]

def pin_records(path):
    """The log's records: (cycle, MOL_2612, MOR_2612, PSG), one at each change."""
    import struct
    d = open(path, "rb").read()
    return [struct.unpack("<QHHH", d[i:i + 14]) for i in range(0, len(d) - 13, 14)]

def fm_level(pin):
    """A ten-bit FM pin as the channel's signed nine-bit output: the pins carry it one higher when it is not negative."""
    return pin - 1 if pin < 512 else pin - 1024

def channel_slot(recs, origin=0):
    """Where in the sample, counted from `origin`, the pins take the most different values: the busiest channel's turn."""
    seen = {}
    for t, mol, _, _ in recs:
        seen.setdefault((t - origin) % SAMPLE, set()).add(mol)
    return max(seen, key=lambda k: len(seen[k])) if seen else None

def fm_series(recs, slot, origin=0):
    """The channel whose turn comes `slot` cycles into each sample from `origin`: {sample number: level} where it shows."""
    return {(t - origin) // SAMPLE: fm_level(mol) for t, mol, _, _ in recs if (t - origin) % SAMPLE == slot}

def off_rest(xs):
    """The first sample that is neither 0 nor -1."""
    return next(i for i, x in enumerate(xs) if x not in (0, -1))

def board_channel(log, settle=1300000, end=None):
    """The board's busiest channel a sample at a time from cycle `settle` to `end`, read a cycle into its turn: (its
    turn's place, its levels)."""
    held = [r for r in pin_records(log) if end is None or r[0] < end]
    recs = [r for r in held if r[0] > settle]
    slot = channel_slot(recs)
    board, i = [], 0
    k = recs[0][0] // SAMPLE
    while k * SAMPLE + slot + 1 < (end or held[-1][0]):
        t = k * SAMPLE + slot + 1
        while i + 1 < len(held) and held[i + 1][0] <= t:
            i += 1
        board.append(fm_level(held[i][1]))
        k += 1
    return slot, board

def fnv(data):
    h = 0x811C9DC5
    for b in data:
        h = ((h ^ b) * 0x01000193) & 0xFFFFFFFF
    return h

FIXTURE_BLOCK = 64

def block_hashes(series, indent="        "):
    """The FNV-1a hash of each block of 64 levels, as little-endian i16, ten to a row of Rust."""
    import struct
    blocks = [fnv(struct.pack("<%dh" % len(series[i:i + FIXTURE_BLOCK]), *series[i:i + FIXTURE_BLOCK])) for i in range(0, len(series), FIXTURE_BLOCK)]
    return "\n".join(indent + " ".join(f"0x{h:08x}," for h in blocks[i:i + 10]) for i in range(0, len(blocks), 10))

def edges(lines):
    """The Z80 reset line's assertions and releases among the bench's z lines."""
    z = [(int(l[1]), l[2]) for l in lines if l[0] == "z"]
    return ([t for (t, v), (_, was) in zip(z[1:], z) if v == "1" and was == "0"],
            [t for (t, v), (_, was) in zip(z[1:], z) if v == "0" and was == "1"])

def ym_strobes(lines, start, end):
    """The YM2612's writes among the bench's b lines from `start` to `end`: (the cycle its strobe rises, port, value)."""
    bus = [(int(l[1]), int(l[3]), int(l[4], 16), int(l[5], 16)) for l in lines if l[0] == "b" and start <= int(l[1]) < end]
    return [(e[0], was[2] & 3, was[3]) for was, e in zip(bus, bus[1:]) if e[1] == 1 and was[1] == 0 and 0x4000 <= was[2] < 0x4004]

# The samples of each voice Nephrite's `board_fm.rs` holds, from its first off rest; `fm-envelope` to before its key-off.
VOICE_SAMPLES = {"fm-attack": 5800, "fm-envelope": 2900, "fm-envelope-ks": 3000}
BOARD_VOICES = (["fm-sine", "fm-tl16", "fm-mul3", "fm-dt7", "fm-feedback5", "fm-chain"] + [f"fm-alg{a}" for a in range(1, 8)]
                + ["fm-attack", "fm-envelope", "fm-envelope-ks"] + [f"fm-dr{r}" for r in (38, 39, 42, 43, *range(46, 62))]
                + [f"fm-ar{r}" for r in range(44, 60)] + [f"fm-rom{i}" for i in range(len(ROM_VOICES))])

def logged(path, make):
    """The bench's output lines kept at `path`, made by `make` if they are not there."""
    if not os.path.exists(path):
        open(path, "w").write("\n".join(" ".join(l) for l in make()) + "\n")
    return [l.split() for l in open(path)]

def board_fm(logs):
    """Nephrite's `board_fm.rs`, from the board's logs in the directory `logs`, each made here if it is missing: every
    voice's pins (`<name>.pins`, 12,000,000 cycles; 10,000,000 for the `fm-rom` voices), one voice's bus (`voice.bus`)
    and four reset sweeps' pins and buses (`reset-sweep<n>`). Nephrite_Native.md section 21."""
    at = lambda name: os.path.join(os.path.abspath(logs), name)
    out = [BOARD_FM_HEAD]
    lines = logged(at("voice.bus"), lambda: bench(SOUNDS["fm-dr39"](), 1750000, zbus=True, pins=True))
    release = edges(lines)[1][0]
    strobes = ym_strobes(lines, release, 1750000)
    assert len(strobes) == 74
    out.append(f"pub const VOICE_RELEASE: u64 = {release};")
    out.append("pub const VOICE_WRITES: [u64; 74] = [\n" + "\n".join("    " + " ".join(f"{t}," for t, _, _ in strobes[i:i + 12]) for i in range(0, 74, 12)) + "\n];\n")
    out.append("pub const VOICES: &[BoardVoice] = &[")
    for name in BOARD_VOICES:
        if not os.path.exists(at(name + ".pins")):
            bench(SOUNDS[name](), 10000000 if name.startswith("fm-rom") else 12000000, audio=at(name + ".pins"))
        _, board = board_channel(at(name + ".pins"))
        b0 = off_rest(board)
        series = board[b0:b0 + VOICE_SAMPLES.get(name, 3000 if name[3:5] in ("dr", "ar") else 5191)]
        out.append(f'    BoardVoice {{ name: "{name}", image: 0x{fnv(SOUNDS[name]()):08x}, samples: {len(series)}, blocks: &[\n{block_hashes(series)}\n    ] }},')
    out.append("];\n")
    replays, values = [], None
    for n in range(4):
        cycles = 19000000
        lines = logged(at(f"reset-sweep{n}.bus"), lambda: [l for l in bench(reset_sweep(60 + n), cycles, audio=at(f"reset-sweep{n}.pins"), zbus=True, pins=True) if l[0] in "zb"])
        asserts, releases = edges(lines)
        for i, (a, r) in enumerate(zip(asserts, releases[1:])):
            end = asserts[i + 1] if i + 1 < len(asserts) else cycles
            strobes = ym_strobes(lines, r, end)
            values = values or [(p, v) for _, p, v in strobes]
            assert values == [(p, v) for _, p, v in strobes]
            _, board = board_channel(at(f"reset-sweep{n}.pins"), settle=r, end=end)
            series = board[off_rest(board):][:REPLAY_SAMPLES]
            assert len(series) == REPLAY_SAMPLES
            replays.append((r - a, [t - r for t, _, _ in strobes], series))
    out.append(board_lfo(at))
    out.append("pub const REPLAY_WRITES: [(u8, u8); 72] = [\n" + "\n".join("    " + " ".join(f"({p}, 0x{v:02x})," for p, v in values[i:i + 12]) for i in range(0, 72, 12)) + "\n];\n")
    out.append("pub const REPLAYS: &[BoardReplay] = &[")
    for held, times, series in sorted(replays):
        out.append(f"    BoardReplay {{ held: {held}, times: &[\n" + "\n".join("        " + " ".join(f"{t}," for t in times[i:i + 12]) for i in range(0, 72, 12))
                   + f"\n    ], blocks: &[\n{block_hashes(series)}\n    ] }},")
    out.append("];")
    return "\n".join(out) + "\n"

REPLAY_SAMPLES = 512

# The LFO's voices (Nephrite_Native.md section 23): the slow ones run 36,000,000 cycles and keep 17,000 samples.
LFO_SLOW = ("fm-lfo0", "fm-lfo1", "fm-lfo2", "fm-lfo3", "fm-lfo4", "fm-lfo5", "fm-lfo-off", "fm-lfo-restart")

def lfo_voice_names():
    return [n for n in SOUNDS if n.startswith(("fm-lfo", "fm-ams", "fm-am-", "fm-pms", "fm-pm-"))]

def lfo_channel(name):
    """The channel a voice plays on: `fm-pm-c<n>s<k>` and `fm-am-c<n>s<k>` name it, `fm-pm-ch3` is channel 3."""
    import re
    m = re.match(r"fm-(?:pm|am)-c(\d)s", name)
    return int(m.group(1)) - 1 if m else 2 if name.startswith("fm-pm-ch3") else 0

def board_lfo(at):
    """The LFO's part of `board_fm.rs`: each LFO voice's channel and samples, its pins made if missing."""
    out = ["\npub const LFO_VOICES: &[BoardLfoVoice] = &["]
    for name in lfo_voice_names():
        slow = name in LFO_SLOW
        if not os.path.exists(at(name + ".pins")):
            bench(SOUNDS[name](), 36000000 if slow else 19000000, audio=at(name + ".pins"))
        _, board = board_channel(at(name + ".pins"))
        b0 = off_rest(board)
        series = board[b0:b0 + (17000 if slow else 8600)]
        out.append(f'    BoardLfoVoice {{ name: "{name}", image: 0x{fnv(SOUNDS[name]()):08x}, channel: {lfo_channel(name)}, samples: {len(series)}, blocks: &[\n{block_hashes(series)}\n    ] }},')
    return "\n".join(out) + "\n];\n"
BOARD_FM_HEAD = '''//! The board's FM pins for `mdboard.py`'s voices and reset sweeps, written by `mdboard.py board-fm`. Times are
//! MCLK2 cycles, two to a master clock; samples are a channel's levels from its first that is neither 0 nor -1, held
//! as the FNV-1a hash of each block of 64 as little-endian i16 (Nephrite_Disputes.md D-16 to D-19,
//! Nephrite_Native.md §21).

/// A voice the 68000 writes after power-on: its program's hash, and the samples of channel 1 held.
pub struct BoardVoice {
    pub name: &'static str,
    pub image: u32,
    pub samples: usize,
    pub blocks: &'static [u32],
}

/// A voice for the LFO, on `channel` (0-5): its program's hash and the samples of that channel held.
pub struct BoardLfoVoice {
    pub name: &'static str,
    pub image: u32,
    pub channel: usize,
    pub samples: usize,
    pub blocks: &'static [u32],
}

/// A voice the Z80 writes again after a pulse of its reset line held `held` cycles: the cycle after the release at
/// which each of `REPLAY_WRITES` has its strobe rise, and 512 samples of channel 1.
pub struct BoardReplay {
    pub held: u64,
    pub times: &'static [u64],
    pub blocks: &'static [u32],
}

pub const BLOCK: usize = 64;

/// Every 68000-written voice makes the same 37 writes at the same cycles from power-on, where the reset line starts
/// asserted: the line's release, and each write's address strobe and data strobe as they rise.'''

def fm_compare(log, trace, channel=0, settle=1300000):
    """The board's busiest channel against Nephrite's `fmtrace` output (six little-endian i16 a sample), each from its
    first sample that is neither 0 nor -1: (samples compared, samples equal, the first differences)."""
    import struct
    slot, board = board_channel(log, settle)
    raw = open(trace, "rb").read()
    neph = list(struct.unpack("<%dh" % (len(raw) // 2), raw))[channel::6]
    b0, n0 = off_rest(board), off_rest(neph)
    n = min(len(board) - b0, len(neph) - n0)
    diffs = [(j, board[b0 + j], neph[n0 + j]) for j in range(n) if board[b0 + j] != neph[n0 + j]]
    return slot, n, n - len(diffs), diffs[:5]

def trial_marks(lines):
    return [int(l[1]) for l in lines if l[0] == "w" and l[2] == "04006" and int(l[3], 16) >= TRIAL_MARK]

def read_after_write_report(nops=0, dac=False, timers=False, cycles=12000000):
    """Runs `read_after_write` and prints, for each place in a slot the read's strobe ended at, how many trials kept
    the written value, how many were left with 0 and how many with anything else."""
    log = os.path.join(WORK, "audio.log")
    lines = bench(read_after_write(nops, dac, timers), cycles, audio=log, zbus=True, pins=True)
    recs, marks = pin_records(log), trial_marks(lines)
    z = [(int(l[1]), l[2]) for l in lines if l[0] == "z"]
    asserted = [t for (t, v), (_, was) in zip(z[1:], z) if v == "1" and was == "0"][-1]
    bus = [(int(l[1]), l[2], l[3]) for l in lines if l[0] == "b"]
    slot = (asserted + CHANNEL_1) % SAMPLE if not dac else channel_slot([r for r in recs if r[0] > marks[0]])
    tally, gaps = {}, set()
    for p, (a, b) in enumerate(zip(marks, marks[1:])):
        ev = [e for e in bus if a <= e[0] < b]
        wr_rise = [e[0] for e, was in zip(ev[1:], ev) if e[2] == "1" and was[2] == "0"][1]
        rd_fall = [e[0] for e, was in zip(ev[1:], ev) if e[1] == "0" and was[1] == "1"][0]
        rd_rise = [e[0] for e, was in zip(ev[1:], ev) if e[1] == "1" and was[1] == "0"][0]
        seen = [fm_level(mol) for t, mol, _, _ in recs if a + 16000 <= t < b and t % SAMPLE == slot]
        if dac:
            outcome = 1 if -256 in seen else 0 if 2 * (0x10 + p) in seen else 2
        else:                                                    # a sine's peak: 63 at TL $10, 31 at $18, 255 at 0
            outcome = 1 if max(seen) >= 250 else 0 if abs(max(seen) - (31 if p & 1 else 63)) <= 2 else 2
        place = (rd_rise - asserted - CHANNEL_1) % SLOT
        tally.setdefault(place, [0, 0, 0])[outcome] += 1
        gaps.add(rd_fall - wr_rise)
    print(f"{'the DAC data' if dac else 'TL'} written, {nops} NOPs, then the status read{' of $03' if timers else ''}; the read's strobe falls {sorted(gaps)} cycles after the write's rises")
    for place in sorted(tally):
        kept, zero, other = tally[place]
        print(f"  read ending {place:2d} cycles into a slot: {kept:2d} kept, {zero:2d} left with 0, {other:2d} with something else")

# Channel 1's turn on the pins comes this many cycles after the Z80's reset line is asserted, in every sample.
CHANNEL_1 = 1226

def reset_replay_report(rounds=200, cycles=11500000):
    """Runs `reset_replay` and prints, for each release of the reset, where channel 1's turn falls after the pulse's
    start and after its end, and the first sample of the replayed attack that leaves rest."""
    log = os.path.join(WORK, "audio.log")
    lines = bench(reset_replay(rounds), cycles, audio=log, pins=True)
    recs = pin_records(log)
    z = [(int(l[1]), l[2]) for l in lines if l[0] == "z"]
    asserts = [t for (t, v), (_, was) in zip(z[1:], z) if v == "1" and was == "0"]
    releases = [t for (t, v), (_, was) in zip(z[1:], z) if v == "0" and was == "1"][1:]
    first = None
    for i, (a, r) in enumerate(zip(asserts, releases)):
        end = asserts[i + 1] if i + 1 < len(asserts) else cycles
        seg = [x for x in recs if r < x[0] < end]
        slot = channel_slot(seg, r)
        series = fm_series(seg, slot, r) if seg else {}
        if len(series) < 50:
            print(f"  reset held {(r - a) / SAMPLE:5.2f} samples: the voice does not sound again")
            continue
        first = first or series
        common = sorted(set(first) & set(series))
        print(f"  reset held {(r - a) / SAMPLE:5.2f} samples: channel 1 comes {(r + slot - a) % SAMPLE} cycles after the pulse's start, "
              f"{slot} after its end; the attack leaves rest {min(series)} samples after the release; "
              f"{sum(first[j] == series[j] for j in common)} of {len(common)} samples as the first replay's")

# ---- the LFO's vibrato on the board (Nephrite_LfoTables.md): six channels, S4 alone in each, at chosen frequencies.

YM_TABLE = 0x4000
# The 68000's loop over (part << 8 | register, value) words at a3: busy wait, write, four NOPs; $FFFF waits, $FFFE ends.
YM_TABLE_LOOP = w(0x321B, 0x341B, 0x4A41, 0x6B24, 0x4A12, 0x6BFC, 0x0801, 0x0008, 0x6608, 0x1481, 0x1542, 0x0001,
                  0x6008, 0x1541, 0x0002, 0x1542, 0x0003, 0x4E71, 0x4E71, 0x4E71, 0x4E71, 0x60D4, 0x0C41, 0xFFFE,
                  0x6706, 0x51CA, 0xFFFE, 0x60C8, 0x60FE)
CHANNEL_KEYS = (0, 1, 2, 4, 5, 6)

def ym_table(entries):
    """The 68000 writes the YM2612 from a table: (part, register, value) entries, or (None, n) for n + 1 dbra rounds."""
    table = b"".join(w(0xFFFF, e[1]) if e[0] is None else w(e[0] << 8 | e[1], e[2]) for e in entries) + w(0xFFFE, 0)
    code = z80_bus() + w(0x45F9, 0x00A0, 0x4000, 0x47F9, 0x0000, YM_TABLE) + YM_TABLE_LOOP
    image = bytearray(program([0x8004, 0x8104, 0x8F02], [], code))
    image += b"\xff" * max(0, YM_TABLE + len(table) - len(image))
    image[YM_TABLE:YM_TABLE + len(table)] = table
    return bytes(image)

def lfo_sweep(configs, rate=7, block=7, hold=12000, mul=1, ams=0, enable_late=False, extra=(), amon=False):
    """Six channels of S4 alone, the LFO on, each config's (frequency, PMS) keyed and held (Nephrite_LfoTables.md §3.1)."""
    e = [(0, 0x22, 0), (0, 0x27, 0), (0, 0x2B, 0)] + [(0, 0x28, k) for k in CHANNEL_KEYS]
    for c in range(6):
        part, i = c // 3, c % 3
        e.append((part, 0xB0 + i, 0x07))
        for slot, op in zip((0x00, 0x08, 0x04, 0x0C), (QUIET, QUIET, QUIET, (mul, 0x00, 0x1F, 0x80 if amon else 0, 0, 0x0F))):
            for base, v in zip((0x30, 0x40, 0x50, 0x60, 0x70, 0x80), op):
                e.append((part, base + slot + i, v))
            e.append((part, 0x90 + slot + i, 0))
    if not enable_late:
        e.append((0, 0x22, 0x08 | rate))
    for n, config in enumerate(configs):
        for c, (fnum, pms) in enumerate(config):
            part, i = c // 3, c % 3
            e += [(part, 0xB4 + i, 0xC0 | ams << 4 | pms), (part, 0xA4 + i, block << 3 | fnum >> 8), (part, 0xA0 + i, fnum & 0xFF)]
        e += [(0, 0x28, k) for k in CHANNEL_KEYS] + [(0, 0x28, 0x80 | k) for k in CHANNEL_KEYS]
        if enable_late and n == 0:
            e.append((0, 0x22, 0x08 | rate))
        e.append((None, hold))
    return ym_table(e + list(extra))

# ---- the vibrato read back (Nephrite_LfoTables.md): each channel's frequency number recovered sample by sample.

TURN_ORDER = (0, 4, 2, 1, 5, 3)   # the channel whose turn is the k-th of six, 336 cycles apart from channel 1's
LFO_SHARD, LFO_HOLD, LFO_STEP = 240, 10000, 20   # items a run, dbra rounds a configuration, samples a PM step at rate 7

def channel_levels(log, slot1, samples):
    """Every channel's level for samples 0 to `samples`, each read a cycle into its turn, channel 1's at `slot1`."""
    import bisect
    recs = pin_records(log)
    times = [r[0] for r in recs]
    out = [None] * 6
    for k, c in enumerate(TURN_ORDER):
        out[c] = [fm_level(recs[max(0, bisect.bisect_right(times, s * SAMPLE + slot1 + 336 * k + 1) - 1)][1]) for s in range(samples)]
    return out

def carrier_levels():
    """A lone carrier at TL 0: its nine-bit level for each ten-bit phase, from the operator tables' formulas."""
    import math
    ls = [round(-math.log2(math.sin((x + 0.5) * math.pi / 512)) * 256) for x in range(256)]
    ex = [round((2 ** (x / 256) - 1) * 1024) for x in range(256)]
    def level(p):
        i = 0xFF - (p & 0xFF) if p & 0x100 else p & 0xFF
        mag = ((ex[255 - (ls[i] & 0xFF)] + 1024) << 2) >> (ls[i] >> 8)
        return (-mag if p & 0x200 else mag) >> 5
    return [level(p) for p in range(1024)]

def ym_register_writes(lines):
    """The YM2612's register writes among the bench's b lines: (the data strobe's cycle, part, register, value)."""
    st = ym_strobes(lines, 0, 1 << 62)
    return [(b[0], a[1] >> 1, a[2], b[2]) for a, b in zip(st, st[1:]) if a[1] in (0, 2) and b[1] == a[1] + 1]

def lfo_items():
    """Every frequency number at every PMS, in the order the runs take them."""
    import random
    items = [(f, p) for p in range(8) for f in range(2048)]
    random.Random(386).shuffle(items)
    return items

def lfo_run(items, at, rate=7, block=7, mul=15):
    """`items`, six (frequency, PMS) a configuration, on the board at multiple `mul`: its pins and bus kept at `at`."""
    items = list(items) + [items[-1]] * (-len(items) % 6)
    configs = [items[i:i + 6] for i in range(0, len(items), 6)]
    cycles = 900000 + (len(configs) + 1) * (LFO_HOLD * 140 + 160000)
    if not os.path.exists(at + ".bus"):
        lines = bench(lfo_sweep(configs, rate=rate, block=block, hold=LFO_HOLD, mul=mul), cycles, audio=at + ".pins", zbus=True, pins=True)
        open(at + ".bus", "w").write("\n".join(" ".join(l) for l in lines if l[0] in "zb") + "\n")
    return configs, cycles

def lfo_fit(at, configs, cycles, block=7, mul=15, step=LFO_STEP, sets=False):
    """Each item's 32 offsets in half steps, kept where every explanation of the record agrees (Nephrite_LfoTables.md §3.2)."""
    import numpy as np
    levels = np.array(carrier_levels())
    off = np.arange(-400, 401)
    writes = ym_register_writes([l.split() for l in open(at + ".bus")])
    enable = [t for t, p, r, v in writes if p == 0 and r == 0x22 and v & 8][0] // SAMPLE
    keys = [t for t, p, r, v in writes if p == 0 and r == 0x28 and v & 0x80]
    starts = [t for t, p, r, v in writes if p == 0 and r == 0xB4]
    ch = channel_levels(at + ".pins", 1223, cycles // SAMPLE - 2)
    def explain(obs, k0, fnum, g0, end):
        g = (2 * fnum + off) & 0xFFF
        inc = ((((g << block) >> 2) * mul) & 0xFFFFF).astype(np.int64)
        states, n, segs = {0}, k0, []
        while n < end:
            span = min(g0 + step * ((n - g0) // step + 1), end) - n
            o = np.asarray(obs[n:n + span])
            moves = []
            for a in states:
                acc = (a + np.outer(inc, np.arange(span))) & 0xFFFFF
                moves += [(a, int(off[i]), (a + span * int(inc[i])) & 0xFFFFF) for i in np.nonzero((levels[acc >> 10] == o[None, :]).all(axis=1))[0]]
            if not moves:
                return None
            segs.append((n, span, moves))
            states, n = {b for _, _, b in moves}, n + span
        # Back from the end: a step's offsets are those of the moves that some whole explanation of the record makes.
        out, live = [], states
        for n, span, moves in reversed(segs):
            kept = [(a, o) for a, o, b in moves if b in live]
            out.append((n, span, {o for _, o in kept}))
            live = {a for a, _ in kept}
        return out[::-1]
    def item(n, c, g0):
        """Every explanation's steps, for each sample the key-on may have reset the phase at that explains the record."""
        end = starts[n + 1] // SAMPLE - 1 if n + 1 < len(configs) else cycles // SAMPLE - 3
        k = keys[6 * n + c] // SAMPLE
        return [seg for k0 in range(k - 1, k + 4) for seg in explain(ch[c], k0, configs[n][c][0], g0, end) or ()]
    # The grid of PM steps: the one phase at which the three deepest items are explained.
    deep = sorted(((configs[n][c][0] * configs[n][c][1], n, c) for n in range(len(configs)) for c in range(6)), reverse=True)[:3]
    g0 = [g for g in range(enable - step, enable + step + 1) if all(item(n, c, g) for _, n, c in deep)][0]
    out = {}
    for n, config in enumerate(configs):
        for c, key in enumerate(config):
            seen = {}
            for s, span, offsets in item(n, c, g0) or ():
                if span == step:
                    seen.setdefault((s - g0) // step % 32, set()).update(offsets)
            got = [seen.get(q, set()) if sets else next(iter(seen[q])) if len(seen.get(q, ())) == 1 else None for q in range(32)]
            was = out.get(tuple(key))
            if was and not sets:
                assert all(a is None or b is None or a == b for a, b in zip(was, got)), key
                got = [b if a is None else a for a, b in zip(was, got)]
            out[tuple(key)] = got
    return out, g0

def lfo_table(logs, first=0, last=None):
    """{(fnum, pms): 32 offsets} from runs in `logs`, made if missing; undecided items are run four times more."""
    global WORK
    items, home, table = lfo_items(), WORK, {}
    def measured(run, at):
        global WORK
        WORK = at + ".work"
        os.makedirs(WORK, exist_ok=True)
        for f in ("tb_md", "68k_ncode.txt", "68k_ucode.txt"):
            if not os.path.exists(os.path.join(WORK, f)):
                os.symlink(os.path.join(home, f), os.path.join(WORK, f))
        configs, cycles = lfo_run(run, at)
        WORK = home
        return lfo_fit(at, configs, cycles)[0]
    for i in list(range(0, len(items), LFO_SHARD))[first:last]:
        table.update(measured(items[i:i + LFO_SHARD], os.path.join(os.path.abspath(logs), f"lfo{i // LFO_SHARD:02d}")))
    open_items = sorted(k for k, v in table.items() if None in v)
    if open_items:
        again = (open_items * 4)[1:] + open_items[:1]
        for k, v in measured(again, os.path.join(os.path.abspath(logs), "lfo-again")).items():
            for s, x in enumerate(v):
                assert x is None or table[k][s] in (None, x), (k, s)
                table[k][s] = table[k][s] if x is None else x
    return table

def lfo_report(table):
    """The table's FNV-1a hash as `lfo_tables.rs` tests it, the undecided items, and the low-bit groups that disagree."""
    import struct
    undecided = [k for k, v in table.items() if None in v]
    groups = {}
    for (f, p), v in table.items():
        groups.setdefault((f >> 4, p), set()).add(tuple(v))
    full = len(table) == 2048 * 8 and not undecided
    h = fnv(b"".join(struct.pack("<32h", *table[(f, p)]) for p in range(8) for f in range(2048))) if full else None
    return h, undecided, sum(1 for g in groups.values() if len(g) > 1)

SOUNDS = {"dac-square": dac_square, "psg-tone": psg_tone, "dac-ramp": dac_ramp}
# The voices the references are compared on (Nephrite_Native.md §19.3): 440 Hz-ish (frequency 1,081, block 4).
A4 = dict(fnum=1081, block=4)
SOUNDS["fm-sine"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), **A4)
SOUNDS["fm-sine-ch5"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), channel=4, **A4)
SOUNDS["fm-tl16"] = lambda: fm_voice((QUIET, QUIET, QUIET, (0x01, 0x10, 0x1F, 0, 0, 0x0F)), **A4)
SOUNDS["fm-mul3"] = lambda: fm_voice((QUIET, QUIET, QUIET, (0x03, 0x00, 0x1F, 0, 0, 0x0F)), **A4)
SOUNDS["fm-chain"] = lambda: fm_voice(((0x01, 0x30, 0x1F, 0, 0, 0x0F), (0x02, 0x28, 0x1F, 0, 0, 0x0F), (0x01, 0x20, 0x1F, 0, 0, 0x0F), SINE), alg=0, **A4)
SOUNDS["fm-feedback5"] = lambda: fm_voice((SINE, QUIET, QUIET, QUIET), fb=5, **A4)
SOUNDS["fm-decay"] = lambda: fm_voice((QUIET, QUIET, QUIET, (0x01, 0x00, 0x1F, 0x08, 0x04, 0x4F)), **A4)
SOUNDS["fm-dt3"] = lambda: fm_voice((QUIET, QUIET, QUIET, (0x31, 0x00, 0x1F, 0, 0, 0x0F)), **A4)
SOUNDS["fm-dt7"] = lambda: fm_voice((QUIET, QUIET, QUIET, (0x71, 0x00, 0x1F, 0, 0, 0x0F)), **A4)
for _alg in range(1, 7):
    SOUNDS[f"fm-alg{_alg}"] = lambda a=_alg: fm_voice(((0x01, 0x28, 0x1F, 0, 0, 0x0F), (0x02, 0x24, 0x1F, 0, 0, 0x0F),
                                                       (0x03, 0x20, 0x1F, 0, 0, 0x0F), (0x01, 0x08, 0x1F, 0, 0, 0x0F)), alg=a, fb=3, **A4)
SOUNDS["fm-alg7"] = lambda: fm_voice(((0x01, 0x10, 0x1F, 0, 0, 0x0F), (0x02, 0x14, 0x1F, 0, 0, 0x0F), (0x03, 0x18, 0x1F, 0, 0, 0x0F),
                                      (0x05, 0x1C, 0x1F, 0, 0, 0x0F)), alg=7, fb=4, **A4)
SOUNDS["fm-envelope"] = lambda: fm_voice((QUIET, QUIET, QUIET, (0x01, 0x00, 0x14, 0x14, 0x0E, 0x48)), extra=[(0x28, 0x00, 43)], **A4)
SOUNDS["fm-envelope-ks"] = lambda: fm_voice((QUIET, QUIET, QUIET, (0x01, 0x00, 0xD2, 0x10, 0x0C, 0x46)), extra=[(0x28, 0x00, 43)], **A4)
SOUNDS["fm-attack"] = lambda: fm_voice((QUIET, QUIET, QUIET, (0x01, 0x00, 0x0C, 0, 0, 0x0F)), **A4)
for _t in (0x40, 0xC0):
    SOUNDS[f"op-alone-{_t:02x}"] = lambda t=_t: op_alone(t)
# The voices of the board's comparison (Nephrite_Disputes.md D-16 to D-18): decays and attacks by rate, and the ROMs' ten.
for _r in (38, 39, 42, 43, *range(46, 62)):
    SOUNDS[f"fm-dr{_r}"] = lambda r=_r: rate_voice(r)
for _r in range(44, 60):
    SOUNDS[f"fm-ar{_r}"] = lambda r=_r: rate_voice(r, attack=True)
for _i in range(len(ROM_VOICES)):
    SOUNDS[f"fm-rom{_i}"] = lambda i=_i: rom_voice(i)
# The LFO (Nephrite_Native.md §23): S4 alone at full level with its AM switch on (bit 7 of $6C), at each LFO speed with
# AMS 3; AMS 1 and 2; the switch off; the LFO off; its speed and its off and on again mid-note; a modulator's AM; PMS.
AM_S4 = (0x01, 0x00, 0x1F, 0x80, 0x00, 0x0F)
for _f in range(8):
    SOUNDS[f"fm-lfo{_f}"] = lambda f=_f: fm_voice((QUIET, QUIET, QUIET, AM_S4), lfo=8 | f, b4=0xC0 | 3 << 4, **A4)
for _a in (1, 2):
    SOUNDS[f"fm-ams{_a}"] = lambda a=_a: fm_voice((QUIET, QUIET, QUIET, AM_S4), lfo=8 | 7, b4=0xC0 | a << 4, **A4)
SOUNDS["fm-am-off"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), lfo=8 | 7, b4=0xC0 | 3 << 4, **A4)
SOUNDS["fm-lfo-off"] = lambda: fm_voice((QUIET, QUIET, QUIET, AM_S4), lfo=7, b4=0xC0 | 3 << 4, **A4)
SOUNDS["fm-lfo-restart"] = lambda: fm_voice((QUIET, QUIET, QUIET, AM_S4), lfo=8 | 6, b4=0xC0 | 3 << 4,
                                              extra=[(0x22, 0x07, 3), (0x22, 0x0E, 2), (0x22, 0x0B, 4)], **A4)
SOUNDS["fm-am-mod"] = lambda: fm_voice(((0x01, 0x30, 0x1F, 0, 0, 0x0F), (0x02, 0x28, 0x1F, 0, 0, 0x0F), (0x01, 0x10, 0x1F, 0x80, 0, 0x0F), SINE),
                                       alg=0, lfo=8 | 6, b4=0xC0 | 2 << 4, **A4)
for _p in range(1, 8):
    SOUNDS[f"fm-pms{_p}"] = lambda p=_p: fm_voice((QUIET, QUIET, QUIET, SINE), lfo=8 | 6, b4=0xC0 | p, **A4)
# Whether the key code follows the vibrato: detune 3 just under the edge of key codes 18 and 19 (frequency 1,151 =
# $47F at block 4), where detune 3 is 9 steps below it and 10 above, PMS 7.
SOUNDS["fm-pm-kc"] = lambda: fm_voice((QUIET, QUIET, QUIET, (0x31, 0x00, 0x1F, 0, 0, 0x0F)), lfo=8 | 6, b4=0xC7, fnum=1151, block=4)
# The same vibrato on channel 1 at channel 3's special frequency below, and on channel 3 without the special mode.
SOUNDS["fm-pm-1500"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), lfo=8 | 6, b4=0xC7, fnum=1500, block=4)
SOUNDS["fm-pm-s1"] = lambda: fm_voice((SINE, QUIET, QUIET, QUIET), lfo=8 | 6, b4=0xC7, fnum=1500, block=4)
# Every operator of every channel alone under the vibrato (frequency 1,500, block 4, PMS 7, rate 6), and S1-S3 of
# channel 1 under the tremolo (AMS 3, rate 7): where in the sample each takes the LFO's count.
def _alone(op, k, sound=SINE):
    ops = [QUIET] * 4
    ops[k] = sound
    return tuple(ops)
for _c in range(6):
    for _k in range(4):
        SOUNDS[f"fm-pm-c{_c + 1}s{_k + 1}"] = lambda c=_c, k=_k: fm_voice(_alone(None, k), channel=c, lfo=8 | 6, b4=0xC7, fnum=1500, block=4)
for _k in range(3):
    SOUNDS[f"fm-am-s{_k + 1}"] = lambda k=_k: fm_voice(_alone(None, k, AM_S4), lfo=8 | 7, b4=0xC0 | 3 << 4, **A4)
for _c, _k in ((1, 0), (3, 0), (5, 0), (5, 1), (5, 2), (5, 3)):
    SOUNDS[f"fm-am-c{_c + 1}s{_k + 1}"] = lambda c=_c, k=_k: fm_voice(_alone(None, k, AM_S4), channel=c, lfo=8 | 7, b4=0xC0 | 3 << 4, **A4)
SOUNDS["fm-pm-ch3-normal"] = lambda: fm_voice((SINE, QUIET, QUIET, QUIET), channel=2, lfo=8 | 6, b4=0xC7, fnum=1500, block=4)
# Channel 3's special mode under the vibrato: S1 alone at its own frequency (1,500, block 4; the channel's 1,081).
SOUNDS["fm-pm-ch3-p0"] = lambda: fm_voice((SINE, QUIET, QUIET, QUIET), channel=2, lfo=8 | 6, b4=0xC0, fnum=1081, block=4,
                                          pre=[(0xAD, 4 << 3 | 1500 >> 8), (0xA9, 1500 & 0xFF), (0x27, 0x40)])
SOUNDS["fm-pm-ch3-swap"] = lambda: fm_voice((SINE, QUIET, QUIET, QUIET), channel=2, lfo=8 | 6, b4=0xC7, fnum=1500, block=4,
                                            pre=[(0xAD, 4 << 3 | 1081 >> 8), (0xA9, 1081 & 0xFF), (0x27, 0x40)])
SOUNDS["fm-pm-ch3"] = lambda: fm_voice((SINE, QUIET, QUIET, QUIET), channel=2, lfo=8 | 6, b4=0xC7, fnum=1081, block=4,
                                       pre=[(0xAD, 4 << 3 | 1500 >> 8), (0xA9, 1500 & 0xFF), (0x27, 0x40)])

# SSG-EG (Nephrite_Native.md section 24): S4 alone with `$9C` set, an instant attack and a decay of rate 42 to the
# bottom unless told otherwise.
def ssg_s4(eg, ar=0x1F, dr=20, sr=0, sl_rr=0xF8, tl=0, **kw):
    return fm_voice((QUIET, QUIET, QUIET, (0x01, tl, ar, dr, sr, sl_rr)), ssg=(0, 0, 0, eg), **dict(A4, **kw))
for _m in range(8, 16):
    SOUNDS[f"fm-ssg{_m:x}"] = lambda m=_m: ssg_s4(m)
    # Released in its second pass or its hold, and keyed again 208 samples into the release.
    SOUNDS[f"fm-ssg-rel{_m:x}"] = lambda m=_m: ssg_s4(m, sl_rr=0xFA, extra=[(0x28, 0x00, 20)])
    SOUNDS[f"fm-ssg-rekey{_m:x}"] = lambda m=_m: ssg_s4(m, sl_rr=0xFA, extra=[(0x28, 0x00, 20), (0x28, 0xF0, 3)])
    # With an attack of rate 38 in every pass.
    SOUNDS[f"fm-ssg-ar{_m:x}"] = lambda m=_m: ssg_s4(m, ar=0x12)
for _m in (8, 10, 13, 14):
    # A decay to SL 4 and a slower sustain rate from there; and the fastest decay.
    SOUNDS[f"fm-ssg-sl{_m:x}"] = lambda m=_m: ssg_s4(m, dr=24, sr=14, sl_rr=0x48)
    SOUNDS[f"fm-ssg-fast{_m:x}"] = lambda m=_m: ssg_s4(m, dr=31)
SOUNDS["fm-ssg-tl"] = lambda: ssg_s4(14, tl=0x18)
SOUNDS["fm-ssg-am"] = lambda: ssg_s4(14, dr=0x80 | 20, lfo=8 | 7, b4=0xC0 | 3 << 4)
# `$9C` written while the envelope runs: SSG-EG turned on in a plain decay, its attack bit set in a pass, and it
# turned off in an inverted pass.
SOUNDS["fm-ssg-on"] = lambda: ssg_s4(0, dr=13, extra=[(0x9C, 0x08, 20)])
SOUNDS["fm-ssg-flip"] = lambda: ssg_s4(8, extra=[(0x9C, 0x0C, 20)])
SOUNDS["fm-ssg-off"] = lambda: ssg_s4(12, extra=[(0x9C, 0x00, 20)])
# `$9C` written in a hold or towards one: the hold left by turning SSG-EG off or by setting its attack bit, and the
# hold bit set in an inverted pass or cleared in a hold.
SOUNDS["fm-ssg-hold-off"] = lambda: ssg_s4(9, extra=[(0x9C, 0x00, 20)])
SOUNDS["fm-ssg-hold-att"] = lambda: ssg_s4(9, extra=[(0x9C, 0x0D, 20)])
SOUNDS["fm-ssg-a-to-b"] = lambda: ssg_s4(10, extra=[(0x9C, 0x0B, 20)])
SOUNDS["fm-ssg-b-to-a"] = lambda: ssg_s4(11, extra=[(0x9C, 0x0A, 21)])
# A key-off in an attack of rate 38, 208 or 347 samples in.
for _m in (12, 14):
    for _w in (3, 5):
        SOUNDS[f"fm-ssg-ar-rel{_m:x}-{_w}"] = lambda m=_m, w=_w: ssg_s4(m, ar=0x12, sl_rr=0xFA, extra=[(0x28, 0x00, w)])
# On a modulator: S3 into S4 (algorithm 4), its passes restarting its phase or turning about.
SOUNDS["fm-ssg-mod8"] = lambda: fm_voice((QUIET, QUIET, (0x01, 0x10, 0x1F, 20, 0, 0xF8), SINE), alg=4, ssg=(0, 0, 8, 0), **A4)
SOUNDS["fm-ssg-mode"] = lambda: fm_voice((QUIET, QUIET, (0x01, 0x10, 0x1F, 20, 0, 0xF8), SINE), alg=4, ssg=(0, 0, 14, 0), **A4)
# Each operator's place: one alone on channel 1 or 6 with pattern 8 or 14.
for _c, _k, _m in ((0, 0, 8), (0, 0, 14), (0, 1, 8), (0, 2, 8), (5, 0, 8), (5, 3, 8), (5, 3, 14)):
    SOUNDS[f"fm-ssg-c{_c + 1}s{_k + 1}-{_m:x}"] = lambda c=_c, k=_k, m=_m: fm_voice(
        _alone(None, k, (0x01, 0x00, 0x1F, 20, 0, 0xF8)), channel=c, ssg=tuple(m if j == k else 0 for j in range(4)), **A4)

# The envelope by operator: one alone with an attack of rate 38, a decay of 42 to SL 4, a sustain rate of 18 and a
# key-off 2,986 samples in, on the operators and channels whose place in the sample differs.
for _c, _k in ((0, 0), (2, 0), (4, 0), (5, 0), (5, 1), (5, 2), (5, 3), (1, 1), (3, 2)):
    SOUNDS[f"fm-eg-c{_c + 1}s{_k + 1}"] = lambda c=_c, k=_k: fm_voice(_alone(None, k, (0x01, 0x00, 0x12, 20, 8, 0x4A)), channel=c,
                                                                     extra=[(0x28, 0x00 | (c // 3) << 2 | c % 3, 43)], **A4)

# CSM (Nephrite_Native.md section 24): channel 3's operators keyed by timer A's overflow every `period` samples, `$27`
# written as `mode` once the timer's value is, and no key-on written unless `keys` gives one.
PLUCK = (0x01, 0x00, 0x1F, 0x00, 0x00, 0x0A)
FALL = (0x01, 0x00, 0x1F, 20, 0x00, 0xFA)
def csm(ops, period=100, mode=0x81, alg=7, keys=0x00, pre=(), **kw):
    value = 1024 - period
    return fm_voice(ops, alg=alg, channel=2, keys=keys, pre=list(pre) + [(0x24, value >> 2), (0x25, value & 3), (0x27, mode)], **dict(A4, **kw))
SOUNDS["fm-csm"] = lambda: csm((QUIET, QUIET, QUIET, PLUCK))
SOUNDS["fm-csm-all"] = lambda: csm(((0x01, 0x10, 0x1F, 0, 0, 0x0A), (0x02, 0x14, 0x1F, 0, 0, 0x0A), (0x03, 0x18, 0x1F, 0, 0, 0x0A), (0x05, 0x1C, 0x1F, 0, 0, 0x0A)))
# Each operator alone under CSM, S1-S3 at their own frequency of 1,500 and S4 at the channel's; twelve NOPs put the
# timer's load where `fm-csm` has it in the sample, and eighteen the key-on of the hand-over voices clear of channel
# 3's own deadline (Nephrite_Disputes.md D-15).
OWN_1500 = [(a + 4, 4 << 3 | 1500 >> 8) for a in (0xA9, 0xAA, 0xA8)] + [(a, 1500 & 0xFF) for a in (0xA9, 0xAA, 0xA8)]
for _k in range(4):
    SOUNDS[f"fm-csm-o{_k + 1}"] = lambda k=_k: csm(_alone(None, k, PLUCK), pre=OWN_1500, pre_nops=12)
for _p in (1, 2, 3):
    SOUNDS[f"fm-csm-p{_p}"] = lambda p=_p: csm((QUIET, QUIET, QUIET, FALL), period=p)
SOUNDS["fm-csm-ar"] = lambda: csm((QUIET, QUIET, QUIET, (0x01, 0x00, 0x14, 0x00, 0x00, 0x0A)))
SOUNDS["fm-csm-ar-p1"] = lambda: csm((QUIET, QUIET, QUIET, (0x01, 0x00, 0x14, 0x00, 0x00, 0x0A)), period=1)
SOUNDS["fm-csm-11"] = lambda: csm((QUIET, QUIET, QUIET, PLUCK), mode=0xC1)
SOUNDS["fm-csm-noload"] = lambda: csm((QUIET, QUIET, QUIET, PLUCK), mode=0x80)
SOUNDS["fm-csm-flags"] = lambda: csm((QUIET, QUIET, QUIET, PLUCK), mode=0x8F)
SOUNDS["fm-csm-key"] = lambda: csm((QUIET, QUIET, QUIET, FALL), keys=0xF0, key_nops=18, extra=[(0x28, 0x02, 20)])
SOUNDS["fm-csm-key-plain"] = lambda: csm((QUIET, QUIET, QUIET, FALL), mode=0x01, keys=0xF0, key_nops=18)
SOUNDS["fm-csm-off"] = lambda: csm((QUIET, QUIET, QUIET, PLUCK), extra=[(0x27, 0x01, 20), (0x27, 0x81, 5)])
# The test register `$21`, a bit at a time, and `$2C`'s.
SOUNDS["fm-test-lfo-am"] = lambda: fm_voice((QUIET, QUIET, QUIET, AM_S4), lfo=8 | 7, b4=0xC0 | 3 << 4, pre=[(0x21, 0x02)], **A4)
SOUNDS["fm-test-lfo-pm"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), lfo=8 | 6, b4=0xC7, pre=[(0x21, 0x02)], **A4)
SOUNDS["fm-test-lfo-late"] = lambda: fm_voice((QUIET, QUIET, QUIET, AM_S4), lfo=8 | 7, b4=0xC0 | 3 << 4, extra=[(0x21, 0x02, 20), (0x21, 0x00, 5)], **A4)
SOUNDS["fm-test-pg"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), extra=[(0x21, 0x08, 20), (0x21, 0x00, 5)], **A4)
SOUNDS["fm-test-eg"] = lambda: fm_voice((QUIET, QUIET, QUIET, (0x01, 0x00, 0x1F, 13, 0x00, 0xFA)), extra=[(0x21, 0x20, 20), (0x21, 0x00, 5)], **A4)
SOUNDS["fm-test-ugly"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), extra=[(0x21, 0x10, 20), (0x21, 0x00, 5)], **A4)
SOUNDS["fm-test-ugly-chain"] = lambda: fm_voice(((0x01, 0x30, 0x1F, 0, 0, 0x0F), (0x02, 0x28, 0x1F, 0, 0, 0x0F), (0x01, 0x20, 0x1F, 0, 0, 0x0F), SINE), alg=0,
                                                extra=[(0x21, 0x10, 20), (0x21, 0x00, 5)], **A4)
SOUNDS["fm-test-timer"] = lambda: csm((QUIET, QUIET, QUIET, PLUCK), pre=[(0x21, 0x04)])
for _p in (7, 24, 48, 50, 96, 97, 120, 200):
    SOUNDS[f"fm-test-timer-{_p}"] = lambda p=_p: csm((QUIET, QUIET, QUIET, PLUCK), period=p, pre=[(0x21, 0x04)])
# Each bit set before the key-on, where the sample its write lands in is of no account.
SOUNDS["fm-test-pg0"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), pre=[(0x21, 0x08)], extra=[(0x21, 0x00, 20)], **A4)
SOUNDS["fm-test-eg0"] = lambda: fm_voice((QUIET, QUIET, QUIET, (0x01, 0x00, 0x1F, 20, 0x00, 0xFA)), pre=[(0x21, 0x20)], **A4)
SOUNDS["fm-test-ugly0"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), pre=[(0x21, 0x10)], **A4)
for _k in range(3):
    SOUNDS[f"fm-test-ugly0-s{_k + 1}"] = lambda k=_k: fm_voice(_alone(None, k), pre=[(0x21, 0x10)], **A4)
SOUNDS["fm-test-ugly0-chain"] = lambda: fm_voice(((0x01, 0x30, 0x1F, 0, 0, 0x0F), (0x02, 0x28, 0x1F, 0, 0, 0x0F), (0x01, 0x20, 0x1F, 0, 0, 0x0F), SINE), alg=0,
                                                 pre=[(0x21, 0x10)], **A4)
SOUNDS["fm-test-ugly0-fb"] = lambda: fm_voice((SINE, QUIET, QUIET, QUIET), fb=5, pre=[(0x21, 0x10)], **A4)
SOUNDS["fm-test-dac0"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), pre=[(0x2A, 0xA0), (0x2B, 0x80), (0x2C, 0x20)], **A4)
SOUNDS["fm-test-dac0-off"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), pre=[(0x2A, 0xA0), (0x2C, 0x20)], **A4)
# Carriers that together pass the nine bits: how a channel's sum is held to them.
LOUD = lambda mul: (mul, 0x00, 0x1F, 0, 0, 0x0F)
SOUNDS["fm-clip7"] = lambda: fm_voice((LOUD(1), LOUD(2), LOUD(3), LOUD(5)), alg=7, **A4)
SOUNDS["fm-clip5"] = lambda: fm_voice(((0x01, 0x18, 0x1F, 0, 0, 0x0F), LOUD(2), LOUD(3), LOUD(5)), alg=5, **A4)
SOUNDS["fm-clip4"] = lambda: fm_voice(((0x01, 0x18, 0x1F, 0, 0, 0x0F), LOUD(1), (0x03, 0x18, 0x1F, 0, 0, 0x0F), LOUD(3)), alg=4, **A4)
SOUNDS["fm-test-read"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), extra=[(0x21, 0x01, 20), (0x21, 0x40, 5), (0x21, 0x80, 5), (0x21, 0xC1, 5), (0x21, 0x00, 5)], **A4)
SOUNDS["fm-test-2c"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), extra=[(0x2C, 0x10, 20), (0x2C, 0x20, 5), (0x2C, 0x40, 5), (0x2C, 0x80, 5), (0x2C, 0x00, 5)], **A4)
SOUNDS["fm-test-2c-dac"] = lambda: fm_voice((QUIET, QUIET, QUIET, SINE), pre=[(0x2A, 0xA0), (0x2B, 0x80)], extra=[(0x2C, 0x20, 20), (0x2C, 0x00, 5)], **A4)

# When a write takes effect (Nephrite_Disputes.md D-15): one write of each kind moved across the sample by `k` NOPs,
# 28 master clocks each, after its wait for the busy flag. Each is S4 of channel 1 unless its line says otherwise.
DECAY = (0x01, 0x00, 0x1F, 20, 0x00, 0xFA)
LANDING = {
    "lfo": lambda k: fm_voice((QUIET, QUIET, QUIET, AM_S4), b4=0xF0, pre=[(0x22, 0x0F)], pre_nops=k, **A4),
    "lfo-off": lambda k: fm_voice((QUIET, QUIET, QUIET, AM_S4), lfo=0x0F, b4=0xF0, extra=[(0x22, 0x00, 2, k)], **A4),
    "ams": lambda k: fm_voice((QUIET, QUIET, QUIET, AM_S4), lfo=0x0F, b4=0xF0, extra=[(0xB4, 0xC0, 2, k)], **A4),
    "amon": lambda k: fm_voice((QUIET, QUIET, QUIET, AM_S4), lfo=0x0F, b4=0xF0, extra=[(0x6C, 0x00, 2, k)], **A4),
    "pms": lambda k: fm_voice((QUIET, QUIET, QUIET, SINE), lfo=0x0E, b4=0xC0, extra=[(0xB4, 0xC7, 2, k)], **A4),
    "pg": lambda k: fm_voice((QUIET, QUIET, QUIET, SINE), extra=[(0x21, 0x08, 2, k)], **A4),
    "eg": lambda k: fm_voice((QUIET, QUIET, QUIET, DECAY), extra=[(0x21, 0x20, 2, k)], **A4),
    "ugly": lambda k: fm_voice((QUIET, QUIET, QUIET, SINE), alg=0, extra=[(0x21, 0x10, 2, k)], **A4),
    "tl": lambda k: fm_voice((QUIET, QUIET, QUIET, SINE), extra=[(0x4C, 0x20, 2, k)], **A4),
    "ssg": lambda k: ssg_s4(8, extra=[(0x9C, 0x0C, 2, k)]),
    "fnum": lambda k: fm_voice((QUIET, QUIET, QUIET, SINE), extra=[(0xA4, 4 << 3 | 1500 >> 8, 2), (0xA0, 1500 & 0xFF, 0, k)], **A4),
    "mul": lambda k: fm_voice((QUIET, QUIET, QUIET, SINE), extra=[(0x3C, 0x03, 2, k)], **A4),
    "fb": lambda k: fm_voice((SINE, QUIET, QUIET, QUIET), extra=[(0xB0, 0x3F, 2, k)], **A4),
    "alg": lambda k: fm_voice((QUIET, QUIET, (0x02, 0x10, 0x1F, 0, 0, 0x0F), SINE), alg=7, extra=[(0xB0, 0x04, 2, k)], **A4),
    "pan": lambda k: fm_voice((QUIET, QUIET, QUIET, SINE), extra=[(0xB4, 0x40, 2, k)], **A4),
    "dr": lambda k: fm_voice((QUIET, QUIET, QUIET, (0x01, 0x00, 0x1F, 0, 0x00, 0xFA)), extra=[(0x6C, 31, 2, k)], **A4),
    "key-off": lambda k: fm_voice((QUIET, QUIET, QUIET, (0x01, 0x00, 0x1F, 0, 0, 0x0F)), extra=[(0x28, 0x00, 2, k)], **A4),
    "dac": lambda k: fm_voice((QUIET, QUIET, QUIET, SINE), pre=[(0x2A, 0x80), (0x2B, 0x80)], extra=[(0x2A, 0xC0, 2, k)], **A4),
    "dac-on": lambda k: fm_voice((QUIET, QUIET, QUIET, SINE), pre=[(0x2A, 0xC0)], extra=[(0x2B, 0x80, 2, k)], **A4),
    "dac-all": lambda k: fm_voice((QUIET, QUIET, QUIET, SINE), pre=[(0x2A, 0xA0)], extra=[(0x2C, 0x20, 2, k)], **A4),
    "special": lambda k: fm_voice((SINE, QUIET, QUIET, QUIET), channel=2, pre=OWN_1500, extra=[(0x27, 0x40, 2, k)], **A4),
    "csm": lambda k: csm((QUIET, QUIET, QUIET, PLUCK), pre_nops=k),
}
for _c in range(6):
    for _k in range(4):
        LANDING[f"key{_c + 1}s{_k + 1}"] = lambda k, c=_c, o=_k: fm_voice(_alone(None, o), channel=c, key_nops=k, **A4)

PAL_PICTURES = {"pal-v30"}

# The voices of SSG-EG, CSM, the timers' test bit, the test register and the channel's sum (Nephrite_Native.md
# section 24): each run 19,000,000 cycles and held for 8,600 samples from the first that is neither 0 nor -1, nor -256
# where the test register's bit 4 leaves a silent channel there; a voice that never leaves rest holds none.
def chip_voice_names():
    return [n for n in SOUNDS if n.startswith(("fm-ssg", "fm-csm", "fm-test", "fm-clip", "fm-eg-"))]

def chip_channel(name):
    import re
    m = re.search(r"-c(\d)s", name)
    return int(m.group(1)) - 1 if m else 2 if name.startswith(("fm-csm", "fm-test-timer")) else 0

def chip_rest(name):
    return (0, -1, -256) if name.startswith("fm-test-ugly0") else (0, -1)

def board_chip(logs):
    """Nephrite's `board_chip.rs`: each of `chip_voice_names`' channel as the board's pins give it, the pins made in
    `logs` if missing."""
    at = lambda name: os.path.join(os.path.abspath(logs), name)
    out = [BOARD_CHIP_HEAD, "pub const CHIP_VOICES: &[BoardChipVoice] = &["]
    for name in chip_voice_names():
        if not os.path.exists(at(name + ".pins")):
            bench(SOUNDS[name](), 19000000, audio=at(name + ".pins"))
        recs = pin_records(at(name + ".pins"))
        slot = [1223, 215, 1895, 887, 1559, 551][chip_channel(name)]
        board, i, k = [], 0, 1300000 // SAMPLE
        while k * SAMPLE + slot + 1 < 19000000:
            t = k * SAMPLE + slot + 1
            while i + 1 < len(recs) and recs[i + 1][0] <= t:
                i += 1
            board.append(fm_level(recs[i][1]) if recs[i][0] <= t else 0)
            k += 1
        rest = chip_rest(name)
        b0 = next((j for j, x in enumerate(board) if x not in rest), None)
        series = [] if b0 is None else board[b0:b0 + 8600]
        out.append(f'    BoardChipVoice {{ name: "{name}", image: 0x{fnv(SOUNDS[name]()):08x}, channel: {chip_channel(name)}, samples: {len(series)}, blocks: &[\n{block_hashes(series)}\n    ] }},')
    out.append("];\n")
    out.append(board_sweeps(at))
    out.append(board_write_times(at, chip_voice_names() + lfo_voice_names()))
    return "\n".join(out)

def ym_latches(lines, start):
    """The YM2612's writes after `start` as the chip takes them: a strobe's fall and 48 cycles, where the strobe rises
    unless the 68000's cycle was held a clock by the bus refresh (Nephrite_Disputes.md D-2)."""
    bus = [(int(l[1]), int(l[3]), int(l[4], 16)) for l in lines if l[0] == "b" and start <= int(l[1])]
    return [e[0] + 48 for was, e in zip(bus, bus[1:]) if e[1] == 0 and was[1] == 1 and 0x4000 <= e[2] < 0x4004]

def board_write_times(at, names):
    """For each voice, the MCLK2 cycles from the reset line's release to the rise of its first YM2612 write's strobe
    and from each strobe to the next, address and data, from the 68000's: their bus logs (`<name>.bus`, the Z80 bus
    and the reset line) made if missing. Every voice releases the reset line at the same cycle."""
    out = ["pub const WRITE_TIMES: &[(&str, &[u32])] = &["]
    release = None
    for name in names:
        if not os.path.exists(at(name + ".bus")):
            cycles = 36000000 if name in LFO_SLOW else 19000000
            lines = [l for l in bench(SOUNDS[name](), cycles, audio=at(name + ".pins"), zbus=True, pins=True) if l[0] in "zb"]
            open(at(name + ".bus"), "w").write("\n".join(" ".join(l) for l in lines) + "\n")
        lines = [l.split() for l in open(at(name + ".bus"))]
        r = edges(lines)[1][0]
        assert release in (None, r), (name, r, release)
        release = r
        times = ym_latches(lines, r)
        out.append(f'    ("{name}", &[{", ".join(str(t - u) for t, u in zip(times, [r] + times))}]),')
    return "\n".join(out) + f"\n];\n\npub const WRITES_RELEASE: u64 = {release};\n"

SWEEPS = [f"key{_c}s{_k}" for _c in range(1, 7) for _k in (1, 2, 3, 4)] + ["csm"]

def sweep_log(kind, k):
    """A landing sweep's run as its log is named: S4's key sweeps carry no operator in the name."""
    return f"sw-{kind[:-2] if kind.startswith('key') and kind.endswith('s4') else kind}-{k}"

def board_sweeps(at):
    """For each of `SWEEPS`, the sample in which each of its 40 runs first sounds, less the first run's: where each
    write lands in the board's sample as the NOPs move it (Nephrite_Native.md section 24)."""
    out = ["pub const SWEEPS: &[(&str, [i8; 40])] = &["]
    for kind in SWEEPS:
        firsts = []
        for k in range(40):
            name = sweep_log(kind, k)
            if not os.path.exists(at(name + ".pins")):
                bench(LANDING[kind](k), 1800000, audio=at(name + ".pins"))
            assert open(at(name + ".bin"), "rb").read() == LANDING[kind](k) if os.path.exists(at(name + ".bin")) else True
            recs = [r for r in pin_records(at(name + ".pins")) if r[0] > 1300000 and fm_level(r[1]) not in (0, -1)]
            firsts.append(recs[0][0] // SAMPLE)
        out.append(f'    ("{kind}", [{", ".join(str(f - firsts[0]) for f in firsts)}]),')
    return "\n".join(out) + "\n];\n"

BOARD_CHIP_HEAD = '''//! The board's FM pins for `mdboard.py`'s voices of SSG-EG, CSM, the timers, the test register and the channel's sum,
//! written by `mdboard.py board-chip`: each voice's channel from its first sample that is neither 0 nor -1 (nor -256
//! for the voices with the test register's bit 4 set before the key-on), 8,600 samples held as the FNV-1a hash of
//! each block of 64 as little-endian i16 (Nephrite_Native.md §24).

/// A voice on `channel` (0-5): its program's hash and the samples held; none for a voice that never sounds.
pub struct BoardChipVoice {
    pub name: &'static str,
    pub image: u32,
    pub channel: usize,
    pub samples: usize,
    pub blocks: &'static [u32],
}
'''

if __name__ == "__main__":
    what = sys.argv[1]
    if what == "sound":
        open(sys.argv[3], "wb").write(SOUNDS[sys.argv[2]]())
        sys.exit()
    if what == "pins":
        bench(SOUNDS[sys.argv[2]](), int(sys.argv[4]) if len(sys.argv) > 4 else 10000000, audio=os.path.abspath(sys.argv[3]))
        sys.exit()
    if what == "fm":
        slot, n, equal, diffs = fm_compare(sys.argv[2], sys.argv[3], int(sys.argv[4]) if len(sys.argv) > 4 else 0)
        print(f"the channel at cycle {slot} of the sample: {n} samples compared, {equal} equal; first differences (sample, board, trace) {diffs}")
        sys.exit()
    if what == "board-bus":
        sys.stdout.write(board_bus(sys.argv[2]))
        sys.exit()
    if what == "board-fm":
        sys.stdout.write(board_fm(sys.argv[2]))
        sys.exit()
    if what == "board-chip":
        sys.stdout.write(board_chip(sys.argv[2]))
        sys.exit()
    if what == "read-after-write":
        read_after_write_report(int(sys.argv[2]) if len(sys.argv) > 2 else 0, "dac" in sys.argv[3:], "timers" in sys.argv[3:])
        sys.exit()
    if what == "status-ports":
        for enable in (1, 0):
            reads = [l[3][-2:] for l in bench(status_ports(), 2600000, status_enable=enable) if l[0] == "w"]
            print(f"ym2612_status_enable {enable}: ports 0-3 under the busy flag ${', $'.join(reads[:4])}; after it ${', $'.join(reads[4:])}")
        sys.exit()
    if what == "lfo":
        table = lfo_table(sys.argv[2], *(int(a) for a in sys.argv[3:5]))
        h, undecided, low = lfo_report(table)
        print(f"{len(table)} items, {len(undecided)} with a step the board leaves open; {low} groups of sixteen frequency numbers "
              f"that differ only in their low four bits and do not agree" + (f"; the table's hash 0x{h:08x}" if h is not None else ""))
        sys.exit()
    if what == "reset-replay":
        reset_replay_report(int(sys.argv[2]) if len(sys.argv) > 2 else 200)
        sys.exit()
    if what == "picture":
        picture(sys.argv[2], int(sys.argv[3]) if len(sys.argv) > 3 else 4, sys.argv[4] if len(sys.argv) > 4 else None)
        sys.exit()
    cycles = int(sys.argv[2]) if len(sys.argv) > 2 else 400000
    if what == "busreq-tight":
        busreq_tight(cycles, 0x8000)
        busreq_tight(cycles, 0x1000)
    else:
        {"window": window, "window-write": lambda c: window(c, 0x32), "busreq": busreq, "rom-loop": rom_loop, "vcounter": vcounter, "interlace": interlace, "mode4-ports": mode4_ports, "vram128": vram128}[what](cycles)
