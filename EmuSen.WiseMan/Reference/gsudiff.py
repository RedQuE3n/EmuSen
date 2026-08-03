#!/usr/bin/env python3
"""Find the first point two GSU instruction streams disagree.

    gsudiff.py <mesen_gsutrace_fNNNNN.bin> <emusen-trace.txt>

The Mesen side comes from MesenProbe's traceUntilFrame argument (18 uint32 per
step: address, opcode, R0-R15). The EmuSen side is the [GSU] lines produced by
`--flag MasterLoggingEnabled --flag SuperFxTraceCountdown=<n>`:

    dotnet run --project EmuSen.Pharaoh -c Release -- <rom> <cap> \
        --commands <script> --flag MasterLoggingEnabled \
        --flag SuperFxTraceCountdown=6000000 | grep '^\\[GSU\\]' > emusen-trace.txt

Both must start from power-on and cover the same frame count.

Why the comparison is shaped this way: the two cores label a step differently.
Mesen primes its pipeline with a NOP at every GSU start and logs R15 before the
fetch, so its (address, opcode) pairing is offset from ours by one - and the
offset grows by one at each start, so no fixed shift aligns them. Register
state is unambiguous though, and the injected NOP changes none of it, so
collapsing runs of identical register tuples on both sides cancels the
difference exactly. Whatever survives that is a real divergence.

See EmuSen_Debugging_Tools_Reference_v5.md §3.39 and Venus_SuperFX.md §9.
"""
import re
import struct
import sys

STEP_WORDS = 18
LINE = re.compile(
    r"\[GSU\] ([0-9A-F]{2}):([0-9A-F]{4}) op=([0-9A-F]{2}).*"
    r"R0=([0-9A-F]{4}) R1=([0-9A-F]{4}) R2=([0-9A-F]{4}) "
    r"R11=([0-9A-F]{4}) R13=([0-9A-F]{4}) R14=([0-9A-F]{4})"
)

# The registers EmuSen's trace line carries; Mesen logs all 16, so the
# comparison is limited to the intersection.
MESEN_COLS = (0, 1, 2, 11, 13, 14)


def load_mesen(path):
    blob = open(path, "rb").read()
    steps = len(blob) // (STEP_WORDS * 4)
    out = []
    for i in range(steps):
        off = i * STEP_WORDS * 4
        addr, opcode = struct.unpack_from("<II", blob, off)
        regs = struct.unpack_from("<16I", blob, off + 8)
        out.append((addr, opcode, tuple(regs[c] for c in MESEN_COLS)))
    return out


def load_emusen(path):
    out = []
    for line in open(path):
        m = LINE.match(line)
        if m:
            g = m.groups()
            addr = (int(g[0], 16) << 16) | int(g[1], 16)
            out.append((addr, int(g[2], 16), tuple(int(x, 16) for x in g[3:])))
    return out


def collapse(steps):
    """Drop steps that left every compared register alone, keeping an index map."""
    kept, index, prev = [], [], None
    for i, step in enumerate(steps):
        if step[2] != prev:
            kept.append(step[2])
            index.append(i)
            prev = step[2]
    return kept, index


def show(label, steps, at, before=16, after=8):
    print(f"\n--- {label} ---")
    for i in range(max(0, at - before), min(len(steps), at + after)):
        addr, opcode, regs = steps[i]
        mark = ">>" if i == at else "  "
        print(
            f"  {mark} [{i:8d}] ${addr:06X} op {opcode:02X} "
            f"R0={regs[0]:04X} R1={regs[1]:04X} R2={regs[2]:04X} "
            f"R11={regs[3]:04X} R13={regs[4]:04X} R14={regs[5]:04X}"
        )


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2

    mesen = load_mesen(sys.argv[1])
    emusen = load_emusen(sys.argv[2])
    print(f"mesen {len(mesen)} steps, emusen {len(emusen)} steps")
    if not mesen or not emusen:
        print("one side is empty - check the trace cap and that both start from power-on")
        return 2

    m_states, m_index = collapse(mesen)
    e_states, e_index = collapse(emusen)
    print(f"after collapsing unchanged states: mesen {len(m_states)}, emusen {len(e_states)}")

    limit = min(len(m_states), len(e_states))
    at = next((i for i in range(limit) if m_states[i] != e_states[i]), None)
    if at is None:
        print(f"\nNo divergence in the {limit} compared states - the streams agree.")
        print("If one side is much shorter, it hit its trace cap; raise it and rerun.")
        return 0

    print(f"\nFIRST DIVERGENCE: mesen step {m_index[at]}, emusen step {e_index[at]}")
    show("Mesen", mesen, m_index[at])
    show("EmuSen", emusen, e_index[at])
    print("\nDisassemble both sides around those addresses (`disasm gsu <addr>`);")
    print("identical registers with different next addresses means a branch condition.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
