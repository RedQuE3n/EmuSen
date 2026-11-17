// A recorded account of the default state cut into the parts theme-audit's tests run side by side, and joined back (man theme-check, PARTS).

use emusen_qt_test::harness::SIZES;

/// How theme-audit begins a word of its own: what it held, and the lines a ruling lets pass; a finding has none.
const OWN: &str = "theme-audit: ";

/// The label the default state's Choose a console over 120 consoles carries after its variant, as theme-audit writes it.
const CROWD: &str = ", 120 consoles";

/// One part of a run: one window size without the crowd (`--sizes WxH --crowd without`), or the crowd at every size (`--crowd only`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Size(i32, i32),
    Crowd,
}

/// Every part, the sizes in the gate's order and the crowd last.
pub fn parts() -> Vec<Part> {
    SIZES.iter().map(|&(w, h)| Part::Size(w, h)).chain([Part::Crowd]).collect()
}

/// Where a line was judged: its screen, its size, its variant and whether over the crowd; None for a line that names no size.
fn place(line: &str) -> Option<(&str, (i32, i32), &str, bool)> {
    let at = line.find(".qml")?;
    let screen = line[..at].rsplit('/').next()?;
    let rest = &line[at + 4..];
    // A finding's `:1 kind ` stands between the file and the size; a ruling's line has none.
    let rest = match rest.strip_prefix(":1 ") {
        Some(kinded) => kinded.split_once(' ')?.1,
        None => rest.strip_prefix(' ')?,
    };
    let head = &rest[..rest.find(':')?];
    let (size, label) = head.split_once(' ')?;
    let (w, h) = size.split_once('x')?;
    let variant = label.split(',').next()?;
    Some((screen, (w.parse().ok()?, h.parse().ok()?), variant, label.ends_with(CROWD)))
}

/// A held line's screen, states and states against its mockup.
fn held(line: &str) -> Option<(&str, usize, usize)> {
    let rest = line.strip_prefix("theme-audit: held ")?;
    let (screen, rest) = rest.split_once(" in ")?;
    let (states, rest) = rest.split_once(" states, ")?;
    Some((screen, states.parse().ok()?, rest.strip_suffix(" against its mockup")?.parse().ok()?))
}

fn held_line(screen: &str, states: usize, mocked: usize) -> String {
    format!("theme-audit: held {screen} in {states} states, {mocked} against its mockup\n")
}

/// The share of a recorded account one part judges, in the account's order, held counts cut as the passes are (man theme-check, PARTS).
pub fn part(recorded: &str, part: Part, canvas: (i32, i32)) -> String {
    let shaped = |(w, h): (i32, i32)| w * canvas.1 == h * canvas.0;
    let at_shape = SIZES.iter().filter(|s| shaped(**s)).count();
    let mut out = String::new();
    for line in recorded.lines() {
        if let Some((screen, states, mocked)) = held(line) {
            let crowd = if screen == "Consoles" { SIZES.len() } else { 0 };
            match part {
                Part::Crowd if crowd > 0 => out.push_str(&held_line(screen, crowd, 0)),
                Part::Crowd => {}
                Part::Size(w, h) => {
                    assert_eq!((states - crowd) % SIZES.len(), 0, "{line}: states judged at every size");
                    assert_eq!(mocked % at_shape, 0, "{line}: against its mockup at each size of the canvas's shape");
                    out.push_str(&held_line(screen, (states - crowd) / SIZES.len(), if shaped((w, h)) { mocked / at_shape } else { 0 }));
                }
            }
            continue;
        }
        let (_, size, _, crowd) = place(line).unwrap_or_else(|| panic!("a line no part judges: {line}"));
        let mine = match part {
            Part::Crowd => crowd,
            Part::Size(w, h) => !crowd && size == (w, h),
        };
        if mine {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// The parts' accounts joined in the order one whole run writes them (man theme-check, PARTS).
pub fn join(parts: &[(Part, String)], variants: &[&str]) -> String {
    let mut screens: Vec<(String, usize, usize)> = Vec::new();
    for (_, account) in parts {
        for (screen, states, mocked) in account.lines().filter_map(held) {
            match screens.iter_mut().find(|(s, _, _)| s == screen) {
                Some((_, n, m)) => (*n, *m) = (*n + states, *m + mocked),
                None => screens.push((screen.to_owned(), states, mocked)),
            }
        }
    }
    let group = |ruled: bool| {
        let mut out = String::new();
        let lines = |part: Part| {
            let account = parts.iter().find(|(p, _)| *p == part).map(|(_, a)| a.as_str()).unwrap_or("");
            account.lines().filter(move |l| l.starts_with(OWN) == ruled && held(l).is_none()).map(|l| (l, place(l).unwrap_or_else(|| panic!("a line no part judges: {l}"))))
        };
        for (screen, _, _) in &screens {
            for variant in variants {
                for &(w, h) in &SIZES {
                    lines(Part::Size(w, h)).filter(|(_, p)| p.0 == screen && p.2 == *variant).for_each(|(l, _)| out.push_str(&format!("{l}\n")));
                }
            }
            lines(Part::Crowd).filter(|(_, p)| p.0 == screen).for_each(|(l, _)| out.push_str(&format!("{l}\n")));
        }
        out
    };
    let held: String = screens.iter().map(|(s, n, m)| held_line(s, *n, *m)).collect();
    format!("{}{held}{}", group(false), group(true))
}
