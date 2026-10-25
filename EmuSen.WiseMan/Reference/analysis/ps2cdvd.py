#!/usr/bin/env python3
"""Writes the EE programs that drive the CDVD's registers on a referee (Sapphire_Native.md §25.3), as emusen-mips
assembly for examples/elf.rs; prints the words each records from 00200000h for ps2referee.py's dump.

    ps2cdvd.py ncmd PLAN OUT.s    N commands timed by COP0 Count until the N status's busy bit clears; four words at
                                  entry (disk type, drive status, N status, S status), then five a command (cycles, N
                                  status, I_STAT, drive status, error). PLAN: dvd, dual, cd, reads, thrdvd, thrcd
    ps2cdvd.py scmd PLAN OUT.s    S commands with parameters; 18 words a command (status, count, sixteen result bytes,
                                  the drive status in the last). PLAN: config, tray, rtc
    ps2cdvd.py nsweep OUT.s       N commands 00h-1Fh but 03h with eleven zero parameters, each timed then broken off;
                                  eight words a command

The registers are reached at BF402000h, which PCSX2's lineage maps for the EE and Play! does not. Each wait is bounded,
so a referee that never clears a bit still ends the record with 5A99D00Eh at 001FFFF0h.
"""
import sys

SEEK, STANDBY, STOP, PAUSE, READCD, READDVD = 5, 2, 3, 4, 6, 8


def le(v, n):
    return [(v >> (8 * i)) & 0xFF for i in range(n)]


def seek(s):
    return (SEEK, le(s, 4))


def rdvd(s, n, tail=(0, 0, 0)):
    return (READDVD, le(s, 4) + le(n, 4) + list(tail))


def rdvd8(s, n):
    """ReadDvd with the eight parameter bytes ps2tek lists, which PCSX2 refuses with error 22h."""
    return (READDVD, le(s, 4) + le(n, 4))


def rcd(s, n, size=0):
    return (READCD, le(s, 4) + le(n, 4) + [0, 0, size])


NPLANS = {
    "dvd": [(STANDBY, []), seek(0), seek(0), seek(8), seek(24), seek(40), seek(1000), seek(14000), seek(28764),
            seek(28764 + 14763), seek(43527 + 14765), seek(500000), seek(0), (PAUSE, []), seek(16), (STOP, []), seek(100),
            rdvd8(200, 1), rdvd8(201, 4), rdvd8(205, 16), seek(300000), seek(300016), seek(300100), seek(310000)],
    "dual": [(STANDBY, []), seek(0), seek(599990), seek(600010), seek(600030), seek(599980), seek(1000000), seek(10),
             rdvd8(599998, 4)],
    "reads": [(STANDBY, []), seek(100), rdvd(100, 1), rdvd(101, 1), rdvd(102, 4), rdvd(106, 16), rdvd(122, 64), rdvd(200, 1),
              rdvd(300, 1), rdvd(20000, 1), rdvd(20001, 1), rdvd(20002, 8, (0, 1, 0)), rdvd(20010, 8, (0, 2, 0)),
              rdvd(20018, 8, (0, 0, 1)), (STOP, []), rdvd(50, 1)],
    "thrdvd": [(STANDBY, []), seek(0), seek(15), seek(31), seek(46), seek(62), seek(62), seek(61), seek(45)],
    "thrcd": [(STANDBY, []), seek(0), seek(7), seek(15), seek(22), seek(30), seek(30), seek(29), seek(21)],
    "cd": [(STANDBY, []), seek(0), seek(4), seek(16), seek(1000), seek(4370), seek(4370 + 4370), seek(8740 + 4372), seek(20000),
           seek(29000), seek(100), rcd(200, 1), rcd(201, 4), rcd(205, 4, 1), rcd(209, 4, 2), (STOP, []), seek(50)],
}

P1, P2 = list(range(0x11, 0x21)), list(range(0xA1, 0xB1))
SPLANS = {
    "config": [(0x40, [1, 0, 1]), (0x42, P1), (0x43, []), (0x40, [0, 0, 1]), (0x41, []), (0x43, []),
               (0x40, [1, 1, 1]), (0x42, P2), (0x43, []), (0x40, [0, 1, 1]), (0x41, []), (0x43, []),
               (0x40, [0, 0, 1]), (0x41, []), (0x43, []), (0x40, [0, 0, 2]), (0x41, []), (0x41, []), (0x43, []),
               (0x40, [0, 2, 1]), (0x41, []), (0x43, []), (0x41, [])],
    "tray": [(0x06, [0]), (0x05, []), (0x06, [1]), (0x05, []), (0x06, [2]), (0x06, [0]), (0x06, [0]), (0x06, [1])],
    "rtc": [(0x08, []), (0x09, [0, 0x59, 0x59, 0x23, 0, 0x31, 0x12, 0x99]), (0x08, []), (0x03, [0]), (0x03, [1]), (0x03, [2]),
            (0x15, []), (0x02, [])],
}

HEAD = ["        lui a0, 0x0020", "        lui s0, 0xBF40", "        ori s0, s0, 0x2000"]
DONE = ["        lui a0, 0x0020", "        lui t0, 0x5A99", "        ori t0, t0, 0xD00E", "        sw t0, -16(a0)",
        "end:    beq zero, zero, end", "        nop"]


def busy_wait(label, done, count="0x0100"):
    return ["        lui t2, " + count, f"{label}:     lbu t0, 5(s0)", "        nop", "        andi t0, t0, 0x80", f"        beq t0, zero, {done}",
            "        nop", "        addiu t2, t2, -1", f"        bne t2, zero, {label}", "        nop"]


def ncmd(plan):
    out = list(HEAD)
    for i, reg in enumerate([15, 10, 5, 23]):
        out += [f"        lbu t0, {reg}(s0)", "        nop", f"        sw t0, {4 * i}(a0)"]
    out += ["        addiu a0, a0, 16"]
    for k, (cmd, params) in enumerate(plan):
        out += ["        addiu t0, zero, 255", "        sb t0, 8(s0)"]
        for b in params:
            out += [f"        addiu t0, zero, {b}", "        sb t0, 5(s0)"]
        out += [f"        addiu t1, zero, {cmd}", "        mfc0 s1, $9", "        sb t1, 4(s0)"] + busy_wait(f"w{k}", f"d{k}", "0x0400")
        out += [f"d{k}:     mfc0 s2, $9", "        subu s2, s2, s1", "        sw s2, 0(a0)"]
        for i, reg in enumerate([5, 8, 10, 6]):
            out += [f"        lbu t0, {reg}(s0)", "        nop", f"        sw t0, {4 * (i + 1)}(a0)"]
        out += ["        addiu a0, a0, 20"]
    return out + DONE, 4 + 5 * len(plan)


def scmd(plan):
    out = list(HEAD)
    for k, (cmd, params) in enumerate(plan):
        out += [f"dr{k}:   lbu t0, 23(s0)", "        nop", "        andi t0, t0, 0x40", f"        bne t0, zero, go{k}", "        nop",
                "        lbu t0, 24(s0)", f"        beq zero, zero, dr{k}", "        nop", f"go{k}:   nop"]
        for b in params:
            out += [f"        addiu t0, zero, {b}", "        sb t0, 23(s0)"]
        out += [f"        addiu t1, zero, {cmd}", "        sb t1, 22(s0)", "        lui t2, 0x0010",
                f"w{k}:    lbu t0, 23(s0)", "        nop", "        andi t0, t0, 0x80", f"        beq t0, zero, i{k}", "        nop",
                "        addiu t2, t2, -1", f"        bne t2, zero, w{k}", "        nop",
                f"i{k}:    lbu t0, 23(s0)", "        nop", "        sw t0, 0(a0)", "        addiu t3, zero, 0", "        addiu t4, a0, 8",
                f"m{k}:    lbu t0, 23(s0)", "        nop", "        andi t0, t0, 0x40", f"        bne t0, zero, o{k}", "        nop",
                "        lbu t0, 24(s0)", "        nop", "        sw t0, 0(t4)", "        addiu t4, t4, 4", "        addiu t3, t3, 1",
                "        slti t5, t3, 16", f"        bne t5, zero, m{k}", "        nop",
                f"o{k}:    sw t3, 4(a0)", "        lbu t0, 10(s0)", "        nop", "        sw t0, 68(a0)", "        addiu a0, a0, 72"]
    return out + DONE, 18 * len(plan)


def nsweep():
    out, n = list(HEAD), 0
    for k, cmd in enumerate(c for c in range(0x20) if c != 3):
        out += ["        addiu t0, zero, 255", "        sb t0, 8(s0)"] + ["        sb zero, 5(s0)"] * 11
        out += [f"        addiu t1, zero, {cmd}", "        mfc0 s1, $9", "        sb t1, 4(s0)"] + busy_wait(f"w{k}", f"d{k}", "0x0010")
        out += [f"d{k}:     mfc0 s2, $9", "        subu s2, s2, s1", "        sw s2, 0(a0)", f"        addiu t0, zero, {cmd}", "        sw t0, 24(a0)"]
        for i, reg in enumerate([5, 8, 10, 6, 4]):
            out += [f"        lbu t0, {reg}(s0)", "        nop", f"        sw t0, {4 * (i + 1)}(a0)"]
        out += ["        sb zero, 7(s0)"] + busy_wait(f"b{k}", f"e{k}", "0x0010")
        out += [f"e{k}:     lbu t0, 5(s0)", "        nop", "        sw t0, 28(a0)", "        addiu a0, a0, 32"]
        n += 1
    return out + DONE, 8 * n


if __name__ == "__main__":
    if len(sys.argv) < 3 or sys.argv[1] not in ("ncmd", "scmd", "nsweep"):
        sys.exit(__doc__)
    mode, target = sys.argv[1], sys.argv[-1]
    lines, words = {"ncmd": lambda: ncmd(NPLANS[sys.argv[2]]), "scmd": lambda: scmd(SPLANS[sys.argv[2]]), "nsweep": nsweep}[mode]()
    with open(target, "w") as f:
        f.write("\n".join(lines) + "\n")
    print(words)
