#!/usr/bin/env python3
"""The DSP-1 replacement's tables, generated from VenusRT_Native.md §40's formula alone, independently of the Rust.

    dsp1_tables.py [OUT]

Writes, as 16-bit little-endian words (fullsnes's "newer" ROM-image order), the sine table S and the derivative table
D of the member §40.3 records: N = 256 entries a turn, amplitude 2^15 saturated to 7FFFh, each entry floored, a
quarter wave mirrored and negated:

    S[i] = Q(32768 * sin(2*pi*k/256))            k the index folded into the first quarter, the sign by half
    D[i] = Q(32768 * (2*pi/256) * cos(2*pi*i/256)) the same rule, a quarter turn on

Without OUT it prints the count of words and a digest only. VenusRT's crate test `the_tables_equal_the_independent_generator`
runs it and compares the words with the core's own, and firmwarecheck.py --forced takes its output as the formula image.
"""
import hashlib
import math
import struct
import sys

N = 256


def wave(scale, phase):
    out = []
    for i0 in range(N):
        i = (i0 + phase) % N
        half = i % (N // 2)
        k = N // 2 - half if half > N // 4 else half
        m = math.floor(32768.0 * scale * math.sin(2 * math.pi * k / N))
        v = -m if i >= N // 2 else m
        out.append(max(-0x8000, min(0x7FFF, v)))
    return out


def tables():
    return wave(1.0, 0) + wave(2 * math.pi / N, N // 4)


def main(argv):
    words = tables()
    data = b"".join(struct.pack("<h", w) for w in words)
    if len(argv) > 1:
        with open(argv[1], "wb") as f:
            f.write(data)
    print(f"{len(words)} words, sha256 {hashlib.sha256(data).hexdigest()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
