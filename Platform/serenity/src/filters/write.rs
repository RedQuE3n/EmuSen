//! A built-in filter written as a slang preset for the chain: each pass's Vulkan GLSL given its uniform block, its
//! samplers and its two stages, the preset's text over them, and the folder named by their content that holds them.
//! See EmuSen_DevicePicture.md §3, §12, §19 and §20, and EmuSen_CRT.md §16.

use std::path::Path;
use super::number::single;
use super::{Axis, Filter, Pass, Scale, Size, Values, glsl};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// The values the runner sets at each draw, which no settings window lists.
pub const PIXEL_SCALE: &str = "emusen_pixel_scale";
pub const ORIGIN_X: &str = "emusen_pixel_origin_x";
pub const ORIGIN_Y: &str = "emusen_pixel_origin_y";
pub const CROP_LEFT: &str = "emusen_crop_left";
pub const CROP_TOP: &str = "emusen_crop_top";
pub const CROP_WIDTH: &str = "emusen_crop_width";
pub const CROP_HEIGHT: &str = "emusen_crop_height";
pub const RAW_WIDTH: &str = "emusen_raw_width";
pub const RAW_HEIGHT: &str = "emusen_raw_height";

/// The lookup image a pass reads the raw frame from.
pub const RAW_TEXTURE: &str = "raw";

fn reserved_default(id: &str) -> f32 {
    match id {
        PIXEL_SCALE | CROP_WIDTH | CROP_HEIGHT => 1.0,
        _ => 0.0,
    }
}

/// A black pixel as a PNG, the raw frame's stand-in until the runner gives the chain one: the bytes Skia encodes.
pub const BLACK_PIXEL: [u8; 86] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x04, 0x73, 0x42, 0x49, 0x54, 0x08, 0x08, 0x08, 0x08, 0x7c, 0x08, 0x64, 0x88, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x08, 0x99, 0x63, 0x60, 0x60, 0x60, 0xf8,
    0x0f, 0x00, 0x01, 0x04, 0x01, 0x00, 0x7d, 0xb2, 0xc8, 0xdf, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

/// One translated pass: its slang text and what the preset says of it.
#[derive(Clone, Debug, PartialEq)]
pub struct Written {
    pub file_name: String,
    pub text: String,
    pub alias: Option<String>,
    pub width: Size,
    pub height: Size,
    pub float: bool,
    pub linear_sampler: bool,
}

fn alias(name: &str) -> String {
    format!("emusen_{name}")
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// `\buniform[ \t]+(\w+)[ \t]+(\w+)[ \t]*;` over a text: each match's span, type and name, left to right.
fn uniforms(text: &str) -> Vec<(usize, usize, String, String)> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut found = Vec::new();
    let mut i = 0;
    let word_run = |from: usize| -> usize {
        let mut k = from;
        while k < chars.len() && is_word(chars[k].1) {
            k += 1;
        }
        k
    };
    let blank_run = |from: usize| -> usize {
        let mut k = from;
        while k < chars.len() && matches!(chars[k].1, ' ' | '\t') {
            k += 1;
        }
        k
    };
    while i < chars.len() {
        let at = chars[i].0;
        let boundary = i == 0 || !is_word(chars[i - 1].1);
        if boundary && text[at..].starts_with("uniform") {
            let k = i + 7;
            let a = blank_run(k);
            let b = word_run(a);
            if a > k && b > a {
                let c = blank_run(b);
                let d = word_run(c);
                if c > b && d > c {
                    let e = blank_run(d);
                    if e < chars.len() && chars[e].1 == ';' {
                        let end = chars[e].0 + 1;
                        let ty: String = chars[a..b].iter().map(|x| x.1).collect();
                        let name: String = chars[c..d].iter().map(|x| x.1).collect();
                        found.push((at, end, ty, name));
                        i = e + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    found
}

fn children_of(glsl: &str) -> Vec<String> {
    uniforms(glsl).into_iter().filter(|u| u.2 == "sampler2D").map(|u| u.3).collect()
}

fn reads_linearly(pass: &Pass, child: &str) -> bool {
    if child == "source" || child == "image" { pass.linear_source } else { pass.linear.iter().any(|l| l == child) }
}

fn index_of_name(passes: &[Pass], name: &str, before: usize) -> Option<usize> {
    (0..before).find(|&k| passes[k].name.as_deref() == Some(name))
}

/// The passes a filter draws on a console, reducing pass first; `nudge` shifts every sampling point for the tie test.
pub fn translate(filter: &Filter, values: &Values, reduce_by: i32, nudge: (f32, f32)) -> Result<Vec<Written>, String> {
    let mut all = Vec::new();
    let reduced = reduce_by > 1;
    if reduced {
        let mut p = Pass::new(glsl::REDUCE.replace("uniform float by;", &format!("const float by = {reduce_by}.0;")), Scale::Source);
        p.name = Some("reduced".to_string());
        p.width = Some(Size::of(Axis::Source, 1.0 / reduce_by as f32));
        p.height = Some(Size::of(Axis::Source, 1.0 / reduce_by as f32));
        all.push(p);
    }
    all.extend(filter.passes_for(values));

    // By libretro's rule an output is sampled with the filter of the pass after it, the frame with the first pass's.
    let mut linear_outputs = BTreeSet::new();
    let mut linear_original = false;
    for j in 0..all.len() {
        for child in children_of(&all[j].glsl) {
            if !reads_linearly(&all[j], &child) {
                continue;
            }
            if child == "source" {
                if j == 0 {
                    linear_original = true;
                } else {
                    linear_outputs.insert(j - 1);
                }
            } else if child == "image" || (child == "original" && !reduced) {
                linear_original = true;
            } else if let Some(k) = index_of_name(&all, if child == "original" { "reduced" } else { &child }, j) {
                linear_outputs.insert(k);
            }
        }
    }

    let mut result = Vec::new();
    for j in 0..all.len() {
        let pass = &all[j];
        let linear_sampler = if j == 0 { linear_original } else { linear_outputs.contains(&(j - 1)) };
        let axis = if pass.scale == Scale::Source { Axis::Source } else { Axis::Viewport };
        let width = pass.width.unwrap_or(Size::of(axis, 1.0));
        let height = pass.height.unwrap_or(Size::of(axis, 1.0));
        let text = pass_text(filter, &all, j, values, reduced && j > 0, nudge)?;
        result.push(Written { file_name: format!("pass{j}.slang"), text, alias: pass.name.as_deref().map(alias), width, height, float: pass.float, linear_sampler });
    }
    Ok(result)
}

struct Child {
    name: String,
    sampler: String,
    linear: bool,
    stretched: bool,
}

fn pass_text(filter: &Filter, passes: &[Pass], index: usize, values: &Values, original_is_reduced: bool, nudge: (f32, f32)) -> Result<String, String> {
    let pass = &passes[index];
    let mut children: Vec<Child> = Vec::new();
    let mut globals = String::new();
    let mut assignments = String::new();
    let mut vectors: BTreeSet<String> = BTreeSet::from(["MVP".to_string()]);
    let mut floats: Vec<String> = Vec::new();
    let mut pragmas = String::new();
    let mut frame_count = false;

    let float = |id: &str, initial: f32, floats: &mut Vec<String>, pragmas: &mut String| {
        if floats.iter().any(|f| f == id) {
            return;
        }
        floats.push(id.to_string());
        pragmas.push_str(&format!("#pragma parameter {id} \"{id}\" {} {} {} 0\n", single(initial), single(initial - 1e6), single(initial + 1e6)));
    };
    let mut semantic = |ty: &str, name: &str, value: &str, needs: &[&str], vectors: &mut BTreeSet<String>| {
        globals.push_str(&format!("{ty} {name};\n"));
        assignments.push_str(&format!("    {name} = {value};\n"));
        for need in needs {
            vectors.insert(need.to_string());
        }
    };

    let found = uniforms(&pass.glsl);
    for (_, _, ty, name) in &found {
        let name = name.as_str();
        if ty == "sampler2D" {
            let history = name.strip_prefix("history").and_then(|rest| rest.parse::<i32>().ok().filter(|_| rest.bytes().all(|b| b.is_ascii_digit()))).filter(|back| *back >= 1);
            let sampler = match name {
                "source" => "Source".to_string(),
                "original" => {
                    if original_is_reduced {
                        alias("reduced")
                    } else {
                        "Original".to_string()
                    }
                }
                "raw" => "raw".to_string(),
                "image" => "Original".to_string(),
                "feedback" if pass.feedback => match &pass.name {
                    Some(own) => alias(own) + "Feedback",
                    None => format!("PassFeedback{index}"),
                },
                _ if history.is_some() => format!("OriginalHistory{}", history.unwrap()),
                _ if index_of_name(passes, name, index).is_some() => alias(name),
                _ => return Err(format!("The '{}' filter asks for a child called '{name}', which no pass provides.", filter.name)),
            };
            let linear = if name == "image" { pass.linear_source } else { reads_linearly(pass, name) && name != "raw" && !name.starts_with("history") };
            children.push(Child { name: name.to_string(), sampler, linear, stretched: name == "image" });
            if name == "image" {
                vectors.insert("OriginalSize".to_string());
                vectors.insert("OutputSize".to_string());
            }
            continue;
        }

        match name {
            "inputSize" => semantic("vec2", name, "SourceSize.xy", &["SourceSize"], &mut vectors),
            "originalSize" => {
                if original_is_reduced {
                    let size = alias("reduced") + "Size";
                    semantic("vec2", name, &format!("{size}.xy"), &[&size], &mut vectors)
                } else {
                    semantic("vec2", name, "OriginalSize.xy", &["OriginalSize"], &mut vectors)
                }
            }
            "outputSize" => semantic("vec2", name, "OutputSize.xy", &["OutputSize"], &mut vectors),
            "viewportSize" => semantic("vec2", name, "FinalViewportSize.xy", &["FinalViewportSize"], &mut vectors),
            "frameCount" => {
                semantic("float", name, "float(FrameCount)", &[], &mut vectors);
                frame_count = true;
            }
            "pixelScale" => {
                semantic("float", name, PIXEL_SCALE, &[], &mut vectors);
                float(PIXEL_SCALE, 1.0, &mut floats, &mut pragmas);
            }
            "pixelOrigin" => {
                semantic("vec2", name, &format!("vec2({ORIGIN_X}, {ORIGIN_Y})"), &[], &mut vectors);
                float(ORIGIN_X, 0.0, &mut floats, &mut pragmas);
                float(ORIGIN_Y, 0.0, &mut floats, &mut pragmas);
            }
            "crop" => {
                semantic("vec4", name, &format!("vec4({CROP_LEFT}, {CROP_TOP}, {CROP_WIDTH}, {CROP_HEIGHT})"), &[], &mut vectors);
                float(CROP_LEFT, 0.0, &mut floats, &mut pragmas);
                float(CROP_TOP, 0.0, &mut floats, &mut pragmas);
                float(CROP_WIDTH, 1.0, &mut floats, &mut pragmas);
                float(CROP_HEIGHT, 1.0, &mut floats, &mut pragmas);
            }
            "rawSize" => {
                semantic("vec2", name, &format!("vec2({RAW_WIDTH}, {RAW_HEIGHT})"), &[], &mut vectors);
                float(RAW_WIDTH, 0.0, &mut floats, &mut pragmas);
                float(RAW_HEIGHT, 0.0, &mut floats, &mut pragmas);
            }
            size if size.ends_with("Size") && index_of_name(passes, &size[..size.len() - 4], index).is_some() => {
                let named = alias(&size[..size.len() - 4]) + "Size";
                semantic("vec2", size, &format!("{named}.xy"), &[&named], &mut vectors);
            }
            id if values.contains_key(id) => float(id, values[id], &mut floats, &mut pragmas),
            _ => return Err(format!("The '{}' filter asks for a uniform called '{name}', which no pass provides.", filter.name)),
        }
    }

    let mut body = String::new();
    let mut last = 0;
    for (start, end, _, _) in &found {
        body.push_str(&pass.glsl[last..*start]);
        last = *end;
    }
    body.push_str(&pass.glsl[last..]);

    let mut block = String::from("layout(std140, set = 0, binding = 0) uniform UBO\n{\n    mat4 MVP;\n");
    for vector in vectors.iter().filter(|v| *v != "MVP") {
        block.push_str(&format!("    vec4 {vector};\n"));
    }
    if frame_count {
        block.push_str("    uint FrameCount;\n");
    }
    for id in &floats {
        block.push_str(&format!("    float {id};\n"));
    }
    block.push_str("};\n");

    let mut samplers = String::new();
    for (binding, child) in children.iter().enumerate() {
        let (n, s) = (&child.name, &child.sampler);
        samplers.push_str(&format!("layout(set = 0, binding = {}) uniform sampler2D {s};\n", binding + 1));
        // The simple effects' image is the frame stretched over the rectangle.
        samplers.push_str(&match (child.stretched, child.linear) {
            (true, true) => format!("vec4 {n}_eval(vec2 p) {{ return texture({s}, p / OutputSize.xy); }}\n"),
            (true, false) => format!("vec4 {n}_eval(vec2 p) {{ ivec2 s = textureSize({s}, 0); return texelFetch({s}, clamp(ivec2(floor(p * OriginalSize.xy / OutputSize.xy)), ivec2(0), s - 1), 0); }}\n"),
            (false, true) => format!("vec4 {n}_eval(vec2 p) {{ return texture({s}, p / vec2(textureSize({s}, 0))); }}\n"),
            (false, false) => format!("vec4 {n}_eval(vec2 p) {{ ivec2 s = textureSize({s}, 0); return texelFetch({s}, clamp(ivec2(floor(p)), ivec2(0), s - 1), 0); }}\n"),
        });
    }

    let write = if pass.float { "FragColor = c;" } else { "FragColor = vec4(c.rgb, 1.0);" };
    let nudged = if nudge.0 == 0.0 && nudge.1 == 0.0 { String::new() } else { format!(" + vec2({}, {})", single(nudge.0), single(nudge.1)) };
    let named = pass.name.as_deref().map_or(String::new(), |n| format!("#pragma name {}\n", alias(n)));
    Ok(format!("#version 450\n{pragmas}{named}{block}{}{samplers}{globals}{body}\nvoid main()\n{{\n{assignments}    vec4 c = filter_main(gl_FragCoord.xy{nudged});\n    {write}\n}}\n", glsl::STAGES))
}

fn axis_lines(text: &mut String, axis: &str, index: usize, size: Size) {
    let kind = match size.axis {
        Axis::Source => "source",
        Axis::Viewport => "viewport",
        Axis::Fixed => "absolute",
    };
    text.push_str(&format!("scale_type_{axis}{index} = {kind}\nscale_{axis}{index} = {}\n", single(size.factor)));
}

/// The preset's text over its passes, and the values its parameters start at.
pub fn preset_text(passes: &[Written], values: &Values) -> String {
    let mut text = format!("shaders = {}\n", passes.len());
    for (i, pass) in passes.iter().enumerate() {
        text.push_str(&format!("shader{i} = {}\n", pass.file_name));
        if let Some(alias) = &pass.alias {
            text.push_str(&format!("alias{i} = {alias}\n"));
        }
        text.push_str(&format!("filter_linear{i} = {}\n", pass.linear_sampler));
        text.push_str(&format!("wrap_mode{i} = clamp_to_edge\n"));
        axis_lines(&mut text, "x", i, pass.width);
        axis_lines(&mut text, "y", i, pass.height);
        text.push_str(&format!("float_framebuffer{i} = {}\n", pass.float));
        text.push_str(&format!("frame_count_mod{i} = 65536\n"));
    }
    if passes.iter().any(|p| p.text.contains("uniform sampler2D raw;")) {
        text.push_str(&format!("textures = {RAW_TEXTURE}\n{RAW_TEXTURE} = {RAW_TEXTURE}.png\n{RAW_TEXTURE}_linear = false\n{RAW_TEXTURE}_wrap_mode = clamp_to_edge\n"));
    }
    for (id, value) in values {
        text.push_str(&format!("{id} = {}\n", single(*value)));
    }
    text
}

/// The values a filter's preset is written with on a console: each setting's chosen value or its default there, then the console's constants.
pub fn values_for(filter: &Filter, console: Option<&str>, chosen: Option<&Values>) -> Values {
    filter.values_for(console, chosen)
}

/// The preset's text and its passes, and the folder name their content gives.
pub fn compose(filter: &Filter, console: Option<&str>, chosen: Option<&Values>, reduce_by: i32, nudge: (f32, f32)) -> Result<(String, Vec<Written>, String), String> {
    let values = values_for(filter, console, chosen);
    let passes = translate(filter, &values, reduce_by, nudge)?;
    let mut used = BTreeMap::new();
    for pass in &passes {
        for line in pass.text.split('\n') {
            if let Some(rest) = line.strip_prefix("#pragma parameter ") {
                let id: String = rest.chars().take_while(|c| is_word(*c)).collect();
                if !id.is_empty() && rest[id.len()..].starts_with(' ') {
                    let value = values.get(&id).copied().unwrap_or_else(|| reserved_default(&id));
                    used.insert(id, value);
                }
            }
        }
    }
    let preset = preset_text(&passes, &used);
    let mut all = preset.clone();
    for pass in &passes {
        all.push('\0');
        all.push_str(&pass.file_name);
        all.push('\0');
        all.push_str(&pass.text);
    }
    let folder: String = Sha256::digest(all.as_bytes()).iter().take(12).map(|b| format!("{b:02x}")).collect();
    Ok((preset, passes, folder))
}

/// Why a preset was not written: a pass asking for what no pass provides, or the file system's words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    Pass(String),
    Io(String),
}

/// Writes the preset into a folder of `root` named by its content and returns the preset's path; the same content is written once.
pub fn write(root: &str, filter: &Filter, console: Option<&str>, chosen: Option<&Values>, reduce_by: i32, nudge: (f32, f32)) -> Result<String, Refusal> {
    let (preset, passes, name) = compose(filter, console, chosen, reduce_by, nudge).map_err(Refusal::Pass)?;
    let folder = Path::new(root).join(&name);
    let path = folder.join("filter.slangp");
    if path.is_file() {
        return Ok(path.to_string_lossy().into_owned());
    }
    let failed = |e: std::io::Error| Refusal::Io(format!("{}: {e}", path.display()));
    std::fs::create_dir_all(&folder).map_err(failed)?;
    for pass in &passes {
        std::fs::write(folder.join(&pass.file_name), &pass.text).map_err(failed)?;
    }
    if preset.contains(&format!("textures = {RAW_TEXTURE}")) {
        std::fs::write(folder.join(format!("{RAW_TEXTURE}.png")), BLACK_PIXEL).map_err(failed)?;
    }
    let temporary = emusen_galaxia::atomic::beside(&path, &format!(".{}.tmp", std::process::id()));
    std::fs::write(&temporary, &preset).map_err(failed)?;
    std::fs::rename(&temporary, &path).map_err(failed)?;
    Ok(path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniforms_are_found_as_the_pattern_finds_them() {
        let found = uniforms("uniform vec2 a;\nxuniform float b;\n  uniform\tsampler2D  c ;\nuniform float;\nuniform mat4 d e;");
        let names: Vec<_> = found.iter().map(|f| (f.2.as_str(), f.3.as_str())).collect();
        assert_eq!(names, vec![("vec2", "a"), ("sampler2D", "c")]);
    }

    #[test]
    fn every_filter_writes_on_every_console_it_suits() {
        for filter in super::super::catalog() {
            let values = filter.values_for(None, None);
            assert!(translate(filter, &values, 1, (0.0, 0.0)).is_ok(), "{}", filter.key);
        }
    }
}
