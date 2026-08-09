-- The known-differences dictionary - see EmuSen_Debugging_Tools_Reference_v5.md §3.49.
--
-- Every row here is a claim about why two emulators disagree. A wrong claim is
-- worse than no claim: it teaches the comparator to explain away a real defect,
-- and it does so silently and forever. So "proven" is not a word an author may
-- simply type - the trigger at the bottom makes it unreachable without a
-- verification row that actually passed.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS definition (
    id              INTEGER PRIMARY KEY,
    slug            TEXT    NOT NULL UNIQUE,
    title           TEXT    NOT NULL,

    -- What is asserted, in one sentence, falsifiably.
    claim           TEXT    NOT NULL,

    -- Why it happens. Null while the difference is characterised but unexplained,
    -- which is a legitimate and common state.
    cause           TEXT,

    -- provisional: observed, not yet demonstrated. Annotates nothing.
    -- proven:      a check exists and has passed. May annotate a finding.
    -- retracted:   was believed and turned out to be false. Kept, never deleted.
    status          TEXT    NOT NULL DEFAULT 'provisional'
                            CHECK (status IN ('provisional', 'proven', 'retracted')),

    -- What the claim is about, so the comparator can look it up.
    scope           TEXT    NOT NULL CHECK (scope IN ('column', 'identity', 'screen', 'global')),
    subject         TEXT,

    -- Which pairing and machine it applies to; '*' is any.
    left_backend    TEXT    NOT NULL DEFAULT '*',
    right_backend   TEXT    NOT NULL DEFAULT '*',
    system          TEXT    NOT NULL DEFAULT '*',

    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    retracted_at    TEXT,
    retracted_why   TEXT,

    CHECK (status <> 'retracted' OR retracted_why IS NOT NULL)
);

-- The observations that motivated the claim. Evidence is not proof; it is what a
-- reader needs in order to disagree with the claim on the merits.
CREATE TABLE IF NOT EXISTS evidence (
    id              INTEGER PRIMARY KEY,
    definition_id   INTEGER NOT NULL REFERENCES definition(id) ON DELETE CASCADE,
    kind            TEXT    NOT NULL CHECK (kind IN ('measurement', 'source', 'counterexample')),
    description     TEXT    NOT NULL,
    detail          TEXT
);

-- The falsifiable part: a predicate a machine can re-run against a real pair of
-- dump sets. A definition with no check can never be promoted, by construction.
CREATE TABLE IF NOT EXISTS assertion (
    id              INTEGER PRIMARY KEY,
    definition_id   INTEGER NOT NULL REFERENCES definition(id) ON DELETE CASCADE,

    -- column-agreement: the agreement rate of <subject> against <op> <value>
    -- identity-field:   the identity field <subject> differs between the two sides
    -- screen-shift:     the whole frame is offset by <value> scanlines
    kind            TEXT    NOT NULL CHECK (kind IN ('column-agreement', 'identity-field', 'screen-shift')),
    subject         TEXT    NOT NULL,
    op              TEXT    NOT NULL CHECK (op IN ('<=', '>=', '=', '<>')),
    value           REAL    NOT NULL,

    -- The dump pair this was demonstrated on, so anyone can repeat it.
    fixture         TEXT
);

-- One row per time an assertion was actually re-run. This is the only thing that
-- can make a definition proven, and staleness is visible because the commit is
-- recorded: an entry verified against code that has since changed is a claim
-- about the past.
CREATE TABLE IF NOT EXISTS verification (
    id              INTEGER PRIMARY KEY,
    definition_id   INTEGER NOT NULL REFERENCES definition(id) ON DELETE CASCADE,
    assertion_id    INTEGER NOT NULL REFERENCES assertion(id) ON DELETE CASCADE,
    ran_at          TEXT    NOT NULL DEFAULT (datetime('now')),
    commit_sha      TEXT,
    passed          INTEGER NOT NULL CHECK (passed IN (0, 1)),

    -- What was actually seen, so a failure is diagnosable without a rerun.
    observed        TEXT
);

CREATE INDEX IF NOT EXISTS definition_lookup ON definition(scope, subject, status);
CREATE INDEX IF NOT EXISTS verification_by_definition ON verification(definition_id, passed);

-- Everything the comparator is allowed to act on, and nothing else.
CREATE VIEW IF NOT EXISTS proven_definition AS
SELECT d.*,
       (SELECT MAX(ran_at) FROM verification v
         WHERE v.definition_id = d.id AND v.passed = 1) AS last_verified_at,
       (SELECT commit_sha FROM verification v
         WHERE v.definition_id = d.id AND v.passed = 1
         ORDER BY ran_at DESC LIMIT 1)                  AS last_verified_commit
FROM definition d
WHERE d.status = 'proven';

-- "Thorough validation and proof before entry", expressed as something an author
-- cannot talk their way past. Writing status='proven' by hand fails here unless a
-- passing verification already exists, and a verification can only be written by
-- the verifier actually running the assertion.
CREATE TRIGGER IF NOT EXISTS definition_proof_required_insert
BEFORE INSERT ON definition
WHEN NEW.status = 'proven'
BEGIN
    SELECT RAISE(ABORT, 'a definition cannot be inserted as proven: run the verifier');
END;

CREATE TRIGGER IF NOT EXISTS definition_proof_required_update
BEFORE UPDATE OF status ON definition
WHEN NEW.status = 'proven'
 AND NOT EXISTS (SELECT 1 FROM verification v WHERE v.definition_id = NEW.id AND v.passed = 1)
BEGIN
    SELECT RAISE(ABORT, 'cannot promote to proven: no passing verification for this definition');
END;

-- A promoted definition must also have had something to verify.
CREATE TRIGGER IF NOT EXISTS definition_needs_assertion
BEFORE UPDATE OF status ON definition
WHEN NEW.status = 'proven'
 AND NOT EXISTS (SELECT 1 FROM assertion a WHERE a.definition_id = NEW.id)
BEGIN
    SELECT RAISE(ABORT, 'cannot promote to proven: no assertion to re-run');
END;
