#!/usr/bin/env python3
"""The ST010 replacement's sine and angle tables, generated from VenusRT_Native.md §43's and §54's formulas alone,
independently of the Rust.

    st010_tables.py [OUT]
    st010_tables.py --atan [OUT]

Writes, as 16-bit little-endian words, the 256-entry sine of the closest member §43 records: amplitude 32,767, each
entry rounded half up, a quarter wave mirrored and negated:

    S[i] = round(32767 * sin(2*pi*k/256))    k the index folded into the first quarter, the sign by half

With --atan it writes instead the 1,024-entry angle of §54.1 that 01h and 05h share, a by rows and b within a row,
each entry the word 256*n:

    n(a, b) = round(256 * atan2(a, b) / (2*pi)), half up    a, b in 0..31, n(0, 0) = 0

Without OUT it prints the count of words and a digest only. VenusRT's crate test `the_st010_sine_equals_the_independent_generator`
runs it and compares the words with the core's own, and firmwarecheck.py --forced takes its output as the formula image.
"""
import hashlib
import math
import struct
import sys

N = 256


def table():
    out = []
    for i in range(N):
        half = i % (N // 2)
        k = N // 2 - half if half > N // 4 else half
        m = min(0x7FFF, math.floor(32767.0 * math.sin(2 * math.pi * k / N) + 0.5))
        out.append(-m if i >= N // 2 else m)
    return out


def atan_table():
    out = []
    for a in range(32):
        for b in range(32):
            n = math.floor(256 * math.atan2(a, b) / (2 * math.pi) + 0.5) if (a, b) != (0, 0) else 0
            out.append(n << 8)
    return out


def main(argv):
    atan = "--atan" in argv
    argv = [a for a in argv if a != "--atan"]
    words = atan_table() if atan else table()
    data = b"".join(struct.pack("<H" if atan else "<h", w) for w in words)
    if len(argv) > 1:
        with open(argv[1], "wb") as f:
            f.write(data)
    print(f"{len(words)} words, sha256 {hashlib.sha256(data).hexdigest()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
