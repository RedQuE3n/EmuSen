#!/usr/bin/env python3
"""The ST010 replacement's sine and angle tables, generated from VenusRT_Native.md §43's and §54's formulas alone,
independently of the Rust.

    st010_tables.py [OUT]
    st010_tables.py --atan [OUT]
    st010_tables.py --perspective [OUT]

Writes, as 16-bit little-endian words, the 256-entry sine of the closest member §43 records: amplitude 32,767, each
entry rounded half up, a quarter wave mirrored and negated:

    S[i] = round(32767 * sin(2*pi*k/256))    k the index folded into the first quarter, the sign by half

With --atan it writes instead the 1,024-entry angle of §54.1 that 01h and 05h share, a by rows and b within a row,
each entry the word 256*n:

    n(a, b) = round(256 * atan2(a, b) / (2*pi)), half up    a, b in 0..31, n(0, 0) = 0

With --perspective it writes instead 07h's 176 line scales of §57, the one sequence the exact search found:

    L(n) = round(39421 / (5*n + 44)), half up    n in 0..175

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


def perspective_table():
    return [(2 * 39421 + q) // (2 * q) for q in (5 * n + 44 for n in range(176))]


def main(argv):
    atan = "--atan" in argv
    persp = "--perspective" in argv
    argv = [a for a in argv if a not in ("--atan", "--perspective")]
    words = atan_table() if atan else perspective_table() if persp else table()
    data = b"".join(struct.pack("<H" if atan or persp else "<h", w) for w in words)
    if len(argv) > 1:
        with open(argv[1], "wb") as f:
            f.write(data)
    print(f"{len(words)} words, sha256 {hashlib.sha256(data).hexdigest()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
