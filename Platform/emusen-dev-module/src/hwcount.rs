//! `hwcount`: runs a program, opens the counters at the first stderr line holding one text and closes them at the first
//! holding another, and prints plain rows: `window`, `thread`, `unit` and `function`. See `man hwcount`.

use crate::hw::{Counter, Sampler, Totals};
use crate::units::{by_place, by_unit, parse_maps, parse_nm, parse_perf_map, parse_table, Places, Row};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const USAGE: &str = "Usage: hwcount --from TEXT [--to TEXT] [--per N] [--units FILE] [--functions N] [--period N] -- <program> [args]\n  See man hwcount.";

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub from: String,
    pub to: Option<String>,
    pub per: u64,
    pub units: Option<String>,
    pub functions: usize,
    pub period: u64,
    pub program: Vec<String>,
}

pub fn options(args: &[String]) -> Result<Options, String> {
    let mut o = Options { from: String::new(), to: None, per: 1, units: None, functions: 0, period: 1_000_000, program: Vec::new() };
    let mut words = args.iter().skip(1);
    while let Some(w) = words.next() {
        let mut value = |name: &str| words.next().cloned().ok_or_else(|| format!("hwcount: {name} needs a value\n{USAGE}"));
        let number = |name: &str, v: String| v.parse::<u64>().ok().filter(|&n| n > 0).ok_or_else(|| format!("hwcount: {name} needs a number above 0\n{USAGE}"));
        match w.as_str() {
            "--from" => o.from = value("--from")?,
            "--to" => o.to = Some(value("--to")?),
            "--per" => o.per = number("--per", value("--per")?)?,
            "--units" => o.units = Some(value("--units")?),
            "--functions" => o.functions = number("--functions", value("--functions")?)? as usize,
            "--period" => o.period = number("--period", value("--period")?)?,
            "--" => {
                o.program = words.cloned().collect();
                break;
            }
            other => return Err(format!("hwcount: {other} is not an option\n{USAGE}")),
        }
    }
    if o.from.is_empty() || o.program.is_empty() {
        return Err(format!("hwcount: --from and a program are required\n{USAGE}"));
    }
    Ok(o)
}

fn thread_name(pid: u32, tid: i32) -> String {
    std::fs::read_to_string(format!("/proc/{pid}/task/{tid}/comm")).map_or("?".to_string(), |s| s.trim().to_string())
}

fn threads(pid: u32) -> Vec<i32> {
    let mut tids: Vec<i32> = std::fs::read_dir(format!("/proc/{pid}/task")).map_or(Vec::new(), |d| d.flatten().filter_map(|e| e.file_name().to_str()?.parse().ok()).collect());
    tids.sort();
    tids
}

type Samples = (Vec<(u64, bool)>, u64);

/// What is open over the window.
struct Window {
    counters: Vec<(Counter, String)>,
    started: Instant,
    drain: Option<(Arc<AtomicBool>, JoinHandle<Samples>)>,
}

fn open_window(pid: u32, o: &Options) -> Result<Window, String> {
    let mut counters = Vec::new();
    for tid in threads(pid) {
        counters.push((Counter::open(tid).map_err(|e| format!("hwcount: the counters: {e}; see kernel.perf_event_paranoid"))?, thread_name(pid, tid)));
    }
    let drain = match o.units {
        Some(_) => {
            let mut sampler = Sampler::open(pid as i32, o.period).map_err(|e| format!("hwcount: the sampler: {e}"))?;
            sampler.start();
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            let handle = std::thread::spawn(move || {
                let (mut samples, mut lost) = (Vec::new(), 0);
                while !flag.load(Ordering::Relaxed) {
                    lost += sampler.drain(&mut samples);
                    std::thread::sleep(Duration::from_millis(20));
                }
                lost += sampler.drain(&mut samples);
                (samples, lost)
            });
            Some((stop, handle))
        }
        None => None,
    };
    counters.iter().for_each(|(c, _)| c.start());
    Ok(Window { counters, started: Instant::now(), drain })
}

fn number(v: f64, per: u64) -> String {
    format!("{:.0}", v / per as f64)
}

fn thread_rows(rows: &[(i32, String, Totals)], secs: f64, per: u64) -> String {
    let mut out = format!("window seconds {secs:.3} per {per} ms_each {:.2}\n", secs * 1000.0 / per as f64);
    for (tid, name, t) in rows {
        let ipc = if t.cycles > 0.0 { t.instr / t.cycles } else { 0.0 };
        out.push_str(&format!("thread {tid} {name} instr {} cycles {} ipc {ipc:.2} branches {} misses {}\n", number(t.instr, per), number(t.cycles, per), number(t.branches, per), number(t.misses, per)));
    }
    out
}

/// The `unit` and `function` rows: sample shares of the sampled thread's own counted totals.
fn sampled_rows(rows: &HashMap<String, Row>, table: &[(String, String)], total: Totals, per: u64, functions: usize) -> String {
    let sum = rows.values().fold(Row::default(), |a, r| Row { instr: a.instr + r.instr, cycles: a.cycles + r.cycles });
    let line = |kind: &str, r: Row, name: &str| {
        let (i, c) = (r.instr as f64 / sum.instr.max(1) as f64, r.cycles as f64 / sum.cycles.max(1) as f64);
        format!("{kind} instr {} cycles {} ipc {:.2} share {:.1}% {name}\n", number(i * total.instr, per), number(c * total.cycles, per), r.ipc(), i * 100.0)
    };
    let mut out = String::new();
    for (unit, row) in by_unit(rows, table) {
        out.push_str(&line("unit", row, &unit));
    }
    let mut named: Vec<(&String, &Row)> = rows.iter().collect();
    named.sort_by(|a, b| b.1.instr.cmp(&a.1.instr).then(a.0.cmp(b.0)));
    for (place, row) in named.into_iter().take(functions) {
        out.push_str(&line("function", *row, place));
    }
    out
}

/// Closes the window and returns its rows; the program must still be there for its maps.
fn close_window(w: Window, pid: u32, o: &Options, table: &[(String, String)]) -> Result<String, String> {
    let secs = w.started.elapsed().as_secs_f64();
    let rows: Vec<(i32, String, Totals)> = w.counters.iter().filter_map(|(c, n)| Some((c.tid, n.clone(), c.read().ok()?))).collect();
    let mut report = thread_rows(&rows, secs, o.per);
    if let Some((stop, handle)) = w.drain {
        stop.store(true, Ordering::Relaxed);
        let (samples, lost) = handle.join().map_err(|_| "hwcount: the sampler stopped".to_string())?;
        let maps = std::fs::read_to_string(format!("/proc/{pid}/maps")).unwrap_or_default();
        let exe: PathBuf = std::fs::read_link(format!("/proc/{pid}/exe")).unwrap_or_default();
        let compiled = std::fs::read_to_string(format!("/tmp/perf-{pid}.map")).unwrap_or_default();
        let nm = Command::new("nm").args(["-n", "--defined-only", "-C"]).arg(&exe).output().map_err(|e| format!("hwcount: nm: {e}"))?;
        let name = exe.file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned());
        let places = Places::new(parse_maps(&maps), parse_nm(&String::from_utf8_lossy(&nm.stdout)), parse_perf_map(&compiled), &name);
        let total = rows.iter().find(|r| r.0 == pid as i32).map_or(Totals::default(), |r| r.2);
        report.push_str(&format!("sampled thread {pid} samples {} lost {lost}\n", samples.len()));
        report.push_str(&sampled_rows(&by_place(&samples, &places), table, total, o.per, o.functions));
    }
    Ok(report)
}

pub fn run(args: &[String]) -> Result<String, String> {
    let o = options(args)?;
    let table = match &o.units {
        Some(f) => parse_table(&std::fs::read_to_string(f).map_err(|e| format!("hwcount: {f}: {e}"))?).map_err(|e| format!("hwcount: {f}: {e}"))?,
        None => Vec::new(),
    };
    let mut child = Command::new(&o.program[0]).args(&o.program[1..]).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("hwcount: {}: {e}", o.program[0]))?;
    let pid = child.id();
    let mut stdout = child.stdout.take().expect("the piped stdout");
    let reader = std::thread::spawn(move || {
        let mut text = Vec::new();
        let _ = stdout.read_to_end(&mut text);
        text
    });
    let (mut window, mut report, mut failed) = (None, None, None);
    for line in BufReader::new(child.stderr.take().expect("the piped stderr")).lines() {
        let Ok(line) = line else { break };
        let _ = writeln!(std::io::stderr(), "{line}");
        if window.is_none() && report.is_none() && failed.is_none() && line.contains(&o.from) {
            match open_window(pid, &o) {
                Ok(w) => window = Some(w),
                Err(words) => failed = Some(words),
            }
        } else if o.to.as_deref().is_some_and(|t| line.contains(t)) {
            if let Some(w) = window.take() {
                match close_window(w, pid, &o, &table) {
                    Ok(rows) => report = Some(rows),
                    Err(words) => failed = Some(words),
                }
            }
        }
    }
    if let Some(w) = window.take() {
        report = Some(close_window(w, pid, &o, &table)?);
    }
    let status = child.wait().map_err(|e| format!("hwcount: {e}"))?;
    let printed = reader.join().unwrap_or_default();
    let _ = std::fs::remove_file(format!("/tmp/perf-{pid}.map"));
    if let Some(words) = failed {
        return Err(words);
    }
    let Some(report) = report else { return Err(format!("hwcount: the program ended ({status}) without the line {:?}", o.from)) };
    Ok(format!("{report}exit {}\n{}", status.code().unwrap_or(-1), String::from_utf8_lossy(&printed)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn a_command_line_is_the_options_or_the_usage() {
        let o = options(&args("hwcount --from a --to b --per 200 --period 4000 --functions 5 -- prog --x 1")).unwrap();
        assert_eq!((o.from.as_str(), o.to.as_deref(), o.per, o.period, o.functions), ("a", Some("b"), 200, 4000, 5));
        assert_eq!(o.program, args("prog --x 1"));
        assert!(options(&args("hwcount -- prog")).unwrap_err().contains("required"));
        assert!(options(&args("hwcount --from a")).unwrap_err().contains("required"));
        assert!(options(&args("hwcount --per 0 --from a -- p")).unwrap_err().contains("above 0"));
        assert!(options(&args("hwcount --nope")).unwrap_err().contains("not an option"));
    }

    #[test]
    fn rows_are_scaled_by_the_divisor() {
        let rows = [(7, "main".to_string(), Totals { cycles: 300.0, instr: 600.0, branches: 60.0, misses: 6.0 })];
        let text = thread_rows(&rows, 2.0, 3);
        assert!(text.contains("window seconds 2.000 per 3 ms_each 666.67"), "{text}");
        assert!(text.contains("thread 7 main instr 200 cycles 100 ipc 2.00 branches 20 misses 2"), "{text}");
    }

    #[test]
    fn a_program_is_counted_between_its_two_lines() {
        let program = "echo begin >&2; i=0; while [ $i -lt 300000 ]; do i=$((i+1)); done; echo end >&2; echo printed";
        let text = run(&["hwcount", "--from", "begin", "--to", "end", "--", "bash", "-c", program].map(str::to_string)).unwrap();
        let thread = text.lines().find(|l| l.starts_with("thread ")).unwrap_or_else(|| panic!("{text}"));
        let instr: u64 = thread.split_whitespace().nth(4).unwrap().parse().unwrap();
        assert!(instr > 1_000_000 && text.contains("exit 0") && text.ends_with("printed\n"), "{text}");
    }

    #[test]
    fn a_line_that_never_comes_is_an_error() {
        let words = run(&["hwcount", "--from", "never", "--", "bash", "-c", "true"].map(str::to_string)).unwrap_err();
        assert!(words.contains("without the line"), "{words}");
    }
}
