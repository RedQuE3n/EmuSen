//! ps1-tests' `gte/test-all` replayed on the engine alone: its 1,150 cases from `tests.c`, each register written in
//! order, the command run, every register read back (BlackMoon_GTE.md §7). Unrun without `EMUSEN_BLACKMOON_CORPUS`.

use crate::Gte;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub struct Case {
    pub name: String,
    pub input: [u32; 64],
    pub opcode: Option<u32>,
    pub output: [u32; 64],
}

fn hex_words(text: &str) -> Vec<u32> {
    text.split("0x").skip(1).filter_map(|t| u32::from_str_radix(t.get(..8)?, 16).ok()).collect()
}

/// The cases of `tests.c`: per test its name, 64 inputs, the opcode (FFFFFFFFh for none) and 64 outputs.
pub fn cases(source: &str) -> Vec<Case> {
    let body = &source[source.find("const struct test_t tests[]").expect("the table")..];
    body.split(".name = \"")
        .skip(1)
        .map(|block| {
            let name = block[..block.find('"').unwrap()].to_owned();
            let words = hex_words(&block[block.find(".input").unwrap()..]);
            assert!(words.len() >= 129, "{name}: {} words", words.len());
            Case {
                name,
                input: words[..64].try_into().unwrap(),
                opcode: Some(words[64]).filter(|&o| o != 0xFFFF_FFFF),
                output: words[65..129].try_into().unwrap(),
            }
        })
        .collect()
}

pub fn corpus() -> Option<PathBuf> {
    std::env::var_os("EMUSEN_BLACKMOON_CORPUS").map(PathBuf::from)
}

/// The engine's 64 reads after the case, as test-all makes them: every register cleared, the inputs written in order.
pub fn replay(case: &Case) -> [u32; 64] {
    let mut g = Gte::new();
    for r in 0..64u8 {
        if r < 32 { g.set_data(r, 0) } else { g.set_control(r - 32, 0) }
    }
    for (r, &v) in case.input.iter().enumerate() {
        if r < 32 { g.set_data(r as u8, v) } else { g.set_control(r as u8 - 32, v) }
    }
    if let Some(op) = case.opcode {
        g.command(op & 0x01FF_FFFF);
    }
    std::array::from_fn(|r| g.register(r))
}

fn command_name(op: Option<u32>) -> String {
    match op {
        None => "registers".into(),
        Some(o) => crate::COPROCESSOR.commands.iter().find(|c| c.select == o & 0x3F).map_or(format!("{:02X}", o & 0x3F), |c| c.name.to_owned()),
    }
}

#[test]
fn test_all_every_case_bit_for_bit() {
    let Some(root) = corpus() else { return };
    let path = root.join("tests/ps1-tests/gte/test-all/tests.c");
    let Ok(source) = std::fs::read_to_string(&path) else { return };
    let all = cases(&source);
    assert_eq!(all.len(), 1150);
    let mut by_command: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let mut by_register: BTreeMap<(String, usize), u32> = BTreeMap::new();
    let mut shown = 0;
    for case in &all {
        let got = replay(case);
        let e = by_command.entry(command_name(case.opcode)).or_default();
        e.0 += 1;
        if got != case.output {
            e.1 += 1;
            for r in (0..64).filter(|&r| got[r] != case.output[r]) {
                *by_register.entry((command_name(case.opcode), r)).or_default() += 1;
                if shown < 400 && std::env::var("GTE_SHOW").is_ok_and(|s| s == command_name(case.opcode)) {
                    shown += 1;
                    println!("{} op {:08X?} r{r}: {:08X} want {:08X}", case.name, case.opcode, got[r], case.output[r]);
                }
            }
        }
    }
    let misses: u32 = by_command.values().map(|v| v.1).sum();
    for (c, (n, m)) in &by_command {
        println!("{c:10} {n:4} cases, {m:4} missed");
    }
    for ((c, r), n) in &by_register {
        println!("  {c:10} r{r:<2} {n}");
    }
    assert_eq!(misses, 0, "{misses} of 1150 cases differ");
}
