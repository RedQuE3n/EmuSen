"""The ingest: what the probe wrote is what the database says it wrote."""
import os
import unittest

import dumpdb
from tests import fixtures


class IngestTests(unittest.TestCase):

    def setUp(self):
        self.temp = fixtures.TempDir()
        self.dir = os.path.join(self.temp.path, "ours")
        self.db = dumpdb.open_db(os.path.join(self.temp.path, "dumps.db"))

    def tearDown(self):
        self.db.close()
        self.temp.cleanup()

    def test_every_crc_survives_the_round_trip(self):
        made = fixtures.rows(40, 100)
        fixtures.write_signature(self.dir, "emusen", made)

        set_id = dumpdb.ingest(self.db, self.dir)
        read = dumpdb.signature(self.db, set_id)

        self.assertEqual(40, len(read))
        for frame, ram, work in made:
            self.assertEqual(ram, read[frame]["ram"])
            self.assertEqual(work, read[frame]["work"])
            self.assertEqual(0, read[frame]["screen"])

    def test_the_identity_block_is_read_field_for_field(self):
        fixtures.write_signature(self.dir, "emusen", fixtures.rows(4, 1),
                                 system="nes", rom="/roms/smb3.nes", board="4",
                                 region="pal", header_trust="archaic", prg=262144,
                                 chr_bytes=131072, save_loaded=True,
                                 screen_format="PaletteIndex16")

        set_id = dumpdb.ingest(self.db, self.dir)
        row = self.db.execute("SELECT * FROM dump_set WHERE id = ?", (set_id,)).fetchone()

        self.assertEqual("emusen", row["backend"])
        self.assertEqual("nes", row["system"])
        self.assertEqual("/roms/smb3.nes", row["rom"])
        self.assertEqual("4", row["board"])
        self.assertEqual("pal", row["region"])
        self.assertEqual("archaic", row["header_trust"])
        self.assertEqual(262144, row["prg_bytes"])
        self.assertEqual(131072, row["chr_bytes"])
        self.assertEqual(1, row["save_loaded"])
        self.assertEqual("PaletteIndex16", row["screen_format"])

    def test_a_directory_with_only_manifests_still_identifies_itself(self):
        fixtures.write_manifest(self.dir, "mesen", 120)

        set_id = dumpdb.ingest(self.db, self.dir)
        row = self.db.execute("SELECT * FROM dump_set WHERE id = ?", (set_id,)).fetchone()

        self.assertIsNotNone(set_id)
        self.assertEqual("mesen", row["backend"])
        self.assertEqual([], dumpdb.frames(self.db, set_id))

    def test_a_text_field_takes_the_manifest_and_a_number_takes_the_csv(self):
        # The inconsistency dumpdb.py's docstring records, pinned so that tidying
        # it later has to be a deliberate change with the comparator re-verified.
        fixtures.write_signature(self.dir, "emusen", fixtures.rows(4, 1),
                                 board="65", prg=1024)
        fixtures.write_manifest(self.dir, "emusen", 0, board="4", prg=999999)

        set_id = dumpdb.ingest(self.db, self.dir)
        row = self.db.execute("SELECT * FROM dump_set WHERE id = ?", (set_id,)).fetchone()

        self.assertEqual("4", row["board"])
        self.assertEqual(1024, row["prg_bytes"])

    def test_an_empty_manifest_field_does_not_overwrite_the_csv(self):
        fixtures.write_signature(self.dir, "emusen", fixtures.rows(4, 1), board="65")
        fixtures.write_manifest(self.dir, "emusen", 0, board="")

        set_id = dumpdb.ingest(self.db, self.dir)
        row = self.db.execute("SELECT * FROM dump_set WHERE id = ?", (set_id,)).fetchone()

        self.assertEqual("65", row["board"])

    def test_identity_comes_from_the_first_manifest_not_the_last(self):
        fixtures.write_manifest(self.dir, "mesen", 9, board="first")
        fixtures.write_manifest(self.dir, "mesen", 120, board="later")

        set_id = dumpdb.ingest(self.db, self.dir)
        row = self.db.execute("SELECT * FROM dump_set WHERE id = ?", (set_id,)).fetchone()

        self.assertEqual("first", row["board"])

    def test_spaces_and_screens_are_recorded_for_every_frame(self):
        for frame in (10, 11, 12):
            fixtures.write_manifest(self.dir, "mesen", frame,
                                    spaces=(("ram", 2048), ("oam", 256)))

        set_id = dumpdb.ingest(self.db, self.dir)

        spaces = self.db.execute(
            "SELECT frame, name, bytes, file FROM space WHERE set_id = ? ORDER BY frame, name",
            (set_id,)).fetchall()
        self.assertEqual(6, len(spaces))
        self.assertEqual(("oam", 256, "mesen_oam_f00010.bin"),
                         (spaces[0]["name"], spaces[0]["bytes"], spaces[0]["file"]))

        screens = self.db.execute(
            "SELECT * FROM screen WHERE set_id = ? ORDER BY frame", (set_id,)).fetchall()
        self.assertEqual(3, len(screens))
        self.assertEqual(256, screens[0]["width"])
        self.assertEqual(240, screens[0]["height"])

    def test_a_null_screen_is_absent_rather_than_zero(self):
        # Only the Rust probe emits one, for a backend with no frame buffer.
        fixtures.write_manifest(self.dir, "mesen", 5, screen=None)

        set_id = dumpdb.ingest(self.db, self.dir)

        self.assertEqual(0, self.db.execute(
            "SELECT COUNT(*) FROM screen WHERE set_id = ?", (set_id,)).fetchone()[0])

    def test_re_ingesting_replaces_rather_than_duplicates(self):
        fixtures.write_signature(self.dir, "emusen", fixtures.rows(40, 100))
        first = dumpdb.ingest(self.db, self.dir)
        first_rows = self.db.execute(
            "SELECT COUNT(*) FROM signature WHERE set_id = ?", (first,)).fetchone()[0]

        second = dumpdb.ingest(self.db, self.dir)
        second_rows = self.db.execute(
            "SELECT COUNT(*) FROM signature WHERE set_id = ?", (second,)).fetchone()[0]

        self.assertEqual(1, self.db.execute("SELECT COUNT(*) FROM dump_set").fetchone()[0])
        self.assertEqual(first_rows, second_rows)

    def test_a_shorter_second_run_does_not_leave_the_first_runs_rows_behind(self):
        fixtures.write_signature(self.dir, "emusen", fixtures.rows(40, 100))
        dumpdb.ingest(self.db, self.dir)

        fixtures.write_signature(self.dir, "emusen", fixtures.rows(5, 100))
        set_id = dumpdb.ingest(self.db, self.dir)

        self.assertEqual(5, len(dumpdb.frames(self.db, set_id)))

    def test_a_directory_with_no_dump_in_it_is_not_a_dump_set(self):
        os.makedirs(self.dir, exist_ok=True)
        self.assertIsNone(dumpdb.ingest(self.db, self.dir))

    def test_the_columns_are_whatever_the_probe_exposed(self):
        fixtures.write_signature(self.dir, "emusen", [(0, 1, 2, 3)],
                                 columns=("ram", "nametable", "chr"))

        set_id = dumpdb.ingest(self.db, self.dir)

        self.assertEqual(["chr", "nametable", "ram", "screen"],
                         dumpdb.columns(self.db, set_id))

    def test_set_for_ingests_a_directory_it_has_not_seen(self):
        fixtures.write_signature(self.dir, "emusen", fixtures.rows(4, 1))

        row = dumpdb.set_for(self.db, self.dir)

        self.assertIsNotNone(row)
        self.assertEqual("emusen", row["backend"])

    def test_the_span_view_reports_what_was_ingested(self):
        fixtures.write_signature(self.dir, "emusen",
                                 [(7, 1, 2), (8, 3, 4), (30, 5, 6)])

        set_id = dumpdb.ingest(self.db, self.dir)
        span = self.db.execute("SELECT * FROM set_span WHERE set_id = ?", (set_id,)).fetchone()

        self.assertEqual(3, span["frames"])
        self.assertEqual(7, span["first_frame"])
        self.assertEqual(30, span["last_frame"])

    def test_deleting_a_set_takes_its_rows_with_it(self):
        fixtures.write_signature(self.dir, "emusen", fixtures.rows(10, 1))
        fixtures.write_manifest(self.dir, "emusen", 0)
        set_id = dumpdb.ingest(self.db, self.dir)

        with self.db:
            self.db.execute("DELETE FROM dump_set WHERE id = ?", (set_id,))

        for table in ("signature", "space", "screen"):
            self.assertEqual(0, self.db.execute(
                f"SELECT COUNT(*) FROM {table} WHERE set_id = ?", (set_id,)).fetchone()[0])


if __name__ == "__main__":
    unittest.main()
