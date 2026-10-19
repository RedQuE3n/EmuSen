//! The rules held to the C# `ScrapeRules` and `ScrapeRedactor`: each fixture was written by the C# on 2026-10-18 from
//! generated inputs (a probe over EmuSen.Mistress, outside the repository), so a difference here is a difference from
//! the C#. See EmuSen_Backend.md §30.7.

use emusen_dotnet::json::{self as dj, Value};
use emusen_scrape::{redact, rules};

fn lines(name: &str) -> Vec<Value> {
    let text = std::fs::read_to_string(format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap();
    text.lines().filter(|l| !l.is_empty()).map(|l| dj::parse(l).unwrap()).collect()
}

fn s(v: &Value, name: &str) -> Option<String> {
    v.get(name).and_then(Value::as_str).map(str::to_string)
}

fn list(v: &Value, name: &str) -> Vec<String> {
    match v.get(name) {
        Some(Value::Array(items)) => items.iter().map(|i| i.as_str().unwrap().to_string()).collect(),
        _ => panic!("{name} is not a list"),
    }
}

#[test]
fn dates_read_as_the_csharp_read_them() {
    let cases = lines("dates.jsonl");
    assert!(cases.len() > 200);
    for c in cases {
        let input = s(&c, "input").unwrap();
        assert_eq!(rules::date(Some(&input)).map(rules::date_text), s(&c, "output"), "{input:?}");
    }
}

#[test]
fn ratings_round_as_the_csharp_rounded_them() {
    let cases = lines("ratings.jsonl");
    assert!(cases.len() > 200);
    for c in cases {
        let note: f64 = match s(&c, "note").unwrap().as_str() {
            "NaN" => f64::NAN,
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            t => t.parse().unwrap(),
        };
        let expected = s(&c, "rating").map(|t| t.parse::<f64>().unwrap());
        let got = rules::rating(Some(note)).map(|f| f as f64);
        assert_eq!(got.map(f64::to_bits), expected.map(f64::to_bits), "note {note}: {got:?} against {expected:?}");
    }
}

#[test]
fn a_file_names_region_order_and_search_text_are_the_csharps() {
    let cases = lines("names.jsonl");
    assert!(cases.len() > 150);
    for c in cases {
        let name = s(&c, "name").unwrap();
        assert_eq!(rules::region_of_file_name(&name).map(str::to_string), s(&c, "region"), "{name:?}");
        assert_eq!(rules::search_name(&name), s(&c, "search").unwrap(), "{name:?}");
        assert_eq!(rules::region_order(&name, "auto", true), list(&c, "order_auto"), "{name:?}");
        assert_eq!(rules::region_order(&name, "auto", false), list(&c, "order_auto_nofallback"), "{name:?}");
        assert_eq!(rules::region_order(&name, "eu", true), list(&c, "order_eu"), "{name:?}");
        assert_eq!(rules::region_order(&name, " ", true), list(&c, "order_blank"), "{name:?}");
    }
}

#[test]
fn redaction_blanks_what_the_csharp_blanked() {
    for secret in ["FAKEDEVID", "FAKE+pass/word", "FAKE%2Bpass%2Fword", "abc"] {
        redact::register(secret);
    }
    let cases = lines("redact.jsonl");
    assert!(cases.len() > 100);
    for c in cases {
        let input = s(&c, "input").unwrap();
        assert_eq!(redact::redact(&input), s(&c, "output").unwrap(), "{input:?}");
    }
}
