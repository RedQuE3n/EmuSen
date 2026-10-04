#!/usr/bin/env python3
"""The ST010 replacement's sine, generated from VenusRT_Native.md §43's formula alone, independently of the Rust.

    st010_tables.py [OUT]

Writes, as 16-bit little-endian words, the 256-entry sine of the closest member §43 records: amplitude 32,767, each
entry rounded half up, a quarter wave mirrored and negated:

    S[i] = round(32767 * sin(2*pi*k/256))    k the index folded into the first quarter, the sign by half

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


def main(argv):
    words = table()
    data = b"".join(struct.pack("<h", w) for w in words)
    if len(argv) > 1:
        with open(argv[1], "wb") as f:
            f.write(data)
    print(f"{len(words)} words, sha256 {hashlib.sha256(data).hexdigest()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
