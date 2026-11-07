//! ScreenScraper's answers as the scraper reads them: each line of the fixture is an answer the C# read on 2026-10-18
//! (generated, mutated and broken from the shape of a real one, none of ScreenScraper's data), with what each of its
//! five readers gave. A reading is held to the model the C# gave; a refusal is held to being one, its words being
//! serde's since 2026-11-07, except the reader's own. See EmuSen_Backend.md §30.7 and EmuSen_RustPlatform.md §47.10.

use emusen_scrape::answer::{self, Game, Quota};
use serde_json::Value;

fn quoted(text: &str) -> String {
    serde_json::to_string(text).unwrap()
}

fn text_or_null(t: &Option<String>) -> String {
    t.as_deref().map_or("null".to_string(), quoted)
}

fn num(n: Option<i64>) -> String {
    n.map_or("null".to_string(), |n| n.to_string())
}

fn double(d: Option<f64>) -> String {
    match d {
        None => "null".into(),
        Some(d) if d.is_nan() => "\"NaN\"".into(),
        Some(d) if d.is_infinite() => if d > 0.0 { "\"Infinity\"".into() } else { "\"-Infinity\"".into() },
        Some(d) => quoted(&format!("{d}")),
    }
}

fn pairs(items: &[answer::Localised]) -> String {
    format!("[{}]", items.iter().map(|l| format!("[{},{}]", quoted(&l.key), quoted(&l.text))).collect::<Vec<_>>().join(","))
}

fn quota(q: &Option<Quota>) -> String {
    match q {
        None => "null".into(),
        Some(q) => format!(
            "[{}]",
            [q.max_threads, q.max_download_kbps, q.requests_today, q.requests_ko_today, q.max_requests_per_minute, q.max_requests_per_day, q.max_requests_ko_per_day]
                .iter()
                .map(|v| num(v.map(i64::from)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn game(g: &Option<Game>) -> String {
    let Some(g) = g else { return "null".into() };
    let genres = g.genres.iter().map(|x| format!("{{\"names\":{},\"main\":{}}}", pairs(&x.names), x.main)).collect::<Vec<_>>().join(",");
    let media = g
        .media
        .iter()
        .map(|m| format!("{{\"ty\":{},\"region\":{},\"url\":{},\"format\":{},\"sha1\":{},\"size\":{}}}", quoted(&m.ty), text_or_null(&m.region), quoted(&m.url), text_or_null(&m.format), text_or_null(&m.sha1), num(m.size)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"id\":{},\"rom_id\":{},\"system_id\":{},\"names\":{},\"synopses\":{},\"dates\":{},\"genres\":[{genres}],\"developer\":{},\"publisher\":{},\"players\":{},\"note\":{},\"media\":[{media}]}}",
        g.id,
        num(g.rom_id),
        num(g.system_id.map(i64::from)),
        pairs(&g.names),
        pairs(&g.synopses),
        pairs(&g.dates),
        text_or_null(&g.developer),
        text_or_null(&g.publisher),
        text_or_null(&g.players),
        double(g.note)
    )
}

fn outcome<T>(result: Result<T, String>, ok: impl FnOnce(T) -> String) -> Value {
    match result {
        Ok(v) => serde_json::from_str(&format!("{{\"ok\":{}}}", ok(v))).unwrap(),
        Err(words) if words == "a game without an id" => serde_json::json!({ "words": words }),
        Err(_) => serde_json::json!({ "refused": true }),
    }
}

/// The C#'s outcome as the scraper now gives it: the model, or a refusal whose words are the C#'s only where they were the reader's own.
fn expected(v: &Value) -> Value {
    match v.get("words").and_then(Value::as_str) {
        Some("a game without an id") => serde_json::json!({ "words": "a game without an id" }),
        Some(_) => serde_json::json!({ "refused": true }),
        None => serde_json::json!({ "ok": v["ok"] }),
    }
}

/// Two outcomes alike, numbers equal by value.
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::String(x), Value::String(y)) => x == y || x.parse::<f64>().ok().zip(y.parse::<f64>().ok()).is_some_and(|(x, y)| x == y),
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same(x, y)),
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w))),
        _ => a == b,
    }
}

/// A fixture line with each lone surrogate escape read as U+FFFD, as text holding half a pair reaches the scraper.
fn mended(line: &str) -> String {
    let unit = |s: &str| u16::from_str_radix(s, 16).ok();
    let (b, mut out, mut i) = (line.as_bytes(), String::new(), 0);
    while i < line.len() {
        let high = line.get(i..i + 2) == Some("\\u") && (i == 0 || b[i - 1] != b'\\');
        let code = if high { line.get(i + 2..i + 6).and_then(unit) } else { None };
        match code {
            Some(c @ 0xD800..=0xDBFF) if line.get(i + 6..i + 8) == Some("\\u") && line.get(i + 8..i + 12).and_then(unit).is_some_and(|d| (0xDC00..=0xDFFF).contains(&d)) => {
                let _ = c;
                out.push_str(&line[i..i + 12]);
                i += 12;
            }
            Some(0xD800..=0xDFFF) => {
                out.push_str("\\uFFFD");
                i += 6;
            }
            _ => {
                let ch = line[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    out
}

#[test]
fn every_answer_reads_as_the_csharp_read_it() {
    let text = std::fs::read_to_string(format!("{}/tests/fixtures/answers.jsonl", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let mut count = 0;
    for line in text.lines().filter(|l| !l.is_empty()) {
        let case: Value = serde_json::from_str(&mended(line)).unwrap();
        let input = case.get("input").and_then(Value::as_str).unwrap().to_string();
        let got = [
            ("jeu_infos", outcome(answer::jeu_infos(&input), |(g, q)| format!("{{\"game\":{},\"quota\":{}}}", game(&g), quota(&q)))),
            ("jeu_recherche", outcome(answer::jeu_recherche(&input), |(gs, q)| format!("{{\"games\":[{}],\"quota\":{}}}", gs.into_iter().map(|g| game(&Some(g))).collect::<Vec<_>>().join(","), quota(&q)))),
            ("user", outcome(answer::user(&input), |q| quota(&q))),
            ("member", outcome(answer::member(&input), |m| m.map_or("null".into(), |m| format!("{{\"name\":{},\"level\":{},\"quota\":{}}}", quoted(&m.name), text_or_null(&m.level), quota(&Some(m.quota)))))),
            ("systems", outcome(answer::systems(&input), |mut s| {
                s.sort_by_key(|(id, _)| *id);
                format!("[{}]", s.iter().map(|(id, n)| format!("[{id},{}]", quoted(n))).collect::<Vec<_>>().join(","))
            })),
        ];
        for (reader, value) in got {
            assert!(same(&value, &expected(case.get(reader).unwrap())), "{reader} of {input}: {value} and the C#'s {}", case.get(reader).unwrap());
        }
        count += 1;
    }
    assert!(count > 150, "{count}");
}
