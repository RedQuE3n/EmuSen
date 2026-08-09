// Everything a dump set is made of, with no knowledge of which emulator made it
// - see EmuSen_Debugging_Tools_Reference_v5.md §3.50.
//
// Every byte written here is the wire format Pharaoh and WiseMan already read,
// and they read the manifest with a regex rather than a JSON parser
// (Pharaoh/Reference/DumpSet.cs). That is why this module hand-rolls its JSON
// instead of reaching for serde: a serialiser is free to reorder keys and
// reflow whitespace, and either would break a consumer that is byte-shaped.
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufWriter, Result, Write};
use std::path::{Path, PathBuf};

use crate::backend::{MemorySpace, ProbeIdentity, ScreenView, find_space};

// The bare filename a consumer reads out of the manifest, which must match what
// BlobCache::write produced byte for byte - hence one formatter, not two.
pub fn blob_name(backend: &str, part: &str, frame: u32, extension: &str) -> String {
    format!("{backend}_{part}_f{frame:05}.{extension}")
}

pub fn blob_path(dir: &str, backend: &str, part: &str, frame: u32, extension: &str) -> PathBuf {
    Path::new(dir).join(blob_name(backend, part, frame, extension))
}

// A blob that has not changed since the last report is hard-linked to the file
// that already holds it rather than written again. CHR is the case that
// motivated this - on an 11-report SMB3 run it was 11 files and one distinct
// value, 30% of the set - but a static scene dedupes the screen just as well.
//
// A link rather than a manifest reference, deliberately: every consumer already
// globs for <backend>_<space>_f<frame>.bin and stats its size, and a "sameAs"
// field would have to be taught to all of them. The file is still there, still
// the right length, and `diff -r` between two dump sets still compares content.
//
// The comparison is byte-exact rather than hashed. A 32-bit digest over 128 KB
// would collide rarely enough to be missed in testing and often enough to
// silently publish one frame's memory as another's, which is the failure §3.49
// argues is worse than not having the feature.
#[derive(Default)]
pub struct BlobCache {
    previous: std::collections::HashMap<String, (u32, Vec<u8>)>,
    pub linked: u32,
    pub written: u32,
}

impl BlobCache {
    pub fn write(&mut self, dir: &str, backend: &str, space: &str, frame: u32, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        let path = blob_path(dir, backend, space, frame, "bin");
        if let Some((last_frame, bytes)) = self.previous.get(space)
            && bytes == data
            && *last_frame != frame
        {
            let source = blob_path(dir, backend, space, *last_frame, "bin");
            // A failed link is not an error: fall back to a full write, which is
            // what the probe did before this existed.
            let _ = std::fs::remove_file(&path);
            if std::fs::hard_link(&source, &path).is_ok() {
                self.linked += 1;
                return Ok(());
            }
        }

        File::create(&path)?.write_all(data)?;
        self.previous.insert(space.to_string(), (frame, data.to_vec()));
        self.written += 1;
        Ok(())
    }
}

// The same path shape, plus the 8-byte magic that tells a trace parser which
// record layout it is looking at (ESCT/ESGT/ESAW).
pub fn write_trace(
    dir: &str,
    backend: &str,
    kind: &str,
    frame: u32,
    magic: &[u8; 8],
    payload: &[u8],
    record_size: usize,
) -> Result<()> {
    let path = blob_path(dir, backend, kind, frame, "bin");
    let mut file = BufWriter::new(File::create(&path)?);
    file.write_all(magic)?;
    file.write_all(payload)?;
    file.flush()?;

    let steps = payload.len().checked_div(record_size).unwrap_or(0);
    println!("  [{kind} {steps} steps -> {}]", path.display());
    // A trailing partial record means the producer and this table disagree about
    // the layout, which would otherwise surface as a step count that is quietly
    // wrong. The C++ probe never checked. See §3.50.
    if record_size != 0 && !payload.len().is_multiple_of(record_size) {
        println!(
            "  [WARN] {kind} payload is {} bytes, not a multiple of the {record_size}-byte record",
            payload.len()
        );
    }
    Ok(())
}

// The manifest is small and fixed-shape, so a JSON dependency would cost more
// than it saves; only the ROM path can contain anything needing an escape.
fn json_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out
}

// <backend>_manifest_f<frame>.json. Additive, and the reason a dump set from
// an emulator this project has never seen is still readable: it names every
// space, its size and file, and the screen's dimensions and pixel format.
pub fn manifest_json(
    backend: &str,
    system: &str,
    rom_path: &str,
    frame: u32,
    spaces: &[MemorySpace<'_>],
    screen: Option<&ScreenView<'_>>,
    identity: &ProbeIdentity,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"backend\": \"{}\",", json_escape(backend));
    let _ = writeln!(out, "  \"system\": \"{}\",", json_escape(system));
    let _ = writeln!(out, "  \"rom\": \"{}\",", json_escape(rom_path));
    let _ = writeln!(out, "  \"frame\": {frame},");

    // An empty string here means "this backend cannot say", which the gate treats
    // as reduced confidence rather than as agreement - see §3.48.
    let _ = writeln!(
        out,
        "  \"identity\": {{ \"board\": \"{}\", \"region\": \"{}\", \"headerTrust\": \"{}\", \"prg\": {}, \"chr\": {}, \"saveLoaded\": {} }},",
        json_escape(&identity.board),
        json_escape(&identity.region),
        json_escape(&identity.header_trust),
        identity.prg_bytes,
        identity.chr_bytes,
        if identity.save_loaded { "true" } else { "false" }
    );

    out.push_str("  \"spaces\": [");

    let mut first = true;
    for space in spaces.iter().filter(|s| !s.data.is_empty()) {
        out.push_str(if first { "\n" } else { ",\n" });
        first = false;
        let _ = write!(
            out,
            "    {{ \"name\": \"{}\", \"size\": {}, \"file\": \"{}\" }}",
            space.name,
            space.size(),
            blob_name(backend, &space.name, frame, "bin")
        );
    }

    out.push_str(if first { "" } else { "\n  " });
    out.push_str("],\n");

    match screen.filter(|s| s.format != crate::backend::ScreenFormat::None) {
        Some(screen) => {
            let _ = writeln!(
                out,
                "  \"screen\": {{ \"width\": {}, \"height\": {}, \"bytes\": {}, \"format\": \"{}\", \"file\": \"{}\" }}",
                screen.width,
                screen.height,
                screen.bytes(),
                screen.format.name(),
                blob_name(backend, "screen", frame, "bin")
            );
        }
        None => out.push_str("  \"screen\": null\n"),
    }

    out.push_str("}\n");
    out
}

#[allow(clippy::too_many_arguments)]
pub fn write_manifest(
    dir: &str,
    backend: &str,
    system: &str,
    rom_path: &str,
    frame: u32,
    spaces: &[MemorySpace<'_>],
    screen: Option<&ScreenView<'_>>,
    identity: &ProbeIdentity,
) -> Result<()> {
    let text = manifest_json(backend, system, rom_path, frame, spaces, screen, identity);
    let path = blob_path(dir, backend, "manifest", frame, "json");
    File::create(path)?.write_all(text.as_bytes())
}

// The ordinary reflected 0xEDB88320 with 0xFFFFFFFF in and out, so a consumer in
// any language agrees without being told. Pharaoh mirrors it in C# at
// Reference/SignatureWriter.cs.
pub fn crc32(data: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

// The "RAM @ $0A00: xx xx ..." report line, off a named space rather than a
// backend-specific pointer. Reads past the end as zero rather than mirroring,
// which is deliberately not what the anchor does - see main.rs::anchor_word.
pub fn hex_line(label: &str, space: &MemorySpace<'_>, addr: u32, length: u32) -> String {
    let mut line = format!("  {label} @ ${addr:05X}:");
    for i in 0..length {
        let byte = space.data.get((addr + i) as usize).copied().unwrap_or(0);
        let _ = write!(line, " {byte:02X}");
    }
    line
}

// One row per frame of hashes, which is what locating a divergence needs and
// what 900 full dumps would cost too much to give - see §3.48.
//
// Memory spaces only, plus a screen column the consumer must not trust unless
// both sides declare the same pixel format: bytes are bytes everywhere, but a
// palette index and an RGBA quad describe the same picture and never hash
// alike. The limitation is declared in the header rather than worked around,
// because memory divergence precedes screen divergence in every case this was
// built for.
pub struct SignatureWriter {
    file: BufWriter<File>,
    columns: Vec<String>,
}

impl SignatureWriter {
    pub fn open(path: &Path) -> Result<SignatureWriter> {
        Ok(SignatureWriter {
            file: BufWriter::new(File::create(path)?),
            columns: Vec::new(),
        })
    }

    pub fn header_text(
        &mut self,
        backend: &str,
        system: &str,
        rom_path: &str,
        identity: &ProbeIdentity,
        screen: Option<&ScreenView<'_>>,
        spaces: &[MemorySpace<'_>],
    ) -> String {
        let mut out = String::new();
        out.push_str("# emusen-probe-signature 1\n");
        let _ = writeln!(out, "# backend={backend}");
        let _ = writeln!(out, "# system={system}");
        let _ = writeln!(out, "# rom={rom_path}");
        let _ = writeln!(out, "# board={}", identity.board);
        let _ = writeln!(out, "# region={}", identity.region);
        let _ = writeln!(out, "# headerTrust={}", identity.header_trust);
        let _ = writeln!(out, "# prg={}", identity.prg_bytes);
        let _ = writeln!(out, "# chr={}", identity.chr_bytes);
        let _ = writeln!(out, "# saveLoaded={}", if identity.save_loaded { 1 } else { 0 });
        let _ = writeln!(
            out,
            "# screenFormat={}",
            screen.map_or("None", |s| s.format.name())
        );

        // The column list is the schema: a consumer intersects on names rather than
        // assuming two backends expose the same spaces, which they do not.
        self.columns = spaces
            .iter()
            .filter(|s| !s.data.is_empty())
            .map(|s| s.name.clone())
            .collect();

        out.push_str("frame");
        for name in &self.columns {
            let _ = write!(out, ",{name}");
        }
        out.push_str(",screen\n");
        out
    }

    pub fn write_header(
        &mut self,
        backend: &str,
        system: &str,
        rom_path: &str,
        identity: &ProbeIdentity,
        screen: Option<&ScreenView<'_>>,
        spaces: &[MemorySpace<'_>],
    ) -> Result<()> {
        let text = self.header_text(backend, system, rom_path, identity, screen, spaces);
        self.file.write_all(text.as_bytes())
    }

    pub fn row_text(
        &self,
        frame: u32,
        spaces: &[MemorySpace<'_>],
        screen: Option<&ScreenView<'_>>,
    ) -> String {
        let mut out = format!("{frame}");
        for name in &self.columns {
            let crc = find_space(spaces, name).map_or(0, |s| crc32(s.data));
            let _ = write!(out, ",{crc:08x}");
        }
        let screen_crc = screen.map_or(0, |s| crc32(s.data));
        let _ = writeln!(out, ",{screen_crc:08x}");
        out
    }

    pub fn write_row(
        &mut self,
        frame: u32,
        spaces: &[MemorySpace<'_>],
        screen: Option<&ScreenView<'_>>,
    ) -> Result<()> {
        let text = self.row_text(frame, spaces, screen);
        self.file.write_all(text.as_bytes())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.file.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::ScreenFormat;

    fn space<'a>(name: &str, data: &'a [u8]) -> MemorySpace<'a> {
        MemorySpace { name: name.to_string(), data }
    }

    #[test]
    fn blob_names_are_five_digit_zero_padded() {
        assert_eq!(blob_name("mesen", "ram", 194, "bin"), "mesen_ram_f00194.bin");
        assert_eq!(blob_name("mesen", "manifest", 0, "json"), "mesen_manifest_f00000.json");
        assert_eq!(blob_name("mesen", "screen", 123456, "bin"), "mesen_screen_f123456.bin");
    }

    // The reflected 0xEDB88320 check value, so a drift in the crate is caught here
    // rather than as a mismatched dump set.
    #[test]
    fn crc32_matches_the_standard_check_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn hex_line_pads_past_the_end_with_zeroes() {
        let s = space("ram", &[0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(hex_line("RAM", &s, 0, 6), "  RAM @ $00000: DE AD BE EF 00 00");
        assert_eq!(hex_line("RAM", &s, 2, 2), "  RAM @ $00002: BE EF");
    }

    #[test]
    fn json_escape_covers_quote_backslash_newline() {
        assert_eq!(json_escape(r#"a"b\c"#), r#"a\"b\\c"#);
        assert_eq!(json_escape("a\nb"), "a\\nb");
        assert_eq!(json_escape("plain/path.nes"), "plain/path.nes");
    }

    // Pinned against a real dump from Reference/dumps/SMB3, plus the identity
    // block that the on-disk set predates.
    #[test]
    fn manifest_matches_the_cpp_byte_for_byte() {
        let spaces = [space("ram", &[0u8; 2048]), space("oam", &[0u8; 256])];
        let pixels = [0u8; 8];
        let screen = ScreenView {
            data: &pixels,
            width: 256,
            height: 240,
            format: ScreenFormat::PaletteIndex16,
        };
        let identity = ProbeIdentity {
            board: "4".to_string(),
            region: "ntsc".to_string(),
            header_trust: String::new(),
            prg_bytes: 262144,
            chr_bytes: 131072,
            save_loaded: false,
        };

        let text = manifest_json("mesen", "nes", "/roms/smb3.nes", 194, &spaces, Some(&screen), &identity);
        assert_eq!(
            text,
            "{\n  \"backend\": \"mesen\",\n  \"system\": \"nes\",\n  \"rom\": \"/roms/smb3.nes\",\n  \"frame\": 194,\n  \"identity\": { \"board\": \"4\", \"region\": \"ntsc\", \"headerTrust\": \"\", \"prg\": 262144, \"chr\": 131072, \"saveLoaded\": false },\n  \"spaces\": [\n    { \"name\": \"ram\", \"size\": 2048, \"file\": \"mesen_ram_f00194.bin\" },\n    { \"name\": \"oam\", \"size\": 256, \"file\": \"mesen_oam_f00194.bin\" }\n  ],\n  \"screen\": { \"width\": 256, \"height\": 240, \"bytes\": 8, \"format\": \"PaletteIndex16\", \"file\": \"mesen_screen_f00194.bin\" }\n}\n"
        );
    }

    // The empty-array and null-screen corners the C++ writes differently from the
    // populated ones: no newline inside the brackets, and a bare null.
    #[test]
    fn manifest_with_no_spaces_and_no_screen() {
        let text = manifest_json("libretro", "unknown", "x.bin", 0, &[], None, &ProbeIdentity::default());
        assert_eq!(
            text,
            "{\n  \"backend\": \"libretro\",\n  \"system\": \"unknown\",\n  \"rom\": \"x.bin\",\n  \"frame\": 0,\n  \"identity\": { \"board\": \"\", \"region\": \"\", \"headerTrust\": \"\", \"prg\": 0, \"chr\": 0, \"saveLoaded\": false },\n  \"spaces\": [],\n  \"screen\": null\n}\n"
        );
    }

    // An empty space is skipped in the manifest exactly as the C++ skipped a null
    // pointer or a zero size, and write_blob writes no file for it either.
    #[test]
    fn manifest_skips_empty_spaces() {
        let spaces = [space("ram", &[1, 2, 3]), space("sram", &[])];
        let text = manifest_json("m", "nes", "r", 1, &spaces, None, &ProbeIdentity::default());
        assert!(text.contains("m_ram_f00001.bin"));
        assert!(!text.contains("sram"));
    }

    #[test]
    fn signature_header_and_row() {
        let file = std::env::temp_dir().join("emusen-probe-sigtest.csv");
        let mut sig = SignatureWriter::open(&file).unwrap();
        let spaces = [space("ram", &[0u8; 4]), space("vram", b"abcd")];
        let identity = ProbeIdentity {
            region: "ntsc".to_string(),
            prg_bytes: 16,
            ..Default::default()
        };

        let header = sig.header_text("core", "nes", "/r.nes", &identity, None, &spaces);
        assert_eq!(
            header,
            "# emusen-probe-signature 1\n# backend=core\n# system=nes\n# rom=/r.nes\n# board=\n# region=ntsc\n# headerTrust=\n# prg=16\n# chr=0\n# saveLoaded=0\n# screenFormat=None\nframe,ram,vram,screen\n"
        );

        // Lowercase, zero-padded to eight, and a missing column hashes as zero -
        // which is also what a present-but-empty space hashes to.
        let row = sig.row_text(7, &spaces, None);
        assert_eq!(row, format!("7,{:08x},{:08x},{:08x}\n", crc32(&[0u8; 4]), crc32(b"abcd"), 0));

        let missing = sig.row_text(8, &spaces[..1], None);
        assert_eq!(missing, format!("8,{:08x},{:08x},{:08x}\n", crc32(&[0u8; 4]), 0, 0));

        let _ = std::fs::remove_file(&file);
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    #[test]
    fn unchanged_blobs_are_linked_not_rewritten() {
        let dir = std::env::temp_dir().join("emusen-probe-cachetest");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let d = dir.to_str().unwrap();

        let mut cache = BlobCache::default();
        let a = vec![7u8; 64];
        let b = vec![9u8; 64];
        cache.write(d, "m", "chr", 0, &a).unwrap();
        cache.write(d, "m", "chr", 1, &a).unwrap();
        cache.write(d, "m", "chr", 2, &a).unwrap();
        cache.write(d, "m", "chr", 3, &b).unwrap();
        cache.write(d, "m", "chr", 4, &b).unwrap();

        assert_eq!((cache.written, cache.linked), (2, 3), "two distinct values, three repeats");
        for frame in 0..=4 {
            let p = blob_path(d, "m", "chr", frame, "bin");
            assert!(p.is_file(), "frame {frame} must still be a readable file");
        }
        assert_eq!(std::fs::read(blob_path(d, "m", "chr", 2, "bin")).unwrap(), a);
        assert_eq!(std::fs::read(blob_path(d, "m", "chr", 4, "bin")).unwrap(), b);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
