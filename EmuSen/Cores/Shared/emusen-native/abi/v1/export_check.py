#!/usr/bin/env python3
"""The core ABI's export check on a built library (EmuSen_CoreAPI.md section 5.4, item 5).

  export_check.py BASELINE LIBRARY

Every required export present, every optional export present if and only if its capability bit is claimed, and no
emusen_core_ symbol the baseline does not list. Presence is read through the loader, which works on every platform;
the listing of symbols needs `nm` and is skipped where there is none (Windows). A library without
emusen_core_abi_version is not on the stable ABI and is reported as such, exit 0, so a pre-stable core passes.
"""

import ctypes
import shutil
import subprocess
import sys


def baseline(path):
    exports, caps = {}, {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            words = line.split()
            if len(words) < 3 or line.startswith("#"):
                continue
            if words[0] == "export":
                exports[words[1]] = words[3] if words[2] == "optional" else None
            elif words[0] == "const" and words[1].startswith("EMUSEN_CAP_"):
                caps[words[1]] = int(words[2], 16)
    return exports, caps


def listed_symbols(library):
    nm = shutil.which("nm")
    if nm is None:
        return None
    flags = ["-gU"] if sys.platform == "darwin" else ["-D", "--defined-only"]
    out = subprocess.run([nm, *flags, library], capture_output=True, text=True, check=True).stdout
    names = set()
    for line in out.splitlines():
        name = line.split()[-1] if line.split() else ""
        name = name[1:] if sys.platform == "darwin" and name.startswith("_") else name
        if name.startswith("emusen_core_"):
            names.add(name)
    return names


def main():
    if len(sys.argv) != 3:
        print(__doc__)
        return 2
    exports, caps = baseline(sys.argv[1])
    lib = ctypes.CDLL(sys.argv[2])
    if not hasattr(lib, "emusen_core_abi_version"):
        print(f"{sys.argv[2]}: not on the stable core ABI (no emusen_core_abi_version); skipped")
        return 0
    lib.emusen_core_abi_version.restype = ctypes.c_uint32
    lib.emusen_core_capabilities.restype = ctypes.c_uint64
    version = lib.emusen_core_abi_version()
    claimed = lib.emusen_core_capabilities()
    problems = []
    if version >> 16 != 1:
        problems.append(f"major {version >> 16}, not 1")
    for name, cap in sorted(exports.items()):
        present = hasattr(lib, name)
        if cap is None and not present:
            problems.append(f"{name} is required and missing")
        elif cap is not None and present != bool(claimed & caps[cap]):
            state = "present" if present else "missing"
            problems.append(f"{name} is {state} but {cap} is {'claimed' if claimed & caps[cap] else 'not claimed'}")
    symbols = listed_symbols(sys.argv[2])
    if symbols is not None:
        for name in sorted(symbols - set(exports)):
            problems.append(f"{name} is exported but not in the baseline")
    for p in problems:
        print(f"error: {p}")
    if not problems:
        listing = f"{len(symbols)} emusen_core_ symbols listed" if symbols is not None else "no nm, symbols not listed"
        print(f"{sys.argv[2]}: ABI {version >> 16}.{version & 0xFFFF}, capabilities {claimed:#x}, exports as the baseline requires ({listing})")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
