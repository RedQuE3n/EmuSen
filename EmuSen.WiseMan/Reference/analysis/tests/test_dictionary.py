"""The proof discipline is a schema constraint, not a convention.

Ported from KnownDifferencesTests.cs. These reach past the Python API and speak
raw SQL on purpose: the point is that the *database* refuses a claim promoted
without evidence, so a test that could only go through code the same module
provides would be testing the wrong layer. See schema.sql's triggers and
EmuSen_Debugging_Tools_Reference_v5.md §3.49.
"""
import os
import sqlite3
import unittest

import dictionary as dictionary_module
from tests import fixtures


class ProofDisciplineTests(unittest.TestCase):

    def setUp(self):
        self.temp = fixtures.TempDir()
        source = dictionary_module.dictionary_directory()
        with open(os.path.join(source, "schema.sql"), encoding="utf-8") as schema:
            text = schema.read()
        with open(os.path.join(self.temp.path, "schema.sql"), "w", encoding="utf-8") as copy:
            copy.write(text)

        # No seed.sql, so the dictionary starts empty and these tests are about
        # the rules rather than about whatever is currently claimed.
        self.dictionary = dictionary_module.open_dictionary(self.temp.path)
        self.raw = sqlite3.connect(
            os.path.join(self.temp.path, dictionary_module.DB_NAME))
        self.raw.execute("PRAGMA foreign_keys = ON")

    def tearDown(self):
        self.raw.close()
        self.dictionary.close()
        self.temp.cleanup()

    def add_provisional(self, slug, with_assertion):
        with self.raw:
            self.raw.execute(
                "INSERT INTO definition (slug, title, claim, scope, subject) "
                "VALUES (?, 'title', 'claim', 'column', 'palette')", (slug,))
        definition_id = self.raw.execute(
            "SELECT id FROM definition WHERE slug = ?", (slug,)).fetchone()[0]

        if with_assertion:
            with self.raw:
                self.raw.execute(
                    "INSERT INTO assertion (definition_id, kind, subject, op, value) "
                    "VALUES (?, 'column-agreement', 'palette', '<=', 0.05)", (definition_id,))
        return definition_id

    def assertion_for(self, definition_id):
        return self.raw.execute(
            "SELECT id FROM assertion WHERE definition_id = ?", (definition_id,)).fetchone()[0]

    def verify(self, definition_id, passed):
        with self.raw:
            self.raw.execute(
                "INSERT INTO verification (definition_id, assertion_id, passed) VALUES (?, ?, ?)",
                (definition_id, self.assertion_for(definition_id), 1 if passed else 0))

    def test_a_definition_cannot_be_written_as_proven(self):
        with self.assertRaises(sqlite3.DatabaseError) as refused:
            with self.raw:
                self.raw.execute(
                    "INSERT INTO definition (slug, title, claim, scope, subject, status) "
                    "VALUES ('sneaky', 'title', 'claim', 'column', 'palette', 'proven')")

        self.assertIn("cannot be inserted as proven", str(refused.exception))

    def test_promotion_fails_when_nothing_was_ever_verified(self):
        definition_id = self.add_provisional("unverified", with_assertion=True)

        promoted, error = self.dictionary.try_promote(definition_id)

        self.assertFalse(promoted)
        self.assertIn("no passing verification", error)

    # Stronger than "promotion is refused": a definition with nothing to re-run
    # cannot have a verification recorded against it at all, because verification
    # references the assertion it ran.
    def test_a_definition_with_nothing_to_re_run_cannot_even_be_verified(self):
        definition_id = self.add_provisional("no-assertion", with_assertion=False)

        with self.assertRaises(sqlite3.DatabaseError) as refused:
            with self.raw:
                self.raw.execute(
                    "INSERT INTO verification (definition_id, assertion_id, passed) "
                    "VALUES (?, 9999, 1)", (definition_id,))

        self.assertIn("FOREIGN KEY", str(refused.exception))
        self.assertFalse(self.dictionary.try_promote(definition_id)[0])

    def test_a_failing_verification_does_not_promote(self):
        definition_id = self.add_provisional("failed", with_assertion=True)
        self.verify(definition_id, passed=False)

        promoted, error = self.dictionary.try_promote(definition_id)

        self.assertFalse(promoted)
        self.assertIn("no passing verification", error)

    def test_a_passing_verification_promotes_and_the_entry_becomes_visible(self):
        definition_id = self.add_provisional("earned", with_assertion=True)
        self.verify(definition_id, passed=True)

        promoted, _ = self.dictionary.try_promote(definition_id)

        self.assertTrue(promoted)
        self.assertIn("earned", [row["slug"] for row in self.dictionary.proven(
            "column", "palette", "nes", "emusen", "mesen")])

    # A provisional claim must be invisible to the comparator, not merely labelled.
    def test_a_provisional_entry_is_never_returned_to_the_comparator(self):
        self.add_provisional("not-yet", with_assertion=True)

        self.assertNotIn("not-yet", [row["slug"] for row in self.dictionary.proven(
            "column", "palette", "nes", "emusen", "mesen")])

    def test_a_retraction_must_say_why(self):
        with self.assertRaises(sqlite3.DatabaseError):
            with self.raw:
                self.raw.execute(
                    "INSERT INTO definition (slug, title, claim, scope, subject, status) "
                    "VALUES ('silent', 'title', 'claim', 'column', 'palette', 'retracted')")

    def test_a_demoted_entry_goes_back_out_of_sight(self):
        definition_id = self.add_provisional("earned-then-lost", with_assertion=True)
        self.verify(definition_id, passed=True)
        self.dictionary.try_promote(definition_id)

        self.dictionary.demote(definition_id)

        self.assertNotIn("earned-then-lost", [row["slug"] for row in self.dictionary.proven(
            "column", "palette", "nes", "emusen", "mesen")])

    def test_the_counts_report_what_is_in_there(self):
        self.add_provisional("one", with_assertion=True)
        self.add_provisional("two", with_assertion=True)

        total, proven, provisional, retracted = self.dictionary.counts()

        self.assertEqual((2, 0, 2, 0), (total, proven, provisional, retracted))


class SeedTests(unittest.TestCase):
    """The committed seed, which is the dictionary a fresh clone gets."""

    def setUp(self):
        self.temp = fixtures.TempDir()
        source = dictionary_module.dictionary_directory()
        for name in ("schema.sql", "seed.sql"):
            with open(os.path.join(source, name), encoding="utf-8") as text:
                body = text.read()
            with open(os.path.join(self.temp.path, name), "w", encoding="utf-8") as copy:
                copy.write(body)
        self.dictionary = dictionary_module.open_dictionary(self.temp.path)

    def tearDown(self):
        self.dictionary.close()
        self.temp.cleanup()

    def test_nothing_ships_proven(self):
        total, proven, provisional, retracted = self.dictionary.counts()

        self.assertGreater(total, 0)
        self.assertEqual(0, proven)
        self.assertEqual(total, provisional + retracted)

    # A claim that turned out to be false ships as history rather than being
    # deleted, so the seed is not all provisional - which its header used to say.
    def test_a_retracted_claim_is_kept_rather_than_removed(self):
        _, _, _, retracted = self.dictionary.counts()

        self.assertGreater(retracted, 0)

    def test_a_fresh_dictionary_annotates_nothing(self):
        self.assertEqual([], self.dictionary.proven("column", "palette", "nes", "emusen", "mesen"))


if __name__ == "__main__":
    unittest.main()
