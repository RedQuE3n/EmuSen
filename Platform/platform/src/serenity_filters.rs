//! Serenity's built-in filters behind the C interface: the catalog, and a filter's values, passes, translation and
//! written preset, so that every front end and DianaSh write the same preset from the same choices. A filter crosses
//! as a document: `{"key"}` for one of the catalog's, or its own description for a filter a caller made. Values cross
//! as JSON objects of each id's 32 bits. Since interface 35. See EmuSen_Serenity.md §3.13.

use crate::serenity::{bits, document, list, number, object, out_json, parameter_of};
use crate::{fail, status, text};
use emusen_dotnet::json::Value;
use emusen_serenity::filters::write::{self, Refusal, Written};
use emusen_serenity::filters::{self, Axis, Filter, Pass, Scale, Setting, Size, Values, glsl, screens};
use std::borrow::Cow;
use std::collections::BTreeMap;

fn values_from(value: &Value) -> Option<Values> {
    let Value::Object(members) = value else { return None };
    members.iter().map(|(id, v)| Some((id.text.clone(), f32::from_bits(number(Some(v))?)))).collect()
}

fn size_from(value: Option<&Value>) -> Option<Option<Size>> {
    match value {
        None | Some(Value::Null) => Some(None),
        Some(v) => {
            let axis = match number(v.get("axis"))? {
                0 => Axis::Source,
                1 => Axis::Viewport,
                _ => Axis::Fixed,
            };
            Some(Some(Size::of(axis, f32::from_bits(number(v.get("factor"))?))))
        }
    }
}

fn pass_from(value: &Value) -> Option<Pass> {
    let flag = |name: &str| value.get(name).and_then(Value::as_bool).unwrap_or(false);
    let mut pass = Pass::new(value.get("glsl")?.as_str()?.to_string(), if number(value.get("scale"))? == 0 { Scale::Source } else { Scale::Viewport });
    pass.linear_source = flag("linear_source");
    pass.history = number(value.get("history")).unwrap_or(0) as i32;
    pass.width = size_from(value.get("width"))?;
    pass.height = size_from(value.get("height"))?;
    pass.float = flag("float");
    pass.feedback = flag("feedback");
    pass.name = value.get("name").and_then(Value::as_str).map(str::to_string);
    pass.linear = list(value.get("linear")).map_or(Some(Vec::new()), |l| l.iter().map(|n| n.as_str().map(str::to_string)).collect())?;
    Some(pass)
}

/// A filter a caller described: its name, passes, settings and constants, and what decides its reducing pass and its raw reads.
fn described(doc: &Value) -> Option<Filter> {
    let name = doc.get("name")?.as_str()?;
    let passes = list(doc.get("passes")).map_or(Some(Vec::new()), |p| p.iter().map(pass_from).collect())?;
    let settings = list(doc.get("settings")).map_or(Some(Vec::new()), |s| s.iter().map(|p| parameter_of(p).map(|parameter| Setting { parameter, choices: None })).collect())?;
    let mut filter = Filter::plain(name, name, passes, None, "", settings);
    filter.raw_encoding = doc.get("raw").and_then(Value::as_str).map(str::to_string);
    if let Some(Value::Object(members)) = doc.get("defaults") {
        filter.console_defaults = Some(members.iter().map(|(console, v)| Some((console.text.clone(), values_from(v)?))).collect::<Option<BTreeMap<_, _>>>()?);
    }
    filter.rows_once = doc.get("rows_once").and_then(Value::as_bool).unwrap_or(false);
    filter.history_depth = number(doc.get("history_depth")).unwrap_or(0) as i32;
    Some(filter)
}

/// # Safety
/// `data` valid for `len` bytes.
unsafe fn filter_of(data: *const u8, len: usize) -> Result<Cow<'static, Filter>, i32> {
    let doc = unsafe { document(data, len) }?;
    if let Some(key) = doc.get("key") {
        let key = key.as_str().unwrap_or_default();
        return filters::find(key).map(Cow::Borrowed).ok_or_else(|| fail(status::BAD_ARGUMENT, format!("no built-in filter is called '{key}'")));
    }
    described(&doc).map(Cow::Owned).ok_or_else(|| fail(status::PARSE, "not a filter's description"))
}

/// Values from `{"id": bits, …}`, or none for a null pointer.
unsafe fn values_of(data: *const u8, len: usize) -> Result<Option<Values>, i32> {
    if data.is_null() {
        return Ok(None);
    }
    let doc = unsafe { document(data, len) }?;
    values_from(&doc).map(Some).ok_or_else(|| fail(status::PARSE, "values are an object of each id's 32 bits"))
}

fn values_json(values: &Values) -> Value {
    object(values.iter().map(|(id, v)| (id.as_str(), bits(*v))).collect())
}

fn size_json(size: Size) -> Value {
    let axis = match size.axis {
        Axis::Source => 0,
        Axis::Viewport => 1,
        Axis::Fixed => 2,
    };
    object(vec![("axis", Value::number(axis)), ("factor", bits(size.factor))])
}

fn strings(items: &[String]) -> Value {
    Value::Array(items.iter().map(Value::string).collect())
}

fn pass_json(p: &Pass) -> Value {
    object(vec![
        ("glsl", Value::string(&p.glsl)),
        ("scale", Value::number(if p.scale == Scale::Source { 0 } else { 1 })),
        ("linear_source", Value::Bool(p.linear_source)),
        ("history", Value::number(p.history)),
        ("width", p.width.map_or(Value::Null, size_json)),
        ("height", p.height.map_or(Value::Null, size_json)),
        ("float", Value::Bool(p.float)),
        ("feedback", Value::Bool(p.feedback)),
        ("name", p.name.as_deref().map_or(Value::Null, Value::string)),
        ("linear", strings(&p.linear)),
    ])
}

fn filter_json(f: &Filter) -> Value {
    let settings = f
        .settings
        .iter()
        .map(|s| {
            let Value::Object(mut members) = crate::serenity::parameter_json(&s.parameter) else { unreachable!() };
            members.push((emusen_dotnet::json::Text::new("choices"), s.choices.as_deref().map_or(Value::Null, strings)));
            Value::Object(members)
        })
        .collect();
    object(vec![
        ("key", Value::string(&f.key)),
        ("name", Value::string(&f.name)),
        ("passes", Value::Array(f.passes.iter().map(pass_json).collect())),
        ("built", Value::Bool(f.build.is_some())),
        ("consoles", f.consoles.as_deref().map_or(Value::Null, strings)),
        ("credit", Value::string(&f.credit)),
        ("settings", Value::Array(settings)),
        ("structural", f.structural.as_deref().map_or(Value::Null, strings)),
        ("raw", f.raw_encoding.as_deref().map_or(Value::Null, Value::string)),
        ("defaults", f.console_defaults.as_ref().map_or(Value::Null, |d| object(d.iter().map(|(c, v)| (c.as_str(), values_json(v))).collect()))),
        ("aspect", f.aspect.map_or(Value::Null, |a| Value::number(a.to_bits()))),
        ("crops_itself", Value::Bool(f.crops_itself)),
        ("rows_once", Value::Bool(f.rows_once)),
        ("history_depth", Value::number(f.history_depth)),
    ])
}

/// The catalog: `{"filters": [...], "screens": [...], "owed": [...], "televisions": [...], "reduce": the reducing pass's GLSL}`; see the header.
///
/// # Safety
/// `out` valid for `cap` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_filters(out: *mut u8, cap: usize) -> i64 {
    let screens = screens::ALL
        .iter()
        .map(|s| {
            object(vec![
                ("console", Value::string(s.console)),
                ("kind", Value::string(s.kind.name())),
                ("profile", Value::string(s.profile)),
                ("hosts_picture", Value::Bool(s.hosts_picture)),
                ("gap", s.gap.map_or(Value::Null, Value::string)),
            ])
        })
        .collect();
    let owed = screens::OWED.iter().map(|(console, note)| object(vec![("console", Value::string(*console)), ("note", Value::string(*note))])).collect();
    let document = object(vec![
        ("filters", Value::Array(filters::catalog().iter().map(filter_json).collect())),
        ("screens", Value::Array(screens)),
        ("owed", Value::Array(owed)),
        ("televisions", Value::Array(screens::televisions().into_iter().map(Value::string).collect())),
        ("reduce", Value::string(glsl::REDUCE)),
    ]);
    unsafe { out_json(&document, out, cap) }
}

/// A filter's values on a console (`console` null for none): each setting's given value or its default there, then the console's constants.
///
/// # Safety
/// Each pointer valid for its length, or null; `out` for `cap`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_filter_values(filter: *const u8, filter_len: usize, console: *const u8, console_len: usize, given: *const u8, given_len: usize, out: *mut u8, cap: usize) -> i64 {
    let run = || -> Result<Values, i32> {
        let filter = unsafe { filter_of(filter, filter_len) }?;
        let console = unsafe { text(console, console_len) }?;
        let given = unsafe { values_of(given, given_len) }?;
        Ok(filter.values_for(console, given.as_ref()))
    };
    match run() {
        Ok(values) => unsafe { out_json(&values_json(&values), out, cap) },
        Err(status) => status as i64,
    }
}

/// The passes a filter is made of for these values, before translation.
///
/// # Safety
/// Each pointer valid for its length, or null; `out` for `cap`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_filter_passes(filter: *const u8, filter_len: usize, values: *const u8, values_len: usize, out: *mut u8, cap: usize) -> i64 {
    let run = || -> Result<Vec<Pass>, i32> {
        let filter = unsafe { filter_of(filter, filter_len) }?;
        let values = unsafe { values_of(values, values_len) }?.unwrap_or_default();
        Ok(filter.passes_for(&values))
    };
    match run() {
        Ok(passes) => unsafe { out_json(&Value::Array(passes.iter().map(pass_json).collect()), out, cap) },
        Err(status) => status as i64,
    }
}

/// Whether a pass made for this console and these values reads the filter's raw encoding: 1 or 0.
///
/// # Safety
/// Each pointer valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_filter_reads_raw(filter: *const u8, filter_len: usize, console: *const u8, console_len: usize, given: *const u8, given_len: usize) -> i32 {
    let run = || -> Result<bool, i32> {
        let filter = unsafe { filter_of(filter, filter_len) }?;
        let console = unsafe { text(console, console_len) }?;
        let given = unsafe { values_of(given, given_len) }?;
        Ok(filter.reads_raw(console, given.as_ref()))
    };
    match run() {
        Ok(reads) => reads as i32,
        Err(status) => status,
    }
}

/// How many rows of a frame are one of the console's lines for the filter's reducing pass; 1 when it has none.
///
/// # Safety
/// `filter` valid for `filter_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_filter_reduce_by(filter: *const u8, filter_len: usize, width: i32, height: i32, lines: i32) -> i32 {
    match unsafe { filter_of(filter, filter_len) } {
        Ok(filter) => filter.reduce_by(width, height, lines),
        Err(status) => status,
    }
}

fn written_json(pass: &Written) -> Value {
    object(vec![
        ("file", Value::string(&pass.file_name)),
        ("text", Value::string(&pass.text)),
        ("alias", pass.alias.as_deref().map_or(Value::Null, Value::string)),
        ("width", size_json(pass.width)),
        ("height", size_json(pass.height)),
        ("float", Value::Bool(pass.float)),
        ("linear", Value::Bool(pass.linear_sampler)),
    ])
}

/// The passes a filter draws with these values, reducing pass first, translated; PARSE with the words for a pass that asks for what no pass provides.
///
/// # Safety
/// Each pointer valid for its length, or null; `out` for `cap`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_filter_translate(filter: *const u8, filter_len: usize, values: *const u8, values_len: usize, reduce_by: i32, nudge_x: f32, nudge_y: f32, out: *mut u8, cap: usize) -> i64 {
    let run = || -> Result<Vec<Written>, i32> {
        let filter = unsafe { filter_of(filter, filter_len) }?;
        let values = unsafe { values_of(values, values_len) }?.unwrap_or_default();
        write::translate(&filter, &values, reduce_by, (nudge_x, nudge_y)).map_err(|words| fail(status::PARSE, words))
    };
    match run() {
        Ok(passes) => unsafe { out_json(&Value::Array(passes.iter().map(written_json).collect()), out, cap) },
        Err(status) => status as i64,
    }
}

/// Writes a filter's preset into a folder of `root` named by its content, once, and gives the preset's path.
///
/// # Safety
/// Each pointer valid for its length, or null; `out` for `cap`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_filter_write(
    filter: *const u8,
    filter_len: usize,
    console: *const u8,
    console_len: usize,
    chosen: *const u8,
    chosen_len: usize,
    reduce_by: i32,
    nudge_x: f32,
    nudge_y: f32,
    root: *const u8,
    root_len: usize,
    out: *mut u8,
    cap: usize,
) -> i64 {
    let run = || -> Result<String, i32> {
        let filter = unsafe { filter_of(filter, filter_len) }?;
        let console = unsafe { text(console, console_len) }?;
        let chosen = unsafe { values_of(chosen, chosen_len) }?;
        let Some(root) = (unsafe { text(root, root_len) })? else { return Err(fail(status::NULL, "no root named")) };
        write::write(root, &filter, console, chosen.as_ref(), reduce_by, (nudge_x, nudge_y)).map_err(|refusal| match refusal {
            Refusal::Pass(words) => fail(status::PARSE, words),
            Refusal::Io(words) => fail(status::IO, words),
        })
    };
    match run() {
        Ok(path) => unsafe { crate::put(path.as_bytes(), out, cap) },
        Err(status) => status as i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_catalog_crosses_and_a_preset_is_written_through_the_interface() {
        let mut out = vec![0u8; 1 << 20];
        let n = unsafe { emusen_serenity_filters(out.as_mut_ptr(), out.len()) };
        assert!(n > 0 && (n as usize) < out.len());
        let catalog = emusen_dotnet::json::parse(std::str::from_utf8(&out[..n as usize]).unwrap()).unwrap();
        let listed = list(catalog.get("filters")).unwrap();
        assert_eq!(listed.len(), filters::catalog().len());
        for (doc, filter) in listed.iter().zip(filters::catalog()) {
            let described = described(doc).unwrap();
            assert_eq!((described.passes.len(), described.settings.len(), described.rows_once), (filter.passes.len(), filter.settings.len(), filter.rows_once), "{}", filter.key);
        }

        let root = std::env::temp_dir().join(format!("emusen-filters-{}", std::process::id())).to_string_lossy().into_owned();
        let crt = br#"{"key":"CRT"}"#;
        let n = unsafe { emusen_serenity_filter_write(crt.as_ptr(), crt.len(), b"NES".as_ptr(), 3, std::ptr::null(), 0, 1, 0.0, 0.0, root.as_ptr(), root.len(), out.as_mut_ptr(), out.len()) };
        assert!(n > 0, "{n}");
        let path = std::str::from_utf8(&out[..n as usize]).unwrap();
        assert!(std::path::Path::new(path).is_file());
        assert!(std::path::Path::new(path).parent().unwrap().join("raw.png").is_file());
        assert_eq!(unsafe { emusen_serenity_filter_reads_raw(crt.as_ptr(), crt.len(), b"NES".as_ptr(), 3, std::ptr::null(), 0) }, 1);
        assert_eq!(unsafe { emusen_serenity_filter_reduce_by(crt.as_ptr(), crt.len(), 512, 448, 224) }, 2);
        let nothing = br#"{"key":"nothing"}"#;
        assert_eq!(unsafe { emusen_serenity_filter_reduce_by(nothing.as_ptr(), nothing.len(), 512, 448, 224) }, status::BAD_ARGUMENT);

        let own = br#"{"name":"own","passes":[{"glsl":"uniform float tint; uniform vec2 secondSize; vec4 filter_main(vec2 coord) { return vec4(tint); }","scale":1}],"settings":[{"id":"tint","description":"Tint","initial":1065353216,"minimum":0,"maximum":1065353216,"step":0}]}"#;
        let tint = br#"{"tint":1065353216}"#;
        let n = unsafe { emusen_serenity_filter_translate(own.as_ptr(), own.len(), tint.as_ptr(), tint.len(), 1, 0.0, 0.0, out.as_mut_ptr(), out.len()) };
        assert_eq!(n, status::PARSE as i64);
        let mut words = vec![0u8; 512];
        let w = unsafe { crate::emusen_platform_last_error(words.as_mut_ptr(), words.len()) } as usize;
        let words = String::from_utf8_lossy(&words[..w]).into_owned();
        assert!(words.contains("The 'own' filter asks for a uniform called 'secondSize'"), "{words}");
        let _ = std::fs::remove_dir_all(&root);
    }
}
