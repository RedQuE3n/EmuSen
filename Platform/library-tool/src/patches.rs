//! A package's soft patches added and removed: `pkg patch add` and `pkg patch rm` (EmuSen_GamePackages.md §3.3, §3.7,
//! §5.4, §15). A patch is copied into the package's root, checked against the image it will be given, and appended to
//! the list; the files are named as RetroArch finds them whenever the list fits its convention (§9 Q10); `patched` is
//! taken again whenever the list changes. The file given is only read, the library never written.

use crate::convert::{obj, Outcome};
use crate::packages::{head, read};
use crate::Context;
use emusen_galaxia::claims;
use emusen_galaxia::dotnet_files;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::json::Value;
use emusen_galaxia::{archive, image_sets, package, rom_hash};
use emusen_patch::package::{self as listing, Listed};
use emusen_patch::{Format, Refusal};

fn text(s: &str) -> Value {
    Value::string(s)
}

/// Names a patch's own file may not take, being a package's own.
const RESERVED: [&str; 3] = [package::FILE_NAME, package::CHEATS_FILE, "settings.json"];

/// The image a package's patches apply to: its game's bytes, the entry inside where the game is a zip, held to the dump
/// recorded so that a patch is never pinned to another file. Words where it is a disc, is gone or is another dump.
pub fn source_image(ctx: &Context, folder: &str, doc: &Value) -> Result<Vec<u8>, String> {
    let roots = ctx.places();
    let roots = roots.roots();
    let game = doc.get("game").and_then(|g| claims::place(&roots, folder, g)).ok_or("its game's library is not set (AppSettings.RomDirectory)")?;
    let container = doc.get("container").and_then(|c| c.get("type")).and_then(Value::as_str);
    if container == Some("chd") || image_sets::is_sheet(&game) || image_sets::is_playlist(&game) || doc.get("dump").and_then(|d| d.get("tracks")).is_some_and(|t| !matches!(t, Value::Null)) {
        return Err("it is a disc game, and disc patches are left to a page of their own (EmuSen_GamePackages.md §9 Q15)".into());
    }
    if !dotnet_files::file_exists(&game) {
        return Err(format!("{} is gone: pkg check looks for it", claims::entry_place(&roots, &game).1));
    }
    let bytes = match doc.get("container").and_then(|c| c.get("entry")).and_then(Value::as_str) {
        Some(entry) if archive::is_archive(&game) => archive::read_entry(&game, entry, emusen_patch::MAX_IMAGE as usize + 1)?,
        _ => {
            let size = std::fs::metadata(&game).map_err(|e| e.to_string())?.len();
            if size > emusen_patch::MAX_IMAGE {
                return Err(format!("the game is {size} bytes, more than the {} a patched image may be", emusen_patch::MAX_IMAGE));
            }
            std::fs::read(&game).map_err(|e| format!("{game}: {e}"))?
        }
    };
    if let Some(file) = doc.get("dump").and_then(|d| d.get("file")).filter(|f| matches!(f, Value::Object(_)))
        && !claims::same_dump(file, &rom_hash::hashes_of(&bytes))
    {
        return Err(format!("{} is not the dump the package was made for ({}): pkg check says more", claims::entry_place(&roots, &game).1, claims::named(file)));
    }
    Ok(bytes)
}

/// The names a list's files take: as RetroArch finds them beside `<stem>` where the list is one BPS, one UPS, or IPS
/// patches alone (`.ips`, `.ips1`, `.ips2` in order); otherwise each its own name, made distinct. Whether it fits.
pub fn names(stem: &str, list: &[(Format, String)]) -> (Vec<String>, bool) {
    let fits = !list.is_empty() && (list.iter().all(|(f, _)| *f == Format::Ips) || (list.len() == 1 && list[0].0 != Format::Ips));
    if fits {
        let named = list.iter().enumerate().map(|(n, (f, _))| if n == 0 { format!("{stem}.{}", f.name()) } else { format!("{stem}.{}{n}", f.name()) }).collect();
        return (named, true);
    }
    let mut named: Vec<String> = Vec::new();
    for (_, own) in list {
        let own = if RESERVED.iter().any(|r| r.eq_ignore_ascii_case(own)) { format!("patch {own}") } else { own.clone() };
        let (base, ext) = (path::file_name_without_extension(Style::Unix, &own).to_string(), path::extension(Style::Unix, &own).to_string());
        let mut candidate = own.clone();
        let mut k = 2;
        while named.iter().any(|n| n.eq_ignore_ascii_case(&candidate)) {
            candidate = format!("{base} ({k}){ext}");
            k += 1;
        }
        named.push(candidate);
    }
    (named, false)
}

/// Each file renamed from its old name to its new, through a hidden name first so that two may swap; refused before
/// anything moves where a new name is taken by a file that is not one of the list's.
fn relayout(folder: &str, moves: &[(String, String)]) -> Result<(), String> {
    let olds: Vec<&String> = moves.iter().map(|(o, _)| o).collect();
    for (old, new) in moves {
        if old != new && dotnet_files::file_exists(&claims::under(folder, new)) && !olds.iter().any(|o| o.eq_ignore_ascii_case(new)) {
            return Err(format!("the package already holds a file named {new} that is not one of its patches, so nothing was changed"));
        }
    }
    let pid = std::process::id();
    let mut staged = Vec::new();
    for (k, (old, new)) in moves.iter().enumerate().filter(|(_, (o, n))| o != n) {
        let hidden = format!(".relayout-{pid}-{k}");
        std::fs::rename(claims::under(folder, old), claims::under(folder, &hidden)).map_err(|e| format!("{old}: {e}"))?;
        staged.push((hidden, new));
    }
    for (hidden, new) in staged {
        std::fs::rename(claims::under(folder, &hidden), claims::under(folder, new)).map_err(|e| format!("{new}: {e}"))?;
    }
    Ok(())
}

fn own_name(entry: &Value) -> String {
    entry.get("name").or_else(|| entry.get("file")).and_then(Value::as_str).map(|n| path::file_name(Style::Unix, &n.replace('\\', "/")).to_string()).unwrap_or_default()
}

fn set_patches(doc: &mut Value, entries: Vec<Value>, patched: Value) {
    claims::set(doc, "patches", Value::Array(entries));
    claims::set(doc, "patched", patched);
}

/// The entries renamed as `names` lays them out, the files moved, and the manifest written with `patched` taken over
/// `image`; the words of the first thing that refused.
fn commit(folder: &str, doc: &mut Value, mut entries: Vec<Value>, moves_from: Vec<String>, image: Option<&[u8]>) -> Result<bool, String> {
    let stem = emusen_galaxia::saves::package_stem(folder).to_string();
    let list: Vec<(Format, String)> = entries.iter().map(|e| (e.get("format").and_then(Value::as_str).and_then(Format::named).unwrap_or(Format::Ips), own_name(e))).collect();
    let (named, fits) = names(&stem, &list);
    let moves: Vec<(String, String)> = moves_from.into_iter().zip(named.iter().cloned()).collect();
    relayout(folder, &moves)?;
    for (e, n) in entries.iter_mut().zip(&named) {
        claims::set(e, "file", text(n));
    }
    let patched = match image {
        Some(img) if !entries.is_empty() => {
            let mut draft = doc.clone();
            claims::set(&mut draft, "patches", Value::Array(entries.clone()));
            claims::set(&mut draft, "patched", Value::Null);
            let listed: Vec<Listed> = listing::read_list(folder, &draft)?;
            let result = listing::apply_listed(&draft, &listed, img)?;
            listing::patched_json(&listed, &result)
        }
        _ => Value::Null,
    };
    set_patches(doc, entries, patched);
    match package::write(&dotnet_files::join(folder, package::FILE_NAME), doc) {
        Ok(Ok(())) => Ok(fits),
        Ok(Err(words)) => Err(words),
        Err(refusal) => Err(refusal.words().to_string()),
    }
}

/// Words for a checksummed patch made for another form of the image: where the system pack's comparable form (a copier
/// header taken off, say) is the patch's source, the words say so, and it is still not applied (§5.4).
fn refused(ctx_system: Option<&str>, current: &[u8], patch: &[u8], refusal: &Refusal) -> String {
    let words = refusal.words();
    if let (Refusal::WrongSource { expected_crc32, .. }, Some(system)) = (refusal, ctx_system)
        && let Some(form) = emusen_systems::openvgdb::bytes(system, current).filter(|f| f.as_slice() != current)
    {
        let mut c = rom_hash::Crc32::default();
        c.append(&form);
        if c.value() == *expected_crc32 && emusen_patch::apply(&form, patch).is_ok() {
            return format!("{words}. It was made for this image's comparable form, {} bytes with CRC-32 {:08x} (its copier header taken off): EmuSen gives the core the image as dumped, so the patch was not added", form.len(), c.value());
        }
    }
    words
}

/// `pkg patch add`: the patch at `file` checked against the image the list so far makes, copied into the package and
/// appended to its list. `added`, `failed`.
pub fn add(ctx: &Context, folder: &str, file: &str) -> Outcome {
    let mut doc = match read(folder) {
        Ok(d) => d,
        Err(words) => return Outcome::said("failed", words),
    };
    let fail = |doc: &Value, words: String| head(doc, Outcome::said("failed", words));
    let size = match std::fs::metadata(file) {
        Ok(m) if m.is_file() => m.len(),
        Ok(_) => return fail(&doc, format!("{file} is not a file")),
        Err(e) => return fail(&doc, format!("{file}: {e}")),
    };
    if size > listing::MAX_PATCH {
        return fail(&doc, format!("{file} is {size} bytes, more than the {} a patch may be", listing::MAX_PATCH));
    }
    let bytes = match std::fs::read(file) {
        Ok(b) => b,
        Err(e) => return fail(&doc, format!("{file}: {e}")),
    };
    let info = match emusen_patch::info(&bytes) {
        Ok(i) => i,
        Err(r) => return fail(&doc, format!("{}: {}", path::file_name(Style::HOST, file), r.words())),
    };
    let image = match source_image(ctx, folder, &doc) {
        Ok(i) => i,
        Err(words) => return fail(&doc, words),
    };
    let listed = match listing::read_list(folder, &doc) {
        Ok(l) => l,
        Err(words) => return fail(&doc, words),
    };
    let current = match listing::apply_listed(&doc, &listed, &image) {
        Ok(c) => c,
        Err(words) => return fail(&doc, format!("the package's own patches do not apply: {words}")),
    };
    let system = doc.get("system").and_then(Value::as_str).and_then(crate::packages::system_named).map(|s| s.id.clone());
    let result = match emusen_patch::apply(&current, &bytes) {
        Ok(r) => r,
        Err(r) => return fail(&doc, format!("{}: {}", path::file_name(Style::HOST, file), refused(system.as_deref(), &current, &bytes, &r))),
    };
    let own = path::file_name(Style::HOST, file).to_string();
    let staged = format!(".adding-{}", std::process::id());
    if let Err(words) = emusen_galaxia::atomic::write(&claims::under(folder, &staged), &bytes) {
        return fail(&doc, words);
    }
    let mut members = vec![("file", text(&staged)), ("format", text(info.format.name())), ("name", text(&own)), ("sha1", text(&rom_hash::sha1_of(&bytes)))];
    match (info.source_crc32, info.target_crc32) {
        (Some(s), Some(t)) => members.extend([("source_crc32", text(&format!("{s:08x}"))), ("target_crc32", text(&format!("{t:08x}")))]),
        _ => members.push(("source_sha1", text(&rom_hash::sha1_of(&current)))),
    }
    let mut entries: Vec<Value> = listing::entries(&doc).to_vec();
    let mut olds: Vec<String> = entries.iter().map(|e| e.get("file").and_then(Value::as_str).unwrap_or("").to_string()).collect();
    entries.push(obj(members));
    olds.push(staged.clone());
    match commit(folder, &mut doc, entries, olds, Some(&image)) {
        Ok(fits) => {
            let name = listing::entries(&doc).last().and_then(|e| e.get("file")).and_then(Value::as_str).unwrap_or("").to_string();
            let mut c = rom_hash::Crc32::default();
            c.append(&result);
            head(&doc, Outcome::new("added").with("patch", text(&name)).with("format", text(info.format.name())).with("retroarch", Value::Bool(fits)).with("patched_crc32", text(&c.hex())).with("patches", Value::number(listing::entries(&doc).len())))
        }
        Err(words) => {
            let _ = std::fs::remove_file(claims::under(folder, &staged));
            fail(&doc, words)
        }
    }
}

/// `pkg patch rm`: patch `n` (from 1, as `pkg ls` numbers them) taken out of the list, its copy in the package
/// removed, and the rest laid out and checked again; refused where the rest would no longer apply. `removed`, `failed`.
pub fn remove(ctx: &Context, folder: &str, n: usize) -> Outcome {
    let mut doc = match read(folder) {
        Ok(d) => d,
        Err(words) => return Outcome::said("failed", words),
    };
    let fail = |doc: &Value, words: String| head(doc, Outcome::said("failed", words));
    let mut entries: Vec<Value> = listing::entries(&doc).to_vec();
    if n == 0 || n > entries.len() {
        return fail(&doc, format!("the package has {} patch(es), and there is no patch {n}", entries.len()));
    }
    let gone = entries.remove(n - 1);
    let gone_file = gone.get("file").and_then(Value::as_str).unwrap_or("").to_string();
    let olds: Vec<String> = entries.iter().map(|e| e.get("file").and_then(Value::as_str).unwrap_or("").to_string()).collect();
    let image = if entries.is_empty() {
        None
    } else {
        let image = match source_image(ctx, folder, &doc) {
            Ok(i) => i,
            Err(words) => return fail(&doc, words),
        };
        let mut draft = doc.clone();
        claims::set(&mut draft, "patches", Value::Array(entries.clone()));
        claims::set(&mut draft, "patched", Value::Null);
        if let Err(words) = listing::read_list(folder, &draft).and_then(|l| listing::apply_listed(&draft, &l, &image)) {
            return fail(&doc, format!("without patch {n} the rest no longer apply ({words}), so nothing was changed"));
        }
        Some(image)
    };
    // The removed file is set aside first, so that the rest may take its name.
    let aside = format!(".removing-{}", std::process::id());
    let gone_at = claims::under(folder, &gone_file);
    let had = listing::inside(&gone_file) && dotnet_files::file_exists(&gone_at);
    if had && let Err(e) = std::fs::rename(&gone_at, claims::under(folder, &aside)) {
        return fail(&doc, format!("{gone_file}: {e}"));
    }
    match commit(folder, &mut doc, entries, olds, image.as_deref()) {
        Ok(fits) => {
            if had {
                let _ = std::fs::remove_file(claims::under(folder, &aside));
            }
            head(&doc, Outcome::new("removed").with("patch", text(&gone_file)).with("retroarch", Value::Bool(fits)).with("patches", Value::number(listing::entries(&doc).len())))
        }
        Err(words) => {
            if had {
                let _ = std::fs::rename(claims::under(folder, &aside), &gone_at);
            }
            fail(&doc, words)
        }
    }
}

/// The list's files and what they make, for `pkg check`: every file held to its SHA-1 and the list applied to the
/// image; words where it does not.
pub fn check(ctx: &Context, folder: &str, doc: &Value) -> Result<(), String> {
    if listing::entries(doc).is_empty() {
        return Ok(());
    }
    let list = listing::read_list(folder, doc)?;
    let image = source_image(ctx, folder, doc)?;
    listing::apply_listed(doc, &list, &image).map(|_| ())
}
