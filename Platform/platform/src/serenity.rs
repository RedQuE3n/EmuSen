//! Serenity's C layer: presets, sources, parameters, the compiler, reflection and the SPIR-V cache.
//!
//! Shaderc and SQLite are the host's, lent once by the handles its own bindings loaded. What a reader makes is
//! structured, so it crosses as JSON (§3.8), with a float as its 32 bits so that a NaN or an infinity crosses whole;
//! a reader's failure is a JSON document too, since it carries more than words. A result that does not fit the
//! caller's buffer waits on the calling thread for `emusen_serenity_take`. A cache is a handle that locks itself, as
//! the passes of a preset are compiled side by side through one. See EmuSen_RustPlatform.md §16.

use crate::endymion::{hand_over, take};
use crate::{fail, required, status, text};
use emusen_galaxia::json::{self, Value};
use emusen_serenity::cache::{self, Cache};
use emusen_serenity::compiler::{Shaderc, Stage};
use emusen_serenity::preset::{self, Preset, ScaleType, Wrap};
use emusen_serenity::reflection::{self, Block, Fault, Member, Reflection, Sampler};
use emusen_serenity::source::{self, Parameter, Source};
use emusen_serenity::sqlite::Sqlite;
use emusen_serenity::{Error, parameters, text as rules, word};
use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};

static SHADERC: OnceLock<Arc<Shaderc>> = OnceLock::new();
static SQLITE: OnceLock<Arc<Sqlite>> = OnceLock::new();

thread_local! {
    static PENDING: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn shaderc() -> Result<Arc<Shaderc>, i32> {
    SHADERC.get().cloned().ok_or_else(|| fail(status::NOT_SUPPORTED, "Shaderc has not been lent to the library"))
}

/// Bytes out by the pending idiom: their length, copied when they fit and kept for `emusen_serenity_take` when not.
unsafe fn out_bytes(bytes: Vec<u8>, out: *mut u8, cap: usize) -> i64 {
    PENDING.with_borrow_mut(|pending| unsafe { hand_over(pending, bytes, out, cap) })
}

unsafe fn out_json(value: &Value, out: *mut u8, cap: usize) -> i64 {
    let mut text = String::new();
    json::write_compact(&mut text, value);
    unsafe { out_bytes(text.into_bytes(), out, cap) }
}

fn object(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(name, value)| (json::Text::new(name), value)).collect())
}

fn optional(text: &Option<String>) -> Value {
    text.as_ref().map_or(Value::Null, Value::string)
}

fn bits(value: f32) -> Value {
    Value::number(value.to_bits())
}

/// A reader's failure, as the exception the C# throws for it.
fn failure(error: &Error) -> Value {
    match error {
        Error::NotFound { message, file } => object(vec![("fail", Value::string("not_found")), ("message", Value::string(message)), ("file", Value::string(file))]),
        Error::InvalidData(message) => object(vec![("fail", Value::string("invalid_data")), ("message", Value::string(message))]),
        Error::Argument { message, parameter } => object(vec![("fail", Value::string("argument")), ("message", Value::string(message)), ("parameter", Value::string(parameter))]),
        Error::Io { message, denied } => object(vec![("fail", Value::string(if *denied { "denied" } else { "io" })), ("message", Value::string(message))]),
    }
}

/// Shaderc as the host has it loaded, with the file it came from when the host can name it (null when not), whose
/// digest the compiler's identity carries. The first lent is kept; the library never closes it.
///
/// # Safety
/// `handle` a live handle to Shaderc from the platform's loader, kept loaded for the process; `path` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_shaderc_lend(handle: *mut c_void, path: *const u8, len: usize) -> i32 {
    if SHADERC.get().is_some() {
        return 0;
    }
    let path = match unsafe { text(path, len) } {
        Ok(path) => path,
        Err(status) => return status,
    };
    match unsafe { Shaderc::lent(handle, path) } {
        Ok(lent) => {
            let _ = SHADERC.set(Arc::new(lent));
            0
        }
        Err(words) => fail(if handle.is_null() { status::NULL } else { status::NOT_SUPPORTED }, words),
    }
}

/// SQLite as the host has it loaded, so that the process holds one SQLite. The first lent is kept, and never closed.
///
/// # Safety
/// `handle` a live handle to SQLite from the platform's loader, kept loaded for the process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_sqlite_lend(handle: *mut c_void) -> i32 {
    if SQLITE.get().is_some() {
        return 0;
    }
    match unsafe { Sqlite::lent(handle) } {
        Ok(lent) => {
            let _ = SQLITE.set(Arc::new(lent));
            0
        }
        Err(words) => fail(if handle.is_null() { status::NULL } else { status::NOT_SUPPORTED }, words),
    }
}

/// What is waiting on this thread from a call whose result did not fit, copied when it fits now and then forgotten.
///
/// # Safety
/// `out` valid for `cap` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_take(out: *mut u8, cap: usize) -> i64 {
    PENDING.with_borrow_mut(|pending| unsafe { take(pending, out, cap) })
}

/// The compiler's identity, which a cache's keys carry.
///
/// # Safety
/// `out` valid for `cap` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_compiler_identity(out: *mut u8, cap: usize) -> i64 {
    match shaderc() {
        Ok(shaderc) => unsafe { crate::put(shaderc.identity().as_bytes(), out, cap) },
        Err(status) => status as i64,
    }
}

fn wrap(wrap: Wrap) -> Value {
    Value::number(match wrap {
        Wrap::ClampToBorder => 0,
        Wrap::ClampToEdge => 1,
        Wrap::Repeat => 2,
        Wrap::MirroredRepeat => 3,
    })
}

fn scale_type(kind: ScaleType) -> Value {
    Value::number(match kind {
        ScaleType::Source => 0,
        ScaleType::Viewport => 1,
        ScaleType::Absolute => 2,
    })
}

fn preset_json(preset: &Preset) -> Value {
    object(vec![
        ("path", Value::string(&preset.path)),
        (
            "passes",
            Value::Array(
                preset
                    .passes
                    .iter()
                    .map(|pass| {
                        object(vec![
                            ("shader", Value::string(&pass.shader_path)),
                            ("alias", optional(&pass.alias)),
                            ("linear", Value::Bool(pass.filter_linear)),
                            ("wrap", wrap(pass.wrap)),
                            ("type_x", scale_type(pass.scale_type_x)),
                            ("type_y", scale_type(pass.scale_type_y)),
                            ("scale_x", bits(pass.scale_x)),
                            ("scale_y", bits(pass.scale_y)),
                            ("float", Value::Bool(pass.float_framebuffer)),
                            ("srgb", Value::Bool(pass.srgb_framebuffer)),
                            ("mipmap", Value::Bool(pass.mipmap_input)),
                            ("mod", Value::number(pass.frame_count_mod)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "textures",
            Value::Array(
                preset
                    .textures
                    .iter()
                    .map(|texture| {
                        object(vec![
                            ("name", Value::string(&texture.name)),
                            ("path", Value::string(&texture.path)),
                            ("linear", Value::Bool(texture.linear)),
                            ("wrap", wrap(texture.wrap)),
                            ("mipmap", Value::Bool(texture.mipmap)),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("parameters", Value::Array(preset.parameters.iter().map(|(key, value)| Value::Array(vec![Value::string(key), bits(*value)])).collect())),
    ])
}

/// A preset read through its references: a JSON document of it, or of why it could not be read.
///
/// # Safety
/// `path` valid for `len` bytes; `out` for `cap`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_preset_load(path: *const u8, len: usize, out: *mut u8, cap: usize) -> i64 {
    let path = match unsafe { required(path, len) } {
        Ok(path) => path,
        Err(status) => return status as i64,
    };
    let document = match preset::load(path) {
        Ok(preset) => preset_json(&preset),
        Err(error) => failure(&error),
    };
    unsafe { out_json(&document, out, cap) }
}

fn parameter_json(parameter: &Parameter) -> Value {
    object(vec![
        ("id", Value::string(&parameter.id)),
        ("description", Value::string(&parameter.description)),
        ("initial", bits(parameter.initial)),
        ("minimum", bits(parameter.minimum)),
        ("maximum", bits(parameter.maximum)),
        ("step", bits(parameter.step)),
    ])
}

fn source_json(source: &Source) -> Value {
    object(vec![
        ("path", Value::string(&source.path)),
        ("vertex", Value::string(&source.vertex)),
        ("fragment", Value::string(&source.fragment)),
        ("name", optional(&source.pass_name)),
        ("format", optional(&source.framebuffer_format)),
        ("parameters", Value::Array(source.parameters.iter().map(parameter_json).collect())),
    ])
}

/// A shader's source with its includes in place, split into its stages: a JSON document of it, or of why it could not be read.
///
/// # Safety
/// `path` valid for `len` bytes; `out` for `cap`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_source_load(path: *const u8, len: usize, out: *mut u8, cap: usize) -> i64 {
    let path = match unsafe { required(path, len) } {
        Ok(path) => path,
        Err(status) => return status as i64,
    };
    let document = match source::load(path) {
        Ok(source) => source_json(&source),
        Err(error) => failure(&error),
    };
    unsafe { out_json(&document, out, cap) }
}

fn number(value: Option<&Value>) -> Option<u32> {
    match value? {
        Value::Number(token) => token.parse().ok(),
        _ => None,
    }
}

fn parameter_of(value: &Value) -> Option<Parameter> {
    let float = |name: &str| number(value.get(name)).map(f32::from_bits);
    Some(Parameter {
        id: value.get("id")?.as_str()?.to_string(),
        description: value.get("description")?.as_str()?.to_string(),
        initial: float("initial")?,
        minimum: float("minimum")?,
        maximum: float("maximum")?,
        step: float("step")?,
    })
}

/// # Safety
/// `data` valid for `len` bytes.
unsafe fn document(data: *const u8, len: usize) -> Result<Value, i32> {
    json::parse(unsafe { required(data, len) }?).map_err(|error| fail(status::PARSE, error.to_string()))
}

fn list(value: Option<&Value>) -> Option<&Vec<Value>> {
    match value? {
        Value::Array(items) => Some(items),
        _ => None,
    }
}

/// The parameters of a preset's passes merged, first declaration winning, each with the preset's value where it gives one.
/// In: `{"sources":[[parameter,…],…],"overrides":[["id",bits],…]}`; out: the merged parameters.
///
/// # Safety
/// `input` valid for `len` bytes; `out` for `cap`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_parameters_merge(input: *const u8, len: usize, out: *mut u8, cap: usize) -> i64 {
    let input = match unsafe { document(input, len) } {
        Ok(input) => input,
        Err(status) => return status as i64,
    };
    let read = || -> Option<Merge> {
        let sources = list(input.get("sources"))?.iter().map(|source| list(Some(source))?.iter().map(parameter_of).collect()).collect::<Option<Vec<Vec<Parameter>>>>()?;
        let overrides = list(input.get("overrides"))?
            .iter()
            .map(|pair| {
                let pair = list(Some(pair))?;
                Some((pair.first()?.as_str()?.to_string(), f32::from_bits(number(pair.get(1))?)))
            })
            .collect::<Option<Vec<(String, f32)>>>()?;
        Some((sources, overrides))
    };
    let Some((sources, overrides)) = read() else { return fail(status::PARSE, "not the sources and overrides of a merge") as i64 };
    let merged = parameters::merge(sources.iter().map(Vec::as_slice), &overrides);
    unsafe { out_json(&Value::Array(merged.iter().map(parameter_json).collect()), out, cap) }
}

/// What a merge is given: each source's parameters, and the preset's values.
type Merge = (Vec<Vec<Parameter>>, Vec<(String, f32)>);

fn stage(stage: u32) -> Result<Stage, i32> {
    match stage {
        0 => Ok(Stage::Vertex),
        1 => Ok(Stage::Fragment),
        _ => Err(fail(status::BAD_ARGUMENT, "no such stage")),
    }
}

/// A stage's SPIR-V: its length in bytes. PARSE for a stage that does not compile, with the C#'s words and Shaderc's.
///
/// # Safety
/// `source` valid for `len` bytes and `name` for `name_len`; `out` for `cap`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_compile(source: *const u8, len: usize, kind: u32, name: *const u8, name_len: usize, out: *mut u8, cap: usize) -> i64 {
    let compiled = (|| {
        let (source, name) = (unsafe { required(source, len) }?, unsafe { required(name, name_len) }?);
        shaderc()?.compile(source, stage(kind)?, name).map_err(|words| fail(status::PARSE, words))
    })();
    match compiled {
        Ok(spirv) => unsafe { out_bytes(spirv, out, cap) },
        Err(status) => status as i64,
    }
}

fn block_json(block: &Option<Block>) -> Value {
    match block {
        None => Value::Null,
        Some(block) => object(vec![
            ("binding", Value::number(block.binding)),
            ("size", Value::number(block.size)),
            ("members", Value::Array(block.members.iter().map(|m| object(vec![("name", Value::string(&m.name)), ("offset", Value::number(m.offset)), ("size", Value::number(m.size))])).collect())),
        ]),
    }
}

fn reflection_json(reflection: &Reflection) -> Value {
    object(vec![
        ("uniforms", block_json(&reflection.uniforms)),
        ("push", block_json(&reflection.push_constants)),
        ("samplers", Value::Array(reflection.samplers.iter().map(|s| object(vec![("name", Value::string(&s.name)), ("binding", Value::number(s.binding))])).collect())),
    ])
}

fn fault_json(fault: &Fault) -> Value {
    let (kind, message) = match fault {
        Fault::NotSpirv(message) => ("not_spirv", message.as_str()),
        Fault::IndexOutOfRange => ("index", "an instruction is shorter than its kind"),
        Fault::ArgumentOutOfRange => ("range", "an entry point's name runs to its end"),
        Fault::Endless => ("endless", "a type holds itself"),
    };
    object(vec![("fail", Value::string(kind)), ("message", Value::string(message))])
}

/// # Safety
/// `data` valid for `len` bytes, or null with `len` 0.
unsafe fn bytes<'a>(data: *const u8, len: usize) -> &'a [u8] {
    if data.is_null() || len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } }
}

/// What a stage's SPIR-V binds: a JSON document of it, or of why the module could not be read.
///
/// # Safety
/// `spirv` valid for `len` bytes; `out` for `cap`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_reflect(spirv: *const u8, len: usize, out: *mut u8, cap: usize) -> i64 {
    let document = match reflection::read(unsafe { bytes(spirv, len) }) {
        Ok(reflection) => reflection_json(&reflection),
        Err(fault) => fault_json(&fault),
    };
    unsafe { out_json(&document, out, cap) }
}

fn block_of(value: Option<&Value>) -> Option<Option<Block>> {
    let value = value?;
    if value.is_null() {
        return Some(None);
    }
    let members = list(value.get("members"))?
        .iter()
        .map(|member| Some(Member { name: member.get("name")?.as_str()?.to_string(), offset: number(member.get("offset"))?, size: number(member.get("size"))? }))
        .collect::<Option<Vec<Member>>>()?;
    Some(Some(Block { binding: number(value.get("binding"))?, size: number(value.get("size"))?, members }))
}

fn reflection_of(value: Option<&Value>) -> Option<Reflection> {
    let value = value?;
    let samplers = list(value.get("samplers"))?
        .iter()
        .map(|sampler| Some(Sampler { name: sampler.get("name")?.as_str()?.to_string(), binding: number(sampler.get("binding"))? }))
        .collect::<Option<Vec<Sampler>>>()?;
    Some(Reflection { uniforms: block_of(value.get("uniforms"))?, push_constants: block_of(value.get("push"))?, samplers })
}

/// Two stages' reflections as one pass's. In: `{"vertex":reflection,"fragment":reflection}`; out: the merged reflection.
///
/// # Safety
/// `input` valid for `len` bytes; `out` for `cap`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_reflect_merge(input: *const u8, len: usize, out: *mut u8, cap: usize) -> i64 {
    let input = match unsafe { document(input, len) } {
        Ok(input) => input,
        Err(status) => return status as i64,
    };
    let (Some(vertex), Some(fragment)) = (reflection_of(input.get("vertex")), reflection_of(input.get("fragment"))) else {
        return fail(status::PARSE, "not the two reflections of a merge") as i64;
    };
    unsafe { out_json(&reflection_json(&reflection::merge(&vertex, &fragment)), out, cap) }
}

/// The module with the inputs no function reads left out of its entry points: its length in bytes, or ABSENT when
/// there is nothing to leave out and the caller keeps its own. BAD_ARGUMENT for an entry point whose name runs to its end.
///
/// # Safety
/// `spirv` valid for `len` bytes; `out` for `cap`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_without_unread_inputs(spirv: *const u8, len: usize, out: *mut u8, cap: usize) -> i64 {
    match reflection::without_unread_inputs(unsafe { bytes(spirv, len) }) {
        Ok(Some(pruned)) => unsafe { out_bytes(pruned, out, cap) },
        Ok(None) => status::ABSENT as i64,
        Err(_) => fail(status::BAD_ARGUMENT, "an entry point's name runs to its end") as i64,
    }
}

// The cache.

pub struct CacheHandle {
    cache: Mutex<Cache>,
}

fn locked(handle: &CacheHandle) -> std::sync::MutexGuard<'_, Cache> {
    handle.cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A cache on a file, with the bound on its bytes and the compiler's identity its keys carry (null for the lent
/// Shaderc's own). It is made even when the file cannot be used, and then says why and compiles everything.
///
/// # Safety
/// `path` valid for `len` bytes; `identity` for `identity_len`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_cache_new(path: *const u8, len: usize, limit: i64, identity: *const u8, identity_len: usize) -> *mut CacheHandle {
    let made = (|| {
        let path = unsafe { required(path, len) }?;
        let identity = match unsafe { text(identity, identity_len) }? {
            Some(identity) => identity.to_string(),
            None => shaderc()?.identity().to_string(),
        };
        let sqlite = SQLITE.get().cloned().ok_or_else(|| fail(status::NOT_SUPPORTED, "SQLite has not been lent to the library"))?;
        Ok::<_, i32>(Cache::open(sqlite, path, limit, &identity))
    })();
    match made {
        Ok(cache) => Box::into_raw(Box::new(CacheHandle { cache: Mutex::new(cache) })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// # Safety
/// `h` from `emusen_serenity_cache_new`, not freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_cache_free(h: *mut CacheHandle) -> i32 {
    if !h.is_null() {
        drop(unsafe { Box::from_raw(h) });
    }
    0
}

macro_rules! cache {
    ($h:expr) => {
        match unsafe { $h.as_ref() } {
            Some(handle) => handle,
            None => return fail(status::NULL, "no handle").into(),
        }
    };
}

/// Lets go of the database; the cache then keeps nothing and still compiles, and its counts stay to be read.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_cache_close(h: *const CacheHandle) -> i32 {
    locked(cache!(h)).close();
    0
}

/// A stage's SPIR-V from the cache, or compiled and kept: its length in bytes. `now` is Unix seconds. PARSE for a
/// stage that does not compile.
///
/// # Safety
/// `h` live; `source` valid for `len` bytes and `name` for `name_len`; `out` for `cap`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_cache_compile(h: *const CacheHandle, source: *const u8, len: usize, kind: u32, name: *const u8, name_len: usize, now: i64, out: *mut u8, cap: usize) -> i64 {
    let handle = cache!(h);
    let compiled = (|| {
        let (source, name) = (unsafe { required(source, len) }?, unsafe { required(name, name_len) }?);
        cache::compile(&handle.cache, &*shaderc()?, source, stage(kind)?, name, now).map_err(|words| fail(status::PARSE, words))
    })();
    match compiled {
        Ok(spirv) => unsafe { out_bytes(spirv, out, cap) },
        Err(status) => status as i64,
    }
}

pub const CACHE_WORKING: u32 = 0;
pub const CACHE_HITS: u32 = 1;
pub const CACHE_MISSES: u32 = 2;
pub const CACHE_STORED: u32 = 3;
pub const CACHE_DAMAGED: u32 = 4;
pub const CACHE_SKIPPED: u32 = 5;
pub const CACHE_EVICTED: u32 = 6;
pub const CACHE_TOUCH_AFTER: u32 = 7;

/// Whether the cache works, one of its counts, or how old a row's use must be before a hit writes it again.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_cache_get(h: *const CacheHandle, which: u32) -> i64 {
    let cache = locked(cache!(h));
    match which {
        CACHE_WORKING => cache.working() as i64,
        CACHE_HITS => cache.counters.hits as i64,
        CACHE_MISSES => cache.counters.misses as i64,
        CACHE_STORED => cache.counters.stored as i64,
        CACHE_DAMAGED => cache.counters.damaged as i64,
        CACHE_SKIPPED => cache.counters.skipped as i64,
        CACHE_EVICTED => cache.counters.evicted as i64,
        CACHE_TOUCH_AFTER => cache.touch_after,
        _ => fail(status::BAD_ARGUMENT, "no such number") as i64,
    }
}

/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_cache_set_touch_after(h: *const CacheHandle, seconds: i64) -> i32 {
    locked(cache!(h)).touch_after = seconds;
    0
}

/// Why the cache is off; ABSENT while it works.
///
/// # Safety
/// `h` live; `out` valid for `cap` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_cache_problem(h: *const CacheHandle, out: *mut u8, cap: usize) -> i64 {
    match &locked(cache!(h)).problem {
        Some(problem) => unsafe { crate::put(problem.as_bytes(), out, cap) },
        None => status::ABSENT as i64,
    }
}

/// The key a compile is kept under: 32 bytes into `out`.
///
/// # Safety
/// The three texts valid for their lengths; `out` valid for 32 bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_cache_key(identity: *const u8, identity_len: usize, source: *const u8, len: usize, kind: u32, name: *const u8, name_len: usize, out: *mut u8) -> i32 {
    let made = (|| Ok::<_, i32>(cache::key(unsafe { required(identity, identity_len) }?, unsafe { required(source, len) }?, stage(kind)?, unsafe { required(name, name_len) }?)))();
    match made {
        Ok(key) if !out.is_null() => {
            unsafe { std::ptr::copy_nonoverlapping(key.as_ptr(), out, 32) };
            0
        }
        Ok(_) => fail(status::NULL, "nowhere to put the key"),
        Err(status) => status,
    }
}

pub const CLASS_SPACE: u32 = 0;
pub const CLASS_WORD: u32 = 1;

/// Whether a UTF-16 code unit is white space or a word character to the readers, for a test that holds both to .NET's own answer.
#[unsafe(no_mangle)]
pub extern "C" fn emusen_serenity_text_class(which: u32, unit: u32) -> i32 {
    let Some(c) = char::from_u32(unit) else { return 0 };
    match which {
        CLASS_SPACE => rules::is_space(c) as i32,
        CLASS_WORD => word::is_word(c) as i32,
        _ => fail(status::BAD_ARGUMENT, "no such class"),
    }
}

pub const NUMBER_FLOAT: u32 = 0;
pub const NUMBER_INTEGER: u32 = 1;

/// A number as the readers parse one: 1 with its 32 bits in `out`, or 0 when the text is not one.
///
/// # Safety
/// `text` valid for `len` bytes; `out` valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_parse_number(which: u32, data: *const u8, len: usize, out: *mut u32) -> i32 {
    let text = match unsafe { required(data, len) } {
        Ok(text) => text,
        Err(status) => return status,
    };
    let parsed = match which {
        NUMBER_FLOAT => rules::parse_float(text).map(f32::to_bits),
        NUMBER_INTEGER => rules::parse_int(text).map(|value| value as u32),
        _ => return fail(status::BAD_ARGUMENT, "no such number"),
    };
    match (parsed, unsafe { out.as_mut() }) {
        (Some(value), Some(out)) => {
            *out = value;
            1
        }
        (Some(_), None) => fail(status::NULL, "nowhere to put the number"),
        (None, _) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reader_s_failure_and_a_reflection_cross_as_json_and_back() {
        let reflection = Reflection {
            uniforms: Some(Block { binding: 1, size: 80, members: vec![Member { name: "MVP \"quoted\" é".into(), offset: 0, size: 64 }] }),
            push_constants: None,
            samplers: vec![Sampler { name: "Source".into(), binding: 2 }],
        };
        let mut text = String::new();
        json::write_compact(&mut text, &reflection_json(&reflection));
        assert_eq!(reflection_of(Some(&json::parse(&text).unwrap())), Some(reflection));
        let mut text = String::new();
        json::write_compact(&mut text, &failure(&Error::NotFound { message: "The preset /x does not exist.".into(), file: "/x".into() }));
        assert_eq!(text, r#"{"fail":"not_found","message":"The preset /x does not exist.","file":"/x"}"#);
        let parameter = Parameter { id: "a".into(), description: "d".into(), initial: f32::from_bits(0xFFC0_0000), minimum: f32::NEG_INFINITY, maximum: 1.5, step: -0.0 };
        let mut text = String::new();
        json::write_compact(&mut text, &parameter_json(&parameter));
        let back = parameter_of(&json::parse(&text).unwrap()).unwrap();
        assert_eq!((back.initial.to_bits(), back.minimum, back.maximum, back.step.to_bits()), (0xFFC0_0000, f32::NEG_INFINITY, 1.5, 0x8000_0000));
    }

    #[test]
    fn nothing_is_compiled_or_cached_before_its_library_is_lent() {
        if SHADERC.get().is_none() {
            assert_eq!(unsafe { emusen_serenity_compile(b"x".as_ptr(), 1, 0, b"n".as_ptr(), 1, std::ptr::null_mut(), 0) }, status::NOT_SUPPORTED as i64);
        }
        if SQLITE.get().is_none() {
            assert!(unsafe { emusen_serenity_cache_new(b"/tmp/x.db".as_ptr(), 9, 1, b"i".as_ptr(), 1) }.is_null());
        }
        assert_eq!(emusen_serenity_text_class(CLASS_WORD, 'é' as u32), 1);
        assert_eq!(emusen_serenity_text_class(CLASS_SPACE, 0xA0), 1);
        assert_eq!(emusen_serenity_text_class(CLASS_WORD, 0xD800), 0);
        let mut bits = 0u32;
        assert_eq!(unsafe { emusen_serenity_parse_number(NUMBER_FLOAT, b"-nan".as_ptr(), 4, &mut bits) }, 1);
        assert_eq!(bits, 0xFFC0_0000);
        assert_eq!(unsafe { emusen_serenity_parse_number(NUMBER_INTEGER, b"1.5".as_ptr(), 3, &mut bits) }, 0);
    }
}
