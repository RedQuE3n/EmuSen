#!/usr/bin/env python3
"""firmwarecheck counts what a replacement repeats of an original - see EmuSen_Debugging_Tools_Reference_v5.md §3.61."""
import io
import os
import re
import tempfile
import unittest

import firmwarecheck


def _original():
    """64 bytes of a fixed congruential sequence, each 200 or more so no offset or count in a report can be one."""
    x, out = 1, bytearray()
    for _ in range(64):
        x = (x * 1103515245 + 12345) % 2**31
        out.append(200 + (x >> 16) % 56)
    return bytes(out)


# A synthetic "original" with no repeated 4-byte string, so each copy below is found at one offset only.
ORIGINAL = _original()


def replacement_with(copies):
    """Bytes 0-63, distinct from the original, with each (length, at, from) run copied from it."""
    out = bytearray(range(64))
    for n, at, src in copies:
        out[at:at + n] = ORIGINAL[src:src + n]
    return bytes(out)


class FirmwareCheckTests(unittest.TestCase):
    def run_check(self, ours, theirs=ORIGINAL, *args):
        with tempfile.TemporaryDirectory() as d:
            a, b = os.path.join(d, "ours.bin"), os.path.join(d, "theirs.bin")
            with open(a, "wb") as f:
                f.write(ours)
            with open(b, "wb") as f:
                f.write(theirs)
            out = io.StringIO()
            code = firmwarecheck.main([a, b, *args], out)
            return code, out.getvalue()

    def test_a_disjoint_replacement_passes_with_nothing_shared(self):
        code, text = self.run_check(bytes(range(64)))
        self.assertEqual(code, 0)
        self.assertIn("equal at the same offset: 0 of 64 (0.0%)", text)
        self.assertIn("longest common run: 0", text)
        self.assertIn("common runs of 4+ bytes: 0", text)

    def test_a_copied_run_is_found_at_both_offsets_and_fails_past_six(self):
        code, text = self.run_check(replacement_with([(7, 10, 30)]))
        self.assertEqual(code, 1)
        self.assertIn("longest common run: 7 bytes, replacement offset 10, original offset 30", text)
        self.assertIn("  7 bytes: replacement offset 10, original offset 30", text)
        self.assertIn("equal at the same offset: 0 of 64", text)

    def test_an_aligned_copy_counts_at_the_same_offset_and_the_thresholds_are_arguments(self):
        ours = replacement_with([(5, 0, 0), (5, 20, 20)])
        code, text = self.run_check(ours)
        self.assertEqual(code, 0)
        self.assertIn("equal at the same offset: 10 of 64 (15.6%); longest aligned run 5 at 0", text)
        self.assertIn("common runs of 4+ bytes: 2", text)
        self.assertEqual(self.run_check(ours, ORIGINAL, "--max-equal", "10")[0], 1)
        self.assertEqual(self.run_check(ours, ORIGINAL, "--max-run", "4")[0], 1)

    def test_the_window_compares_only_its_offsets_and_reports_them_whole(self):
        ours = replacement_with([(8, 40, 40)])
        code, text = self.run_check(ours, ORIGINAL, "--window", "0:32")
        self.assertEqual(code, 0)
        self.assertIn("equal at the same offset: 0 of 32", text)
        code, text = self.run_check(ours, ORIGINAL, "--window", "32:64")
        self.assertEqual(code, 1)
        self.assertIn("longest common run: 8 bytes, replacement offset 40, original offset 40", text)

    def test_the_report_holds_no_byte_of_the_original(self):
        _, text = self.run_check(replacement_with([(7, 10, 30), (5, 50, 2)]))
        numbers = {int(n) for n in re.findall(r"\b\d+\b", text)}
        self.assertFalse(numbers & set(ORIGINAL), "a byte value of the original appears in the report")
        tokens = {int(t, 16) for t in re.findall(r"\b[0-9A-Fa-f]{2}\b", text)}
        self.assertFalse(tokens & set(ORIGINAL), "a byte of the original appears in hex")

    def test_the_longest_run_agrees_with_a_brute_force_search(self):
        ours = replacement_with([(6, 3, 50), (4, 30, 9), (3, 60, 0)])
        best = max((n for i in range(64) for j in range(64)
                    for n in [next(k for k in range(65) if i + k >= 64 or j + k >= 64 or ours[i + k] != ORIGINAL[j + k])]), default=0)
        self.assertEqual(firmwarecheck.longest_common(ours, ORIGINAL)[0], best)


if __name__ == "__main__":
    unittest.main()
