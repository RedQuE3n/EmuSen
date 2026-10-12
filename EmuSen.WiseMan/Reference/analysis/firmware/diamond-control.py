#!/usr/bin/env python3
"""The control firmwarecheck.py --control takes for Diamond's open kernel - see EmuSen_Firmware.md §0.

    diamond-control.py <ps1-tests release folder> <size> <out>

The bodies (each PS-EXE past its 2,048-byte header) of ps1-tests' release programs, in sorted path order, cut to <size>
bytes: compiled MIPS code that has nothing to do with any BIOS, built at run time from the local corpus and never kept.
"""
import glob
import os
import sys


def main(argv):
    folder, size, out = argv[0], int(argv[1], 0), argv[2]
    body = bytearray()
    for path in sorted(glob.glob(os.path.join(folder, "**", "*.exe"), recursive=True)):
        with open(path, "rb") as f:
            body += f.read()[2048:]
        if len(body) >= size:
            break
    with open(out, "wb") as f:
        f.write(bytes(body[:size]))
    return 0 if len(body) >= size else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
