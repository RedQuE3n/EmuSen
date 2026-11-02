//! The studio's three data files: `Testing/inputs.json` (what no graph shows), `Testing/tiers.json` (what each tier
//! selects and runs) and `~/.config/EmuSen/testing/runners.json` (the household's machines). Each carries a format
//! number, and a field a reader does not know is ignored. See EmuSen_Testing.md §4.

use emusen_dotnet::json::{self, Value};
use regex::Regex;

pub const INPUTS_FORMAT: u32 = 1;
pub const TIERS_FORMAT: u32 = 1;

/// What a matched line does: narrows a reached project to named units, names a recording's owner, or declares the
/// file nobody's input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Folder,
    Recording,
    None,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub pattern: Regex,
    pub source: String,
    pub kind: LineKind,
    pub select: Vec<String>,
    pub since: String,
    pub reason: String,
}

#[derive(Clone, Debug, Default)]
pub struct Inputs {
    pub lines: Vec<Line>,
    /// unit id -> extra input paths its key hashes.
    pub extra: Vec<(String, Vec<String>)>,
    /// File endings a recording has; one no recording line claims selects the tier.
    pub recording_endings: Vec<String>,
}

fn strings(v: Option<&Value>) -> Vec<String> {
    match v {
        Some(Value::Array(items)) => items.iter().filter_map(|i| i.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    }
}

fn number(v: Option<&Value>) -> Option<f64> {
    match v {
        Some(Value::Number(t)) => t.parse().ok(),
        _ => None,
    }
}

fn format_of(doc: &Value, key: &str, want: u32, file: &str) -> Result<(), String> {
    match number(doc.get(key)) {
        Some(n) if n as u32 == want => Ok(()),
        Some(n) => Err(format!("{file}: format {n} is not {want}, the one this studio reads")),
        None => Err(format!("{file}: no \"{key}\" format number")),
    }
}

impl Inputs {
    pub fn parse(text: &str) -> Result<Inputs, String> {
        let doc = json::parse(text).map_err(|e| format!("Testing/inputs.json: {e:?}"))?;
        format_of(&doc, "inputs", INPUTS_FORMAT, "Testing/inputs.json")?;
        let mut out = Inputs { recording_endings: strings(doc.get("recording_endings")), ..Inputs::default() };
        if let Some(Value::Array(lines)) = doc.get("lines") {
            for (i, l) in lines.iter().enumerate() {
                let source = l.get("path").and_then(Value::as_str).ok_or_else(|| format!("Testing/inputs.json: line {i} has no path"))?.to_string();
                let pattern = Regex::new(&source).map_err(|e| format!("Testing/inputs.json: line {i}: {e}"))?;
                let kind = match l.get("kind").and_then(Value::as_str).unwrap_or("folder") {
                    "folder" => LineKind::Folder,
                    "recording" => LineKind::Recording,
                    "none" => LineKind::None,
                    other => return Err(format!("Testing/inputs.json: line {i}: kind {other} is folder, recording or none")),
                };
                let since = l.get("since").and_then(Value::as_str).unwrap_or("").to_string();
                let reason = l.get("reason").and_then(Value::as_str).unwrap_or("").to_string();
                if since.is_empty() || reason.is_empty() {
                    return Err(format!("Testing/inputs.json: line {i} ({source}) needs a dated reason"));
                }
                out.lines.push(Line { pattern, source, kind, select: strings(l.get("select")), since, reason });
            }
        }
        if let Some(Value::Array(extra)) = doc.get("extra") {
            for e in extra {
                if let Some(unit) = e.get("unit").and_then(Value::as_str) {
                    out.extra.push((unit.to_string(), strings(e.get("paths"))));
                }
            }
        }
        Ok(out)
    }

    pub fn matching(&self, path: &str) -> Vec<&Line> {
        self.lines.iter().filter(|l| l.pattern.is_match(path)).collect()
    }

    pub fn is_recording(&self, path: &str) -> bool {
        self.recording_endings.iter().any(|e| path.ends_with(e.as_str()))
    }
}

#[derive(Clone, Debug, Default)]
pub struct Tier {
    pub name: String,
    pub exclude: Vec<String>,
    pub profile: Option<String>,
    pub reuse: bool,
    pub reuse_days: f64,
    pub depth: Option<usize>,
    pub always: Vec<String>,
    pub named_only: Vec<String>,
}

pub fn parse_tier(text: &str, name: &str) -> Result<Tier, String> {
    let doc = json::parse(text).map_err(|e| format!("Testing/tiers.json: {e:?}"))?;
    format_of(&doc, "tiers", TIERS_FORMAT, "Testing/tiers.json")?;
    let t = doc.get(name).ok_or_else(|| format!("Testing/tiers.json has no tier {name}"))?;
    Ok(Tier {
        name: name.to_string(),
        exclude: strings(t.get("exclude")),
        profile: t.get("profile").and_then(Value::as_str).map(str::to_string),
        reuse: t.get("reuse").and_then(Value::as_bool).unwrap_or(false),
        reuse_days: number(t.get("reuse_days")).unwrap_or(7.0),
        depth: number(t.get("depth")).map(|d| d as usize),
        always: strings(t.get("always")),
        named_only: strings(t.get("named_only")),
    })
}

/// A runner's slots per class; the desktop's two CPU slots and one GPU slot when runners.json is absent.
pub fn runner_classes(text: Option<&str>, runner: &str) -> Vec<(String, usize)> {
    let fallback = vec![("cpu".to_string(), 2), ("gpu".to_string(), 1)];
    let Some(doc) = text.and_then(|t| json::parse(t).ok()) else { return fallback };
    match doc.get(runner).and_then(|r| r.get("classes")) {
        Some(Value::Object(members)) => members.iter().filter_map(|(k, v)| number(Some(v)).map(|n| (k.text.clone(), n as usize))).collect(),
        _ => fallback,
    }
}
