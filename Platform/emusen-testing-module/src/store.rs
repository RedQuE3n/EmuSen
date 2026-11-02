//! The store: one SQLite file at `~/.local/share/EmuSen/testing/studio.db` (`EMUSEN_TESTING_DB` for another), opened
//! through the platform's sqlite crate, its schema the committed `Testing/schema/*.sql` applied forward by
//! `user_version`. See EmuSen_Testing.md §4.

use emusen_sqlite::{Bound, Connection, Sqlite};
use std::path::PathBuf;
use std::sync::Arc;

/// The numbered migrations, each a committed file that sets `user_version` to its number.
pub const MIGRATIONS: &[(i64, &str)] = &[(1, include_str!("../../../Testing/schema/001-studio.sql"))];

pub struct Store {
    pub db: Connection,
}

pub fn db_path() -> PathBuf {
    std::env::var_os("EMUSEN_TESTING_DB").map(PathBuf::from).unwrap_or_else(|| crate::repo::home().join(".local/share/EmuSen/testing/studio.db"))
}

/// The SQLite the process has, else `EMUSEN_SQLITE`, else the program's shipped copy from the NuGet cache, else the system's.
fn sqlite() -> Result<Arc<Sqlite>, String> {
    #[cfg(unix)]
    unsafe {
        libc::dlopen(c"libm.so.6".as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL);
    }
    let mut candidates: Vec<String> = std::env::var("EMUSEN_SQLITE").into_iter().collect();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(emusen_sqlite::SHIPPED_NAME).display().to_string());
        }
    }
    if let Ok(versions) = std::fs::read_dir(crate::repo::home().join(".nuget/packages/sqlitepclraw.lib.e_sqlite3")) {
        let mut found: Vec<String> = versions.flatten().map(|v| v.path().join("runtimes/linux-x64/native/libe_sqlite3.so")).filter(|p| p.exists()).map(|p| p.display().to_string()).collect();
        found.sort();
        candidates.extend(found.into_iter().rev());
    }
    candidates.extend(["libsqlite3.so.0".to_string(), "libsqlite3.dylib".to_string(), "sqlite3.dll".to_string()]);
    Sqlite::of_process(&candidates).map(Arc::new)
}

pub fn now() -> String {
    iso(std::time::SystemTime::now())
}

/// A time as UTC `YYYY-MM-DDTHH:MM:SSZ`.
pub fn iso(t: std::time::SystemTime) -> String {
    let secs = t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    let (y, m, d) = civil(days);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Days since 1970-01-01 as a proleptic Gregorian date (Howard Hinnant's algorithm).
fn civil(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `14d`, `12h` or `30m` as an ISO time that long ago.
pub fn since(text: &str) -> Result<String, String> {
    let (n, unit) = text.split_at(text.len().saturating_sub(1));
    let n: u64 = n.parse().map_err(|_| format!("{text} is not a span like 14d, 12h or 30m"))?;
    let secs = match unit {
        "d" => n * 86400,
        "h" => n * 3600,
        "m" => n * 60,
        _ => return Err(format!("{text} is not a span like 14d, 12h or 30m")),
    };
    Ok(iso(std::time::SystemTime::now() - std::time::Duration::from_secs(secs)))
}

impl Store {
    pub fn open() -> Result<Store, String> {
        Store::open_at(db_path())
    }

    pub fn open_at(path: PathBuf) -> Result<Store, String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let sqlite = sqlite()?;
        let mut db = Connection::open(sqlite, &path.display().to_string()).map_err(|f| format!("{}: {}", path.display(), f.message))?;
        db.set_timeout(emusen_sqlite::COMMAND_TIMEOUT);
        let s = Store { db };
        s.exec("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        s.migrate()?;
        Ok(s)
    }

    pub fn version(&self) -> Result<i64, String> {
        let mut v = 0;
        self.db.query("PRAGMA user_version", &[], |r| v = r.int64(0)).map_err(|f| f.message)?;
        Ok(v)
    }

    fn migrate(&self) -> Result<(), String> {
        let have = self.version()?;
        for (n, sql) in MIGRATIONS.iter().filter(|(n, _)| *n > have) {
            self.exec("BEGIN IMMEDIATE")?;
            if let Err(e) = self.exec(sql) {
                let _ = self.exec("ROLLBACK");
                return Err(format!("migration {n}: {e}"));
            }
            self.exec("COMMIT")?;
        }
        Ok(())
    }

    pub fn exec(&self, sql: &str) -> Result<(), String> {
        self.db.execute(sql).map(|_| ()).map_err(|f| f.message)
    }

    pub fn run(&self, sql: &str, p: &[(&str, Bound)]) -> Result<(), String> {
        self.db.query(sql, p, |_| {}).map(|_| ()).map_err(|f| format!("{}: {sql}", f.message))
    }

    /// The text columns of every row, `None` for a NULL; the columns are those the SELECT names.
    pub fn rows(&self, sql: &str, p: &[(&str, Bound)]) -> Result<Vec<Vec<Option<String>>>, String> {
        let n = out_len_hint(sql) as i32;
        let mut out = Vec::new();
        self.db.query(sql, p, |r| out.push((0..n).map(|i| r.text_or_null(i)).collect())).map_err(|f| format!("{}: {sql}", f.message))?;
        Ok(out)
    }

    pub fn unit(&self, id: &str) -> Result<(), String> {
        let (kind, name) = id.split_once(':').unwrap_or(("", id));
        self.run("INSERT OR IGNORE INTO unit (id, kind, name, first_seen) VALUES ($id, $kind, $name, $now)", &[("$id", Bound::Text(id)), ("$kind", Bound::Text(kind)), ("$name", Bound::Text(name)), ("$now", Bound::Text(&now()))])
    }

    pub fn runner(&self, id: &str) -> Result<(), String> {
        self.run("INSERT INTO runner (id, kind, seen) VALUES ($id, $id, $now) ON CONFLICT(id) DO UPDATE SET seen = $now", &[("$id", Bound::Text(id)), ("$now", Bound::Text(&now()))])
    }

    /// A new trial's number.
    pub fn new_trial(&self, t: &NewTrial) -> Result<i64, String> {
        self.runner(&t.runner)?;
        let mut id = 0;
        self.db
            .query(
                "INSERT INTO trial (started, finished, tree, base, branch, checkout, tier, runner, requested_by, verdict, seconds, note, source) VALUES ($started, $finished, $tree, $base, $branch, $checkout, $tier, $runner, $by, $verdict, $seconds, $note, $source) RETURNING id",
                &[
                    ("$started", Bound::Text(&t.started)),
                    ("$finished", t.finished.as_deref().map_or(Bound::Null, Bound::Text)),
                    ("$tree", Bound::Text(&t.tree)),
                    ("$base", t.base.as_deref().map_or(Bound::Null, Bound::Text)),
                    ("$branch", t.branch.as_deref().map_or(Bound::Null, Bound::Text)),
                    ("$checkout", t.checkout.as_deref().map_or(Bound::Null, Bound::Text)),
                    ("$tier", Bound::Text(&t.tier)),
                    ("$runner", Bound::Text(&t.runner)),
                    ("$by", t.requested_by.as_deref().map_or(Bound::Null, Bound::Text)),
                    ("$verdict", t.verdict.as_deref().map_or(Bound::Null, Bound::Text)),
                    ("$seconds", t.seconds.map_or(Bound::Null, Bound::Real)),
                    ("$note", t.note.as_deref().map_or(Bound::Null, Bound::Text)),
                    ("$source", t.source.as_deref().map_or(Bound::Null, Bound::Text)),
                ],
                |r| id = r.int64(0),
            )
            .map_err(|f| f.message)?;
        Ok(id)
    }

    pub fn finish_trial(&self, id: i64, verdict: &str, seconds: f64, note: Option<&str>) -> Result<(), String> {
        self.run(
            "UPDATE trial SET finished = $now, verdict = $verdict, seconds = $seconds, note = COALESCE($note, note) WHERE id = $id",
            &[("$now", Bound::Text(&now())), ("$verdict", Bound::Text(verdict)), ("$seconds", Bound::Real(seconds)), ("$note", note.map_or(Bound::Null, Bound::Text)), ("$id", Bound::Integer(id))],
        )
    }

    pub fn has_source(&self, source: &str) -> Result<bool, String> {
        Ok(!self.rows("SELECT id FROM trial WHERE source = $s", &[("$s", Bound::Text(source))])?.is_empty())
    }

    pub fn selection(&self, trial: i64, unit: &str, reason: &str, via: &str, key: Option<&str>, reused_from: Option<i64>) -> Result<(), String> {
        self.unit(unit)?;
        self.run(
            "INSERT OR REPLACE INTO selection (trial, unit, reason, via, key, reused_from) VALUES ($t, $u, $r, $v, $k, $f)",
            &[("$t", Bound::Integer(trial)), ("$u", Bound::Text(unit)), ("$r", Bound::Text(reason)), ("$v", Bound::Text(via)), ("$k", key.map_or(Bound::Null, Bound::Text)), ("$f", reused_from.map_or(Bound::Null, Bound::Integer))],
        )
    }

    pub fn unit_run(&self, trial: i64, r: &UnitRun) -> Result<(), String> {
        self.unit(&r.unit)?;
        self.run(
            "INSERT OR REPLACE INTO unit_run (trial, unit, outcome, passed, failed, skipped, flaky, seconds, build_seconds, log) VALUES ($t, $u, $o, $p, $f, $s, $k, $sec, $b, $log)",
            &[
                ("$t", Bound::Integer(trial)),
                ("$u", Bound::Text(&r.unit)),
                ("$o", Bound::Text(&r.outcome)),
                ("$p", Bound::Integer(r.passed)),
                ("$f", Bound::Integer(r.failed)),
                ("$s", Bound::Integer(r.skipped)),
                ("$k", Bound::Integer(r.flaky)),
                ("$sec", r.seconds.map_or(Bound::Null, Bound::Real)),
                ("$b", r.build_seconds.map_or(Bound::Null, Bound::Real)),
                ("$log", r.log.as_deref().map_or(Bound::Null, Bound::Text)),
            ],
        )
    }

    pub fn result(&self, trial: i64, unit: &str, t: &crate::reports::TestRow) -> Result<(), String> {
        self.run(
            "INSERT OR IGNORE INTO result (trial, unit, test, outcome, seconds, attempts, device, message) VALUES ($t, $u, $n, $o, $s, $a, $d, $m)",
            &[
                ("$t", Bound::Integer(trial)),
                ("$u", Bound::Text(unit)),
                ("$n", Bound::Text(&t.test)),
                ("$o", Bound::Text(t.outcome.word())),
                ("$s", t.seconds.map_or(Bound::Null, Bound::Real)),
                ("$a", Bound::Integer(t.attempts as i64)),
                ("$d", Bound::Null),
                ("$m", t.message.as_deref().map_or(Bound::Null, Bound::Text)),
            ],
        )
    }

    /// The quarantined tests still in force, as (test, owner, cause, until).
    pub fn quarantined(&self) -> Result<Vec<(String, String, String, String)>, String> {
        Ok(self
            .rows("SELECT test, owner, cause, until FROM quarantine WHERE until >= $now ORDER BY test", &[("$now", Bound::Text(&now()))])?
            .into_iter()
            .map(|r| (r[0].clone().unwrap_or_default(), r[1].clone().unwrap_or_default(), r[2].clone().unwrap_or_default(), r[3].clone().unwrap_or_default()))
            .collect())
    }

    /// The trial whose green result for `unit` under `key` may stand in for a run: same key and runner, not older than
    /// `days`, and no flaky or failed result for the unit under any key in 30 days.
    pub fn reusable(&self, unit: &str, key: &str, runner: &str, days: f64) -> Result<Option<(i64, String, i64)>, String> {
        let oldest = since(&format!("{}m", (days * 1440.0) as u64))?;
        let month = since("30d")?;
        let unsettled = self.rows("SELECT 1 FROM unit_run u JOIN trial t ON t.id = u.trial WHERE u.unit = $u AND t.started >= $m AND (u.flaky > 0 OR u.outcome = 'flaky') LIMIT 1", &[("$u", Bound::Text(unit)), ("$m", Bound::Text(&month))])?;
        if !unsettled.is_empty() {
            return Ok(None);
        }
        let found = self.rows(
            "SELECT t.id, t.started, u.passed FROM selection s JOIN unit_run u ON u.trial = s.trial AND u.unit = s.unit JOIN trial t ON t.id = s.trial \
             WHERE s.unit = $u AND s.key = $k AND t.runner = $r AND u.outcome = 'ok' AND s.reused_from IS NULL AND t.started >= $o ORDER BY t.id DESC LIMIT 1",
            &[("$u", Bound::Text(unit)), ("$k", Bound::Text(key)), ("$r", Bound::Text(runner)), ("$o", Bound::Text(&oldest))],
        )?;
        Ok(found.first().map(|r| (r[0].as_deref().unwrap_or("0").parse().unwrap_or(0), r[1].clone().unwrap_or_default(), r[2].as_deref().unwrap_or("0").parse().unwrap_or(0))))
    }
}

/// How many columns a SELECT names, for reading its rows as text.
fn out_len_hint(sql: &str) -> usize {
    let upper = sql.to_ascii_uppercase();
    let Some(start) = upper.find("SELECT") else { return 1 };
    let end = upper[start..].find(" FROM ").map_or(upper.len(), |e| start + e);
    let cols = &sql[start + 6..end];
    let mut depth = 0;
    let mut n = 1;
    for c in cols.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => n += 1,
            _ => {}
        }
    }
    n
}

pub struct NewTrial {
    pub started: String,
    pub finished: Option<String>,
    pub tree: String,
    pub base: Option<String>,
    pub branch: Option<String>,
    pub checkout: Option<String>,
    pub tier: String,
    pub runner: String,
    pub requested_by: Option<String>,
    pub verdict: Option<String>,
    pub seconds: Option<f64>,
    pub note: Option<String>,
    pub source: Option<String>,
}

impl NewTrial {
    pub fn new(tree: &str, tier: &str, runner: &str) -> NewTrial {
        NewTrial { started: now(), finished: None, tree: tree.into(), base: None, branch: None, checkout: None, tier: tier.into(), runner: runner.into(), requested_by: None, verdict: None, seconds: None, note: None, source: None }
    }
}

#[derive(Clone, Debug, Default)]
pub struct UnitRun {
    pub unit: String,
    pub outcome: String,
    pub passed: i64,
    pub failed: i64,
    pub skipped: i64,
    pub flaky: i64,
    pub seconds: Option<f64>,
    pub build_seconds: Option<f64>,
    pub log: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_date_is_the_proleptic_gregorian_one() {
        assert_eq!(iso(std::time::UNIX_EPOCH), "1970-01-01T00:00:00Z");
        assert_eq!(iso(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_793_577_600)), "2026-11-02T00:00:00Z");
        assert_eq!(out_len_hint("SELECT a, COUNT(b, c), d FROM x"), 3);
        assert!(since("14x").is_err());
    }
}
