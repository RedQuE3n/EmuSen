//! Result files read into test rows: nextest's JUnit (with its `flakyFailure` and `rerunFailure` records), the TRX
//! `dotnet test --logger trx` writes, and the two text logs the backfill reads, `cargo test`'s and `dotnet test`'s
//! default output. The XML reader is the few rules these two formats use: elements, attributes, text, CDATA and the
//! five entities with character references. See EmuSen_Testing.md §3.5.

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Fail,
    Skip,
    /// Failed, then passed on a rerun.
    Flaky,
}

impl Outcome {
    pub fn word(self) -> &'static str {
        match self {
            Outcome::Pass => "pass",
            Outcome::Fail => "fail",
            Outcome::Skip => "skip",
            Outcome::Flaky => "flaky",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TestRow {
    /// The class (dotnet) or binary (cargo) the test belongs to.
    pub group: String,
    pub test: String,
    pub outcome: Outcome,
    pub seconds: Option<f64>,
    pub attempts: u32,
    pub message: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Open(String, Vec<(String, String)>, bool),
    Close(String),
    Text(String),
}

pub fn unescape(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        let Some(end) = after.find(';').filter(|&e| e <= 10) else {
            out.push('&');
            rest = after;
            continue;
        };
        let name = &after[..end];
        let ch = match name {
            "lt" => Some('<'),
            "gt" => Some('>'),
            "amp" => Some('&'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            n if n.starts_with("#x") => u32::from_str_radix(&n[2..], 16).ok().and_then(char::from_u32),
            n if n.starts_with('#') => n[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        };
        match ch {
            Some(c) => out.push(c),
            None => {
                out.push('&');
                out.push_str(name);
                out.push(';');
            }
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// The document's elements and text in order.
pub fn events(xml: &str) -> Vec<Event> {
    let mut out = Vec::new();
    let mut rest = xml;
    while !rest.is_empty() {
        let Some(lt) = rest.find('<') else {
            if !rest.trim().is_empty() {
                out.push(Event::Text(unescape(rest)));
            }
            break;
        };
        if lt > 0 && !rest[..lt].trim().is_empty() {
            out.push(Event::Text(unescape(&rest[..lt])));
        }
        rest = &rest[lt..];
        if let Some(r) = rest.strip_prefix("<![CDATA[") {
            let end = r.find("]]>").unwrap_or(r.len());
            out.push(Event::Text(r[..end].to_string()));
            rest = r.get(end + 3..).unwrap_or("");
        } else if let Some(r) = rest.strip_prefix("<!--") {
            rest = r.find("-->").map_or("", |e| &r[e + 3..]);
        } else if rest.starts_with("<?") || rest.starts_with("<!") {
            rest = rest.find('>').map_or("", |e| &rest[e + 1..]);
        } else if let Some(r) = rest.strip_prefix("</") {
            let end = r.find('>').unwrap_or(r.len());
            out.push(Event::Close(r[..end].trim().to_string()));
            rest = r.get(end + 1..).unwrap_or("");
        } else {
            let r = &rest[1..];
            let mut end = 0;
            let mut quote: Option<char> = None;
            for (i, c) in r.char_indices() {
                match (quote, c) {
                    (Some(q), c) if c == q => quote = None,
                    (None, '"') | (None, '\'') => quote = Some(c),
                    (None, '>') => {
                        end = i;
                        break;
                    }
                    _ => {}
                }
                end = i + c.len_utf8();
            }
            let tag = &r[..end];
            let closed = tag.ends_with('/');
            let tag = tag.trim_end_matches('/');
            let name_end = tag.find(|c: char| c.is_whitespace()).unwrap_or(tag.len());
            let name = tag[..name_end].to_string();
            let mut attrs = Vec::new();
            let mut a = &tag[name_end..];
            while let Some(eq) = a.find('=') {
                let key = a[..eq].trim().to_string();
                let v = a[eq + 1..].trim_start();
                let Some(q) = v.chars().next().filter(|c| *c == '"' || *c == '\'') else { break };
                let Some(close) = v[1..].find(q) else { break };
                attrs.push((key, unescape(&v[1..1 + close])));
                a = &v[close + 2..];
            }
            out.push(Event::Open(name, attrs, closed));
            rest = r.get(end + 1..).unwrap_or("");
        }
    }
    out
}

fn attr<'a>(attrs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// nextest's JUnit: one row per testcase; a `flakyFailure` without a `failure` is a flaky pass, and each recorded
/// attempt counts.
pub fn junit(xml: &str) -> Vec<TestRow> {
    let mut rows = Vec::new();
    let mut suite = String::new();
    let mut current: Option<TestRow> = None;
    let mut in_message: Option<String> = None;
    for e in events(xml) {
        match e {
            Event::Open(name, attrs, closed) => match name.as_str() {
                "testsuite" => suite = attr(&attrs, "name").unwrap_or("").to_string(),
                "testcase" => {
                    let row = TestRow {
                        group: attr(&attrs, "classname").unwrap_or(&suite).to_string(),
                        test: attr(&attrs, "name").unwrap_or("").to_string(),
                        outcome: Outcome::Pass,
                        seconds: attr(&attrs, "time").and_then(|t| t.parse().ok()),
                        attempts: 1,
                        message: None,
                    };
                    if closed {
                        rows.push(row);
                    } else {
                        current = Some(row);
                    }
                }
                "failure" | "error" | "flakyFailure" | "flakyError" | "rerunFailure" | "rerunError" | "skipped" => {
                    if let Some(row) = current.as_mut() {
                        match name.as_str() {
                            "failure" | "error" => row.outcome = Outcome::Fail,
                            "skipped" => row.outcome = Outcome::Skip,
                            "flakyFailure" | "flakyError" => {
                                if row.outcome == Outcome::Pass {
                                    row.outcome = Outcome::Flaky;
                                }
                                row.attempts += 1;
                            }
                            _ => row.attempts += 1,
                        }
                        if row.message.is_none() {
                            row.message = attr(&attrs, "message").map(str::to_string);
                        }
                        if !closed {
                            in_message = Some(name.clone());
                        }
                    }
                }
                _ => {}
            },
            Event::Text(t) => {
                if let (Some(_), Some(row)) = (&in_message, current.as_mut()) {
                    if row.message.is_none() {
                        row.message = Some(t.lines().take(3).collect::<Vec<_>>().join(" ").chars().take(400).collect());
                    }
                }
            }
            Event::Close(name) => {
                if in_message.as_deref() == Some(name.as_str()) {
                    in_message = None;
                }
                if name == "testcase" {
                    if let Some(row) = current.take() {
                        rows.push(row);
                    }
                }
            }
        }
    }
    rows
}

/// "00:01:02.5000000" as seconds.
fn trx_duration(text: &str) -> Option<f64> {
    let mut parts = text.split(':');
    let h: f64 = parts.next()?.parse().ok()?;
    let m: f64 = parts.next()?.parse().ok()?;
    let s: f64 = parts.next()?.parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + s)
}

/// A TRX file: each UnitTestResult with its class from the TestDefinitions; a nested class counts as its outer one.
pub fn trx(xml: &str) -> Vec<TestRow> {
    let mut class_of: HashMap<String, String> = HashMap::new();
    let mut results: Vec<(String, TestRow)> = Vec::new();
    let mut unit_test: Option<String> = None;
    let mut in_message = false;
    for e in events(xml) {
        match e {
            Event::Open(name, attrs, closed) => match name.as_str() {
                "UnitTestResult" => {
                    let outcome = match attr(&attrs, "outcome").unwrap_or("") {
                        "Passed" => Outcome::Pass,
                        "NotExecuted" | "Inconclusive" | "NotRunnable" => Outcome::Skip,
                        _ => Outcome::Fail,
                    };
                    let row = TestRow { group: String::new(), test: attr(&attrs, "testName").unwrap_or("").to_string(), outcome, seconds: attr(&attrs, "duration").and_then(trx_duration), attempts: 1, message: None };
                    results.push((attr(&attrs, "testId").unwrap_or("").to_string(), row));
                    let _ = closed;
                }
                "Message" => in_message = !closed,
                "UnitTest" => unit_test = attr(&attrs, "id").map(str::to_string),
                "TestMethod" => {
                    if let (Some(id), Some(class)) = (&unit_test, attr(&attrs, "className")) {
                        let class = class.split(',').next().unwrap_or(class).trim();
                        class_of.insert(id.clone(), class.split('+').next().unwrap_or(class).to_string());
                    }
                }
                _ => {}
            },
            Event::Text(t) => {
                if in_message {
                    if let Some((_, row)) = results.last_mut() {
                        if row.message.is_none() {
                            row.message = Some(t.lines().take(3).collect::<Vec<_>>().join(" ").chars().take(400).collect());
                        }
                    }
                }
            }
            Event::Close(name) => {
                if name == "Message" {
                    in_message = false;
                }
            }
        }
    }
    results
        .into_iter()
        .map(|(id, mut row)| {
            row.group = class_of.get(&id).cloned().unwrap_or_else(|| class_from_name(&row.test));
            row
        })
        .collect()
}

/// `Ns.Class.Method(args)` as `Ns.Class`.
pub fn class_from_name(test: &str) -> String {
    let bare = test.split('(').next().unwrap_or(test);
    bare.rsplit_once('.').map_or(bare, |(c, _)| c).to_string()
}

/// `cargo test` output: each "test x ... ok" line under its "Running" binary, the build's seconds from "Finished",
/// the profile, and whether the run failed.
#[derive(Debug, Default)]
pub struct CargoLog {
    pub rows: Vec<TestRow>,
    pub build_seconds: Option<f64>,
    pub test_seconds: f64,
    pub profile: Option<String>,
    pub failed: bool,
    pub passed: i64,
    pub failures: i64,
    pub ignored: i64,
}

/// "10m 33s", "25.22s" or "1h 2m 3s" as seconds.
pub fn span_seconds(text: &str) -> Option<f64> {
    let mut total = 0.0;
    let mut any = false;
    for part in text.split_whitespace() {
        let (n, unit) = part.split_at(part.find(|c: char| c.is_ascii_alphabetic())?);
        let n: f64 = n.parse().ok()?;
        total += n * match unit {
            "h" => 3600.0,
            "m" => 60.0,
            "s" => 1.0,
            "ms" => 0.001,
            _ => return None,
        };
        any = true;
    }
    any.then_some(total)
}

pub fn cargo_log(text: &str) -> CargoLog {
    let mut log = CargoLog::default();
    let mut binary = String::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("Running ") {
            binary = rest.split(" (").nth(1).map(|p| p.trim_end_matches(')').rsplit('/').next().unwrap_or("").rsplit_once('-').map_or(p, |(n, _)| n).to_string()).unwrap_or_default();
            if binary.is_empty() {
                binary = rest.to_string();
            }
        } else if let Some(rest) = t.strip_prefix("Finished `") {
            log.profile = rest.split('`').next().map(str::to_string);
            log.build_seconds = rest.rsplit(" in ").next().and_then(span_seconds).map(|s| s + log.build_seconds.unwrap_or(0.0));
        } else if let Some(rest) = line.strip_prefix("test ").filter(|_| !t.starts_with("test result: ")) {
            if let Some((name, verdict)) = rest.rsplit_once(" ... ") {
                let outcome = match verdict.trim() {
                    "ok" => Outcome::Pass,
                    v if v.starts_with("ignored") => Outcome::Skip,
                    "FAILED" => Outcome::Fail,
                    _ => continue,
                };
                log.rows.push(TestRow { group: binary.clone(), test: name.trim().to_string(), outcome, seconds: None, attempts: 1, message: None });
            }
        } else if let Some(rest) = t.strip_prefix("test result: ") {
            let num = |word: &str| rest.split(';').find_map(|p| p.trim().strip_suffix(word).and_then(|n| n.trim().rsplit(' ').next().and_then(|x| x.parse::<i64>().ok())));
            log.passed += num("passed").unwrap_or(0);
            log.failures += num("failed").unwrap_or(0);
            log.ignored += num("ignored").unwrap_or(0);
            if rest.starts_with("FAILED") {
                log.failed = true;
            }
            if let Some(s) = rest.rsplit("finished in ").next().and_then(|s| s.trim_end_matches('s').parse::<f64>().ok()) {
                log.test_seconds += s;
            }
        } else if t.starts_with("error[") || t.starts_with("error:") {
            log.failed = true;
        }
    }
    log
}

/// `dotnet test`'s summary line and its failed tests: (passed, failed, skipped, seconds, failed names).
#[derive(Debug, Default, PartialEq)]
pub struct DotnetLog {
    pub summary: bool,
    pub passed: i64,
    pub failed: i64,
    pub skipped: i64,
    pub seconds: Option<f64>,
    pub failures: Vec<String>,
}

/// "29 m 28 s", "1 h 2 m", "45 s", "512 ms" or "< 1 ms" as seconds.
pub fn dotnet_duration(text: &str) -> Option<f64> {
    let words: Vec<&str> = text.split_whitespace().filter(|w| *w != "<").collect();
    let mut total = 0.0;
    for pair in words.chunks(2) {
        let n: f64 = pair.first()?.parse().ok()?;
        total += n * match *pair.get(1)? {
            "h" => 3600.0,
            "m" => 60.0,
            "s" => 1.0,
            "ms" => 0.001,
            _ => return None,
        };
    }
    Some(total)
}

pub fn dotnet_log(text: &str) -> DotnetLog {
    let mut log = DotnetLog::default();
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("Failed ").filter(|_| line.starts_with("  ")) {
            if let Some(name) = rest.rsplit_once(" [").map(|(n, _)| n) {
                log.failures.push(name.trim().to_string());
            }
        }
        if (t.starts_with("Passed!") || t.starts_with("Failed!") || t.starts_with("Skipped!")) && t.contains("Total:") {
            let num = |key: &str| t.split(key).nth(1).and_then(|r| r.trim_start().split([',', ' ']).next()).and_then(|n| n.parse::<i64>().ok());
            log.summary = true;
            log.passed += num("Passed:").unwrap_or(0);
            log.failed += num("Failed:").unwrap_or(0);
            log.skipped += num("Skipped:").unwrap_or(0);
            log.seconds = t.split("Duration:").nth(1).and_then(|r| dotnet_duration(r.split(" - ").next().unwrap_or(""))).map(|s| s + log.seconds.unwrap_or(0.0));
        }
    }
    log
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nextest_junit_gives_flaky_and_failed_tests_with_their_attempts() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="nextest-run" tests="3" failures="1" errors="0">
  <testsuite name="emusen-core" tests="3" disabled="0" errors="0" failures="1">
    <testcase name="tests::steady" classname="emusen-core" time="0.010"/>
    <testcase name="tests::racy" classname="emusen-core" time="0.020">
      <flakyFailure type="test failure" message="assertion &amp; more"><![CDATA[thread panicked]]></flakyFailure>
    </testcase>
    <testcase name="tests::broken" classname="emusen-core" time="0.030">
      <failure type="test failure">left != right</failure>
      <rerunFailure type="test failure">left != right</rerunFailure>
      <system-out>noise</system-out>
    </testcase>
  </testsuite>
</testsuites>"#;
        let rows = junit(xml);
        assert_eq!(rows.len(), 3);
        assert_eq!((rows[0].outcome, rows[0].attempts), (Outcome::Pass, 1));
        assert_eq!((rows[1].outcome, rows[1].attempts, rows[1].message.as_deref()), (Outcome::Flaky, 2, Some("assertion & more")));
        assert_eq!((rows[2].outcome, rows[2].attempts), (Outcome::Fail, 2));
        assert_eq!(rows[2].message.as_deref(), Some("left != right"));
    }

    #[test]
    fn a_trx_file_gives_each_test_its_class() {
        let xml = r#"<TestRun><Results>
<UnitTestResult testId="a" testName="W.Fit.Holds(x: 1)" duration="00:00:01.5000000" outcome="Passed" />
<UnitTestResult testId="b" testName="W.Fit.Breaks" duration="00:01:00" outcome="Failed"><Output><ErrorInfo><Message>Assert.Equal() Failure</Message></ErrorInfo></Output></UnitTestResult>
<UnitTestResult testId="c" testName="W.Outer+Inner.Skipped" outcome="NotExecuted" />
</Results><TestDefinitions>
<UnitTest name="W.Fit.Holds" id="a"><TestMethod codeBase="x.dll" className="W.Fit" name="Holds" /></UnitTest>
<UnitTest name="W.Fit.Breaks" id="b"><TestMethod className="W.Fit, W, Version=1.0" name="Breaks" /></UnitTest>
<UnitTest name="W.Outer+Inner.Skipped" id="c"><TestMethod className="W.Outer+Inner" name="Skipped" /></UnitTest>
</TestDefinitions></TestRun>"#;
        let rows = trx(xml);
        assert_eq!(rows.iter().map(|r| (r.group.as_str(), r.outcome)).collect::<Vec<_>>(), vec![("W.Fit", Outcome::Pass), ("W.Fit", Outcome::Fail), ("W.Outer", Outcome::Skip)]);
        assert_eq!(rows[0].seconds, Some(1.5));
        assert_eq!(rows[1].message.as_deref(), Some("Assert.Equal() Failure"));
    }

    #[test]
    fn the_text_logs_give_counts_times_and_failures() {
        let c = cargo_log("   Compiling x v0.1.0\n    Finished `test-fast` profile [optimized] target(s) in 1m 05s\n     Running unittests src/lib.rs (target/test-fast/deps/emusen_x-1a2b)\n\nrunning 2 tests\ntest a::one ... ok\ntest a::two ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s\n");
        assert_eq!((c.profile.as_deref(), c.build_seconds, c.passed, c.failures, c.failed), (Some("test-fast"), Some(65.0), 1, 1, true));
        assert_eq!(c.rows.iter().map(|r| (r.group.as_str(), r.test.as_str(), r.outcome)).collect::<Vec<_>>(), vec![("emusen_x", "a::one", Outcome::Pass), ("emusen_x", "a::two", Outcome::Fail)]);
        let d = dotnet_log("  Failed W.Fit.Breaks [12 ms]\n  Error Message:\n\nFailed!  - Failed:     2, Passed:  3257, Skipped:    88, Total:  3347, Duration: 29 m 28 s - EmuSen.WiseMan.dll (net10.0)\n");
        assert_eq!((d.passed, d.failed, d.skipped, d.seconds), (3257, 2, 88, Some(1768.0)));
        assert_eq!(d.failures, vec!["W.Fit.Breaks".to_string()]);
        assert_eq!(dotnet_duration("< 1 ms"), Some(0.001));
    }
}
