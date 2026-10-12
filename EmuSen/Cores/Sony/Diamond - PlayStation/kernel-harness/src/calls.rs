//! The call table read from the kernel's table.rs, one owner for each call's status, and printed as
//! Diamond_OpenKernel.md §5's table.

use std::fmt::Write;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub table: char,
    pub number: u32,
    pub status: char,
    pub name: String,
}

pub const ENTRIES: [(char, u32); 3] = [('A', 0xC0), ('B', 0x100), ('C', 0x80)];

/// Every `t[0xNN] = e!(S, "name", path);` line under each `build_x` function; an unlisted entry jumps to 0.
pub fn parse(source: &str) -> Vec<Call> {
    let mut calls = Vec::new();
    let mut table = None;
    for line in source.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("const fn build_") {
            table = rest.chars().next().map(|c| c.to_ascii_uppercase());
            continue;
        }
        let (Some(tab), Some(rest)) = (table, t.strip_prefix("t[0x")) else { continue };
        let (number, rest) = rest.split_once("] = e!(").expect("an entry");
        let (status, rest) = rest.split_once(", \"").expect("a status");
        let (name, _) = rest.split_once('"').expect("a name");
        calls.push(Call {
            table: tab,
            number: u32::from_str_radix(number, 16).unwrap(),
            status: status.chars().next().unwrap(),
            name: name.into(),
        });
    }
    let mut all = Vec::new();
    for (tab, n) in ENTRIES {
        for number in 0..n {
            all.push(calls.iter().find(|c| c.table == tab && c.number == number).cloned().unwrap_or(Call {
                table: tab,
                number,
                status: 'N',
                name: "jump_to_00000000h".into(),
            }));
        }
    }
    all
}

pub fn source() -> String {
    std::fs::read_to_string(crate::image::kernel_dir().join("src/table.rs")).expect("table.rs")
}

pub fn status_word(s: char) -> &'static str {
    match s {
        'D' => "implemented",
        'X' => "absent from an open kernel",
        'N' => "no function, as documented",
        'U' => "undocumented, returns 0",
        _ => "?",
    }
}

/// The counts and the table, each run of equal "no function" entries on one row.
pub fn markdown(calls: &[Call]) -> String {
    let mut out = String::new();
    let count = |s| calls.iter().filter(|c| c.status == s).count();
    writeln!(
        out,
        "{} entries: {} implemented, {} absent from an open kernel, {} no function as documented, {} undocumented.\n",
        calls.len(),
        count('D'),
        count('X'),
        count('N'),
        count('U')
    )
    .unwrap();
    out += "| Call | Name | Status |\n|---|---|---|\n";
    let mut i = 0;
    while i < calls.len() {
        let c = &calls[i];
        let mut j = i;
        while j + 1 < calls.len()
            && calls[j + 1].table == c.table
            && c.status == 'N'
            && calls[j + 1].status == 'N'
            && calls[j + 1].name == c.name
        {
            j += 1;
        }
        let number = if j > i {
            format!("{}({:02X}h..{:02X}h)", c.table, c.number, calls[j].number)
        } else {
            format!("{}({:02X}h)", c.table, c.number)
        };
        writeln!(out, "| {number} | `{}` | {} |", c.name, status_word(c.status)).unwrap();
        i = j + 1;
    }
    out
}
