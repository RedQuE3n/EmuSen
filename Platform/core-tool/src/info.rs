//! A libretro core information file (`<stem>_libretro.info`, libretro-core-info's `00_example_libretro.info`): `key =
//! "value"` lines, `#` comments, read as plain data with every value bounded (EmuSen_Cores.md §4.2).

/// The longest value kept; a longer one is cut there, since nothing a window shows needs more.
const LONGEST: usize = 4096;

/// One firmware file a core names (`firmwareN_desc`, `_path`, `_opt`); the player's own, never fetched (EmuSen_Cores.md §4.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Firmware {
    pub desc: String,
    pub path: String,
    pub optional: bool,
}

/// A core's information, keyed on its file stem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreInfo {
    pub stem: String,
    fields: Vec<(String, String)>,
}

fn value_of(raw: &str) -> String {
    let raw = raw.trim();
    let text = match raw.strip_prefix('"') {
        Some(rest) => rest.find('"').map_or(rest, |end| &rest[..end]),
        None => raw.split('#').next().unwrap_or("").trim(),
    };
    text.chars().take(LONGEST).collect()
}

impl CoreInfo {
    /// The text of a `.info` read; a line that is not `key = value` is skipped, and a key given twice keeps its first.
    pub fn parse(stem: &str, text: &str) -> CoreInfo {
        let mut fields: Vec<(String, String)> = Vec::new();
        for line in text.lines() {
            let line = line.trim_start_matches('\u{FEFF}').trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else { continue };
            let key = key.trim();
            if key.is_empty() || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') || fields.iter().any(|(k, _)| k == key) {
                continue;
            }
            fields.push((key.to_string(), value_of(value)));
        }
        CoreInfo { stem: stem.to_string(), fields }
    }

    /// A key's value, none where it is absent or blank.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str()).filter(|v| !v.is_empty())
    }

    /// A `|`-separated value split, each part trimmed, blanks dropped.
    pub fn list(&self, key: &str) -> Vec<String> {
        self.get(key).map(|v| v.split('|').map(str::trim).filter(|p| !p.is_empty()).map(str::to_string).collect()).unwrap_or_default()
    }

    /// A `"true"` or `"false"` value; absent is false.
    pub fn flag(&self, key: &str) -> bool {
        self.get(key).is_some_and(|v| v.eq_ignore_ascii_case("true"))
    }

    /// `display_name`, else the stem.
    pub fn name(&self) -> &str {
        self.get("display_name").unwrap_or(&self.stem)
    }

    /// The firmware files `firmware_count` names, in order, those without a path left out.
    pub fn firmware(&self) -> Vec<Firmware> {
        let count = self.get("firmware_count").and_then(|c| c.parse::<usize>().ok()).unwrap_or(0).min(64);
        (0..count)
            .filter_map(|n| {
                let path = self.get(&format!("firmware{n}_path"))?.to_string();
                Some(Firmware { desc: self.get(&format!("firmware{n}_desc")).unwrap_or(&path).to_string(), path, optional: self.flag(&format!("firmware{n}_opt")) })
            })
            .collect()
    }

    /// Why EmuSen's host cannot run the core from its information alone, or none (EmuSen_Cores.md §4.6 kind 1): a
    /// `required_hw_api` whose every choice is OpenGL, OpenGL ES or Direct3D, since the host renders through software and
    /// Vulkan only.
    pub fn unsupported(&self) -> Option<String> {
        let apis = self.list("required_hw_api");
        let gl_only = !apis.is_empty()
            && apis.iter().all(|a| {
                let a = a.to_ascii_lowercase();
                a.starts_with("opengl") || a.starts_with("gles") || a.starts_with("direct3d") || a.starts_with("d3d")
            });
        gl_only.then(|| format!("needs {}, and EmuSen's host renders through software and Vulkan only", apis.join(" or ")))
    }
}
