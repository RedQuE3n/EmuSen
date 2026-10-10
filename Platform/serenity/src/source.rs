//! A `.slang` file with its includes in place, split into its two stages, and what its pragmas declared: the C#
//! `SlangSource`. See EmuSen_Serenity.md §7.2.

use crate::text::{parse_float, pragma, quoted, skip_spaces, spaces, word, word_ends};
use crate::{Error, full, read_lines, resolve};
use emusen_galaxia::dotnet_path::{Style, directory_name};
use emusen_galaxia::tree::file_exists;

/// One `#pragma parameter`: its id, what it is called, its default and its range.
#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub id: String,
    pub description: String,
    pub initial: f32,
    pub minimum: f32,
    pub maximum: f32,
    pub step: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub path: String,
    pub vertex: String,
    pub fragment: String,
    pub pass_name: Option<String>,
    pub framebuffer_format: Option<String>,
    pub parameters: Vec<Parameter>,
}

pub const MAX_INCLUDE_DEPTH: usize = 32;

/// `^\s*#\s*include\s+"([^"]+)"`.
fn include(line: &str) -> Option<&str> {
    let rest = skip_spaces(skip_spaces(line).strip_prefix('#')?).strip_prefix("include")?;
    quoted(spaces(rest)?).map(|(name, _)| name).filter(|name| !name.is_empty())
}

/// `^\s*#\s*pragma\s+include_optional\s+"([^"]+)"`.
fn optional_include(line: &str) -> Option<&str> {
    quoted(pragma(line, "include_optional")?).map(|(name, _)| name).filter(|name| !name.is_empty())
}

/// `^\s*#\s*pragma\s+stage\s+(vertex|fragment)\b`.
fn stage(line: &str) -> Option<&'static str> {
    let rest = pragma(line, "stage")?;
    ["vertex", "fragment"].into_iter().find(|name| rest.strip_prefix(name).is_some_and(word_ends))
}

/// `^\s*#\s*pragma\s+parameter\s+(\S+)\s+"([^"]*)"\s+(\S+)\s+(\S+)\s+(\S+)(?:\s+(\S+))?`.
fn parameter(line: &str) -> Option<Parameter> {
    let (id, rest) = word(pragma(line, "parameter")?)?;
    let (description, rest) = quoted(spaces(rest)?)?;
    let (initial, rest) = word(spaces(rest)?)?;
    let (minimum, rest) = word(spaces(rest)?)?;
    let (maximum, rest) = word(spaces(rest)?)?;
    let step = spaces(rest).and_then(word).map(|(step, _)| step);
    let number = |text: &str| parse_float(text).unwrap_or(0.0);
    Some(Parameter { id: id.to_string(), description: description.to_string(), initial: number(initial), minimum: number(minimum), maximum: number(maximum), step: step.map_or(0.0, number) })
}

/// Includes are textual and relative to the file that names them; an optional one that is absent is simply skipped.
fn expand(path: &str, lines: &mut Vec<String>, depth: usize) -> Result<(), Error> {
    if depth > MAX_INCLUDE_DEPTH {
        return Err(Error::InvalidData(format!("{path}: includes go deeper than {MAX_INCLUDE_DEPTH}; taken to be a loop.")));
    }
    if !file_exists(path) {
        return Err(Error::NotFound { message: format!("The shader file {path} does not exist."), file: path.to_string() });
    }
    let directory = directory_name(Style::HOST, path).unwrap_or_default();
    for line in read_lines(path)? {
        if let Some(name) = include(&line) {
            expand(&resolve(&directory, name)?, lines, depth + 1)?;
            continue;
        }
        if let Some(name) = optional_include(&line) {
            let target = resolve(&directory, name)?;
            if file_exists(&target) {
                expand(&target, lines, depth + 1)?;
            }
            continue;
        }
        lines.push(line);
    }
    Ok(())
}

pub fn load(path: &str) -> Result<Source, Error> {
    let whole = full(path)?;
    let mut lines = Vec::new();
    expand(&whole, &mut lines, 0)?;

    let (mut vertex, mut fragment) = (String::new(), String::new());
    let mut parameters: Vec<Parameter> = Vec::new();
    let (mut name, mut format, mut current) = (None, None, None);

    // A pragma line is kept as a blank so the compiler's line numbers still point at the right line of the expanded text.
    for line in &lines {
        let mut keep = line.as_str();
        if let Some(found) = stage(line) {
            current = Some(found);
            keep = "";
        } else if let Some((found, _)) = pragma(line, "name").and_then(word) {
            name = Some(found.to_string());
            keep = "";
        } else if let Some((found, _)) = pragma(line, "format").and_then(word) {
            format = Some(found.to_string());
            keep = "";
        } else if let Some(found) = parameter(line) {
            if !parameters.iter().any(|known| known.id == found.id) {
                parameters.push(found);
            }
            keep = "";
        }
        if current != Some("fragment") {
            vertex.push_str(keep);
        }
        vertex.push('\n');
        if current != Some("vertex") {
            fragment.push_str(keep);
        }
        fragment.push('\n');
    }

    if current.is_none() {
        return Err(Error::InvalidData(format!("{path} has no #pragma stage.")));
    }
    Ok(Source { path: whole, vertex, fragment, pass_name: name, framebuffer_format: format, parameters })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pragmas_are_read_as_the_expressions_read_them() {
        assert_eq!(include("#include \"a.inc\""), Some("a.inc"));
        assert_eq!(include("  #  include   \"dir/a b.inc\" // note"), Some("dir/a b.inc"));
        assert_eq!(include("#include \"\""), None);
        assert_eq!(include("#include <a.inc>"), None);
        assert_eq!(include("#include\"a.inc\""), None);
        assert_eq!(optional_include("#pragma include_optional \"x\""), Some("x"));
        assert_eq!(optional_include("#pragma include_optional x"), None);
        assert_eq!(stage("#pragma stage vertex"), Some("vertex"));
        assert_eq!(stage("#pragma stage fragment // note"), Some("fragment"));
        assert_eq!(stage("#pragma stage vertexes"), None);
        assert_eq!(stage("#pragma stage fragment_"), None);
        assert_eq!(stage("#pragma stage vertex-shader"), Some("vertex"));
        assert_eq!(stage("#pragma stage Vertex"), None);
    }

    #[test]
    fn a_parameter_has_five_or_six_parts_and_a_number_that_is_not_one_is_zero() {
        let p = parameter("#pragma parameter GAIN \"The gain\" 1.5 0 2 0.25").unwrap();
        assert_eq!((p.id.as_str(), p.description.as_str(), p.initial, p.minimum, p.maximum, p.step), ("GAIN", "The gain", 1.5, 0.0, 2.0, 0.25));
        let p = parameter("#pragma parameter a \"\" x 1 2").unwrap();
        assert_eq!((p.initial, p.minimum, p.maximum, p.step), (0.0, 1.0, 2.0, 0.0));
        let p = parameter("#pragma parameter \"q\" \"d\" 1 2 3 4 5").unwrap();
        assert_eq!((p.id.as_str(), p.step), ("\"q\"", 4.0));
        assert!(parameter("#pragma parameter a \"d\" 1 2").is_none());
        assert!(parameter("#pragma parameter a d 1 2 3").is_none());
        assert!(parameter("#pragma parameter a\"d\" 1 2 3").is_none());
    }
}
