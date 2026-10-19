//! ScreenScraper's answers read as the C# `ScreenScraperJson` read them: each line of the fixture is an answer the C#
//! read on 2026-10-18 (generated, mutated and broken from the shape of a real one, none of ScreenScraper's data), with
//! what each of its five readers gave or the words of what it threw. See EmuSen_Backend.md §30.7.

use emusen_dotnet::json::{self as dj, Value};
use emusen_scrape::answer::{self, Game, Quota};

fn text_or_null(t: &Option<String>) -> String {
    t.as_deref().map_or("null".to_string(), dj::quoted)
}

fn num(n: Option<i64>) -> String {
    n.map_or("null".to_string(), |n| n.to_string())
}

fn double(d: Option<f64>) -> String {
    match d {
        None => "null".into(),
        Some(d) if d.is_nan() => "\"NaN\"".into(),
        Some(d) if d.is_infinite() => if d > 0.0 { "\"Infinity\"".into() } else { "\"-Infinity\"".into() },
        Some(d) => dj::quoted(&dj::format_f64(d).unwrap()),
    }
}

fn pairs(items: &[answer::Localised]) -> String {
    format!("[{}]", items.iter().map(|l| format!("[{},{}]", dj::quoted(&l.key), dj::quoted(&l.text))).collect::<Vec<_>>().join(","))
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
        .map(|m| format!("{{\"ty\":{},\"region\":{},\"url\":{},\"format\":{},\"sha1\":{},\"size\":{}}}", dj::quoted(&m.ty), text_or_null(&m.region), dj::quoted(&m.url), text_or_null(&m.format), text_or_null(&m.sha1), num(m.size)))
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
    let text = match result {
        Ok(v) => format!("{{\"ok\":{}}}", ok(v)),
        Err(words) => format!("{{\"words\":{}}}", dj::quoted(&words)),
    };
    dj::parse(&text).unwrap()
}

/// The C#'s outcome with the exception's type left out, which the scraper does not keep.
fn expected(v: &Value) -> Value {
    match v {
        Value::Object(members) => Value::Object(members.iter().filter(|(k, _)| k.text != "error").cloned().collect()),
        other => other.clone(),
    }
}

#[test]
fn every_answer_reads_as_the_csharp_read_it() {
    let text = std::fs::read_to_string(format!("{}/tests/fixtures/answers.jsonl", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let mut count = 0;
    for line in text.lines().filter(|l| !l.is_empty()) {
        let case = dj::parse(line).unwrap();
        let input = case.get("input").and_then(Value::as_str).unwrap().to_string();
        let got = [
            ("jeu_infos", outcome(answer::jeu_infos(&input), |(g, q)| format!("{{\"game\":{},\"quota\":{}}}", game(&g), quota(&q)))),
            ("jeu_recherche", outcome(answer::jeu_recherche(&input), |(gs, q)| format!("{{\"games\":[{}],\"quota\":{}}}", gs.into_iter().map(|g| game(&Some(g))).collect::<Vec<_>>().join(","), quota(&q)))),
            ("user", outcome(answer::user(&input), |q| quota(&q))),
            ("member", outcome(answer::member(&input), |m| m.map_or("null".into(), |m| format!("{{\"name\":{},\"level\":{},\"quota\":{}}}", dj::quoted(&m.name), text_or_null(&m.level), quota(&Some(m.quota)))))),
            ("systems", outcome(answer::systems(&input), |mut s| {
                s.sort_by_key(|(id, _)| *id);
                format!("[{}]", s.iter().map(|(id, n)| format!("[{id},{}]", dj::quoted(n))).collect::<Vec<_>>().join(","))
            })),
        ];
        for (reader, value) in got {
            assert_eq!(value, expected(case.get(reader).unwrap()), "{reader} of {input}");
        }
        count += 1;
    }
    assert!(count > 150, "{count}");
}
