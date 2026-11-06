//! Gate G5's goldens: twelve commercial games from the library, three anchors each, VenusRT's picture against Mesen's
//! run as a black box (VenusRT_Native.md §41.6); moved from WiseMan's VenusRtGoldenTests on 2026-11-06.

mod common;

use std::path::{Path, PathBuf};

use common::differential::{self as diff, MESEN_ROW_OFFSET, Press};
use common::grader::Picture;
use common::programs;
use emusen_goldens::md5::md5_hex;

const VARIABLE: &str = "EMUSEN_VENUSRT_GOLDENS";
const WINDOW: u32 = 30;

/// One anchor: a frame, the scene Mesen shows there, and what VenusRT's picture was recorded as: "equal" or a cause, then its hash.
struct Anchor(u32, &'static str, &'static str);

struct Golden {
    file: &'static str,
    md5: &'static str,
    why: &'static str,
    presses: Vec<Press>,
    anchors: Vec<Anchor>,
}

fn p(frame: u32, button: usize) -> Press {
    Press { frame, button, frames: 5 }
}

const DOWN: usize = 5;
const START: usize = 3;
const A: usize = 8;

/// Chosen from Mesen's pictures alone before VenusRT was run on them; ROMs are found by MD5, never committed.
fn goldens() -> Vec<Golden> {
    let g = |file, md5, why, presses: Vec<Press>, anchors: Vec<Anchor>| Golden { file, md5, why, presses, anchors };
    vec![
        g("Super Mario RPG - Legend of the Seven Stars (U) [!].smc", "d0b68d68d9efc0558242f5476d1c5b81", "SA-1", vec![], vec![Anchor(300, "the Square logo", "equal 2c3d80054787292a"), Anchor(1200, "the garden at night, attract", "drift, equal 41 frames on (D-6, D-37) 13e331854643b792"), Anchor(2400, "Mario on the map, attract", "drift, equal 42 frames on (D-6, D-37) a5fcdffe4cafff7e")]),
        g("Super Mario World 2 - Yoshi's Island (U) (V1.1).smc", "2150266ad9ea89aed241e949966514ae", "GSU-2", vec![], vec![Anchor(600, "\"A long, long time ago\" under the Nintendo board", "equal a21eacbcf0811b17"), Anchor(1800, "the stork over the night sky", "the stork alone, not logged ec703527e8bde99c"), Anchor(3000, "\"SCRREEEECH!!!\"", "equal 8a43e8202ff19c7c")]),
        g("Super Mario Kart (U) [!].smc", "8b13d7d413545582099ab35298405417", "DSP-1B", vec![], vec![Anchor(600, "the title with karts passing", "equal d7ce4fb7029b6aa0"), Anchor(2400, "the attract race, start", "equal c4af9d906ebf92f6"), Anchor(3300, "the attract race, later", "equal c243a180725238c2")]),
        g("F1 ROC II - Race of Champions (U).smc", "cbf9f26076044edc913aabc22adbd690", "ST010", vec![], vec![Anchor(600, "the title menu", "equal 66ea1481506030a9"), Anchor(1500, "the first attract race", "equal f6b026b5ef303d92"), Anchor(3300, "the second attract race", "equal 9f1ff8d28c746c58")]),
        g("Metal Combat - Falcon's Revenge (U) [!].smc", "7f159d48965a2802d9fd76c6acaed5af", "OBC1", vec![], vec![Anchor(600, "\"Nintendo presents\" over the intro", "equal c204d5fde1e0d310"), Anchor(1500, "the title", "equal 3db27678fd849af4"), Anchor(3000, "the attract fight", "equal 5b32b8bf5dc7db29")]),
        g("Captain America and The Avengers (U).smc", "b96b8908f88e999be63239444e4997df", "tricky list, open bus", vec![], vec![Anchor(600, "the copyright page", "equal d9352985ec7ef486"), Anchor(1500, "the title", "equal efaf9b4ebd1782a2"), Anchor(2100, "Captain America's card", "equal 30185da569fd6392")]),
        g("ActRaiser (U) [!].smc", "2c740f599af49e4db5864817ce57cbdf", "tricky list, BRK/COP", vec![p(1000, DOWN), p(1100, START), p(1300, START), p(1500, A), p(1700, START)], vec![Anchor(600, "the sword in the intro", "D-7 58294b58aba81507"), Anchor(900, "the title menu", "equal f180d4a48a9a8e53"), Anchor(1400, "\"Please create a name\" in the sky palace", "equal c9e375eca963092a")]),
        g("SMW.smc", "dbe1f3c8f3a0b2db52b7d59417891117", "tricky list, ORA [d]", vec![p(600, START), p(700, A), p(800, A), p(1300, A), p(1500, A), p(1700, A)], vec![Anchor(150, "\"Nintendo Presents\"", "equal 7a654f58801248e8"), Anchor(1200, "the welcome message at Yoshi's house", "equal 759afbb3411436bd"), Anchor(2400, "the overworld map", "equal eb6cc5041c418bdb")]),
        g("Hook (U) (2648).smc", "291e0beb91390a96cd1effd619157275", "tricky list, VRAM writes during display", vec![], vec![Anchor(300, "the Sony Imagesoft logo", "equal 9d9e4fac812bc8ec"), Anchor(600, "the copyright page", "equal 2eecc820a49d9233"), Anchor(1500, "the intro's dialogue", "equal ce6979de75867d63")]),
        g("Breath of Fire (U) [!].smc", "cdef890e99a0fb62c4d449d867c4244a", "tricky list, VRAM reads", vec![p(1900, START), p(2100, START), p(2300, A), p(2500, A)], vec![Anchor(700, "the copyright page", "equal 4e1f799880fbdabd"), Anchor(1800, "the title", "equal 2e13fe99b52a9989"), Anchor(2700, "the name entry", "equal 571411f365f99535")]),
        g("Axelay (U).smc", "d6ddca61a7f31eed247c429b0e528f3c", "tricky list, offset-per-tile", vec![], vec![Anchor(600, "the Konami logo", "equal 2d92193a131065e0"), Anchor(1500, "the city in the intro", "D-7 ddc596bb8602f751"), Anchor(3000, "the planet in the intro", "D-7 ac943fa62256caff")]),
        g("NHL '94 (U) [!].smc", "960f6297b96ee768b97bd795bb1136f7", "tricky list, mode 7 scroll latch (the intro)", vec![], vec![Anchor(600, "the EA Sports logo", "equal c415a598c615d12e"), Anchor(1500, "the title and licence", "equal cf5277f7f77fb355"), Anchor(2200, "the credits with a spinning puck", "the puck alone, not logged 4095c4a9591bb755")]),
    ]
}

fn md5_of(path: &Path) -> String {
    std::fs::read(path).map(|b| md5_hex(&b)).unwrap_or_default()
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("sfc") || x.eq_ignore_ascii_case("smc")) {
            out.push(p);
        }
    }
}

/// A golden's ROM by name under SNES/, else by MD5 among the library's files.
fn find(library: &Path, g: &Golden) -> Option<PathBuf> {
    let named = library.join("SNES").join(g.file);
    if named.exists() && md5_of(&named) == g.md5 {
        return Some(named);
    }
    let mut files = Vec::new();
    walk(library, &mut files);
    files.into_iter().find(|f| md5_of(f) == g.md5)
}

/// A golden left out of a run, and whether that fails it: a game the library lacks passes, one whose firmware is missing fails (VenusRT_Native.md §70.4).
struct Skipped {
    file: &'static str,
    why: String,
    fails: bool,
}

fn plan<'a>(library: Option<&Path>, goldens: &'a [Golden], firmware: &[PathBuf]) -> (Vec<(&'a Golden, PathBuf)>, Vec<Skipped>) {
    let (mut runs, mut skips) = (Vec::new(), Vec::new());
    for g in goldens {
        match library.filter(|l| l.is_dir()) {
            None => skips.push(Skipped { file: g.file, why: format!("{VARIABLE} unset or naming no folder"), fails: false }),
            Some(l) => match find(l, g) {
                None => skips.push(Skipped { file: g.file, why: "not in the library".into(), fails: false }),
                Some(found) => {
                    let image = std::fs::read(&found).unwrap();
                    match diff::dsp_request(&image) {
                        Some((name, _)) if diff::dsp_firmware(&image, firmware).is_none() => skips.push(Skipped { file: g.file, why: format!("in the library, but no {name} among the player's images"), fails: true }),
                        _ => runs.push((g, found)),
                    }
                }
            },
        }
    }
    (runs, skips)
}

/// FNV-1a over a picture's size and its 15-bit words.
fn hash(p: &Picture) -> String {
    let mut h: u64 = 14695981039346656037;
    let mut add = |v: u32| h = (h ^ v as u64).wrapping_mul(1099511628211);
    add(p.width as u32);
    add(p.height as u32);
    for &px in &p.pixels {
        add((px & 0x7FFF) as u32);
    }
    format!("{h:016x}")
}

#[test]
fn the_goldens_cover_every_coprocessor_with_three_anchors_each() {
    let gs = goldens();
    assert_eq!(12, gs.len());
    assert!(gs.iter().all(|g| g.anchors.len() == 3));
    for chip in ["SA-1", "GSU", "DSP", "ST01", "OBC1"] {
        assert!(gs.iter().any(|g| g.why.contains(chip)), "{chip}");
    }
    let mut md5s: Vec<&str> = gs.iter().map(|g| g.md5).collect();
    md5s.sort();
    md5s.dedup();
    assert_eq!(gs.len(), md5s.len());
}

#[test]
fn a_game_in_the_library_without_its_firmware_fails_the_run_and_an_absent_one_is_listed() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("skips-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let (library, firmware) = (root.join("library"), root.join("firmware"));
    std::fs::create_dir_all(library.join("SNES")).unwrap();
    std::fs::create_dir_all(&firmware).unwrap();
    let rom = library.join("SNES").join("dsp-game.sfc");
    let image = programs::nec_dsp_board("PILOTWINGS");
    std::fs::write(&rom, &image).unwrap();
    let (name, size) = diff::dsp_request(&image).expect("a DSP request");
    let md5 = Box::leak(md5_hex(&image).into_boxed_str());
    let present = Golden { file: "dsp-game.sfc", md5, why: "DSP-1", presses: vec![], anchors: vec![Anchor(10, "a frame", "")] };
    let absent = Golden { file: "absent.sfc", md5: "00000000000000000000000000000000", why: "plain", presses: vec![], anchors: vec![Anchor(10, "a frame", "")] };
    let both = [present, absent];
    let (runs, skips) = plan(Some(&library), &both, &[firmware.clone()]);
    assert!(runs.is_empty());
    assert_eq!(vec![("dsp-game.sfc", true), ("absent.sfc", false)], skips.iter().map(|s| (s.file, s.fails)).collect::<Vec<_>>());
    assert!(skips[0].why.contains(&name));
    std::fs::write(firmware.join(&name), vec![0u8; size as usize]).unwrap();
    let (runs, skips) = plan(Some(&library), &both, &[firmware.clone()]);
    assert_eq!(vec![rom.clone()], runs.iter().map(|r| r.1.clone()).collect::<Vec<_>>());
    assert_eq!(vec![("absent.sfc", false)], skips.iter().map(|s| (s.file, s.fails)).collect::<Vec<_>>());
    let all = goldens();
    let (runs, skips) = plan(None, &all, &[firmware]);
    assert!(runs.is_empty());
    assert_eq!(all.iter().map(|g| g.file).collect::<Vec<_>>(), skips.iter().map(|s| s.file).collect::<Vec<_>>());
    assert!(skips.iter().all(|s| !s.fails));
}

/// Opt-in: each anchor's VenusRT picture equal to one of Mesen's within the window, or the cause recorded, and its hash as recorded; every skipped game listed.
#[test]
fn the_goldens_against_mesen() {
    let library = std::env::var_os(VARIABLE).map(PathBuf::from);
    let all = goldens();
    let (runs, skips) = plan(library.as_deref(), &all, &diff::firmware_folders());
    // A scratch copy outside the tree, as a commercial image is never written into the repository.
    let scratch = std::env::temp_dir().join(format!("venusrt-goldens-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let (mut failures, mut table) = (Vec::new(), vec!["game\tframe\tscene\tresult\tlag\tdiffering\tvenusrt_hash\trecorded".to_string()]);
    for s in &skips {
        table.push(format!("{}\t\t{}\tnot run\t\t\t\t", s.file, s.why));
        if s.fails {
            failures.push(format!("{}: {}, so its anchors were not checked", s.file, s.why));
        }
    }
    for (g, found) in runs {
        let rom = scratch.join(format!("{}.{}", g.md5, found.extension().unwrap().to_string_lossy()));
        std::fs::copy(&found, &rom).unwrap();
        let frames: Vec<u32> = g.anchors.iter().map(|a| a.0).collect();
        let ours = diff::run_default(&rom, &frames, &g.presses, false);
        for a in &g.anchors {
            let mine = ours.at(a.0).unwrap().picture.clone().unwrap();
            let h = hash(&mine);
            let (mut result, mut lag, mut differing) = ("no probe".to_string(), String::new(), String::new());
            if diff::mesen_probe().is_some() {
                let cache = scratch.join("mesen");
                let window: Vec<u32> = (a.0 - WINDOW..=a.0 + WINDOW).collect();
                let theirs = diff::run_mesen(&cache, &rom, &window, &g.presses, false);
                assert!(theirs.at(a.0).and_then(|s| s.picture.as_ref()).is_some_and(|p| p.pixels.iter().any(|&x| x & 0x7FFF != 0)), "{} {}: Mesen's anchor picture is blank", g.file, a.0);
                let best = theirs.snapshots.iter().map(|s| (s.frame, diff::pictures(Some(&mine), s.picture.as_ref(), MESEN_ROW_OFFSET).1)).min_by_key(|&(f, d)| (d, f.abs_diff(a.0))).unwrap();
                result = if best.1 == 0 { "equal" } else { "differs" }.into();
                lag = (best.0 as i64 - a.0 as i64).to_string();
                differing = best.1.to_string();
                let _ = std::fs::remove_dir_all(&cache);
                if !a.2.is_empty() && (result == "equal") != a.2.starts_with("equal ") {
                    failures.push(format!("{} {}: {result}, recorded {}", g.file, a.0, a.2));
                }
            }
            if !a.2.is_empty() && !a.2.ends_with(&format!(" {h}")) {
                failures.push(format!("{} {}: hash {h}, recorded {}", g.file, a.0, a.2));
            }
            table.push(format!("{}\t{}\t{}\t{result}\t{lag}\t{differing}\t{h}\t{}", g.file, a.0, a.1, a.2));
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    for line in &table {
        println!("{line}");
    }
    if let Some(report) = std::env::var_os("EMUSEN_VENUSRT_REPORT") {
        std::fs::write(report, table.join("\n") + "\n").unwrap();
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
