//! The SNES test-ROM grader, VenusRT's bare machine on the core ABI and its differential against Mesen run as a black
//! box through the reference probe, and the corpus baseline they record (VenusRT_Native.md §3, §68.3); moved from
//! WiseMan's VenusRtTestRomRunnerTests on 2026-11-06.

mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use common::differential::{self as diff, MESEN_ROW_OFFSET, Press};
use common::grader::{self, Outcome, Picture, Snapshot, SnesRun, Verdict};
use common::golden::library;
use common::programs;

/// Where the corpus run's table goes when set.
const REPORT_VARIABLE: &str = "EMUSEN_VENUSRT_REPORT";

fn run_with(vram: Vec<u8>, cgram: Option<Vec<u8>>, earlier: Option<Vec<u8>>) -> Vec<Snapshot> {
    let mut shots = Vec::new();
    if let Some(e) = earlier {
        shots.push(Snapshot { frame: 1800, spaces: BTreeMap::from([("vram".to_string(), e)]), picture: None });
    }
    shots.push(Snapshot { frame: 3600, spaces: BTreeMap::from([("vram".to_string(), vram), ("cgram".to_string(), cgram.unwrap_or_else(|| vec![0; 512]))]), picture: None });
    shots
}

fn vram(texts: &[(usize, &str)]) -> Vec<u8> {
    let mut v = vec![0u8; 0x10000];
    for (word, text) in texts {
        for (i, b) in text.bytes().enumerate() {
            v[(word + i) * 2] = b;
        }
    }
    v
}

fn grade(path: &str, shots: &[Snapshot]) -> Verdict {
    grader::grade(Path::new(path), shots)
}

#[test]
fn each_suites_protocol_reads_its_pass_its_failure_and_its_silence() {
    let gilyon = "/corpus/roms/gilyon-snes-tests-v1.4/cputest/cputest-full.sfc";
    let v = grade(gilyon, &run_with(vram(&[(0x32, "Success"), (0x6E, "0649")]), None, None));
    assert_eq!((v.outcome, v.protocol.as_str(), v.detail.as_str()), (Outcome::Passed, "gilyon", "last=0649"));
    let v = grade(gilyon, &run_with(vram(&[(0x32, "Failed"), (0x6E, "0024")]), None, None));
    assert_eq!((v.outcome, v.protocol.as_str(), v.detail.as_str()), (Outcome::Failed, "gilyon", "test=0024"));
    assert_eq!(Outcome::Incomplete, grade(gilyon, &run_with(vram(&[]), None, None)).outcome);

    let blargg = "/corpus/src/higan-snes-test-roms/blargg-spc-6/spc_smp.sfc";
    assert_eq!(Outcome::Passed, grade(blargg, &run_with(vram(&[]), Some(vec![0x00, 0x7C]), None)).outcome);
    assert_eq!(Outcome::Failed, grade(blargg, &run_with(vram(&[]), Some(vec![0x1F, 0x00]), None)).outcome);
    assert_eq!(Outcome::Incomplete, grade(blargg, &run_with(vram(&[]), None, None)).outcome);

    let text = "/corpus/src/higan-snes-test-roms/jonasquinn-test-roms/test_x.sfc";
    assert_eq!(Outcome::Passed, grade(text, &run_with(vram(&[(0x40, "Timer test"), (0x60, "Passed")]), None, None)).outcome);
    assert_eq!(Outcome::Failed, grade(text, &run_with(vram(&[(0x40, "Passed 1"), (0x60, "Failed 2")]), None, None)).outcome);
    assert_eq!(Outcome::Done, grade(text, &run_with(vram(&[(0x40, "A=1234 Done")]), None, None)).outcome);
    assert_eq!(Outcome::Visual, grade(text, &run_with(vram(&[(0x40, "Press start")]), None, None)).outcome);
    assert_eq!(Outcome::NoDump, grade(text, &[]).outcome);
}

/// A 512x478 reference frame holds each line twice; a 512-wide picture is compared with it column for column, a 256-wide one pixel for pair.
#[test]
fn a_hi_res_reference_frame_is_compared_line_for_doubled_line() {
    let pic = |w: usize, h: usize, f: &dyn Fn(usize, usize) -> i64| Picture { width: w, height: h, pixels: (0..w * h).map(|i| (f(i % w, i / w) & 0x7FFF) as u16).collect() };
    let wide = pic(512, 478, &|x, y| (y as i64 / 2 - 7) * 600 + x as i64);
    let mut ours = pic(512, 224, &|x, y| y as i64 * 600 + x as i64);
    assert_eq!((512 * 224, 0), diff::pictures(Some(&ours), Some(&wide), MESEN_ROW_OFFSET));
    ours.pixels[5 * 512 + 9] ^= 1;
    assert_eq!((512 * 224, 1), diff::pictures(Some(&ours), Some(&wide), MESEN_ROW_OFFSET));
    let narrow = pic(256, 224, &|x, y| y as i64 * 600 + 2 * x as i64);
    assert_eq!((512 * 224, 256 * 224), diff::pictures(Some(&narrow), Some(&wide), MESEN_ROW_OFFSET));
    let mut tall = pic(512, 448, &|x, y| (y as i64 / 2) * 600 + x as i64);
    assert_eq!((512 * 448, 0), diff::pictures(Some(&tall), Some(&wide), MESEN_ROW_OFFSET));
    tall.pixels[3 * 512] ^= 1;
    assert_eq!((512 * 448, 1), diff::pictures(Some(&tall), Some(&wide), MESEN_ROW_OFFSET));
}

fn scratch(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// byuu's ROMs pass on a blue backdrop and fail on a red one, where their folder's source writes both colours.
#[test]
fn a_backdrop_verdict_counts_only_where_the_source_writes_it() {
    let root = scratch("backdrop");
    let dir = root.join("jonasquinn-test-roms").join("t");
    std::fs::create_dir_all(&dir).unwrap();
    let rom = dir.join("test_nmi.smc");
    let other = root.join("jonasquinn-test-roms").join("u").join("x.smc");
    std::fs::write(dir.join("test_nmi.asm"), "pass() {\n  lda #$00 : sta $2122\n  lda #$7c : sta $2122\n}\nfail() {\n  lda #$1f : sta $2122\n}\n").unwrap();
    let g = |p: &Path, cg: [u8; 2]| grader::grade(p, &run_with(vram(&[]), Some(cg.to_vec()), None)).outcome;
    assert_eq!(Outcome::Passed, g(&rom, [0x00, 0x7C]));
    assert_eq!(Outcome::Failed, g(&rom, [0x1F, 0x00]));
    assert_eq!(Outcome::Visual, g(&rom, [0x10, 0x42]));
    assert_eq!(Outcome::Visual, g(&other, [0x00, 0x7C]));
}

/// PeterLemon's tests pass by showing PASS on a screen that has stopped changing, and fail by showing FAIL at all.
#[test]
fn a_peterlemon_test_passes_only_on_a_steady_screen() {
    let dir = scratch("peterlemon");
    let rom = dir.join("CPUADC.sfc");
    std::fs::write(dir.join("CPUADC.asm"), "PrintText(Pass, 8, 32)\n").unwrap();
    let (pass, fail) = (vram(&[(0x100, "PASS")]), vram(&[(0x100, "PASS"), (0x120, "FAIL")]));
    assert_eq!(Outcome::Passed, grader::grade(&rom, &run_with(pass.clone(), None, Some(pass.clone()))).outcome);
    assert_eq!(Outcome::Incomplete, grader::grade(&rom, &run_with(pass.clone(), None, Some(vram(&[])))).outcome);
    assert_eq!(Outcome::Failed, grader::grade(&rom, &run_with(fail.clone(), None, Some(fail))).outcome);
}

fn one(frame: u32, vram_bytes: Vec<u8>, picture: Picture) -> SnesRun {
    SnesRun { engine: "x".into(), snapshots: vec![Snapshot { frame, spaces: BTreeMap::from([("vram".to_string(), vram_bytes)]), picture: Some(picture) }], audio_rate: 0, audio: Vec::new() }
}

#[test]
fn the_differential_counts_every_differing_byte_and_pixel_and_nothing_else() {
    let mut px = vec![0u16; 256 * 225];
    px[256 * 3 + 7] = 0x001F;
    let mut shifted = vec![0u16; 256 * 224];
    shifted[256 * 2 + 7] = 0x801F;
    let a = one(1, vec![1, 2, 3], Picture { width: 256, height: 224, pixels: shifted });
    let b = one(1, vec![1, 9, 3], Picture { width: 256, height: 225, pixels: px });
    let d = &diff::compare(&a, &b, 1)[0];
    assert_eq!(1, d.space_bytes["vram"]);
    assert_eq!((256 * 224, 0), (d.pixels_compared, d.pixels_differing));
    assert_eq!(2, diff::compare(&a, &b, 0)[0].pixels_differing);
    assert!(diff::compare(&a, &a, 0)[0].identical());

    let tone: Vec<i16> = (0..32000 * 2).map(|i: i32| if (i / 2) % 3200 < 1600 && (i / 2) / 3200 % 2 == 0 { 8000 } else { 0 }).collect();
    let mut later = vec![0i16; 533 * 2 * 5];
    later.extend_from_slice(&tone);
    later.truncate(tone.len());
    let sound = |audio: Vec<i16>| SnesRun { engine: "a".into(), snapshots: Vec::new(), audio_rate: 32000, audio };
    let s = diff::audio(&sound(tone.clone()), &sound(later), 30);
    assert!(s.correlation > 0.95, "{s:?}");
    assert!((4..=6).contains(&s.lag_windows), "{s:?}");
    assert!(diff::audio(&sound(vec![0; 64000]), &sound(tone), 30).first_silent);

    let chirp = |rate: i32, seconds: i32, amp: f64, delay: i32| -> Vec<i16> {
        (0..rate * seconds)
            .flat_map(|i| {
                let t = (i - delay) as f64 / rate as f64;
                let v = if i < delay { 0 } else { (amp * (2.0 * std::f64::consts::PI * (100.0 * t + 90.0 * t * t)).sin()) as i16 };
                [v, v]
            })
            .collect()
    };
    let at32 = SnesRun { engine: "a".into(), snapshots: Vec::new(), audio_rate: 32000, audio: chirp(32000, 4, 8000.0, 0) };
    let at48 = SnesRun { engine: "b".into(), snapshots: Vec::new(), audio_rate: 48000, audio: chirp(48000, 4, 4000.0, 30) };
    let w = diff::samples(&at32, &at48, 32000, 1600, 60);
    assert!(w.correlation > 0.999 && w.least > 0.99 && w.relative_error < 0.05, "{w:?}");
    assert_eq!((20, 20), (w.lag_from, w.lag_to));
    assert!((1.98..=2.02).contains(&w.gain), "{w:?}");
}

/// Since 2026-10-18 the library is on the core ABI alone: no pre-stable export is left in it (EmuSen_Backend.md §19.9).
#[test]
fn venusrts_library_is_on_the_core_abi_alone_and_its_state_is_its_own() {
    let lib = emusen_core_host::Library::load(&library()).unwrap();
    for name in ["emusen_native_interface_version", "emusen_native_capabilities", "emusen_native_create", "emusen_native_advance", "emusen_native_free"] {
        assert!(!lib.has(name), "{name} is still exported");
    }
    let mut m = diff::venusrt_machine(&vec![0u8; 0x8000 + 512], None, None).unwrap();
    for _ in 0..3 {
        m.advance().unwrap();
    }
    assert_eq!(3, m.frame_count());
    let info = m.frame_info().unwrap();
    assert_eq!((256, 224), (info.width, info.height));
    assert_eq!(32000, m.audio_rate());
    let sizes: Vec<usize> = grader::SPACES.iter().map(|n| diff::read_space(&mut m, n).len()).collect();
    assert_eq!(vec![0x10000, 0x200, 0x220, 0x20000, 0x10000], sizes);
    let mut state = vec![0u8; m.state_size(0).unwrap()];
    m.state_save(0, &mut state).unwrap();
    assert_eq!(b"VNRT", &state[..4]);
    state[..4].copy_from_slice(b"SNES");
    assert!(m.state_load(&state).is_err());
    assert_eq!(-9, diff::venusrt_machine(&vec![0u8; 0x4000], None, None).err().unwrap().status);
}

fn temp_rom(name: &str, bytes: &[u8]) -> PathBuf {
    let p = scratch(name).join("game.sfc");
    std::fs::write(&p, bytes).unwrap();
    p
}

#[test]
fn the_runner_drives_venusrt_to_the_frames_asked_for() {
    let rom = temp_rom("runner", &programs::board(&[]));
    let run = diff::run_venusrt(&rom, &[20, 10], &[Press { frame: 2, button: 3, frames: 4 }], true, true, None);
    assert_eq!(vec![10, 20], run.snapshots.iter().map(|s| s.frame).collect::<Vec<_>>());
    assert_eq!(Outcome::Visual, grader::grade(&rom, &run.snapshots).outcome);
    assert!(diff::compare(&run, &run, 0).iter().all(|d| d.identical()));
}

/// A boot image handed to the runner is the one the SPC700 starts from: the open program writes $AA and $BB through $F4 and $F5 to the RAM beneath, a synthetic loop at its own vector does not.
#[test]
fn the_runner_starts_venusrt_from_a_boot_image_when_given_one() {
    let rom = temp_rom("boot", &programs::board(&[]));
    let mut lp = [0u8; 64];
    (lp[0], lp[1], lp[62], lp[63]) = (0x2F, 0xFE, 0xC0, 0xFF);
    let own = diff::run_venusrt(&rom, &[10], &[], false, false, None).snapshots[0].spaces["apuram"].clone();
    let given = diff::run_venusrt(&rom, &[10], &[], false, false, Some(&lp)).snapshots[0].spaces["apuram"].clone();
    assert_eq!((0xAA, 0xBB), (own[0xF4], own[0xF5]));
    assert_ne!((0xAA, 0xBB), (given[0xF4], given[0xF5]));
}

/// Mesen's picture rows against a 224-line engine's: the offset that makes steady pictures agree is the one the differential uses.
#[test]
fn the_picture_offset_is_the_one_steady_pictures_agree_at() {
    let (Some(root), Some(_)) = (grader::corpus_root(), diff::mesen_probe()) else {
        println!("{} unset or no probe, not run", grader::CORPUS_VARIABLE);
        return;
    };
    let cache = root.join("runs").join("mesen");
    let mut best = Vec::new();
    for rom in ["roms/240pSuite-SNES-1.03/240pSuite.sfc", "roms/gilyon-snes-tests-v1.4/spctest/spctest.sfc", "src/higan-snes-test-roms/PeterLemon/SNES-CPUTest-CPU/ADC/CPUADC.sfc"] {
        let path = root.join(rom);
        if !path.exists() {
            continue;
        }
        let a = diff::run_default(&path, &[1800, 3600], &[], false);
        let b = diff::run_mesen(&cache, &path, &[1800, 3600], &[], false);
        let scores: Vec<(usize, usize)> = (0..16).map(|o| diff::pictures(a.snapshots[1].picture.as_ref(), b.snapshots[1].picture.as_ref(), o)).collect();
        let at = (0..16).min_by_key(|&o| (scores[o].1, o)).unwrap();
        println!("{rom}: best offset {at}, {} of {} differ; offset 0: {}", scores[at].1, scores[at].0, scores[0].1);
        best.push(at);
    }
    assert!(!best.is_empty());
    assert!(best.iter().all(|&o| o == MESEN_ROW_OFFSET), "{best:?}");
}

/// Opt-in: each game in EMUSEN_VENUSRT_GAMES against Mesen every tenth frame, and where each space and the picture first part.
#[test]
fn the_games_pictures_and_spaces_against_mesen() {
    let (Some(dir), Some(root), Some(_)) = (std::env::var_os("EMUSEN_VENUSRT_GAMES").map(PathBuf::from), grader::corpus_root(), diff::mesen_probe()) else {
        println!("EMUSEN_VENUSRT_GAMES, the corpus or the probe unset, not run");
        return;
    };
    let until: u32 = std::env::var("EMUSEN_VENUSRT_GAMES_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(600);
    let frames: Vec<u32> = (1..=until / 10).map(|i| i * 10).collect();
    let cache = root.join("runs").join("mesen");
    let mut table = vec!["game\tmesen_lit\tvenusrt_lit\tvram\tcgram\toam\tpicture\tpictures_differing\taudio\tsamples\tat_last".to_string()];
    let mut roms: Vec<PathBuf> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.path()).filter(|p| p.is_file()).collect();
    roms.sort();
    for rom in roms {
        let theirs = diff::run_mesen(&cache, &rom, &frames, &[], true);
        let ours = diff::run_default(&rom, &frames, &[], true);
        let sound = diff::audio(&ours, &theirs, 30);
        let wave = diff::samples(&ours, &theirs, 32000, 1600, 60);
        let d = diff::compare(&ours, &theirs, MESEN_ROW_OFFSET);
        let lit = |r: &SnesRun| r.snapshots.iter().find(|s| s.picture.as_ref().is_some_and(|p| p.pixels.iter().any(|&x| x & 0x7FFF != 0))).map_or("never".to_string(), |s| s.frame.to_string());
        let first = |n: &dyn Fn(&diff::FrameDiff) -> usize| d.iter().find(|f| n(f) != 0).map_or("never".to_string(), |f| f.frame.to_string());
        let space = |f: &diff::FrameDiff, s: &str| f.space_bytes.get(s).copied().unwrap_or(0);
        let last = d.last().unwrap();
        table.push(
            [
                rom.file_name().unwrap().to_string_lossy().into_owned(),
                lit(&theirs),
                lit(&ours),
                first(&|f| space(f, "vram")),
                first(&|f| space(f, "cgram")),
                first(&|f| space(f, "oam")),
                first(&|f| f.pixels_differing),
                format!("{} of {}", d.iter().filter(|f| f.pixels_differing != 0).count(), d.len()),
                format!("r {:.3} lag {}{}{}", sound.correlation, sound.lag_windows, if sound.first_silent { " venusrt silent" } else { "" }, if sound.second_silent { " mesen silent" } else { "" }),
                format!("r {:.4} least {:.4} lag {}..{} gain {:.3} error {:.4} over {}s", wave.correlation, wave.least, wave.lag_from, wave.lag_to, wave.gain, wave.relative_error, wave.seconds),
                format!("vram {}B cgram {}B oam {}B picture {}/{}px", space(last, "vram"), space(last, "cgram"), space(last, "oam"), last.pixels_differing, last.pixels_compared),
            ]
            .join("\t"),
        );
    }
    for line in &table {
        println!("{line}");
    }
    if let Some(report) = std::env::var_os(REPORT_VARIABLE) {
        std::fs::write(report, table.join("\n") + "\n").unwrap();
    }
}

/// The corpus over VenusRT and Mesen, graded by each suite's protocol, against the committed baseline; C# Venus's columns are frozen there since it was deleted (VenusRT_Native.md §68.3).
#[test]
fn the_corpus_reproduces_the_recorded_baseline() {
    let Some(root) = grader::corpus_root() else {
        println!("{} unset, not run", grader::CORPUS_VARIABLE);
        return;
    };
    let mesen = diff::mesen_probe().is_some() && std::env::var("EMUSEN_VENUSRT_NO_MESEN").as_deref() != Ok("1");
    let cache = root.join("runs").join("mesen");
    let threads: usize = std::env::var("EMUSEN_VENUSRT_THREADS").ok().and_then(|t| t.parse().ok()).unwrap_or(4);
    let baseline = std::fs::read_to_string(common::golden::directory().join("VenusRtBaseline.tsv")).unwrap();
    let want: BTreeMap<String, Vec<String>> = baseline.lines().skip(1).filter(|l| !l.is_empty()).map(|l| {
        let c: Vec<String> = l.split('\t').map(str::to_string).collect();
        (c[0].clone(), c)
    }).collect();
    let roms = grader::unique_with_md5(&root);
    let rows = std::sync::Mutex::new(BTreeMap::new());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let verdict_cells = |v: &Verdict| [v.outcome.word().to_string(), if v.outcome.self_graded() || v.outcome == Outcome::Done { format!("{} {}", v.protocol, v.detail).trim().to_string() } else { String::new() }];
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some((md5, rel)) = roms.get(i) else { break };
                let path = root.join(rel);
                let frames = grader::frames(rel);
                let empty = vec![String::new(); 15];
                let frozen = want.get(md5).unwrap_or(&empty);
                let cell = |i: usize| frozen.get(i).cloned().unwrap_or_default();
                let mut cells = vec![md5.clone(), rel.clone(), grader::suite(rel).to_string(), cell(3), cell(4)];
                let mut theirs = None;
                if mesen {
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| diff::run_mesen(&cache, &path, frames, &[], false))) {
                        Ok(r) => {
                            cells.extend(verdict_cells(&grader::grade(&path, &r.snapshots)));
                            theirs = Some(r);
                        }
                        Err(_) => cells.extend(["Threw".to_string(), "panic".to_string()]),
                    }
                    cells.push(cell(7));
                    let steady = theirs.as_ref().is_some_and(|r| r.snapshots.len() == 2 && r.snapshots[0].spaces.get("vram") == r.snapshots[1].spaces.get("vram"));
                    cells.push(if steady { "steady" } else { "moving" }.to_string());
                } else {
                    cells.extend([cell(5), cell(6), cell(7), cell(8)]);
                }
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| diff::run_default(&path, frames, &[], false))) {
                    Ok(run) => {
                        cells.extend(verdict_cells(&grader::grade(&path, &run.snapshots)));
                        let against = theirs.as_ref().and_then(|t| diff::compare(&run, t, MESEN_ROW_OFFSET).pop());
                        for (k, space) in ["vram", "cgram", "oam"].iter().enumerate() {
                            cells.push(match &against {
                                None => frozen.get(11 + k).cloned().unwrap_or_else(|| "n/a".into()),
                                Some(a) => match a.space_bytes.get(*space).copied().unwrap_or(0) {
                                    0 => "same".into(),
                                    n => format!("{n}B"),
                                },
                            });
                        }
                        cells.push(match &against {
                            None => frozen.get(14).cloned().unwrap_or_else(|| "n/a".into()),
                            Some(a) if a.pixels_compared == 0 => "n/a".into(),
                            Some(a) if a.pixels_differing == 0 => "same".into(),
                            Some(a) => format!("{}px", a.pixels_differing),
                        });
                    }
                    Err(_) => cells.extend(["Threw", "panic", "n/a", "n/a", "n/a", "n/a"].map(str::to_string)),
                }
                rows.lock().unwrap().insert(md5.clone(), cells);
            });
        }
    });
    let rows = rows.into_inner().unwrap();
    let lines: Vec<&Vec<String>> = roms.iter().map(|(md5, _)| &rows[md5]).collect();
    if let Some(report) = std::env::var_os(REPORT_VARIABLE) {
        let header = "md5\trom\tsuite\tvenus\tvenus_detail\tmesen\tmesen_detail\tvram_last\tmesen_vram\tvenusrt\tvenusrt_detail\tvenusrt_vram\tvenusrt_cgram\tvenusrt_oam\tvenusrt_picture";
        let body: Vec<String> = lines.iter().map(|c| c.join("\t")).collect();
        std::fs::write(report, format!("{header}\n{}\n", body.join("\n"))).unwrap();
    }
    let differences: Vec<String> = lines
        .iter()
        .filter(|c| match want.get(&c[0]) {
            None => true,
            Some(w) => w[9] != c[9] || (c[9] != "Visual" && w[10] != c[10]) || (mesen && (w[5] != c[5] || (c[5] != "Visual" && w[6] != c[6]))),
        })
        .map(|c| format!("{}: VenusRT {}, Mesen {}", c[1], c[9], c[5]))
        .collect();
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}
