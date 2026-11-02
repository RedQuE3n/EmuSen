-- The testing studio's store, version 1: what ran, on which build, runner and device, why it was selected, how long
-- each test took, what flaked, what was quarantined, and the queue's history. Rows are the record; result files under
-- ~/.cache/emusen/testing/trials/<n>/ are referenced by path and may be pruned. Migrations are numbered files applied
-- forward only, each setting user_version. See EmuSen_Testing.md §4.
PRAGMA user_version = 1;

-- A machine that runs units. Its caps decide which units may go there; a GPU unit records its device per result.
CREATE TABLE IF NOT EXISTS runner (
    id TEXT PRIMARY KEY,
    kind TEXT,
    caps TEXT,
    threads INTEGER,
    ram_gb INTEGER,
    seen TEXT
);

-- Every unit ever seen, so results keep their meaning after a unit is renamed or retired.
CREATE TABLE IF NOT EXISTS unit (
    id TEXT PRIMARY KEY,
    kind TEXT,
    name TEXT,
    component TEXT,
    tier TEXT,
    tags TEXT,
    needs TEXT,
    inputs TEXT,
    first_seen TEXT,
    retired TEXT
);

-- A build a trial ran or reused, so a unit's time can be split into building and testing.
CREATE TABLE IF NOT EXISTS build (
    id INTEGER PRIMARY KEY,
    trial INTEGER REFERENCES trial,
    tree TEXT,
    checkout TEXT,
    kind TEXT,
    profile TEXT,
    toolchain TEXT,
    runner TEXT REFERENCES runner,
    started TEXT,
    seconds REAL,
    outcome TEXT,
    reused_from INTEGER REFERENCES build
);

-- One run of a selection: a land, a developer's fast loop, a nightly, or a log read back by the backfill. source names
-- the file a backfilled or ingested trial came from, so reading it twice adds nothing.
CREATE TABLE IF NOT EXISTS trial (
    id INTEGER PRIMARY KEY,
    started TEXT,
    finished TEXT,
    tree TEXT,
    base TEXT,
    branch TEXT,
    checkout TEXT,
    tier TEXT,
    runner TEXT REFERENCES runner,
    requested_by TEXT,
    verdict TEXT,
    seconds REAL,
    note TEXT,
    source TEXT UNIQUE
);

-- Why each unit was in a trial (impact's rule and the file that brought it), and the key it ran under; a reused
-- unit names the trial whose green result stood in for it.
CREATE TABLE IF NOT EXISTS selection (
    trial INTEGER REFERENCES trial,
    unit TEXT REFERENCES unit,
    reason TEXT,
    via TEXT,
    key TEXT,
    reused_from INTEGER REFERENCES trial,
    PRIMARY KEY (trial, unit)
);

-- A unit's outcome in a trial with its counts and wall time: the one row a log without per-test lines still gives.
CREATE TABLE IF NOT EXISTS unit_run (
    trial INTEGER REFERENCES trial,
    unit TEXT REFERENCES unit,
    outcome TEXT,
    passed INTEGER,
    failed INTEGER,
    skipped INTEGER,
    flaky INTEGER,
    seconds REAL,
    build_seconds REAL,
    log TEXT,
    PRIMARY KEY (trial, unit)
);

-- One test's outcome in a trial: pass, fail, skip or flaky (failed, then passed on a rerun), with its attempts.
CREATE TABLE IF NOT EXISTS result (
    trial INTEGER REFERENCES trial,
    unit TEXT REFERENCES unit,
    test TEXT,
    outcome TEXT,
    seconds REAL,
    attempts INTEGER,
    device TEXT,
    message TEXT,
    output TEXT,
    PRIMARY KEY (trial, unit, test)
);
CREATE INDEX IF NOT EXISTS result_by_test ON result (unit, test);

-- A recording's owner, hash and last cause; filled by golden in phase 2, read by impact's recording rule's reports.
CREATE TABLE IF NOT EXISTS recording (
    path TEXT PRIMARY KEY,
    owner TEXT REFERENCES unit,
    kind TEXT,
    hash TEXT,
    rerecorded_in INTEGER REFERENCES trial,
    cause TEXT,
    commit_sha TEXT
);

-- The files a unit's tests executed, recorded on the nightly in phase 3, to narrow impact inside a project.
CREATE TABLE IF NOT EXISTS coverage (
    unit TEXT REFERENCES unit,
    file TEXT,
    recorded INTEGER REFERENCES trial,
    PRIMARY KEY (unit, file)
);

-- A nightly failure in a unit a land's impact left out although the land changed one of its inputs: impact's misses.
CREATE TABLE IF NOT EXISTS miss (
    nightly INTEGER REFERENCES trial,
    unit TEXT,
    test TEXT,
    land INTEGER REFERENCES trial,
    changed TEXT,
    fixed_by TEXT
);

-- A flaky test allowed to pass on a rerun, by name, with an owner, a cause and an expiry; nothing else is.
CREATE TABLE IF NOT EXISTS quarantine (
    test TEXT PRIMARY KEY,
    unit TEXT,
    owner TEXT,
    cause TEXT,
    since TEXT,
    until TEXT
);

-- Who asked the queue for a slot, when they got it and when it was given back.
CREATE TABLE IF NOT EXISTS queue (
    id INTEGER PRIMARY KEY,
    class TEXT,
    requester TEXT,
    submitted TEXT,
    started TEXT,
    finished TEXT,
    scope TEXT,
    lease TEXT
);

-- A test with two outcomes under one key, or a pass after a failed attempt.
CREATE VIEW IF NOT EXISTS flaky AS
    SELECT r.unit, r.test, COUNT(*) AS runs, SUM(r.outcome NOT IN ('pass', 'skip')) AS failures, MAX(r.attempts) AS attempts
    FROM result r JOIN selection s ON s.trial = r.trial AND s.unit = r.unit
    WHERE s.key IS NOT NULL
    GROUP BY r.unit, r.test, s.key
    HAVING (failures > 0 AND failures < runs) OR MAX(r.attempts) > 1;
