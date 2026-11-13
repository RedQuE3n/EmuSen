//! What a console's game picture is drawn through, read from the player's display settings (EmuSen_Qt.md §1.8.10): the
//! screen filter its `ScreenFilter` names with the values `ShaderParameters` holds, the overscan its `Overscan` crops,
//! and its `PictureShape`. Plain data in, plain data out: Serenity's catalogue and the graphics model, as the platform
//! library gives them, and the rectangle and crop the window draws by.

use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The filter a console with none chosen, or one no longer offered for it, is drawn through: its pixels as they are.
pub const PLAIN: &str = "effect.None.nearest";

/// What a console's `ScreenFilter` holds before a preset of the shader pack, by its path in the pack's folder
/// (Galaxia's `DIR_SHADERS`), in place of a built-in filter's name.
pub const PACK_PREFIX: &str = "slang:";

/// The preset of the shader pack a stored `ScreenFilter` names, by its path in the pack's folder; none for a built-in.
pub fn pack_preset(stored: &str) -> Option<&str> {
    stored.strip_prefix(PACK_PREFIX).filter(|p| !p.is_empty())
}

/// The filter uniforms a filter that crops itself reads its crop from (Serenity's `filters::write`).
pub const CROP_UNIFORMS: [&str; 4] = ["emusen_crop_left", "emusen_crop_top", "emusen_crop_width", "emusen_crop_height"];

/// The share of the picture hidden at each edge.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Crop {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Crop {
    pub fn is_none(&self) -> bool {
        *self == Crop::default()
    }

    /// The part of the picture left: (x, y, width, height), each a share of the whole.
    pub fn kept(&self) -> [f64; 4] {
        [self.left, self.top, 1.0 - self.left - self.right, 1.0 - self.top - self.bottom]
    }
}

/// A console's picture as its settings ask for it.
#[derive(Clone, Debug, PartialEq)]
pub struct Display {
    /// The catalogue key of the filter drawn through, or the stored value of a preset of the shader pack.
    pub filter: String,
    /// The preset of the shader pack drawn through, by its path in the pack's folder; none for a built-in filter.
    pub preset: Option<String>,
    /// The player's values for the filter's settings, by id.
    pub chosen: BTreeMap<String, f32>,
    /// What the overscan setting hides.
    pub crop: Crop,
    /// Whether the filter crops the picture itself, by its uniforms, rather than the window by a clip.
    pub filter_crops: bool,
    /// Each of the frame's pixels square, rather than the screen the console was made for.
    pub square: bool,
}

impl Default for Display {
    fn default() -> Display {
        Display { filter: PLAIN.to_owned(), preset: None, chosen: BTreeMap::new(), crop: Crop::default(), filter_crops: false, square: false }
    }
}

/// Where the picture is drawn: its width over its height, and what the window clips from the picture it draws.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Shown {
    pub aspect: f64,
    pub clip: Crop,
}

fn text(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

/// A stored number, written with a point or a comma.
fn number(t: &str) -> Option<f64> {
    t.trim().replace(',', ".").parse::<f64>().ok().filter(|n| n.is_finite())
}

fn bits(v: &Value) -> Option<f64> {
    v.as_u64().map(f64::from_bits)
}

/// `console`'s picture from Serenity's `catalogue` (`emusen_serenity_filters`) and the graphics model `graphics`.
pub fn read(catalogue: &Value, graphics: &Value, console: &str) -> Display {
    read_for_game(catalogue, graphics, console, None)
}

/// `console`'s picture for a game whose own crop, games.db's, is `game`: it stands in for the console's overscan.
pub fn read_for_game(catalogue: &Value, graphics: &Value, console: &str, game: Option<Crop>) -> Display {
    let setting = |key: &str| text(graphics.get("Consoles").and_then(|c| c.get(console)).and_then(|c| c.get(key)));
    let filters = catalogue["filters"].as_array().map(Vec::as_slice).unwrap_or_default();
    let suits = |f: &&Value| f["consoles"].as_array().is_none_or(|list| list.iter().any(|c| c == console));
    let stored = setting("ScreenFilter");
    let held_for = |name: &str| graphics.get("ShaderParameters").and_then(|p| p.get(console)).and_then(|p| p.get(name)).cloned();
    if let Some(preset) = pack_preset(&stored) {
        let mut display = Display { filter: stored.clone(), preset: Some(preset.to_owned()), ..Display::default() };
        for (id, value) in held_for(&stored).as_ref().and_then(Value::as_object).into_iter().flatten() {
            if let Some(value) = number(&text(Some(value))) {
                display.chosen.insert(id.clone(), value as f32);
            }
        }
        return with_place(display, catalogue, &setting, console, game);
    }
    let filter = filters.iter().filter(suits).find(|f| f["name"] == stored.as_str() || f["key"] == stored.as_str());
    let filter = filter.or_else(|| filters.iter().find(|f| f["key"] == PLAIN));
    let mut display = Display::default();
    if let Some(filter) = filter {
        display.filter = text(filter.get("key"));
        display.filter_crops = filter["crops_itself"] == true;
        let held = held_for(&text(filter.get("name")));
        for s in filter["settings"].as_array().map(Vec::as_slice).unwrap_or_default() {
            let id = text(s.get("id"));
            if let Some(value) = number(&text(held.as_ref().and_then(|h| h.get(&id)))) {
                display.chosen.insert(id, value as f32);
            }
        }
    }
    with_place(display, catalogue, &setting, console, game)
}

/// Whether `console` drove a television, so its picture has an overscan to crop.
pub fn television(catalogue: &Value, console: &str) -> bool {
    catalogue["screens"].as_array().is_some_and(|all| all.iter().any(|s| s["console"] == console && s["kind"] == "Television"))
}

/// The most a crop may hide at one edge.
fn most(catalogue: &Value) -> f64 {
    bits(&catalogue["crop"]["most_at_an_edge"]).unwrap_or(0.0)
}

/// The `Overscan` value for a crop of the player's own edges, its `Crop*` settings.
pub const CUSTOM: &str = "custom";

/// What an `Overscan` value hides: none, the part a television showed, or the `Crop*` settings' percentages.
pub fn overscan(catalogue: &Value, value: &str, setting: &dyn Fn(&str) -> String) -> Crop {
    let most = most(catalogue);
    match value {
        "tv" => {
            let past = bits(&catalogue["crop"]["television"]).unwrap_or(0.0);
            let edge = ((1.0 - 1.0 / (1.0 + past)) / 2.0).clamp(0.0, most);
            Crop { left: edge, top: edge, right: edge, bottom: edge }
        }
        CUSTOM => {
            let edge = |key: &str| (number(&setting(key)).unwrap_or(0.0) / 100.0).clamp(0.0, most);
            Crop { left: edge("CropLeft"), top: edge("CropTop"), right: edge("CropRight"), bottom: edge("CropBottom") }
        }
        _ => Crop::default(),
    }
}

/// The crop and the shape the console's settings ask for, on a display whose filter is read; a game's own crop in
/// place of the console's overscan.
fn with_place(mut display: Display, catalogue: &Value, setting: &dyn Fn(&str) -> String, console: &str, game: Option<Crop>) -> Display {
    if television(catalogue, console) {
        let most = most(catalogue);
        display.crop = match game {
            Some(c) => Crop { left: c.left.clamp(0.0, most), top: c.top.clamp(0.0, most), right: c.right.clamp(0.0, most), bottom: c.bottom.clamp(0.0, most) },
            None => overscan(catalogue, &setting("Overscan"), setting),
        };
    }
    display.square = setting("PictureShape") == "square";
    display
}

impl Crop {
    /// A game's crop as games.db holds it, a percentage at each edge (the library's `crop` read); none for none.
    pub fn from_record(record: &Value) -> Option<Crop> {
        let edge = |k: &str| record.get(k).and_then(Value::as_f64).map(|v| v / 100.0);
        Some(Crop { left: edge("left")?, top: edge("top")?, right: edge("right")?, bottom: edge("bottom")? })
    }

    /// The library's `set_crop` change of `path` to this crop, in percentages.
    pub fn set_record(&self, path: &str) -> Value {
        json!({ "op": "set_crop", "path": path, "left": self.left * 100.0, "right": self.right * 100.0, "top": self.top * 100.0, "bottom": self.bottom * 100.0 })
    }

    /// The same crop within a rounding's reach.
    pub fn near(&self, other: &Crop) -> bool {
        let a = [self.left, self.top, self.right, self.bottom];
        let b = [other.left, other.top, other.right, other.bottom];
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-9)
    }
}

impl Display {
    /// The values the preset is written with, as `emusen_serenity_filter_write` takes them; none for the defaults.
    pub fn chosen_json(&self) -> Option<Value> {
        (!self.chosen.is_empty()).then(|| Value::Object(self.chosen.iter().map(|(id, v)| (id.clone(), json!(v.to_bits()))).collect()))
    }

    /// The uniforms the chain is given over its preset's: a pack preset's values chosen, or the crop, for a filter
    /// that crops itself.
    pub fn uniforms(&self) -> Vec<(String, f32)> {
        if self.preset.is_some() {
            return self.chosen.iter().map(|(id, v)| (id.clone(), *v)).collect();
        }
        if !self.filter_crops {
            return Vec::new();
        }
        CROP_UNIFORMS.iter().zip(self.crop.kept()).map(|(id, v)| ((*id).to_owned(), v as f32)).collect()
    }

    /// The picture's place for a frame whose console's screen is `screen` wide over high and whose pixels drawn square
    /// are `square`: the shape the setting asks for, narrowed by the crop, and the crop the window clips when the filter
    /// does not make it.
    pub fn shown(&self, screen: f64, square: f64) -> Shown {
        let whole = if self.square && square > 0.0 { square } else { screen };
        let [_, _, w, h] = self.crop.kept();
        let aspect = if whole > 0.0 && h > 0.0 { whole * w / h } else { whole };
        Shown { aspect, clip: if self.filter_crops { Crop::default() } else { self.crop } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalogue() -> Value {
        let b = |v: f64| json!(v.to_bits());
        json!({
            "filters": [
                {"key": "CRT", "name": "CRT", "consoles": ["NES", "PlayStation"], "crops_itself": true,
                 "settings": [{"id": "quality"}, {"id": "mask"}]},
                {"key": "LCD", "name": "Game Boy LCD", "consoles": ["GB"], "crops_itself": false, "settings": []},
                {"key": PLAIN, "name": "None", "consoles": null, "crops_itself": false, "settings": []}
            ],
            "screens": [{"console": "PlayStation", "kind": "Television"}, {"console": "GB", "kind": "Lcd"}],
            "crop": {"television": b(0.07), "most_at_an_edge": b(0.25)}
        })
    }

    #[test]
    fn a_console_with_nothing_chosen_is_drawn_plain_whole_and_at_its_screen_s_shape() {
        let d = read(&catalogue(), &json!({}), "PlayStation");
        assert_eq!(d, Display::default());
        assert_eq!(d.shown(4.0 / 3.0, 2.0), Shown { aspect: 4.0 / 3.0, clip: Crop::default() });
        assert_eq!((d.chosen_json(), d.uniforms()), (None, Vec::new()));
    }

    #[test]
    fn the_crt_chosen_is_drawn_with_the_player_s_values_and_crops_itself() {
        let graphics = json!({
            "Consoles": {"PlayStation": {"ScreenFilter": "CRT", "Overscan": "tv"}},
            "ShaderParameters": {"PlayStation": {"CRT": {"quality": "2", "unknown": "5"}}}
        });
        let d = read(&catalogue(), &graphics, "PlayStation");
        assert_eq!(d.filter, "CRT");
        assert_eq!(d.chosen_json(), Some(json!({"quality": 2.0f32.to_bits()})), "only the filter's own settings");
        let edge = (1.0 - 1.0 / 1.07) / 2.0;
        assert_eq!(d.crop, Crop { left: edge, top: edge, right: edge, bottom: edge });
        let shown = d.shown(4.0 / 3.0, 2.0);
        assert_eq!(shown.clip, Crop::default(), "the CRT crops the picture itself");
        assert!((shown.aspect - 4.0 / 3.0).abs() < 1e-12, "an even crop keeps the shape: {}", shown.aspect);
        let uniforms = d.uniforms();
        assert_eq!(uniforms.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(), CROP_UNIFORMS);
        assert_eq!(uniforms.iter().map(|(_, v)| *v).collect::<Vec<_>>(), [edge as f32, edge as f32, (1.0 - 2.0 * edge) as f32, (1.0 - 2.0 * edge) as f32]);
    }

    #[test]
    fn a_custom_crop_is_clipped_by_the_window_for_a_plain_picture_and_held_to_a_quarter() {
        let graphics = json!({"Consoles": {"PlayStation": {"Overscan": "custom", "CropLeft": "10", "CropRight": "99", "CropTop": "2,5", "PictureShape": "square"}}});
        let d = read(&catalogue(), &graphics, "PlayStation");
        assert_eq!(d.filter, PLAIN);
        assert_eq!(d.crop, Crop { left: 0.1, top: 0.025, right: 0.25, bottom: 0.0 });
        let shown = d.shown(4.0 / 3.0, 2.0);
        assert_eq!(shown.clip, d.crop);
        assert!((shown.aspect - 2.0 * 0.65 / 0.975).abs() < 1e-12, "square pixels, narrowed by the crop: {}", shown.aspect);
    }

    #[test]
    fn a_pack_preset_is_drawn_from_its_path_with_the_player_s_values_as_its_uniforms() {
        let graphics = json!({
            "Consoles": {"PlayStation": {"ScreenFilter": "slang:crt/glow.slangp", "Overscan": "tv"}},
            "ShaderParameters": {"PlayStation": {"slang:crt/glow.slangp": {"glow": "0.25", "width": "3"}, "CRT": {"quality": "2"}}}
        });
        let d = read(&catalogue(), &graphics, "PlayStation");
        assert_eq!((d.filter.as_str(), d.preset.as_deref()), ("slang:crt/glow.slangp", Some("crt/glow.slangp")));
        assert_eq!(d.chosen_json(), Some(json!({"glow": 0.25f32.to_bits(), "width": 3.0f32.to_bits()})), "the preset's own values, not the CRT's");
        assert_eq!(d.uniforms(), [("glow".to_owned(), 0.25), ("width".to_owned(), 3.0)]);
        assert!(!d.filter_crops && d.crop.left > 0.0, "the window clips the overscan of a preset's picture");
        assert_eq!(pack_preset("slang:"), None);
        assert_eq!(pack_preset("CRT"), None);
    }

    #[test]
    fn a_game_s_own_crop_stands_in_for_its_console_s_overscan_on_a_television_alone() {
        let graphics = json!({"Consoles": {"PlayStation": {"Overscan": "tv"}, "GB": {}}});
        let own = Crop::from_record(&json!({"left": 10.0, "right": 40.0, "top": 0.0, "bottom": 2.5})).unwrap();
        assert_eq!(own, Crop { left: 0.1, top: 0.0, right: 0.4, bottom: 0.025 });
        let d = read_for_game(&catalogue(), &graphics, "PlayStation", Some(own));
        assert_eq!(d.crop, Crop { left: 0.1, top: 0.0, right: 0.25, bottom: 0.025 }, "held to a quarter at an edge");
        assert_eq!(read_for_game(&catalogue(), &graphics, "PlayStation", None).crop, overscan(&catalogue(), "tv", &|_| String::new()));
        assert_eq!(read_for_game(&catalogue(), &graphics, "GB", Some(own)).crop, Crop::default(), "an LCD has no overscan");
        assert_eq!(Crop::from_record(&Value::Null), None);
        let record = own.set_record("/r/a.bin");
        assert_eq!((record["op"].as_str(), record["left"].as_f64(), record["bottom"].as_f64()), (Some("set_crop"), Some(10.0), Some(2.5)));
        assert!(Crop::from_record(&record).unwrap().near(&own));
    }

    #[test]
    fn a_filter_for_another_console_and_an_lcd_s_overscan_are_not_drawn() {
        let graphics = json!({"Consoles": {"GB": {"ScreenFilter": "CRT", "Overscan": "tv"}, "PlayStation": {"ScreenFilter": "Game Boy LCD"}}});
        let gb = read(&catalogue(), &graphics, "GB");
        assert_eq!((gb.filter.as_str(), gb.crop), (PLAIN, Crop::default()), "the CRT is not offered for the GB, nor a crop for its LCD");
        assert_eq!(read(&catalogue(), &graphics, "PlayStation").filter, PLAIN);
    }
}
