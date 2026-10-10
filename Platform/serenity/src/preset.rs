//! A `.slangp` preset read through its `#reference` chain, each path resolved against the file that wrote it: the C#
//! `SlangPreset`. See EmuSen_Serenity.md §7.1.

use crate::text::{self, is_space, parse_float, parse_int, skip_spaces};
use crate::{Error, full, read_lines, resolve};
use emusen_galaxia::dotnet_path::{Style, directory_name, equals_ignore_case};
use emusen_galaxia::dotnet_text::lower_invariant;
use emusen_galaxia::tree::file_exists;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleType {
    Source,
    Viewport,
    Absolute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wrap {
    ClampToBorder,
    ClampToEdge,
    Repeat,
    MirroredRepeat,
}

/// One pass of a preset, as its keys say; an unscaled pass is source 1.0, or the viewport when it is the last.
#[derive(Clone, Debug, PartialEq)]
pub struct PassSpec {
    pub shader_path: String,
    pub alias: Option<String>,
    pub filter_linear: bool,
    pub wrap: Wrap,
    pub scale_type_x: ScaleType,
    pub scale_type_y: ScaleType,
    pub scale_x: f32,
    pub scale_y: f32,
    pub float_framebuffer: bool,
    pub srgb_framebuffer: bool,
    pub mipmap_input: bool,
    pub frame_count_mod: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureSpec {
    pub name: String,
    pub path: String,
    pub linear: bool,
    pub wrap: Wrap,
    pub mipmap: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Preset {
    pub path: String,
    pub passes: Vec<PassSpec>,
    pub textures: Vec<TextureSpec>,
    /// Every key that is not a pass's, a texture's or the preset's own, which RetroArch reads as a parameter's value, in the order each was first written.
    pub parameters: Vec<(String, f32)>,
}

/// How deep a chain of references may go before it is taken to be a loop.
pub const MAX_REFERENCE_DEPTH: usize = 16;

const PASS_KEYS: [&str; 16] = [
    "shader", "alias", "filter_linear", "wrap_mode", "scale_type", "scale_type_x", "scale_type_y", "scale", "scale_x", "scale_y", "float_framebuffer", "srgb_framebuffer",
    "mipmap_input", "frame_count_mod", "rgb10_framebuffer", "feedback_pass",
];

/// The keys as they were first written, each with its last value and the directory of the file that gave it.
#[derive(Default)]
struct Values {
    entries: Vec<(String, String, String)>,
}

impl Values {
    fn set(&mut self, key: String, value: String, directory: &str) {
        match self.entries.iter_mut().find(|entry| entry.0 == key) {
            Some(entry) => (entry.1, entry.2) = (value, directory.to_string()),
            None => self.entries.push((key, value, directory.to_string())),
        }
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.entries.iter().find(|entry| entry.0 == key).map(|entry| entry.1.as_str())
    }

    fn resolve(&self, key: &str) -> Result<String, Error> {
        let entry = self.entries.iter().find(|entry| entry.0 == key).expect("asked only for a key that is there");
        resolve(&entry.2, &entry.1)
    }
}

/// `^\s*([A-Za-z0-9_\-\.]+)\s*=\s*(.*)$`: the key and everything after the equals sign and its spaces.
fn pair(line: &str) -> Option<(&str, &str)> {
    let rest = skip_spaces(line);
    let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))).unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    let value = skip_spaces(&rest[end..]).strip_prefix('=')?;
    Some((&rest[..end], skip_spaces(value)))
}

/// `^\s*#reference\s+"?([^"]+)"?`: the file referred to, trimmed. The white space may give its last character back to
/// the name, as the expression does when nothing else follows it, and that name trims to nothing.
fn reference(line: &str) -> Option<String> {
    let after = skip_spaces(line).strip_prefix("#reference")?;
    let rest = text::spaces(after)?;
    let run = |from: &str| from.split('"').next().filter(|name| !name.is_empty()).map(|name| name.trim_matches(is_space).to_string());
    if let Some(name) = rest.strip_prefix('"').and_then(run).or_else(|| if rest.starts_with('"') { None } else { run(rest) }) {
        return Some(name);
    }
    // Two or more spaces and then a quote or the line's end: the last space is the name.
    (after.chars().take_while(|&c| is_space(c)).count() >= 2).then(String::new)
}

/// A quoted value runs to its closing quote, or, as RetroArch tolerates, to a comment or the line's end when it has none.
fn value(text: &str) -> String {
    let text = text.trim_matches(is_space);
    if let Some(inner) = text.strip_prefix('"') {
        if let Some(close) = inner.find('"') {
            return inner[..close].to_string();
        }
        return match inner.find('#') {
            Some(comment) => inner[..comment].trim_matches(is_space).to_string(),
            None => inner.trim_matches(is_space).to_string(),
        };
    }
    match text.find(['#', ' ', '\t']) {
        Some(end) => text[..end].to_string(),
        None => text.to_string(),
    }
}

fn boolean(value: Option<&str>) -> bool {
    value.is_some_and(|value| equals_ignore_case(value, "true") || value == "1")
}

fn float(value: Option<&str>, fallback: f32) -> f32 {
    value.and_then(parse_float).unwrap_or(fallback)
}

fn lower(text: &str) -> String {
    text.chars().map(lower_invariant).collect()
}

fn wrap(name: Option<&str>) -> Wrap {
    match name.map(lower).as_deref() {
        Some("clamp_to_edge") => Wrap::ClampToEdge,
        Some("repeat") => Wrap::Repeat,
        Some("mirrored_repeat") => Wrap::MirroredRepeat,
        _ => Wrap::ClampToBorder,
    }
}

/// Whether a key is one of a pass's: one of the sixteen names and then digits to its end.
fn is_pass_key(key: &str) -> bool {
    PASS_KEYS.iter().any(|name| key.strip_prefix(name).is_some_and(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit())))
}

/// The referenced files first, then this one's own keys over them, as RetroArch applies a `#reference`.
fn read(path: &str, values: &mut Values, depth: usize) -> Result<(), Error> {
    if depth > MAX_REFERENCE_DEPTH {
        return Err(Error::InvalidData(format!("{path}: references go deeper than {MAX_REFERENCE_DEPTH}; taken to be a loop.")));
    }
    if !file_exists(path) {
        return Err(Error::NotFound { message: format!("The preset {path} does not exist."), file: path.to_string() });
    }
    let directory = directory_name(Style::HOST, path).unwrap_or_default();
    let mut own = Vec::new();
    for raw in read_lines(path)? {
        if let Some(target) = reference(&raw) {
            read(&resolve(&directory, &target)?, values, depth + 1)?;
            continue;
        }
        let line = raw.trim_start_matches(is_space);
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some((key, rest)) = pair(&raw) {
            own.push((key.to_string(), value(rest)));
        }
    }
    for (key, value) in own {
        values.set(key, value, &directory);
    }
    Ok(())
}

pub fn load(path: &str) -> Result<Preset, Error> {
    let mut values = Values::default();
    let whole = full(path)?;
    read(&whole, &mut values, 0)?;

    let count = match values.get("shaders").and_then(parse_int) {
        Some(count) if count >= 1 => count,
        _ => return Err(Error::InvalidData(format!("{path} says no number of shaders."))),
    };

    let mut passes = Vec::new();
    for i in 0..count {
        if values.get(&format!("shader{i}")).is_none() {
            return Err(Error::InvalidData(format!("{path} has no shader{i}.")));
        }
        let both = values.get(&format!("scale_type{i}"));
        let x = values.get(&format!("scale_type_x{i}")).or(both);
        let y = values.get(&format!("scale_type_y{i}")).or(both);
        let last = i == count - 1;
        let kind = |name: Option<&str>| match name {
            None => Ok(if last { ScaleType::Viewport } else { ScaleType::Source }),
            Some(name) => match lower(name).as_str() {
                "source" => Ok(ScaleType::Source),
                "viewport" => Ok(ScaleType::Viewport),
                "absolute" => Ok(ScaleType::Absolute),
                _ => Err(Error::InvalidData(format!("{path}: unknown scale type {name}."))),
            },
        };
        let scale = |axis: &str| float(values.get(&format!("scale_{axis}{i}")).or_else(|| values.get(&format!("scale{i}"))), 1.0);
        passes.push(PassSpec {
            shader_path: values.resolve(&format!("shader{i}"))?,
            alias: values.get(&format!("alias{i}")).map(str::to_string),
            filter_linear: boolean(values.get(&format!("filter_linear{i}"))),
            wrap: wrap(values.get(&format!("wrap_mode{i}"))),
            scale_type_x: kind(x)?,
            scale_type_y: kind(y)?,
            scale_x: scale("x"),
            scale_y: scale("y"),
            float_framebuffer: boolean(values.get(&format!("float_framebuffer{i}"))),
            srgb_framebuffer: boolean(values.get(&format!("srgb_framebuffer{i}"))),
            mipmap_input: boolean(values.get(&format!("mipmap_input{i}"))),
            frame_count_mod: float(values.get(&format!("frame_count_mod{i}")), 0.0) as i32,
        });
    }

    let names: Vec<String> = values.get("textures").unwrap_or("").split(';').map(|name| name.trim_matches(is_space).to_string()).filter(|name| !name.is_empty()).collect();
    let mut textures = Vec::new();
    for name in &names {
        if values.get(name).is_none() {
            return Err(Error::InvalidData(format!("{path} names a texture {name} with no file.")));
        }
        textures.push(TextureSpec {
            name: name.clone(),
            path: values.resolve(name)?,
            linear: boolean(values.get(&format!("{name}_linear"))),
            wrap: wrap(values.get(&format!("{name}_wrap_mode"))),
            mipmap: boolean(values.get(&format!("{name}_mipmap"))),
        });
    }

    let reserved = |key: &str| {
        matches!(key, "shaders" | "textures" | "parameters")
            || names.iter().any(|name| key.strip_prefix(name.as_str()).is_some_and(|suffix| matches!(suffix, "" | "_linear" | "_wrap_mode" | "_mipmap")))
    };
    let mut parameters: Vec<(String, f32)> = Vec::new();
    for (key, value, _) in &values.entries {
        if reserved(key) || is_pass_key(key) {
            continue;
        }
        if let Some(number) = parse_float(value) {
            parameters.push((key.clone(), number));
        }
    }

    Ok(Preset { path: whole, passes, textures, parameters })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_is_a_key_and_a_value_and_a_value_ends_where_retroarch_ends_it() {
        assert_eq!(pair("  shaders = 3"), Some(("shaders", "3")));
        assert_eq!(pair("a.b-c_d=\"x y\" # note"), Some(("a.b-c_d", "\"x y\" # note")));
        assert_eq!(pair("= 3"), None);
        assert_eq!(pair("key value"), None);
        assert_eq!(pair("ké = 1"), None);
        assert_eq!(value("\"x y\" # note"), "x y");
        assert_eq!(value("\"open # note"), "open");
        assert_eq!(value("\"# a comment at once"), "");
        assert_eq!(value("\"open  "), "open");
        assert_eq!(value("plain#note"), "plain");
        assert_eq!(value("two words"), "two");
        assert_eq!(value("tab\tbed"), "tab");
        assert_eq!(value("nb\u{A0}sp"), "nb\u{A0}sp");
        assert_eq!(value(""), "");
    }

    #[test]
    fn a_reference_is_read_as_the_expression_reads_it() {
        assert_eq!(reference("#reference \"a/b.slangp\"").as_deref(), Some("a/b.slangp"));
        assert_eq!(reference("  #reference   a b.slangp  ").as_deref(), Some("a b.slangp"));
        assert_eq!(reference("#reference \"open").as_deref(), Some("open"));
        assert_eq!(reference("#reference a\"b").as_deref(), Some("a"));
        assert_eq!(reference("#reference\"a\""), None);
        assert_eq!(reference("#reference "), None);
        assert_eq!(reference("#reference  ").as_deref(), Some(""));
        assert_eq!(reference("#reference \""), None);
        assert_eq!(reference("#reference  \"").as_deref(), Some(""));
        assert_eq!(reference("#reference \"\"x"), None);
        assert_eq!(reference("#reference  \"\"x").as_deref(), Some(""));
        assert_eq!(reference("# reference a"), None);
        assert_eq!(reference("x #reference a"), None);
    }

    #[test]
    fn a_pass_key_is_a_name_and_digits() {
        assert!(is_pass_key("shader0") && is_pass_key("scale_type_x12") && is_pass_key("feedback_pass3") && is_pass_key("scale7"));
        assert!(!is_pass_key("shader") && !is_pass_key("shaders") && !is_pass_key("scale_type_z0") && !is_pass_key("shader0a") && !is_pass_key("xshader0"));
    }

    #[test]
    fn a_boolean_a_wrap_and_a_float_fall_back_as_the_csharp_does() {
        assert!(boolean(Some("TRUE")) && boolean(Some("1")) && !boolean(Some("yes")) && !boolean(Some(" true")) && !boolean(None));
        assert_eq!(wrap(Some("REPEAT")), Wrap::Repeat);
        assert_eq!(wrap(Some("Mirrored_Repeat")), Wrap::MirroredRepeat);
        assert_eq!(wrap(Some("clamp_to_border")), Wrap::ClampToBorder);
        assert_eq!(wrap(Some("nonsense")), Wrap::ClampToBorder);
        assert_eq!(float(Some("x"), 1.0), 1.0);
        assert_eq!(float(Some("2.5"), 1.0), 2.5);
        assert_eq!(float(Some("NaN"), 0.0) as i32, 0);
        assert_eq!(float(Some("1e20"), 0.0) as i32, i32::MAX);
    }
}
