//! `results`: the store's answers as lines. The last trials, the slowest units, the flaky tests, a unit's or a test's
//! history, why each unit was in a trial, impact's misses, the quarantine, and everything as JSON lines. Read-only.
//! See EmuSen_Testing.md §3.5 and `man results`.

use crate::store::{since, Store};
use crate::trial::clock;
use emusen_sqlite::Bound;

fn cell(r: &[Option<String>], i: usize) -> &str {
    r.get(i).and_then(|c| c.as_deref()).unwrap_or("-")
}

pub fn last(store: &Store, n: usize) -> Result<String, String> {
    crate::trial::ls(store, n)
}

/// Units by median wall time over the window, the slowest first.
pub fn slowest(store: &Store, n: usize, tier: Option<&str>, from: &str) -> Result<String, String> {
    let rows = store.rows(
        "SELECT u.unit, u.seconds FROM unit_run u JOIN trial t ON t.id = u.trial WHERE u.seconds IS NOT NULL AND u.outcome <> 'reused' AND t.started >= $from AND ($tier = '' OR t.tier = $tier) ORDER BY u.unit",
        &[("$from", Bound::Text(from)), ("$tier", Bound::Text(tier.unwrap_or("")))],
    )?;
    let mut by: std::collections::BTreeMap<String, Vec<f64>> = std::collections::BTreeMap::new();
    for r in rows {
        by.entry(cell(&r, 0).to_string()).or_default().push(cell(&r, 1).parse().unwrap_or(0.0));
    }
    let mut med: Vec<(f64, usize, f64, String)> = by
        .into_iter()
        .map(|(u, mut v)| {
            v.sort_by(|a, b| a.total_cmp(b));
            (v[v.len() / 2], v.len(), *v.last().unwrap_or(&0.0), u)
        })
        .collect();
    med.sort_by(|a, b| b.0.total_cmp(&a.0));
    Ok(med.into_iter().take(n).map(|(m, runs, max, u)| format!("{}\tmedian\t{runs} runs\tmax {}\t{u}\n", clock(m), clock(max))).collect())
}

/// Tests that flaked (passed on a rerun, or two outcomes under one key) since `from`, with their runners.
pub fn flaky(store: &Store, from: &str) -> Result<String, String> {
    let mut out = String::new();
    for r in store.rows(
        "SELECT r.unit, r.test, COUNT(*), SUM(r.outcome = 'flaky'), GROUP_CONCAT(DISTINCT t.runner) FROM result r JOIN trial t ON t.id = r.trial WHERE r.outcome = 'flaky' AND t.started >= $from GROUP BY r.unit, r.test ORDER BY 3 DESC",
        &[("$from", Bound::Text(from))],
    )? {
        out.push_str(&format!("flaky\t{}\t{}\t{} flaky runs\ton {}\n", cell(&r, 0), cell(&r, 1), cell(&r, 3), cell(&r, 4)));
    }
    for r in store.rows("SELECT unit, test, runs, failures FROM flaky", &[])? {
        out.push_str(&format!("two outcomes\t{}\t{}\t{} of {} runs failed under one key\n", cell(&r, 0), cell(&r, 1), cell(&r, 3), cell(&r, 2)));
    }
    if out.is_empty() {
        out.push_str("no flaky tests recorded in the window\n");
    }
    Ok(out)
}

pub fn unit(store: &Store, unit: &str) -> Result<String, String> {
    let rows = store.rows(
        "SELECT t.id, t.started, t.tier, substr(t.tree, 1, 9), u.outcome, u.passed, u.failed, COALESCE(u.seconds, 0), COALESCE(s.reason, '-') FROM unit_run u JOIN trial t ON t.id = u.trial LEFT JOIN selection s ON s.trial = u.trial AND s.unit = u.unit WHERE u.unit = $u OR u.unit LIKE '%:' || $u ORDER BY t.id DESC LIMIT 50",
        &[("$u", Bound::Text(unit))],
    )?;
    Ok(rows.iter().map(|r| format!("{}\t{}\t{}\t{}\t{}\t{} passed, {} failed\t{}\t{}\n", cell(r, 0), cell(r, 1), cell(r, 2), cell(r, 3), cell(r, 4), cell(r, 5), cell(r, 6), clock(cell(r, 7).parse().unwrap_or(0.0)), cell(r, 8))).collect())
}

pub fn test(store: &Store, name: &str) -> Result<String, String> {
    let rows = store.rows(
        "SELECT t.id, t.started, t.runner, r.unit, r.test, r.outcome, r.attempts, COALESCE(r.seconds, 0) FROM result r JOIN trial t ON t.id = r.trial WHERE r.test LIKE '%' || $n || '%' ORDER BY t.id DESC LIMIT 50",
        &[("$n", Bound::Text(name))],
    )?;
    Ok(rows.iter().map(|r| format!("{}\t{}\t{}\t{}\t{}\t{}\t{} attempt(s)\t{:.3}s\n", cell(r, 0), cell(r, 1), cell(r, 2), cell(r, 3), cell(r, 4), cell(r, 5), cell(r, 6), cell(r, 7).parse::<f64>().unwrap_or(0.0))).collect())
}

pub fn selected(store: &Store, trial: &str) -> Result<String, String> {
    let rows = store.rows(
        "SELECT unit, reason, COALESCE(via, ''), COALESCE(key, '-'), COALESCE(reused_from, '') FROM selection WHERE trial = $t ORDER BY unit",
        &[("$t", Bound::Text(trial))],
    )?;
    Ok(rows.iter().map(|r| format!("{}\t{}\t{}\tkey {}{}\n", cell(r, 0), cell(r, 1), cell(r, 2), &cell(r, 3)[..12.min(cell(r, 3).len())], if cell(r, 4).is_empty() { String::new() } else { format!("\treused from trial {}", cell(r, 4)) })).collect())
}

pub fn misses(store: &Store, from: &str) -> Result<String, String> {
    let rows = store.rows("SELECT m.unit, COALESCE(m.test, ''), m.land, COALESCE(m.changed, ''), COALESCE(m.fixed_by, '') FROM miss m JOIN trial t ON t.id = m.nightly WHERE t.started >= $from", &[("$from", Bound::Text(from))])?;
    if rows.is_empty() {
        return Ok("no misses recorded (the nightly ledger fills from phase 2)\n".into());
    }
    Ok(rows.iter().map(|r| format!("miss\t{}\t{}\tland trial {}\tchanged {}\t{}\n", cell(r, 0), cell(r, 1), cell(r, 2), cell(r, 3), cell(r, 4))).collect())
}

pub fn quarantine(store: &Store) -> Result<String, String> {
    let q = store.quarantined()?;
    if q.is_empty() {
        return Ok("nothing is quarantined\n".into());
    }
    Ok(q.iter().map(|(t, o, c, u)| format!("quarantined\t{t}\tuntil {u}\towner {o}\t{c}\n")).collect())
}

fn json(text: &str) -> String {
    emusen_dotnet::json::quoted(text)
}

/// Every trial since `from` as one JSON object a line, with its unit runs.
pub fn export(store: &Store, from: &str) -> Result<String, String> {
    let mut out = String::new();
    for t in store.rows("SELECT id, started, COALESCE(tree, ''), COALESCE(tier, ''), COALESCE(branch, ''), COALESCE(verdict, ''), COALESCE(seconds, 0), COALESCE(runner, ''), COALESCE(source, '') FROM trial WHERE started >= $from ORDER BY id", &[("$from", Bound::Text(from))])? {
        let units: Vec<String> = store
            .rows("SELECT unit, outcome, passed, failed, skipped, COALESCE(seconds, 0) FROM unit_run WHERE trial = $t ORDER BY unit", &[("$t", Bound::Text(cell(&t, 0)))])?
            .iter()
            .map(|u| format!("{{\"unit\":{},\"outcome\":{},\"passed\":{},\"failed\":{},\"skipped\":{},\"seconds\":{}}}", json(cell(u, 0)), json(cell(u, 1)), cell(u, 2), cell(u, 3), cell(u, 4), cell(u, 5)))
            .collect();
        out.push_str(&format!(
            "{{\"trial\":{},\"started\":{},\"tree\":{},\"tier\":{},\"branch\":{},\"verdict\":{},\"seconds\":{},\"runner\":{},\"source\":{},\"units\":[{}]}}\n",
            cell(&t, 0),
            json(cell(&t, 1)),
            json(cell(&t, 2)),
            json(cell(&t, 3)),
            json(cell(&t, 4)),
            json(cell(&t, 5)),
            cell(&t, 6),
            json(cell(&t, 7)),
            json(cell(&t, 8)),
            units.join(",")
        ));
    }
    Ok(out)
}

/// The window's start: `--since` or, without it, the beginning.
pub fn window(given: Option<&str>) -> Result<String, String> {
    match given {
        Some(s) => since(s),
        None => Ok("0000".to_string()),
    }
}
