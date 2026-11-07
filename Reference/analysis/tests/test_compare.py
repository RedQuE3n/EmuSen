"""The gates that decide whether a difference means anything.

Ported from ComparabilityGateTests.cs when the comparator moved to Python; the
fixtures are deliberately the same, so a verdict that changed could not be blamed
on the fixture. See EmuSen_Debugging_Tools_Reference_v5.md §3.48 and §3.48a.
"""
import os
import unittest

import compare
import dumpdb
from dumpset import DumpSet
from tests import fixtures


class GateTests(unittest.TestCase):

    def setUp(self):
        self.temp = fixtures.TempDir()
        self.db = dumpdb.open_db(os.path.join(self.temp.path, "dumps.db"))
        self.dictionary = _empty_dictionary(self.temp.path)

    def tearDown(self):
        self.dictionary.close()
        self.db.close()
        self.temp.cleanup()

    def set(self, backend, board="4", region="ntsc", trust="clean",
            screen_format="Rgba8888", rows=None):
        directory = os.path.join(self.temp.path, backend)
        fixtures.write_signature(directory, backend, rows, board=board, region=region,
                                 header_trust=trust, screen_format=screen_format)
        return DumpSet.load(self.db, directory)

    def compare(self, ours, theirs, frame=20, phase_window=4, inputs=()):
        return compare.compare(ours, theirs, frame, phase_window, list(inputs),
                               dictionary=self.dictionary)

    def test_a_board_mismatch_is_reported_before_any_pixel_is_compared(self):
        ours = self.set("emusen", board="65", rows=fixtures.rows(40, 100))
        theirs = self.set("mesen", board="4", trust="",
                          screen_format="PaletteIndex16", rows=fixtures.rows(40, 100))

        report = self.compare(ours, theirs)

        self.assertEqual(compare.NOT_COMPARABLE, report.verdict)
        self.assertIn("board differs", report.summary)

    # The case the gate found on its first real outing - see Moon_Memory.md §4.14.
    def test_a_region_mismatch_is_not_comparable_even_when_the_boards_agree(self):
        ours = self.set("emusen", board="69", region="ntsc", rows=fixtures.rows(40, 100))
        theirs = self.set("mesen", board="69", region="pal", trust="",
                          screen_format="PaletteIndex16", rows=fixtures.rows(40, 100))

        report = self.compare(ours, theirs)

        self.assertEqual(compare.NOT_COMPARABLE, report.verdict)
        self.assertIn("region differs", report.summary)

    # A constant column agrees at every offset, so it must not decide alignment.
    def test_a_column_that_never_changes_does_not_decide_the_alignment(self):
        ours = self.set("emusen", rows=fixtures.rows(60, 500))
        theirs = self.set("mesen", trust="", rows=fixtures.rows(60, 498))

        result = compare.first_divergence(ours, theirs)

        # Our frame f carries ram 500+f and theirs 498+f, so ours[f] == theirs[f+2].
        self.assertEqual(2, result.offset)

    def test_columns_are_rated_by_how_often_they_actually_agree(self):
        ours = self.set("emusen", rows=fixtures.rows(60, 500))
        theirs = self.set("mesen", trust="", rows=fixtures.rows(60, 500))

        result = compare.first_divergence(ours, theirs)

        self.assertEqual(compare.STABLE, _rating(result, "ram"))

    # No stable column means no verdict, rather than a number nobody should trust.
    def test_an_unreliable_column_cannot_carry_a_divergence_claim(self):
        noisy = [(i, (i * 7919) & 0xFFFFFFFF, (i * 104729) & 0xFFFFFFFF) for i in range(60)]

        # Differing screen formats too, so not even the screen column survives.
        ours = self.set("emusen", rows=fixtures.rows(60, 500))
        theirs = self.set("mesen", trust="", screen_format="PaletteIndex16", rows=noisy)

        result = compare.first_divergence(ours, theirs)

        self.assertEqual(compare.NEVER, _rating(result, "ram"))
        self.assertTrue(result.inconclusive)
        self.assertIsNone(result.frame)

    def test_a_stable_column_locates_the_frame_the_streams_parted(self):
        ours = self.set("emusen", rows=fixtures.rows(200, 500))
        theirs = self.set("mesen", trust="", rows=fixtures.rows(200, 500, diverge_at=150))

        result = compare.first_divergence(ours, theirs)

        self.assertFalse(result.inconclusive)
        self.assertEqual(150, result.frame)

    def test_screen_format_differences_drop_the_screen_column_rather_than_failing(self):
        ours = self.set("emusen", rows=fixtures.rows(40, 100))
        theirs = self.set("mesen", trust="", screen_format="PaletteIndex16",
                          rows=fixtures.rows(40, 100))

        result = compare.first_divergence(ours, theirs)

        self.assertNotIn("screen", result.columns)

    # The structural columns are rated like any other and then barred, so that a
    # dictionary claim about them stays verifiable while they carry no evidence.
    def test_a_structural_column_is_rated_but_never_carries_evidence(self):
        rows = [(i, 1, 2, 3) for i in range(40)]
        rows[20] = (20, 1, 2, 99)

        ours_dir = os.path.join(self.temp.path, "ours-structural")
        theirs_dir = os.path.join(self.temp.path, "theirs-structural")
        fixtures.write_signature(ours_dir, "emusen", rows, columns=("ram", "work", "nametable"))
        fixtures.write_signature(theirs_dir, "mesen", [(i, 1, 2, 3) for i in range(40)],
                                 columns=("ram", "work", "nametable"))

        result = compare.first_divergence(DumpSet.load(self.db, ours_dir),
                                          DumpSet.load(self.db, theirs_dir))

        self.assertIn("nametable", result.columns)
        self.assertNotIn("nametable", result.usable)
        self.assertIsNone(result.frame)


class FormattingTests(unittest.TestCase):
    """The two renderings that had to match .NET exactly - see compare.py's header."""

    def test_a_signed_offset_prints_a_sign_either_way_but_a_bare_zero(self):
        self.assertEqual("+3", compare._signed(3))
        self.assertEqual("-3", compare._signed(-3))
        self.assertEqual("0", compare._signed(0))

    def test_percentages_round_half_to_even_the_way_dotnet_does(self):
        # Measured against .NET 10; the retired prediction was that these needed
        # rounding away from zero instead.
        self.assertEqual("0.12", compare._fixed(0.125, 2))
        self.assertEqual("0.62", compare._fixed(0.625, 2))
        self.assertEqual("0.38", compare._fixed(0.375, 2))
        self.assertEqual("0.88", compare._fixed(0.875, 2))
        self.assertEqual("12.2", compare._fixed(12.25, 1))
        self.assertEqual("12.8", compare._fixed(12.75, 1))


def _rating(result, name):
    for column, rating, _ in result.profiles:
        if column == name:
            return rating
    return None


def _empty_dictionary(directory):
    """A dictionary with nothing proven in it, which is what a fresh one is.

    The gates are being tested here, not the annotations; seeding real claims
    would make these tests fail the day one is legitimately promoted.
    """
    import sqlite3

    import dictionary as dictionary_module

    db = sqlite3.connect(os.path.join(directory, "empty-dictionary.db"))
    db.row_factory = sqlite3.Row
    schema = os.path.join(dictionary_module.dictionary_directory(), "schema.sql")
    with open(schema, encoding="utf-8") as text:
        db.executescript(text.read())
    return dictionary_module.Dictionary(db)


if __name__ == "__main__":
    unittest.main()
