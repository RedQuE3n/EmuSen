//! `emusen.testing`, the testing studio: `units`, `impact`, `trial`, `results`, `ingest` and `queue`, each printing
//! lines that pipe into the next and into grep, sort and wc. See EmuSen_Testing.md and each command's man page.

mod adapters;
mod catalogue;
mod config;
mod graph;
mod impact;
mod ingest;
mod queue;
mod reports;
mod repo;
mod results;
mod store;
mod trial;

#[cfg(test)]
mod tests;

use dianash_module::{Answer, Call, Manifest, Module};
use repo::Repo;

pub const COMMANDS: &[(&str, bool, &str)] = &[
    ("units", true, "  units [--kind cargo|dotnet] [--tier T] [--crates] [--key UNIT] [--profile P] [--rev REV] [--repo DIR]\n                                the catalogue of test units, one per line"),
    ("impact", true, "  impact [--base REV] [--head REV] [--tier T] [--depth N] [--explain FILE] [--blast FILE] [--repo DIR]\n                                the units a change can affect, with the rule and the file"),
    ("trial", false, "  trial [--tier T] [--from FILE] [--no-reuse] [--dry-run] [--on RUNNER] [--requested-by WHO] | verdict SHA | ls | show N | quarantine ...\n                                run a selection, judge it and record it"),
    ("results", true, "  results [--last N] [--slowest N] [--flaky] [--misses] [--selected N] [--quarantine] [unit U | test T] [--tier T] [--since D] | export\n                                the store's answers"),
    ("ingest", false, "  ingest FILE... --runner R --tree SHA [--trial N] [--tier T] | --backfill [DIR]\n                                read TRX or JUnit results, or land.sh's old logs, into the store"),
    ("queue", false, "  queue | ask CLASS [--gb N] [--as WHO]\n                                the runner's slots, and a slot for a heavy job"),
];

pub struct TestingCommands;

pub static MODULE: TestingCommands = TestingCommands;

/// A command line's flags and words: `--flag value` pairs (a flag in `switches` takes none) and the rest in order.
pub struct Args {
    pub flags: Vec<(String, String)>,
    pub words: Vec<String>,
}

pub fn parse(args: &[String], switches: &[&str], valued: &[&str]) -> Result<Args, String> {
    let mut a = Args { flags: Vec::new(), words: Vec::new() };
    let mut it = args.iter().skip(1);
    while let Some(w) = it.next() {
        if switches.contains(&w.as_str()) {
            a.flags.push((w.clone(), String::new()));
        } else if valued.contains(&w.as_str()) {
            let v = it.next().ok_or_else(|| format!("{} needs a value", w))?;
            a.flags.push((w.clone(), v.clone()));
        } else if w.starts_with("--") {
            return Err(format!("{w} is not an option"));
        } else {
            a.words.push(w.clone());
        }
    }
    Ok(a)
}

impl Args {
    pub fn get(&self, flag: &str) -> Option<&str> {
        self.flags.iter().rev().find(|(f, _)| f == flag).map(|(_, v)| v.as_str())
    }

    pub fn has(&self, flag: &str) -> bool {
        self.flags.iter().any(|(f, _)| f == flag)
    }

    pub fn number(&self, flag: &str, default: usize) -> Result<usize, String> {
        self.get(flag).map_or(Ok(default), |v| v.parse().map_err(|_| format!("{flag} needs a number")))
    }
}

fn ok(out: String) -> Answer {
    Answer { out, err: String::new(), status: 0 }
}

fn units(args: &[String]) -> Result<Answer, String> {
    let a = parse(args, &["--crates"], &["--kind", "--tier", "--key", "--profile", "--rev", "--repo"])?;
    let repo = Repo::find(a.get("--repo"))?;
    let rev = a.get("--rev").unwrap_or("HEAD");
    let snap = catalogue::Snapshot::read(&repo, rev)?;
    if let Some(unit) = a.get("--key") {
        let profile = a.get("--profile").unwrap_or("test-fast");
        return match catalogue::key(&repo, &snap, unit, profile, rev == "HEAD")? {
            Some(k) => Ok(ok(format!("{}\t{unit}\n{}\n", k.key, k.lines.iter().map(|l| format!("  {l}")).collect::<Vec<_>>().join("\n")))),
            None => Ok(Answer { out: format!("none\t{unit}\tan input has uncommitted changes\n"), err: String::new(), status: 1 }),
        };
    }
    let lines = catalogue::unit_lines(&snap, a.get("--kind"), a.get("--tier"), a.has("--crates"));
    Ok(ok(lines.iter().map(|l| format!("{l}\n")).collect()))
}

fn impact_cmd(args: &[String]) -> Result<Answer, String> {
    let a = parse(args, &[], &["--base", "--head", "--tier", "--depth", "--explain", "--blast", "--repo"])?;
    let repo = Repo::find(a.get("--repo"))?;
    let base = a.get("--base").unwrap_or("WiseMan");
    let head = a.get("--head");
    let rev = head.unwrap_or("HEAD");
    let snap = catalogue::Snapshot::read(&repo, rev)?;
    let mut tier = match &snap.tiers_text {
        Some(t) => config::parse_tier(t, a.get("--tier").unwrap_or("land"))?,
        None => config::Tier { name: a.get("--tier").unwrap_or("land").to_string(), ..config::Tier::default() },
    };
    if let Some(d) = a.get("--depth") {
        tier.depth = Some(d.parse().map_err(|_| "--depth needs a number".to_string())?);
    }
    let files = match a.get("--explain") {
        Some(f) => vec![f.to_string()],
        None => {
            let mut f = repo.changed(base, rev)?;
            if head.is_none() {
                f.extend(repo.uncommitted()?);
                f.sort();
                f.dedup();
            }
            f
        }
    };
    let failed_last = match (head.is_none(), store::Store::open()) {
        (true, Ok(s)) => {
            let branch = repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
            s.rows(
                "SELECT unit FROM unit_run WHERE outcome = 'fail' AND trial = (SELECT MAX(id) FROM trial WHERE branch = $b AND source IS NULL)",
                &[("$b", emusen_sqlite::Bound::Text(branch.trim()))],
            )
            .unwrap_or_default()
            .into_iter()
            .filter_map(|r| r[0].clone())
            .collect()
        }
        _ => Vec::new(),
    };
    let sel = impact::impact(&repo, &snap, &impact::Ask { files: files.clone(), tier: &tier, failed_last });
    let err: String = sel.notes.iter().map(|n| format!("impact: {n}\n")).collect();
    if let Some(blast) = a.get("--blast") {
        let text = if blast == "-" { String::new() } else { std::fs::read_to_string(blast).map_err(|e| format!("{blast}: {e}"))? };
        let b = impact::read_blast(&snap, &text);
        let lines = impact::compare(&sel, &b);
        return Ok(Answer { out: lines.iter().map(|l| format!("{l}\n")).collect(), err, status: 0 });
    }
    if a.get("--explain").is_some() {
        let f = &files[0];
        let mut out = String::new();
        if let Some(owner) = snap.graph.owner(f) {
            let chain = snap.graph.reverse_closure(&owner, tier.depth);
            out.push_str(&format!("owner\t{owner}\nreaches\t{}\n", chain.join("  ")));
        } else {
            out.push_str("owner\tnone\n");
        }
        for l in snap.inputs.matching(f) {
            out.push_str(&format!("line\t{:?}\t{}\t{} ({})\n", l.kind, l.source, l.reason, l.since));
        }
        for p in &sel.picks {
            out.push_str(&format!("{}\n", p.line()));
        }
        return Ok(Answer { out, err, status: 0 });
    }
    Ok(Answer { out: sel.picks.iter().map(|p| format!("{}\n", p.line())).collect(), err, status: 0 })
}

fn trial_cmd(call: &Call, args: &[String]) -> Result<Answer, String> {
    match args.get(1).map(String::as_str) {
        Some("verdict") => {
            let sha = args.get(2).ok_or("trial verdict needs a SHA")?;
            let (out, status) = trial::verdict(&store::Store::open()?, sha)?;
            return Ok(Answer { out, err: String::new(), status });
        }
        Some("ls") => return Ok(ok(trial::ls(&store::Store::open()?, 20)?)),
        Some("show") => return Ok(ok(trial::show(&store::Store::open()?, args.get(2).ok_or("trial show needs a trial number")?)?)),
        Some("quarantine") => {
            let a = parse(&args[1..], &[], &["--owner", "--cause", "--unit", "--days"])?;
            let s = store::Store::open()?;
            return match a.words.first().map(String::as_str) {
                Some("add") => {
                    let test = a.words.get(1).ok_or("trial quarantine add needs a test name")?;
                    Ok(ok(trial::quarantine_add(&s, test, a.get("--unit").unwrap_or(""), a.get("--owner").unwrap_or(""), a.get("--cause").unwrap_or(""), a.number("--days", 14)? as u64)?))
                }
                Some("rm") => {
                    let test = a.words.get(1).ok_or("trial quarantine rm needs a test name")?;
                    s.run("DELETE FROM quarantine WHERE test = $t", &[("$t", emusen_sqlite::Bound::Text(test))])?;
                    Ok(ok(format!("released\t{test}\n")))
                }
                _ => Ok(ok(results::quarantine(&s)?)),
            };
        }
        _ => {}
    }
    let a = parse(args, &["--no-reuse", "--dry-run"], &["--tier", "--from", "--on", "--requested-by", "--base", "--repo"])?;
    let o = trial::Options {
        tier: a.get("--tier").unwrap_or("land").to_string(),
        from: a.get("--from").map(str::to_string),
        no_reuse: a.has("--no-reuse"),
        dry_run: a.has("--dry-run"),
        runner: a.get("--on").unwrap_or("desktop").to_string(),
        requested_by: a.get("--requested-by").map(str::to_string),
        base: a.get("--base").unwrap_or("WiseMan").to_string(),
        repo: a.get("--repo").map(str::to_string),
    };
    if o.runner != "desktop" {
        return Err(format!("trial: runner {} has no adapter yet (phase 2 adds the laptop, the Mac and the handheld)", o.runner));
    }
    let (out, err, status) = trial::run(call.stdin(), &o, &|| call.cancelled())?;
    Ok(Answer { out, err, status })
}

fn results_cmd(args: &[String]) -> Result<Answer, String> {
    let a = parse(args, &["--flaky", "--misses", "--quarantine"], &["--last", "--slowest", "--selected", "--tier", "--since"])?;
    let s = store::Store::open()?;
    let from = results::window(a.get("--since"))?;
    let out = match a.words.first().map(String::as_str) {
        Some("unit") => results::unit(&s, a.words.get(1).ok_or("results unit needs a unit")?)?,
        Some("test") => results::test(&s, a.words.get(1).ok_or("results test needs a test name")?)?,
        Some("export") => results::export(&s, &from)?,
        Some(other) => return Err(format!("results: {other} is not unit, test or export")),
        None if a.get("--slowest").is_some() => results::slowest(&s, a.number("--slowest", 20)?, a.get("--tier"), &from)?,
        None if a.has("--flaky") => results::flaky(&s, &from)?,
        None if a.has("--misses") => results::misses(&s, &from)?,
        None if a.has("--quarantine") => results::quarantine(&s)?,
        None if a.get("--selected").is_some() => results::selected(&s, a.get("--selected").unwrap_or(""))?,
        None => results::last(&s, a.number("--last", 10)?)?,
    };
    Ok(ok(out))
}

fn ingest_cmd(args: &[String]) -> Result<Answer, String> {
    let a = parse(args, &["--backfill"], &["--runner", "--tree", "--trial", "--tier", "--repo"])?;
    let s = store::Store::open()?;
    if a.has("--backfill") {
        let repo = Repo::find(a.get("--repo"))?;
        let base = a.words.first().map(std::path::PathBuf::from).unwrap_or_else(|| repo::home().join(".cache/emusen/tmp"));
        let t = ingest::backfill(&s, &repo, &base)?;
        return Ok(ok(format!(
            "backfill\t{} trials ({} green, {} red, {} unknown)\t{} unit runs\t{} test rows\nsources\t{} land folders ({} cargo logs, {} dotnet logs), {} transcripts ({} named a land folder's branch), {} full-suite logs; {} already ingested; {} crate logs whose crate is gone\n",
            t.trials, t.green, t.red, t.unknown, t.unit_runs, t.tests, t.land_dirs, t.cargo_logs, t.dotnet_logs, t.land_logs, t.land_logs_matched, t.full_logs, t.skipped_sources, t.unresolved_crates
        )));
    }
    if a.words.is_empty() {
        return Err("ingest needs result files, or --backfill".into());
    }
    let runner = a.get("--runner").ok_or("ingest needs --runner")?;
    let tree = a.get("--tree").ok_or("ingest needs --tree")?;
    let trial = a.get("--trial").map(|t| t.parse::<i64>().map_err(|_| "--trial needs a number".to_string())).transpose()?;
    Ok(ok(ingest::files(&s, &a.words, runner, tree, trial, a.get("--tier").unwrap_or("ingest"))?))
}

fn queue_cmd(call: &Call, args: &[String]) -> Result<Answer, String> {
    let a = parse(args, &[], &["--gb", "--min", "--as"])?;
    let classes = config::runner_classes(std::fs::read_to_string(repo::home().join(".config/EmuSen/testing/runners.json")).ok().as_deref(), "desktop");
    match a.words.first().map(String::as_str) {
        None => Ok(ok(queue::list(&classes).iter().map(|l| format!("{l}\n")).collect())),
        Some("ask") => {
            let class = a.words.get(1).ok_or("queue ask needs a class (cpu or gpu)")?;
            let count = classes.iter().find(|(c, _)| c == class).map(|(_, n)| *n).ok_or_else(|| format!("queue: the desktop has no {class} slots"))?;
            let gb: u32 = a.get("--gb").map_or(Ok(10), |g| g.parse().map_err(|_| "--gb needs a number".to_string()))?;
            let who = a.get("--as").map(str::to_string).unwrap_or_else(|| std::env::var("USER").unwrap_or_default());
            let slot = queue::ask(class, count, &who, &|| call.cancelled()).ok_or("queue ask: stopped")?;
            if let Ok(s) = store::Store::open() {
                let _ = s.run("INSERT INTO queue (class, requester, submitted, started, scope) VALUES ($c, $w, $n, $n, $s)", &[("$c", emusen_sqlite::Bound::Text(class)), ("$w", emusen_sqlite::Bound::Text(&who)), ("$n", emusen_sqlite::Bound::Text(&store::now())), ("$s", emusen_sqlite::Bound::Text(&slot.display().to_string()))]);
            }
            Ok(ok(format!("{}\n", queue::prefix(&slot, gb))))
        }
        Some(other) => Err(format!("queue: {other} is not ask")),
    }
}

impl Module for TestingCommands {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(|| Manifest { id: "emusen.testing", version: env!("CARGO_PKG_VERSION"), services: &[], commands: COMMANDS, aliases: &[], jobs: &[], settings: &[], approvals: false }.json())
    }

    fn run(&self, call: &Call) -> i32 {
        let args = call.args();
        let command = call.command();
        let result = match command.as_str() {
            "units" => units(&args),
            "impact" => impact_cmd(&args),
            "trial" => trial_cmd(call, &args),
            "results" => results_cmd(&args),
            "ingest" => ingest_cmd(&args),
            "queue" => queue_cmd(call, &args),
            other => Err(format!("emusen.testing has no command '{other}'")),
        };
        call.answer(match result {
            Ok(a) => a,
            Err(words) => Answer { out: String::new(), err: format!("{command}: {words}\n"), status: 2 },
        })
    }
}
