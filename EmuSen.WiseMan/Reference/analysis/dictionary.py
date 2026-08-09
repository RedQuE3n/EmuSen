#!/usr/bin/env python3
"""The known-differences dictionary: built from committed SQL, queried at runtime.

Every row is a claim about why two emulators disagree, and a wrong claim is worse
than no claim - it teaches the comparator to explain away a real defect, silently
and forever. So the schema does not trust its authors: three triggers make
`status = 'proven'` unreachable without a verification row that actually passed,
and a verification row can only be written by the verifier having re-run the
assertion. Writing 'proven' by hand fails.

This module is the reader and the recorder. It cannot promote anything by itself;
`TryPromote` asks the database, and the database refuses unless the evidence is
already there. That is the point - see EmuSen_Debugging_Tools_Reference_v5.md
§3.49 and schema.sql's own header.

Only proven entries are ever returned to the comparator. A provisional claim
explains nothing.
"""
import os
import sqlite3

DB_NAME = "known-differences.db"


def dictionary_directory():
    """Where the schema, the seed and the database live: beside this module."""
    return os.path.dirname(os.path.abspath(__file__))


def open_dictionary(directory=None):
    """Open the dictionary, building it from schema.sql + seed.sql when absent.

    Rebuilt only when the file is missing, which means editing schema.sql has no
    effect on an existing database. That is a known gap rather than a design -
    this database accumulates the verification history that is the only thing
    making 'proven' reachable, so it cannot simply be deleted to pick up a schema
    change. See EmuSen_Stack.md §4.3.
    """
    directory = directory or dictionary_directory()
    path = os.path.join(directory, DB_NAME)
    fresh = not os.path.exists(path)

    db = sqlite3.connect(path)
    db.row_factory = sqlite3.Row

    if fresh:
        with open(os.path.join(directory, "schema.sql"), encoding="utf-8") as schema:
            db.executescript(schema.read())
        seed = os.path.join(directory, "seed.sql")
        if os.path.exists(seed):
            with open(seed, encoding="utf-8") as rows:
                db.executescript(rows.read())

    return Dictionary(db)


class Dictionary:

    def __init__(self, db):
        self.db = db

    def close(self):
        self.db.close()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()
        return False

    def proven(self, scope, subject, system, left_backend, right_backend):
        """Everything proven that could bear on this scope/subject and this pairing.

        A definition with a null subject matches any subject: it is a claim about
        the scope as a whole.
        """
        return self.db.execute(
            """SELECT slug, title, claim, cause, status, scope, subject,
                      left_backend, right_backend, system,
                      last_verified_at, last_verified_commit
                 FROM proven_definition
                WHERE scope = ?
                  AND (? IS NULL OR subject IS NULL OR subject = ?)
                  AND (system = '*' OR system = ?)
                  AND (left_backend = '*' OR left_backend = ?)
                  AND (right_backend = '*' OR right_backend = ?)""",
            (scope, subject, subject, system, left_backend, right_backend)).fetchall()

    def assertions(self):
        """Every re-runnable predicate belonging to a claim that is still standing."""
        return self.db.execute(
            """SELECT a.id, a.definition_id, d.slug, a.kind, a.subject, a.op, a.value, a.fixture
                 FROM assertion a
                 JOIN definition d ON d.id = a.definition_id
                WHERE d.status <> 'retracted'""").fetchall()

    def record_verification(self, assertion, passed, observed, commit):
        with self.db:
            self.db.execute(
                """INSERT INTO verification (definition_id, assertion_id, passed, observed, commit_sha)
                   VALUES (?, ?, ?, ?, ?)""",
                (assertion["definition_id"], assertion["id"], 1 if passed else 0, observed, commit))

    def try_promote(self, definition_id):
        """(promoted, error). Promotion goes through the trigger, so a refusal is the
        schema speaking and not this module deciding."""
        try:
            with self.db:
                changed = self.db.execute(
                    "UPDATE definition SET status = 'proven' "
                    "WHERE id = ? AND status = 'provisional'", (definition_id,)).rowcount
            return changed > 0, ""
        except sqlite3.DatabaseError as refused:
            return False, str(refused)

    def demote(self, definition_id):
        with self.db:
            self.db.execute("UPDATE definition SET status = 'provisional' "
                            "WHERE id = ? AND status = 'proven'", (definition_id,))

    def counts(self):
        row = self.db.execute(
            """SELECT COUNT(*),
                      COALESCE(SUM(status = 'proven'), 0),
                      COALESCE(SUM(status = 'provisional'), 0),
                      COALESCE(SUM(status = 'retracted'), 0)
                 FROM definition""").fetchone()
        return row[0], row[1], row[2], row[3]
