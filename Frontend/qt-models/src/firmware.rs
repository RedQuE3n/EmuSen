//! A console's firmware as needs and their words, the one owner for every screen that shows it (EmuSen_Interface.md
//! §18.10, EmuSen_BigPicture.md §52.4, EmuSen_Qt.md §7.7.E): Settings ▸ Consoles ▸ <console> ▸ Firmware and Cores'
//! Firmware page. The files are the runtime's `firmware_status` (EmuSen_Cores.md §3.11, §3.13); files that meet the
//! same need, as a console's regional BIOSes, are one need. Nothing here knows a console.

use emusen_crystal_client::shapes::{FirmwareFile, FirmwareItem};

/// One need of a console's firmware and the files that meet it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Need {
    /// The need in the console's words: a file's label, or the systems table's `firmware_name` for several.
    pub name: String,
    /// The last word of its name less any bracket, as a short line says it ("BIOS").
    pub word: String,
    pub files: Vec<FirmwareFile>,
}

impl Need {
    /// The file the player's folders hold, if one.
    pub fn found(&self) -> Option<&FirmwareFile> {
        self.files.iter().find(|f| f.status == "present")
    }

    /// No file is found and one is required.
    pub fn missing(&self) -> bool {
        self.found().is_none() && self.files.iter().any(|f| f.status == "missing")
    }

    /// The copy recognised, by its file's name less the extension in capitals ("SCPH-1001").
    pub fn recognised(&self) -> String {
        self.found().map(|f| std::path::Path::new(&f.file).file_stem().unwrap_or_default().to_string_lossy().to_uppercase()).unwrap_or_default()
    }

    /// The files' names, as a sentence lists them.
    pub fn file_names(&self) -> String {
        self.files.iter().map(|f| f.file.as_str()).collect::<Vec<_>>().join(", ")
    }
}

/// A console's files, each with the system that declares it, grouped into needs: each file once, by its name; a need of
/// one file named by its label, of several by `table_name` where the systems table gives one.
pub fn needs(files: &[(String, FirmwareFile)], table_name: Option<&str>) -> Vec<Need> {
    let mut seen: Vec<String> = Vec::new();
    let mut groups: Vec<(String, Vec<FirmwareFile>)> = Vec::new();
    for (at, (system, file)) in files.iter().enumerate() {
        if seen.iter().any(|s| s.eq_ignore_ascii_case(&file.file)) {
            continue;
        }
        seen.push(file.file.clone());
        let need = if file.need.is_empty() { format!("{system}:#{at}") } else { format!("{system}:{}", file.need) };
        match groups.iter_mut().find(|(n, _)| *n == need) {
            Some((_, list)) => list.push(file.clone()),
            None => groups.push((need, vec![file.clone()])),
        }
    }
    groups
        .into_iter()
        .map(|(_, files)| {
            let label = if files[0].label.is_empty() { files[0].file.clone() } else { files[0].label.clone() };
            let table = table_name.filter(|t| !t.is_empty());
            let name = if files.len() > 1 { table.map_or(label.clone(), str::to_owned) } else { label };
            let word = table.unwrap_or(&name).split('(').next().unwrap_or_default().split_whitespace().last().unwrap_or("firmware").to_owned();
            Need { name, word, files }
        })
        .collect()
}

/// The firmware in a few words for a console's line in a list; none where it has none to speak of (`needed_by_all` false:
/// a cartridge console, whose files are chips only some games carry).
pub fn short(needs: &[Need], needed_by_all: bool) -> Option<String> {
    if needs.is_empty() || !needed_by_all {
        return None;
    }
    Some(if let Some(found) = needs.iter().find(|n| n.found().is_some()) {
        format!("✓ {} found", found.word)
    } else if let Some(missing) = needs.iter().find(|n| n.missing()) {
        format!("⚠ {} not found", missing.word)
    } else {
        "open replacement".to_owned()
    })
}

/// The firmware in one line: none needed, found, needed by every game and not found, or the open replacement in its
/// place.
pub fn line(needs: &[Need], needed_by_all: bool) -> String {
    if needs.is_empty() || !needed_by_all {
        return "✓ No firmware needed".to_owned();
    }
    if let Some(found) = needs.iter().find(|n| n.found().is_some()) {
        return format!("✓ {} found", found.word);
    }
    if let Some(missing) = needs.iter().find(|n| n.missing()) {
        return format!("⚠ {} not found: add it under Firmware", missing.word);
    }
    format!("No {} of your own: the open replacement runs in its place", needs[0].word)
}

/// The Firmware row's value on a console's page.
pub fn value(needs: &[Need], needed_by_all: bool) -> String {
    let found = needs.iter().any(|n| n.found().is_some());
    let missing = needs.iter().any(Need::missing);
    match (needs.is_empty(), needed_by_all) {
        (true, _) => "none needed",
        (false, false) if found => "✓ Found",
        (false, false) if missing => "only some games",
        (false, false) => "none needed",
        (false, true) if found => "✓ Found",
        (false, true) if missing => "⚠ Not found",
        (false, true) => "Not found",
    }
    .to_owned()
}

/// A need's row value on the Firmware page.
pub fn status(need: &Need, needed_by_all: bool) -> String {
    match need.found() {
        Some(_) => format!("✓ Found · {}", need.recognised()),
        None => short_status(need, needed_by_all),
    }
}

/// A need's value where a row is narrow, as a card's is: the copy recognised left to the pane.
pub fn short_status(need: &Need, needed_by_all: bool) -> String {
    if need.found().is_some() {
        "✓ Found".to_owned()
    } else if need.missing() && needed_by_all {
        "⚠ Not found".to_owned()
    } else if need.missing() {
        "Only some games".to_owned()
    } else if needed_by_all {
        "Not found".to_owned()
    } else {
        "Not needed".to_owned()
    }
}

/// A pane's words: its plain lines first (the first is the sentence), then its muted lines.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Words {
    pub lines: Vec<String>,
    pub muted: Vec<String>,
}

/// The pane of a console's Firmware row: what is found, and that it is recognised by content and never downloaded.
pub fn pane(needs: &[Need], needed_by_all: bool, console: &str) -> Words {
    if needs.is_empty() {
        return Words { lines: vec![format!("The {console} needs no firmware.")], muted: vec!["EmuSen never downloads firmware.".to_owned()] };
    }
    let mut muted = Vec::new();
    let absent = needs.iter().filter(|n| n.missing()).count();
    if !needed_by_all && absent > 0 {
        muted.push(format!("{} only some games use not found.", crate::words::counted(absent, "chip file", "chip files")));
    }
    muted.push("Recognised by its content in your firmware folders. EmuSen never downloads firmware; without your own, the open replacement runs most games.".to_owned());
    Words { lines: vec![line(needs, needed_by_all)], muted }
}

/// A need's pane: what runs, your own copy when found, else the open replacement; then what the runtime's overview says
/// of each file: where it was found, why one of the wrong size is unused, what the open version changes.
pub fn need_pane(need: &Need, needed_by_all: bool, console: &str, engine: &str, replaced: bool, overview: &[FirmwareItem]) -> Words {
    let files = need.file_names();
    let (said, muted) = if need.found().is_some() {
        let runs = if replaced { "runs the open replacement instead, as Open replacement says Always" } else { "uses it" };
        (format!("✓ Your own {} ({}) was recognised by its content, and {engine} {runs}.", need.name, need.recognised()), "EmuSen never downloads firmware.".to_owned())
    } else if need.missing() && needed_by_all {
        (format!("⚠ No {} of your own was found, and the {console}'s games need one.", need.name), format!("Put your own copy ({files}) in your firmware folder, then Check again."))
    } else if need.missing() {
        (
            format!("{}: needed only by the games that carry its chip; none of your own was found.", need.name),
            format!("Put your own copy ({}) in your firmware folder for those games, then Check again.", need.files[0].file),
        )
    } else {
        (
            format!("No {} of your own was found, so {engine} runs its open replacement in its place.", need.name),
            format!("Your own copy ({files}) is used once it is found. EmuSen never downloads firmware."),
        )
    };
    let mut words = Words { lines: vec![said], muted: vec![muted] };
    for file in &need.files {
        let Some(item) = overview.iter().find(|i| i.file.eq_ignore_ascii_case(&file.file)) else { continue };
        if let Some(folder) = &item.found_in {
            words.muted.push(format!("{}: found in {folder}.", file.file));
        }
        if item.wrong_size {
            let folder = item.wrong_size_in.clone().unwrap_or_else(|| "EmuSen's folder".to_owned());
            words.lines.push(format!("{}: not used, the file in {folder} is not {} bytes.", file.file, crate::words::grouped(item.size as usize)));
        }
        if item.found_in.is_none() {
            match item.effect.as_deref() {
                Some("exact") => words.muted.push("Exact: games run the same with the open version.".to_owned()),
                Some("accuracy") => {
                    if let Some(cost) = first_sentence(item.cost.as_deref()) {
                        words.muted.push(format!("The open version: {cost}"));
                    }
                }
                _ => {}
            }
        }
    }
    let mut kept: Vec<String> = Vec::new();
    for line in words.muted.drain(..) {
        if !kept.contains(&line) {
            kept.push(line);
        }
    }
    words.muted = kept;
    words
}

/// The first sentence of a core's words on a cost; the rest is detail for the core's own page.
fn first_sentence(text: Option<&str>) -> Option<String> {
    let text = text?.trim();
    if text.is_empty() {
        return None;
    }
    Some(match text.find(". ") {
        Some(end) => text[..=end].to_owned(),
        None => text.to_owned(),
    })
}
