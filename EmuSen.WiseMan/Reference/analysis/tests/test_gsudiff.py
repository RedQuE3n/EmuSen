#!/usr/bin/env python3
"""gsudiff reads the probe's ESGT trace - see EmuSen_Debugging_Tools_Reference_v5.md §3.53."""
import os
import struct
import tempfile
import unittest

import gsudiff

# Three steps whose compared registers differ each step, so a correct reader
# sees three distinct states and the collapse keeps all of them.
STEPS = [
    (0x700000, 0x3D, [0x0001, 0x0002, 0x0003] + [0] * 8 + [0x000B, 0, 0x000D, 0x000E, 0]),
    (0x700003, 0x10, [0x0011, 0x0012, 0x0013] + [0] * 8 + [0x001B, 0, 0x001D, 0x001E, 0]),
    (0x700006, 0x4C, [0x0021, 0x0022, 0x0023] + [0] * 8 + [0x002B, 0, 0x002D, 0x002E, 0]),
]


def probe_trace(steps=STEPS, magic=gsudiff.MAGIC, tail=b""):
    blob = bytearray(magic + b"\x01\x00\x00\x00")
    for addr, opcode, regs in steps:
        record = bytearray(gsudiff.RECORD_BYTES)
        record[0:4] = struct.pack("<I", addr & 0xFFFFFF)
        record[4] = opcode
        record[8:40] = struct.pack("<16H", *regs)
        record[40:44] = struct.pack("<I", 6)
        blob += record
    return bytes(blob) + tail


def emusen_trace(steps=STEPS):
    return "".join(
        f"[GSU] {addr >> 16:02X}:{addr & 0xFFFF:04X} op={opcode:02X} "
        f"R0={r[0]:04X} R1={r[1]:04X} R2={r[2]:04X} "
        f"R11={r[11]:04X} R13={r[13]:04X} R14={r[14]:04X}\n"
        for addr, opcode, r in steps
    )


class ProbeTraceTests(unittest.TestCase):
    def written(self, blob, name="mesen_gsutrace_f00100.bin"):
        path = os.path.join(self.dir.name, name)
        with open(path, "wb") as out:
            out.write(blob)
        return path

    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.dir.cleanup)

    # The regression: the header used to be read as the first record, so step 0
    # carried $54475345 - "ESGT" - as its address.
    def test_the_magic_header_is_not_read_as_a_step(self):
        steps = gsudiff.load_probe(self.written(probe_trace()))
        self.assertEqual(len(steps), len(STEPS))
        self.assertEqual(steps[0][0], 0x700000)
        self.assertNotEqual(steps[0][0], 0x54475345)

    def test_records_decode_to_what_the_probe_wrote(self):
        steps = gsudiff.load_probe(self.written(probe_trace()))
        for got, (addr, opcode, regs) in zip(steps, STEPS):
            self.assertEqual(got[0], addr)
            self.assertEqual(got[1], opcode)
            self.assertEqual(got[2], tuple(regs[c] for c in gsudiff.COMPARED))

    def test_a_file_without_the_magic_is_refused(self):
        with self.assertRaises(ValueError):
            gsudiff.load_probe(self.written(probe_trace(magic=b"XXXX")))

    def test_a_partial_trailing_record_is_ignored(self):
        steps = gsudiff.load_probe(self.written(probe_trace(tail=b"\x00" * 9)))
        self.assertEqual(len(steps), len(STEPS))

    # The whole point of the fix: identical streams must report agreement.
    def test_identical_streams_collapse_to_the_same_states(self):
        theirs = gsudiff.load_probe(self.written(probe_trace()))
        path = os.path.join(self.dir.name, "emusen-trace.txt")
        with open(path, "w") as out:
            out.write(emusen_trace())
        ours = gsudiff.load_emusen(path)
        self.assertEqual(gsudiff.collapse(theirs)[0], gsudiff.collapse(ours)[0])


class LabelTests(unittest.TestCase):
    # The label follows the file, so a nestopia trace is not reported as Mesen's.
    def test_the_backend_comes_from_the_filename(self):
        self.assertEqual(gsudiff.backend_of("/d/mesen_gsutrace_f00100.bin"), "mesen")
        self.assertEqual(gsudiff.backend_of("gambatte_gsutrace_f00001.bin"), "gambatte")
        self.assertEqual(gsudiff.backend_of("trace.bin"), "reference")


if __name__ == "__main__":
    unittest.main()
