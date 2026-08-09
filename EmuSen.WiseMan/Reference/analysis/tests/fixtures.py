"""Dump sets written as files, because that is what a dump set is.

Deliberately the same shape as the C# fixture builder these tests replace
(ComparabilityGateTests.WriteSet), so a verdict that changed between the two
implementations could not be blamed on the fixture differing.
"""
import json
import os
import shutil
import struct
import tempfile


class TempDir:
    """A directory that removes itself, for use in setUp/tearDown."""

    def __init__(self, prefix="emusen-analysis-"):
        self.path = tempfile.mkdtemp(prefix=prefix)

    def cleanup(self):
        shutil.rmtree(self.path, ignore_errors=True)


def write_signature(directory, backend, rows, columns=("ram", "work"),
                    system="nes", rom="/fake.nes", board="4", region="ntsc",
                    header_trust="clean", prg=65536, chr_bytes=65536,
                    save_loaded=False, screen_format="Rgba8888"):
    """Write one <backend>_sig.csv. `rows` is [(frame, crc, crc, ...)] over `columns`."""
    os.makedirs(directory, exist_ok=True)
    text = ["# emusen-probe-signature 1",
            f"# backend={backend}",
            f"# system={system}",
            f"# rom={rom}",
            f"# board={board}",
            f"# region={region}",
            f"# headerTrust={header_trust}",
            f"# prg={prg}",
            f"# chr={chr_bytes}",
            f"# saveLoaded={1 if save_loaded else 0}",
            f"# screenFormat={screen_format}",
            "frame," + ",".join(columns) + ",screen"]
    for row in rows:
        frame, crcs = row[0], row[1:]
        text.append(str(frame) + "," + ",".join(f"{crc:08x}" for crc in crcs) + ",00000000")

    path = os.path.join(directory, f"{backend}_sig.csv")
    with open(path, "w", encoding="utf-8") as sig:
        sig.write("\n".join(text) + "\n")
    return path


def rows(count, ram_seed, diverge_at=-1, work=0x1000):
    """The C# fixture's row generator: one column that moves, one that does not."""
    made = []
    for i in range(count):
        ram = 0xDEAD0000 + i if 0 <= diverge_at <= i else ram_seed + i
        made.append((i, ram, work))
    return made


def write_manifest(directory, backend, frame, spaces=(("ram", 2048),), screen=(256, 240),
                   system="nes", rom="/fake.nes", board="4", region="ntsc",
                   header_trust="clean", prg=65536, chr_bytes=65536, save_loaded=False,
                   screen_format="Rgba8888"):
    """Write one manifest, byte-compatible with what both probes emit."""
    os.makedirs(directory, exist_ok=True)
    report = {
        "backend": backend,
        "system": system,
        "rom": rom,
        "frame": frame,
        "identity": {"board": board, "region": region, "headerTrust": header_trust,
                     "prg": prg, "chr": chr_bytes, "saveLoaded": save_loaded},
        "spaces": [{"name": name, "size": size, "file": f"{backend}_{name}_f{frame:05d}.bin"}
                   for name, size in spaces],
    }
    if screen is None:
        report["screen"] = None
    else:
        width, height = screen
        report["screen"] = {"width": width, "height": height, "bytes": width * height * 4,
                            "format": screen_format,
                            "file": f"{backend}_screen_f{frame:05d}.bin"}

    path = os.path.join(directory, f"{backend}_manifest_f{frame:05d}.json")
    with open(path, "w", encoding="utf-8") as manifest:
        json.dump(report, manifest, indent=2)
    return path


def write_screen(directory, backend, frame, pixels, width=4, height=4, fmt="Rgba8888"):
    """Write a raw screen blob. `pixels` is a flat list of (r, g, b) triples."""
    os.makedirs(directory, exist_ok=True)
    if fmt == "PaletteIndex16":
        data = b"".join(struct.pack("<H", p) for p in pixels)
    else:
        data = b"".join(bytes((r, g, b, 0xFF)) for r, g, b in pixels)

    path = os.path.join(directory, f"{backend}_screen_f{frame:05d}.bin")
    with open(path, "wb") as screen:
        screen.write(data)
    return path
