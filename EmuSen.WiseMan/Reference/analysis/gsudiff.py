#!/usr/bin/env python3
"""Find the first point two GSU instruction streams disagree.

    gsudiff.py <backend>_gsutrace_fNNNNN.bin <emusen-trace.txt>

The reference side is whatever backend the probe was driving; the file names it,
and this tool reads the prefix rather than assuming an emulator. The format is
the probe's own `ESGT` trace, not any emulator's: an 8-byte magic header
followed by 48-byte records - address (u24), opcode, prefix regs, SFR, R0-R15 as
u16, then the instruction's cycle cost. Produce one with the probe's
`traceUntilFrame` positional. The EmuSen side is the [GSU] lines from:

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
difference exactly.

Retired for cross-core SuperFX work, and kept for the single-stream case.
Collapsing cancels a pipeline offset but not two cores running the same jobs in
a different *order*, which is what made it report a confident divergence at step
9 in the §10.7 investigation. §3.40's resyncing differ is the tool for that; see
Venus_SuperFX.md §9. Until 2026-08-09 this script still read the pre-Rust-port
72-byte record and no header, so it reported the magic bytes as an address and
diverged at step 0 against a stream identical to its own - see §3.53.

See EmuSen_Debugging_Tools_Reference_v5.md §3.39, §3.40 and Venus_SuperFX.md §9.
"""
import os
import re
import struct
import sys

MAGIC = b"ESGT"
HEADER_BYTES = 8
RECORD_BYTES = 48
LINE = re.compile(
    r"\[GSU\] ([0-9A-F]{2}):([0-9A-F]{4}) op=([0-9A-F]{2}).*"
    r"R0=([0-9A-F]{4}) R1=([0-9A-F]{4}) R2=([0-9A-F]{4}) "
    r"R11=([0-9A-F]{4}) R13=([0-9A-F]{4}) R14=([0-9A-F]{4})"
)

# The registers EmuSen's trace line carries; the probe records all 16, so the
# comparison is limited to the intersection.
COMPARED = (0, 1, 2, 11, 13, 14)


def backend_of(path):
    """The producing backend, from `<backend>_gsutrace_fNNNNN.bin`."""
    stem = os.path.basename(path)
    return stem.split("_")[0] if "_" in stem else "reference"


def load_probe(path):
    with open(path, "rb") as handle:
        blob = handle.read()
    if blob[:4] != MAGIC:
        raise ValueError(
            f"{path} does not start with {MAGIC.decode()} - "
            "it is not a probe GSU trace, or it predates the header"
        )

    body = blob[HEADER_BYTES:]
    if len(body) % RECORD_BYTES:
        print(
            f"[WARN] {len(body)} bytes is not a multiple of the {RECORD_BYTES}-byte "
            "record; the tail is ignored"
        )

    out = []
    for off in range(0, len(body) - RECORD_BYTES + 1, RECORD_BYTES):
        addr = struct.unpack_from("<I", body, off)[0] & 0xFFFFFF
        opcode = body[off + 4]
        regs = struct.unpack_from("<16H", body, off + 8)
        out.append((addr, opcode, tuple(regs[c] for c in COMPARED)))
    return out


def load_emusen(path):
    out = []
    with open(path) as handle:
        for line in handle:
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

    label = backend_of(sys.argv[1])
    try:
        theirs = load_probe(sys.argv[1])
    except (OSError, ValueError) as error:
        print(f"[ERROR] {error}")
        return 2
    ours = load_emusen(sys.argv[2])

    print(f"{label} {len(theirs)} steps, emusen {len(ours)} steps")
    if not theirs or not ours:
        print("one side is empty - check the trace cap and that both start from power-on")
        return 2

    t_states, t_index = collapse(theirs)
    o_states, o_index = collapse(ours)
    print(f"after collapsing unchanged states: {label} {len(t_states)}, emusen {len(o_states)}")

    limit = min(len(t_states), len(o_states))
    at = next((i for i in range(limit) if t_states[i] != o_states[i]), None)
    if at is None:
        print(f"\nNo divergence in the {limit} compared states - the streams agree.")
        print("If one side is much shorter, it hit its trace cap; raise it and rerun.")
        return 0

    print(f"\nFIRST DIVERGENCE: {label} step {t_index[at]}, emusen step {o_index[at]}")
    show(label, theirs, t_index[at])
    show("EmuSen", ours, o_index[at])
    print("\nDisassemble both sides around those addresses (`disasm gsu <addr>`);")
    print("identical registers with different next addresses means a branch condition.")
    print("If the two runs order the same jobs differently, this is bookkeeping, not a")
    print("bug - use §3.40's resyncing differ instead.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
