#!/usr/bin/env python3
"""Reads the RAM records of Diamond's rewrites of ps1-tests' TTY programs, as the original would have printed them.

See Diamond_Records.md for each program's record layout, why it is laid out so, and how the checks below are read.

    psxrecords.py render PROGRAM WORDS
    psxrecords.py equal PROGRAM WORDS TTY
    psxrecords.py console PROGRAM WORDS LOG
    psxrecords.py console PROGRAM --tty TTY LOG
    psxrecords.py mask < TTY

PROGRAM is `timers`, `access-time`, `chopping`, `bandwidth`, `chain-looping`, `otc-test`, `dpcr` or
`io-access-bitwidth`. WORDS is a file of the record's words from 80100000h: psxreferee.py's `ram` output, or plain
hexadecimal words (Diamond's `run --bare --dump`). `render` prints the text the release would print; `equal` compares
the record with a TTY of the release program, by the program's checks; `console` grades the record, or with --tty a
release's TTY, against ps1-tests' console log by the rules of WiseMan's TTY gate and Diamond_Referees.md §6. The last
three programs' records are printf calls, rendered with the release's own formats, read from the build 158 corpus that
EMUSEN_DIAMOND_PS1TESTS names (Diamond_Records.md §10). `mask` copies a TTY with io-access-bitwidth's BIOS row reduced
to whether each value is the ROM's own word, so that a referee's TTY of the release is never kept with BIOS words in it.
Exit codes: 0 done, 2 a usage error.
"""
import os
import re
import statistics
import struct
import sys
from pathlib import Path

BASE = 0x80100000
SIZE = {"timers": 870, "access-time": 102, "chopping": 780, "bandwidth": 56, "chain-looping": 12,
        "otc-test": 3072, "dpcr": 3072, "io-access-bitwidth": 3072}
CALLS = {"otc-test": ("dma/otc-test/otc-test.exe", "dma/otc-test"), "dpcr": ("dma/dpcr/dpcr.exe", "dma/dpcr"),
         "io-access-bitwidth": ("cpu/io-access-bitwidth/io-access-bitwidth.exe", "cpu/io-access-bitwidth")}
ROM_CELL = {1: "(ROM)", 2: "(written)"}

MODES = [["System clock", "Dot clock", "System clock", "Dot clock"],
         ["System clock", "Hblank", "System clock", "Hblank"],
         ["System clock", "System clock", "System clock/8", "System clock/8"]]
SYNCS = [["0 = Pause counter during Hblanks", "1 = Reset counter to 0 at Hblanks",
          "2 = Reset counter to 0 at Hblanks and pause outside of Hblank", "3 = Pause until Hblank occurs once, then switch to Free Run"],
         ["0 = Pause counter during Vblanks", "1 = Reset counter to 0 at Vblanks",
          "2 = Reset counter to 0 at Vblanks and pause outside of Vblank", "3 = Pause until Vblank occurs once, then switch to Free Run"],
         ["0 = Stop counter at current value", "1 = Free Run", "2 = Free Run", "3 = Stop counter at current value"]]
WIDTHS = [256, 320, 368, 512, 640]
DOTCLOCKS = ["5.32224 MHz", "6.65280 MHz", "7.60320 MHz", "10.64448 MHz", "13.30560 MHz"]
REGIONS = [("RAM", 0x80000000), ("BIOS", 0xBFC00000), ("SCRATCHPAD", 0x1F800000), ("EXPANSION1", 0x1F000000),
           ("EXPANSION2", 0x1F802000), ("EXPANSION3", 0x1FA00000), ("DMAC_CTRL", 0x1F8010F0), ("JOY_STAT", 0x1F801044),
           ("SIO_STAT", 0x1F801054), ("RAM_SIZE", 0x1F801060), ("I_STAT", 0x1F801070), ("TIMER0_VAL", 0x1F801100),
           ("CDROM_STAT", 0x1F801800), ("GPUSTAT", 0x1F801814), ("MDECSTAT", 0x1F801824), ("SPUCNT", 0x1F801DAA),
           ("CACHECTRL", 0xFFFE0130)]
BANDWIDTH = ["vramToCpu", "cpuToVram", "vramToVram", "FillScreen GP0(2)", "Rectangle", "Rectangle (semitransparent)",
             "Rectangle textured", "Rectangle textured (semi)", "Polygon quad", "Polygon quad (semi)", "Polygon quad textured",
             "Polygon quad textured (semi)", "Polygon quad (1/4 off screen)", "Polygon quad (1/2 off screen)"]


def die(message, code=2):
    print(message, file=sys.stderr)
    sys.exit(code)


def words(path):
    """The record's words from 80100000h: psxreferee.py's block for that address, or a plain list of words."""
    blocks, at = {}, BASE
    for line in Path(path).read_text().splitlines():
        if line.startswith("== RAM "):
            at = int(line.split()[2], 16)
            continue
        blocks.setdefault(at, []).extend(int(w, 16) for w in line.split() if re.fullmatch(r"[0-9A-Fa-f]{8}", w))
    if BASE not in blocks:
        die(f"no words from {BASE:08X}h")
    return blocks[BASE]


def signed(w):
    return w - (1 << 32) if w & 0x80000000 else w


def timers_parts(ws):
    """Rows as (label, values): the delay and frame rows, the sync rows, then the target and flag checks."""
    row = lambda at: [signed(w) for w in ws[at // 4:at // 4 + 10]]
    rows, k = [], 0
    for timer in range(3):
        for mode in range(2):
            m = mode * 2 if timer == 2 else mode
            for kind in ("1000", "5000", "frame"):
                rows.append((("delay", timer, MODES[timer][m], kind), row(40 * k)))
                k += 1
    sync = []
    for mode in range(2):
        for res in range(5):
            for s in range(4):
                sync.append(((0, mode, res, s), row(0x300 + 40 * (20 * mode + 4 * res + s))))
    for i, (timer, mode) in enumerate((t, m) for t in (1, 2) for m in range(2)):
        for s in range(4):
            sync.append(((timer, mode, None, s), row(0xA00 + 40 * (4 * i + s))))
    target = ws[0xD00 // 4:0xD00 // 4 + 17]
    flags = ws[0xD80 // 4:0xD80 // 4 + 6]
    return rows, sync, target, flags


def render_timers(ws):
    rows, sync, target, flags = timers_parts(ws)
    out = ["timers test", "", "Forced NTSC system."]
    for (_, timer, clock, kind), vs in rows:
        what = f"{int(kind):4d} cycles delay" if kind != "frame" else "      frame delay"
        out.append(f"Timer{timer}, clock: {clock:>14s}, {what}: " + "".join(f"{v:6d}, " for v in vs))
    for (timer, mode, res, s), vs in sync:
        if s == 0:
            out.append("")
            if timer == 0:
                out.append(f"Testing Timer0 sync modes (clock source = {MODES[0][mode]}, 100 delay cycles, resolution = {WIDTHS[res]}x240, dotclock @ {DOTCLOCKS[res]}, not resetted between runs)")
            else:
                out.append(f"Testing Timer{timer} sync modes (clock source = {MODES[timer][mode]}, 100 delay cycles, not resetted between runs)")
        out.append(f"{SYNCS[timer][s]:<61s}" + "".join(f"{v:6d}, " for v in vs))
    out += ["", "Check when Timer2 counter (System clock) is reset when reaching target (= 10)."]
    for i in range(12):
        out.append(f"counter == {i:2d}, {target[i]:5d} ticks  " + ("<<<< This value should be non-zero" if i == 10 else ""))
    names = ["Reached target:                      ", "Reached 0xFFFF:                      ",
             "Counter reset AFTER reaching target: ", "Target + 1 not reached:              "]
    for i, (name, good) in enumerate(zip(names, (1, 0, 1, 1))):
        v = target[13 + i]
        out.append(f"{name}{v} ({'ok' if v == good else 'fail'})")
    out += ["", "", "Check if write to mode register acknowledges the targetReached flag",
            f"Write 0x0000 to Timer0.mode, read 0x{flags[0]:04x}", f"Write 0xffff to Timer0.mode, read 0x{flags[1]:04x}", "",
            f"Read mode after waiting for targetReached: 0x{flags[2]:04x}", f"Read again: 0x{flags[3]:04x}", "",
            f"Write mode after waiting for targetReached and then read: 0x{flags[4]:04x}", f"Read again: 0x{flags[5]:04x}", "", "", "Done."]
    return out


def render_access(ws):
    out = ["cpu/access-time", "", "SEGMENT    (   ADDRESS)    8bit    16bit    32bit    (cpu cycles per bitsize)"]
    for r, (name, at) in enumerate(REGIONS):
        cells = "".join(f"{ws[6 * r + 2 * k]:2d}.{ws[6 * r + 2 * k + 1]:<2d}    " for k in range(3))
        out.append(f"{name:<10s} (0x{at:08x})   " + cells)
    return out + ["Done."]


def render_chopping(ws):
    out = ["dma/chopping (DMA Channel 2 CPU->VRAM, chopping test)"]
    for i in range(130):
        mode, size, a, b, cycles, busy = ws[6 * i:6 * i + 6]
        if i < 65:
            head = f"DMA2, SyncMode: {mode}, {size:4d} bytes, blockSize: {a:3d}                            ... "
        elif i == 65:
            head = f"DMA2, SyncMode: {mode}, {size:4d} bytes, no chopping                               ... "
        else:
            head = f"DMA2, SyncMode: {mode}, {size:4d} bytes, dmaWindow: {a:3d} words, cpuWindow: {b:3d} clks ... "
        out.append(head + f"took {cycles:5d} CPU cycles. " + ("(GPU busy)" if busy else ""))
    return out + ["Done."]


def render_bandwidth(ws):
    out = ["gpu/bandwidth"]
    for i, name in enumerate(BANDWIDTH):
        q, r, h, mbs = ws[4 * i:4 * i + 4]
        out.append(f"{name:<30s} dT: {q:5d}.{r:<5d} ms (hblanks: {h:5d}), speed: {mbs} MB/s")
    return out + ["Done"]


def render_chain(ws):
    out = ["dma/chain-looping", "Tests CPU behavior with various invalid GPU linked list chains.", ""]
    names = ["Testing with no DMA active...", "Testing with empty but complete chain...", "Testing single self-referencing chain...",
             "Testing double self-referencing chain..."]
    text = lambda v: "true" if v == 1 else "false"
    for i, name in enumerate(names):
        ticks, finished, irq = ws[3 * i:3 * i + 3]
        out += [name, f"  Work took {ticks} ticks" + (f", finished = {text(finished)}, irq = {text(irq)}" if i else "")]
    return out


def release(program):
    """The release's PS-EXE from the corpus, and its load address."""
    root = os.environ.get("EMUSEN_DIAMOND_PS1TESTS") or die("EMUSEN_DIAMOND_PS1TESTS names no ps1-tests build 158")
    image = Path(root, CALLS[program][0]).read_bytes()
    return image, struct.unpack("<I", image[0x18:0x1C])[0]


def cstring(exe, address):
    image, load = exe
    at = 0x800 + address - load
    if not 0x800 <= at < len(image):
        return "<%08x>" % address
    return image[at:image.index(b"\0", at)].decode("latin-1")


def cformat(fmt, args, exe):
    """C's printf for the conversions the three programs use: d i u x X c s %, with flags, width and precision."""
    out, args = [], list(args)
    for m in re.finditer(r"%([-+ #0]*)(\d*)(?:\.(\d+))?(?:hh|h|ll|l)?([diuxXcs%])|[^%]+|%", fmt):
        if not m.group(4):
            out.append(m.group(0))
            continue
        flags, width, precision, conv = m.group(1), int(m.group(2) or 0), m.group(3), m.group(4)
        if conv == "%":
            out.append("%")
            continue
        v = args.pop(0) if args else 0
        if conv == "s":
            text, zero = cstring(exe, v), False
            if precision is not None:
                text = text[:int(precision)]
        elif conv == "c":
            text, zero = chr(v & 0xFF), False
        else:
            if conv in "di":
                n = v - (1 << 32) if v & 0x80000000 else v
                sign, digits = ("-" if n < 0 else "+" if "+" in flags else " " if " " in flags else ""), str(abs(n))
            else:
                sign, digits = "", ("%x" % v if conv == "x" else "%X" % v if conv == "X" else str(v))
            if precision is not None:
                digits = digits.zfill(int(precision))
            prefix = ("0x" if conv == "x" else "0X") if "#" in flags and conv in "xX" and v else ""
            zero = "0" in flags and "-" not in flags and precision is None
            text = sign + prefix + (digits.zfill(width - len(sign) - len(prefix)) if zero else digits)
        out.append(text.ljust(width) if "-" in flags else text.rjust(width))
    return "".join(out)


def render_calls(program, ws):
    """The record's printf calls, six words each, rendered with the release's formats; io-access-bitwidth's BIOS cells by ROM_CELL."""
    exe = release(program)
    text = []
    for i in range(0, len(ws) - 5, 6):
        fmt, *args = ws[i:i + 6]
        if fmt == 0:
            break
        text.append("%10s    " % ROM_CELL[fmt] if fmt in ROM_CELL else cformat(cstring(exe, fmt), args, exe))
    return "".join(text).split("\n")


def mask_bios(lines):
    """io-access-bitwidth's BIOS row in a TTY, each value as ROM_CELL says: the ROM's word is taken as the 32-bit table's first cell."""
    out, width, word = [], 32, None
    for line in lines:
        m = re.match(r"^Reading as (\d+)bit", line)
        if m:
            width = int(m.group(1))
        m = re.match(r"^(BIOS\s+\(0xbfc00000\) {3})(.*)$", line)
        if m:
            cells = m.group(2).split()
            if word is None and cells and cells[0].startswith("0x"):
                word = int(cells[0], 16)
            mask = (1 << width) - 1
            shown = [c if not c.startswith(("0x", "0")) or word is None else
                     ROM_CELL[1] if int(c, 16) == word & mask else ROM_CELL[2] for c in cells]
            line = m.group(1) + "".join("%10s    " % c if c in ROM_CELL.values() else "%-10s    " % c for c in shown)
        out.append(line.rstrip())
    return out


def from_heading(program, lines):
    heading = CALLS[program][1]
    at = next((i for i, line in enumerate(lines) if line.startswith(heading)), len(lines))
    return [line.rstrip() for line in lines[at:]]


def equal_calls(program, rec, tty):
    """The record's text against the release's, line by line from the heading on, empty lines left out as DuckStation's log leaves them."""
    a = [l for l in from_heading(program, mask_bios(rec)) if l]
    b = [l for l in from_heading(program, mask_bios(tty)) if l]
    off = [(i, x, y) for i, (x, y) in enumerate(zip(a, b)) if x != y]
    report = [f"line {i}: '{x}' against '{y}'" for i, x, y in off[:12]] + ([f"and {len(off) - 12} lines more"] if len(off) > 12 else [])
    if len(a) != len(b):
        report.append(f"{len(a)} lines against {len(b)}")
    report.append(f"{len(b) - len(off)} of {len(b)} lines equal")
    return not off and len(a) == len(b), report


def bitwidth_cells(lines):
    """Each table's cells by (width, row): the reads after each write width."""
    cells, width = {}, None
    for line in mask_bios(lines):
        m = re.match(r"^Reading as (\d+)bit", line)
        if m:
            width = int(m.group(1))
            continue
        m = re.match(r"^(\S+)\s+\(0x[0-9a-f]+\)\s+(.*)$", line)
        if m and width and m.group(1) != "SEGMENT":
            cells[(width, m.group(1))] = re.findall(r"\(\w+\)|--CRASH--|\S+", m.group(2))
    return cells


def console_calls(program, rec, log):
    """otc-test and dpcr by the TTY gate's rule and line by line against the log; io-access-bitwidth cell by cell."""
    if program == "io-access-bitwidth":
        ours, theirs = bitwidth_cells(rec), bitwidth_cells(log)
        report, same, total = [], 0, 0
        for key, vs in theirs.items():
            mine = ours.get(key, [])
            for k, v in enumerate(vs):
                total += 1
                got = mine[k] if k < len(mine) else "missing"
                if got == v:
                    same += 1
                else:
                    report.append(f"read as {key[0]} bits, {key[1]}, written as {(8, 16, 32)[k]}: {got} against {v}")
        return report + [f"{same} of {total} cells as the console's"]
    a, b = from_heading(program, rec), from_heading(program, log)
    fails = [l for l in a if l.startswith("fail")]
    passed = sum(l.startswith("pass") for l in a)
    gate = not fails and passed > 0 and "Done." in a
    found, at = 0, 0
    for line in (l for l in b if l):
        k = next((i for i in range(at, len(a)) if a[i] == line), None)
        if k is not None:
            found, at = found + 1, k + 1
    extra = [l for l in a if l and l not in b and not l.startswith("fail")]
    failed = {}
    for l in fails:
        failed.setdefault(l[7:].split(":")[0], []).append(l)
    report = [f"not in the log: '{l}'" for l in extra]
    report += [f"fail in {name}, {len(ls)} assertions; the first: {ls[0][7:]}" for name, ls in failed.items()]
    report.append(f"the gate: {'pass' if gate else 'fail'} ({passed} pass lines, {len(fails)} fail lines)")
    report.append(f"{found} of {sum(1 for l in b if l)} log lines found in order")
    return report


RENDER = {"timers": render_timers, "access-time": render_access, "chopping": render_chopping, "bandwidth": render_bandwidth,
          "chain-looping": render_chain}

TIMER_ROW = re.compile(r"^Timer(\d), clock:\s*(.+?),\s*(.+?):\s*(.*)$")
SYNC_ROW = re.compile(r"^(\d = .+?)\s{2,}(-?\d+,.*)$")


def numbers(text):
    return [int(v) for v in re.findall(r"-?\d+", text)]


def timer_rows(lines):
    return [(m.group(1, 2, 3), numbers(m.group(4))) for m in map(TIMER_ROW.match, lines) if m]


def sync_rows(lines):
    return [numbers(m.group(2)) for m in map(SYNC_ROW.match, lines) if m]


def sync_tables(lines):
    """Each "Testing Timer..." heading with the sync rows under it, in order."""
    tables = []
    for line in lines:
        if line.startswith("Testing Timer"):
            tables.append((line, []))
        elif tables and (m := SYNC_ROW.match(line)):
            tables[-1][1].append(numbers(m.group(2)))
    return tables


def sync_shape(rec, log):
    """The sync tables against the log's, table by table in order, as WiseMan's gate grades them (Diamond_Native.md §11.12): each row's first
    sample at most 1.2 times the log's largest first sample in the same table and 2 more, each rise at most 1.1 times
    its largest rise and 2 more, and the rows of sync modes 0 and 3, which neither reset nor wrap here, never falling."""
    report, graded, off = [], 0, 0
    ours = sync_tables(rec)
    for i, (heading, rows) in enumerate(sync_tables(log)):
        mine = ours[i][1] if i < len(ours) and ours[i][0] == heading else None
        graded += 1
        if mine is None or len(mine) != len(rows):
            off += 1
            report.append(f"{heading}: missing")
            continue
        first = max(r[0] for r in rows)
        rise = max([b - a for r in rows for a, b in zip(r, r[1:])] + [0])
        bad = []
        for s, vs in enumerate(mine):
            if vs[0] > 1.2 * first + 2:
                bad.append(f"mode {s} starts at {vs[0]}, the log's largest {first}")
            if any(b - a > 1.1 * rise + 2 for a, b in zip(vs, vs[1:])):
                bad.append(f"mode {s} rises faster than the log's {rise}")
            if s in (0, 3) and any(b < a for a, b in zip(vs, vs[1:])):
                bad.append(f"mode {s} falls")
        if bad:
            off += 1
            report.append(f"{heading[8:60]}...: " + "; ".join(bad))
    report.append(f"{graded - off} of {graded} sync tables in the log's shape")
    return report


def step(vs):
    """The commonest rise between successive samples: the loop's cycles in the counter's ticks."""
    rises = [b - a for a, b in zip(vs, vs[1:]) if b > a]
    return statistics.mode(rises) if len(rises) >= 2 else None


def equal_timers(rec, tty):
    """The checks of Diamond_Records.md §2.3, row class by row class."""
    report, ok = [], True
    for (key, mine), (_, theirs) in zip(timer_rows(rec), timer_rows(tty)):
        label = "Timer%s/%s/%s" % key
        if key[2].endswith("frame delay"):
            a, b = statistics.median(mine[2:]), statistics.median(theirs[2:])
            report.append(f"{label}: settled median {a} against {b}, {a - b:+} (reported, not graded: Diamond_Records.md §2.3)")
        elif key[1] == "System clock":
            same = mine == theirs
            ok &= same
            report.append(f"{label}: {'equal' if same else f'{mine} against {theirs}'}")
        else:
            both = mine + theirs
            same = max(both) - min(both) <= 1
            ok &= same
            report.append(f"{label}: {'within one tick, as the release' if same else f'{sorted(set(mine))} against {sorted(set(theirs))}'}")
    pairs = [(step(x), step(y)) for x, y in zip(sync_rows(rec), sync_rows(tty))]
    counted = [(x, y) for x, y in pairs if x is not None and y is not None]
    report += [f"sync row {i}: rises by {x} against {y}" for i, (x, y) in enumerate(pairs) if x is not None and y is not None and x != y]
    report.append(f"sync rows: {sum(x == y for x, y in counted)} of the {len(counted)} that count in both rise by the same step")
    checks = lambda lines: [l for l in lines if l.startswith(("Reached", "Counter reset", "Target"))]
    same = checks(rec) == checks(tty)
    ok &= same
    report.append(f"the target check's four booleans: {'equal' if same else 'differ'}")
    counts = lambda lines: [numbers(l)[1] for l in lines if l.startswith("counter ==")]
    report.append(f"the target check's counts: {counts(rec)} against {counts(tty)}, not graded (Diamond_Records.md §2.3)")
    reads = lambda lines: [l for l in lines if l.startswith(("Write", "Read"))]
    off = [f"'{a}' against '{b}'" for a, b in zip(reads(rec), reads(tty)) if a != b]
    ok &= not off
    report.append("the mode reads: " + ("equal" if not off else "; ".join(off)))
    return ok, report


def equal_lines(rec, tty, pattern):
    """Every row the pattern finds, in order, by its numbers."""
    a = [numbers(l) for l in rec if re.match(pattern, l)]
    b = [numbers(l) for l in tty if re.match(pattern, l)]
    off = [(i, x, y) for i, (x, y) in enumerate(zip(a, b)) if x != y]
    report = [f"row {i}: {x} against {y}" for i, x, y in off[:8]] + ([f"and {len(off) - 8} rows more"] if len(off) > 8 else [])
    if off:
        gaps = [next(p - q for p, q in zip(x, y) if p != q) for _, x, y in off if len(x) == len(y)]
        report.append("the first differing number, record less TTY: " + ", ".join(f"{g:+} in {gaps.count(g)}" for g in sorted(set(gaps))))
    if len(a) != len(b):
        report.append(f"{len(a)} rows against {len(b)}")
    report.append(f"{len(b) - len(off)} of {len(b)} rows equal")
    return not off and len(a) == len(b), report


def equal(program, rec, tty):
    if program in CALLS:
        return equal_calls(program, rec, tty)
    if program == "timers":
        return equal_timers(rec, tty)
    if program == "access-time":
        return equal_lines(rec, tty, r"^\S+\s+\(\s*0x")
    if program == "chain-looping":
        return equal_lines(rec, tty, r"^  Work took")
    if program == "chopping":
        busy = lambda ls: [("GPU busy" in l) for l in ls if "took" in l]
        ok, report = equal_lines(rec, tty, r"^DMA2")
        marks = sum(x == y for x, y in zip(busy(rec), busy(tty)))
        return ok, report + [f"{marks} of {len(busy(tty))} busy marks equal"]
    hblanks = lambda ls: [int(m.group(1)) for m in (re.search(r"\(hblanks:\s*(\d+)\)", l) for l in ls) if m]
    a, b = hblanks(rec), hblanks(tty)
    report = [f"{BANDWIDTH[i]}: {x} hblanks against {y}, {x - y:+}" for i, (x, y) in enumerate(zip(a, b)) if x != y]
    return a == b, report + [f"{sum(x == y for x, y in zip(a, b))} of {len(b)} rows' hblanks equal"]


def hundredths(cell):
    whole, _, cents = cell.partition(".")
    return int(whole) + (int(cents) / 100 if cents else 0)


def table(lines):
    rows = {}
    for line in lines:
        m = re.match(r"^(\S+)\s+\(\s*0x[0-9a-fA-F]+\)\s+(.*)$", line)
        if m:
            rows[m.group(1)] = [hundredths(c) for c in m.group(2).split()[:3]]
    return rows


def console(program, rec, log):
    """WiseMan's TTY gate (TtyReader.cs) for timers, access-time and bandwidth; Diamond_Referees.md §6 for chopping."""
    report = []
    if program in CALLS:
        return console_calls(program, rec, log)
    if program == "timers":
        ours = {key: vs[2:] for key, vs in timer_rows(rec)}
        graded = off = 0
        for key, vs in timer_rows(log):
            row = "Timer%s/%s/%s" % key
            tol = (0 if "Hblank" in key[1] else 10 if "Dot clock" in key[1] else 60) if key[2].endswith("frame delay") else \
                (2 if "/8" in key[1] or "Dot clock" in key[1] or "Hblank" in key[1] else 3)
            mean = statistics.mean(vs[2:])
            graded += 1
            bad = [v for v in ours.get(key, []) if abs(v - mean) > tol]
            if bad or key not in ours:
                off += 1
                report.append(f"{row}: {bad[:1] or 'missing'} against {mean:.1f} (tolerance {tol})")
        report.append(f"{graded - off} of {graded} rows within their tolerances")
        report += sync_shape(rec, log)
    elif program == "access-time":
        ours, theirs = table(rec), table(log)
        cells = [(r, i, ours[r][i], v) for r, vs in theirs.items() for i, v in enumerate(vs) if r in ours]
        near = lambda d: sum(abs(a - b) <= d for _, _, a, b in cells)
        report += [f"{r}/{(8, 16, 32)[i]}: {a:.2f} against {b:.2f}" for r, i, a, b in cells if abs(a - b) > 0.2]
        report.append(f"{near(0.2)} of {len(cells)} cells within 0.2 cycles, {near(0.5)} within 0.5")
    elif program == "chain-looping":
        want = [l for l in log if l.startswith("  Work took")]
        have = [l for l in rec if l.startswith("  Work took")]
        words_alike = sum(re.sub(r"\d+", "#", a) == re.sub(r"\d+", "#", b) for a, b in zip(have, want))
        within = sum(abs(numbers(a)[0] - numbers(b)[0]) <= 0.10 * numbers(b)[0] for a, b in zip(have, want))
        report += [f"{a.strip()} against {b.strip()}" for a, b in zip(have, want)]
        report.append(f"{words_alike} of {len(want)} lines' words equal, {within} within 10%")
    elif program == "chopping":
        a = [(numbers(l.split("took")[1])[0], "GPU busy" in l) for l in rec if "took" in l]
        b = [(numbers(l.split("took")[1])[0], "GPU busy" in l) for l in log if "took" in l]
        within = sum(abs(x[0] - y[0]) <= 0.10 * y[0] for x, y in zip(a, b))
        marks = sum(x[1] == y[1] for x, y in zip(a, b))
        report.append(f"{within} of {len(b)} rows within 10%, {marks} of {len(b)} busy marks as the console's")
    else:
        h = lambda ls: {m.group(1).strip(): int(m.group(2)) for m in (re.match(r"^(.+?)\s+dT:.*\(hblanks:\s*(\d+)\)", l) for l in ls) if m}
        ours, theirs = h(rec), h(log)
        report += [f"{k}: {ours.get(k)} against {v}" for k, v in theirs.items() if abs(ours.get(k, 0) - v) > 0.02 * v]
        report.append(f"{sum(abs(ours.get(k, 0) - v) <= 0.02 * v for k, v in theirs.items())} of {len(theirs)} rows within 2%")
    return report


def main(argv):
    if argv[:1] == ["mask"]:
        print("\n".join(mask_bios(sys.stdin.read().replace("\r", "").splitlines())))
        return
    if len(argv) < 3 or argv[0] not in ("render", "equal", "console") or (argv[1] not in RENDER and argv[1] not in CALLS):
        die(__doc__)
    mode, program = argv[0], argv[1]
    if argv[2] == "--tty":
        rec = [l.rstrip() for l in Path(argv[3]).read_text(errors="replace").replace("\r", "").splitlines()]
        argv = argv[:2] + argv[3:]
    else:
        ws = words(argv[2])
        if len(ws) < SIZE[program]:
            die(f"{program}: {len(ws)} words, the record needs more")
        rec = render_calls(program, ws) if program in CALLS else RENDER[program](ws)
    if mode == "render":
        print("\n".join(rec))
        return
    other = [l.rstrip() for l in Path(argv[3]).read_text(errors="replace").replace("\r", "").splitlines()]
    if mode == "equal":
        ok, report = equal(program, rec, other)
        print("\n".join(report))
        print("equal" if ok else "differs")
    else:
        print("\n".join(console(program, rec, other)))


if __name__ == "__main__":
    main(sys.argv[1:])
