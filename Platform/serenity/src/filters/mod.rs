//! The built-in screen filters and the presets they are written as: each filter's passes in Vulkan GLSL, made from its
//! parameters' values where the filter is computed (the CRT), the uniforms a player may set, the consoles it suits, and
//! each console's constants. Any front end, and DianaSh, writes the same preset from the same choices. See
//! EmuSen_Serenity.md §3 and §3.13, EmuSen_DevicePicture.md §3 and §20, and EmuSen_CRT.md.

pub mod crt;
pub mod glsl;
pub mod number;
pub mod panels;
pub mod screens;
pub mod write;

use crate::source::Parameter;
use std::collections::BTreeMap;

/// Values by parameter id.
pub type Values = BTreeMap<String, f32>;

/// How big a pass draws: the game's own picture, or the rectangle it is shown in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scale {
    Source,
    Viewport,
}

/// What one axis of a pass is measured against: the frame, the rectangle shown, or a count of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Source,
    Viewport,
    Fixed,
}

/// One axis of a pass's size: the frame's or the rectangle's times a factor, or a fixed count.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub axis: Axis,
    pub factor: f32,
}

impl Size {
    pub fn of(axis: Axis, factor: f32) -> Size {
        Size { axis, factor }
    }
}

/// One pass in Vulkan GLSL, and what the preset says of it.
#[derive(Clone, Debug, PartialEq)]
pub struct Pass {
    pub glsl: String,
    pub scale: Scale,
    /// Reads "source" (or "image") linearly.
    pub linear_source: bool,
    /// How many earlier frames it reads, as history1 to historyN.
    pub history: i32,
    pub width: Option<Size>,
    pub height: Option<Size>,
    /// Drawn into a half-float surface.
    pub float: bool,
    /// Reads its own output of the previous frame as "feedback".
    pub feedback: bool,
    /// The child name later passes read this pass's output by.
    pub name: Option<String>,
    /// Children besides "source" that are sampled linearly.
    pub linear: Vec<String>,
}

impl Pass {
    pub fn new(glsl: String, scale: Scale) -> Pass {
        Pass { glsl, scale, linear_source: false, history: 0, width: None, height: None, float: false, feedback: false, name: None, linear: Vec::new() }
    }
}

/// A uniform a player may set, and the names of its values when it is one of a few choices.
#[derive(Clone, Debug, PartialEq)]
pub struct Setting {
    pub parameter: Parameter,
    pub choices: Option<Vec<String>>,
}

pub(crate) fn setting(id: &str, description: &str, initial: f32, minimum: f32, maximum: f32, step: f32) -> Setting {
    Setting { parameter: Parameter { id: id.to_string(), description: description.to_string(), initial, minimum, maximum, step }, choices: None }
}

pub(crate) fn choice(id: &str, description: &str, initial: i32, choices: &[&str]) -> Setting {
    let mut s = setting(id, description, initial as f32, 0.0, (choices.len() - 1) as f32, 1.0);
    s.choices = Some(choices.iter().map(|c| c.to_string()).collect());
    s
}

/// A screen filter: its passes, the consoles it suits (none for every one), whose work it rests on, and its settings.
#[derive(Clone, Debug)]
pub struct Filter {
    /// The name the catalog finds it by: its own, or a simple effect's with its sampling.
    pub key: String,
    pub name: String,
    pub passes: Vec<Pass>,
    pub consoles: Option<Vec<String>>,
    pub credit: String,
    pub settings: Vec<Setting>,
    /// Passes made from the values, in place of `passes`.
    pub build: Option<fn(&Values) -> Vec<Pass>>,
    /// The settings whose value decides which passes exist.
    pub structural: Option<Vec<String>>,
    /// The encoding of the picture before the palette that a pass declaring "raw" reads.
    pub raw_encoding: Option<String>,
    /// Each console's constants and defaults; "" is any console without its own.
    pub console_defaults: Option<BTreeMap<String, Values>>,
    pub aspect: Option<f64>,
    pub crops_itself: bool,
    pub rows_once: bool,
    pub history_depth: i32,
}

/// The entry of `console_defaults` that any console without its own takes.
pub const ANY_CONSOLE: &str = "";

/// The most rows of a frame one of the console's lines may take for the reducing pass to bring them back.
pub const MOST_ROWS_TO_A_LINE: i32 = 8;

impl Filter {
    pub fn plain(key: &str, name: &str, passes: Vec<Pass>, consoles: Option<&[&str]>, credit: &str, settings: Vec<Setting>) -> Filter {
        Filter {
            key: key.to_string(),
            name: name.to_string(),
            passes,
            consoles: consoles.map(|c| c.iter().map(|s| s.to_string()).collect()),
            credit: credit.to_string(),
            settings,
            build: None,
            structural: None,
            raw_encoding: None,
            console_defaults: None,
            aspect: None,
            crops_itself: false,
            rows_once: false,
            history_depth: 0,
        }
    }

    /// How many earlier frames any variant of the filter may read.
    pub fn history(&self) -> i32 {
        self.passes.iter().map(|p| p.history).fold(self.history_depth, i32::max)
    }

    /// A console's values: its own entry, or the one for any console, or none.
    pub fn defaults_for(&self, console: Option<&str>) -> Option<&Values> {
        let defaults = self.console_defaults.as_ref()?;
        console.and_then(|c| defaults.get(c)).or_else(|| defaults.get(ANY_CONSOLE))
    }

    /// What a setting starts at on a console.
    pub fn default_for(&self, parameter: &Parameter, console: Option<&str>) -> f32 {
        self.defaults_for(console).and_then(|v| v.get(&parameter.id)).copied().unwrap_or(parameter.initial)
    }

    /// Every value the passes read on a console: each setting's given value or its default there, then the console's constants.
    pub fn values_for(&self, console: Option<&str>, given: Option<&Values>) -> Values {
        let mut values = Values::new();
        for s in &self.settings {
            let value = given.and_then(|g| g.get(&s.parameter.id)).copied().unwrap_or_else(|| self.default_for(&s.parameter, console));
            values.insert(s.parameter.id.clone(), value);
        }
        if let Some(constants) = self.defaults_for(console) {
            for (id, value) in constants {
                values.entry(id.clone()).or_insert(*value);
            }
        }
        values
    }

    /// The passes for these values.
    pub fn passes_for(&self, values: &Values) -> Vec<Pass> {
        match self.build {
            Some(build) => build(values),
            None => self.passes.clone(),
        }
    }

    /// Whether a pass made for this console and these values reads the raw encoding, so a front end asks its core for it.
    pub fn reads_raw(&self, console: Option<&str>, given: Option<&Values>) -> bool {
        self.raw_encoding.is_some() && self.passes_for(&self.values_for(console, given)).iter().any(|p| p.glsl.contains("uniform sampler2D raw;"))
    }

    /// How many rows of a frame are one of the console's lines, for a filter that counts lines and keeps no history; one otherwise.
    pub fn reduce_by(&self, width: i32, height: i32, lines: i32) -> i32 {
        if self.rows_once && self.history() == 0 && lines > 0 && height > lines && height % lines == 0 && height / lines <= MOST_ROWS_TO_A_LINE && width % (height / lines) == 0 {
            height / lines
        } else {
            1
        }
    }
}

/// Every built-in filter, in the order a front end offers them, then the simple effects nearest and linear.
pub fn catalog() -> &'static [Filter] {
    static CATALOG: std::sync::OnceLock<Vec<Filter>> = std::sync::OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut all = vec![crt::filter(), panels::lottes()];
        all.extend(panels::handhelds());
        all.extend(panels::effects());
        all
    })
}

/// A filter by its key.
pub fn find(key: &str) -> Option<&'static Filter> {
    catalog().iter().find(|f| f.key == key)
}
