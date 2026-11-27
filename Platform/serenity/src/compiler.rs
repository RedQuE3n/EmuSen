//! A stage's Vulkan GLSL to SPIR-V through Shaderc, optimised for performance while keeping reflection names.

use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::mem::ManuallyDrop;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Vertex,
    Fragment,
}

impl Stage {
    /// The stage as the C# enumeration prints it, which the cache's key and a failure's words carry.
    pub fn name(self) -> &'static str {
        match self {
            Stage::Vertex => "Vertex",
            Stage::Fragment => "Fragment",
        }
    }
}

// `shaderc_optimization_level_performance`, `shaderc_target_env_vulkan`, `shaderc_env_version_vulkan_1_1`, `shaderc_source_language_glsl`.
const OPTIMIZATION_PERFORMANCE: c_int = 2;
const TARGET_VULKAN: c_int = 0;
const VULKAN_1_1: u32 = (1 << 22) | (1 << 12);
const LANGUAGE_GLSL: c_int = 0;
const KIND_VERTEX: c_int = 0;
const KIND_FRAGMENT: c_int = 1;
const STATUS_SUCCESS: c_int = 0;
const ENTRY: &CStr = c"main";

/// Every option a compile is given; the cache's key carries them.
pub const OPTIONS: &str = "optimization=Performance target=Vulkan/Vulkan11 language=Glsl entry=main defines=none";

macro_rules! functions {
    ($($field:ident = $name:literal: fn($($arg:ty),*) $(-> $ret:ty)?;)*) => {
        #[allow(clippy::type_complexity)]
        struct Functions {
            $($field: unsafe extern "C" fn($($arg),*) $(-> $ret)?,)*
        }

        /// The functions of Shaderc's that are called, by their C names.
        pub const NAMES: &[&str] = &[$($name),*];

        impl Functions {
            /// # Safety
            /// `library` must be Shaderc, so that each name has the signature declared for it.
            unsafe fn load(library: &libloading::Library) -> Result<Functions, String> {
                Ok(Functions {
                    $($field: *unsafe { library.get::<unsafe extern "C" fn($($arg),*) $(-> $ret)?>($name) }
                        .map_err(|_| format!("Shaderc has no {}", $name))?,)*
                })
            }
        }
    };
}

functions! {
    compiler_initialize = "shaderc_compiler_initialize": fn() -> *mut c_void;
    compiler_release = "shaderc_compiler_release": fn(*mut c_void);
    options_initialize = "shaderc_compile_options_initialize": fn() -> *mut c_void;
    options_release = "shaderc_compile_options_release": fn(*mut c_void);
    options_set_optimization_level = "shaderc_compile_options_set_optimization_level": fn(*mut c_void, c_int);
    options_set_generate_debug_info = "shaderc_compile_options_set_generate_debug_info": fn(*mut c_void);
    options_set_target_env = "shaderc_compile_options_set_target_env": fn(*mut c_void, c_int, u32);
    options_set_source_language = "shaderc_compile_options_set_source_language": fn(*mut c_void, c_int);
    compile_into_spv = "shaderc_compile_into_spv": fn(*mut c_void, *const c_char, usize, c_int, *const c_char, *const c_char, *mut c_void) -> *mut c_void;
    result_release = "shaderc_result_release": fn(*mut c_void);
    result_get_length = "shaderc_result_get_length": fn(*mut c_void) -> usize;
    result_get_compilation_status = "shaderc_result_get_compilation_status": fn(*mut c_void) -> c_int;
    result_get_bytes = "shaderc_result_get_bytes": fn(*mut c_void) -> *const c_char;
    result_get_error_message = "shaderc_result_get_error_message": fn(*mut c_void) -> *const c_char;
    get_spv_version = "shaderc_get_spv_version": fn(*mut u32, *mut u32);
}

struct Runtime {
    f: Functions,
    _library: ManuallyDrop<libloading::Library>,
    owned: bool,
    identity: String,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if self.owned {
            unsafe { ManuallyDrop::drop(&mut self._library) };
        }
    }
}

struct ThreadCompiler {
    runtime: Arc<Runtime>,
    compiler: *mut c_void,
    options: *mut c_void,
}

impl Drop for ThreadCompiler {
    fn drop(&mut self) {
        unsafe {
            (self.runtime.f.options_release)(self.options);
            (self.runtime.f.compiler_release)(self.compiler);
        }
    }
}

thread_local! {
    static COMPILER: RefCell<Option<ThreadCompiler>> = const { RefCell::new(None) };
}

/// Shaderc and what its output can depend on.
pub struct Shaderc {
    runtime: Arc<Runtime>,
}

// Each worker thread keeps its own compiler and options handles so concurrent compiles share nothing.
unsafe impl Send for Shaderc {}
unsafe impl Sync for Shaderc {}

impl Shaderc {
    /// Shaderc from its file.
    pub fn open(path: &str) -> Result<Shaderc, String> {
        let library = unsafe { libloading::Library::new(path) }.map_err(|e| format!("{path} could not be loaded: {e}"))?;
        let f = unsafe { Functions::load(&library) }?;
        Ok(Shaderc::made(f, library, true, Some(path)))
    }

    /// Shaderc as the program ships it, found as .NET's own probing finds a native library: beside the program, then in
    /// its `runtimes/<rid>/native` folder, then by the system's loader under its name. See EmuSen_RustPlatform.md §33.
    pub fn find(base_directory: &str) -> Result<Shaderc, String> {
        let mut tried = Vec::new();
        for folder in [base_directory.to_string(), format!("{base_directory}/runtimes/{RID}/native")] {
            let path = std::path::Path::new(&folder).join(FILE_NAME);
            if path.is_file() {
                match Shaderc::open(&path.to_string_lossy()) {
                    Ok(shaderc) => return Ok(shaderc),
                    Err(words) => tried.push(words),
                }
            } else {
                tried.push(format!("{} is not there", path.display()));
            }
        }
        for &name in SYSTEM_NAMES {
            match unsafe { libloading::Library::new(name) } {
                Ok(library) => {
                    let f = unsafe { Functions::load(&library) }?;
                    let file = file_of(f.compiler_initialize as *const c_void);
                    return Ok(Shaderc::made(f, library, true, file.as_deref()));
                }
                Err(e) => tried.push(format!("{name}: {e}")),
            }
        }
        Err(format!("Shaderc could not be found ({})", tried.join("; ")))
    }

    /// Shaderc as the host has it loaded, by its loader's handle, with the file it came from when the host can name it.
    ///
    /// # Safety
    /// `handle` must be a live handle to Shaderc from the platform's loader, kept loaded for as long as this lives.
    pub unsafe fn lent(handle: *mut c_void, path: Option<&str>) -> Result<Shaderc, String> {
        if handle.is_null() {
            return Err("no Shaderc handle".to_string());
        }
        #[cfg(unix)]
        let library: libloading::Library = unsafe { libloading::os::unix::Library::from_raw(handle) }.into();
        #[cfg(windows)]
        let library: libloading::Library = unsafe { libloading::os::windows::Library::from_raw(handle as isize) }.into();
        let f = unsafe { Functions::load(&library) }?;
        Ok(Shaderc::made(f, library, false, path))
    }

    fn made(f: Functions, library: libloading::Library, owned: bool, path: Option<&str>) -> Shaderc {
        let (mut version, mut revision) = (0u32, 0u32);
        unsafe { (f.get_spv_version)(&mut version, &mut revision) };
        let identity = format!("shaderc {}; spv {version:X}.{revision}; binding {} {}; {OPTIONS}", digest(path), env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        Shaderc { runtime: Arc::new(Runtime { f, _library: ManuallyDrop::new(library), owned, identity }) }
    }

    /// The compiler itself: the library's own bytes, its SPIR-V version, this binding and the options.
    pub fn identity(&self) -> &str {
        &self.runtime.identity
    }

    fn take_thread_compiler(&self) -> Result<ThreadCompiler, String> {
        if let Some(existing) = COMPILER.with_borrow_mut(|slot| slot.take().filter(|tc| Arc::ptr_eq(&tc.runtime, &self.runtime))) {
            return Ok(existing);
        }
        unsafe {
            let f = &self.runtime.f;
            let compiler = (f.compiler_initialize)();
            let options = (f.options_initialize)();
            if compiler.is_null() || options.is_null() {
                if !options.is_null() {
                    (f.options_release)(options);
                }
                if !compiler.is_null() {
                    (f.compiler_release)(compiler);
                }
                return Err("Shaderc could not initialize compiler state".to_string());
            }
            (f.options_set_optimization_level)(options, OPTIMIZATION_PERFORMANCE);
            (f.options_set_generate_debug_info)(options);
            (f.options_set_target_env)(options, TARGET_VULKAN, VULKAN_1_1);
            (f.options_set_source_language)(options, LANGUAGE_GLSL);
            Ok(ThreadCompiler { runtime: self.runtime.clone(), compiler, options })
        }
    }

    /// The stage's SPIR-V, or the error words for a stage that does not compile, with Shaderc's own after them.
    pub fn compile(&self, source: &str, stage: Stage, name: &str) -> Result<Vec<u8>, String> {
        // The file name is handed over up to its first NUL, as the C# binding hands it.
        let file = CString::new(name.split('\0').next().unwrap_or_default()).expect("no NUL is left");
        let kind = if stage == Stage::Vertex { KIND_VERTEX } else { KIND_FRAGMENT };
        let tc = self.take_thread_compiler()?;
        let f = &self.runtime.f;
        let answer = unsafe {
            let result = (f.compile_into_spv)(tc.compiler, source.as_ptr().cast(), source.len(), kind, file.as_ptr(), ENTRY.as_ptr(), tc.options);
            let answer = if (f.result_get_compilation_status)(result) != STATUS_SUCCESS {
                let words = (f.result_get_error_message)(result);
                let words = if words.is_null() { String::new() } else { String::from_utf8_lossy(CStr::from_ptr(words).to_bytes()).into_owned() };
                Err(format!("{name} ({}) does not compile:\n{words}", stage.name()))
            } else {
                let bytes = (f.result_get_bytes)(result);
                let length = (f.result_get_length)(result);
                Ok(if bytes.is_null() || length == 0 { Vec::new() } else { std::slice::from_raw_parts(bytes.cast::<u8>(), length).to_vec() })
            };
            (f.result_release)(result);
            answer
        };
        COMPILER.with_borrow_mut(|slot| *slot = Some(tc));
        answer
    }
}

/// The file .NET ships Shaderc as on this system.
pub const FILE_NAME: &str = if cfg!(windows) {
    "shaderc_shared.dll"
} else if cfg!(target_os = "macos") {
    "libshaderc_shared.dylib"
} else {
    "libshaderc_shared.so"
};

/// The names the system's loader is asked for when the program ships no copy.
const SYSTEM_NAMES: &[&str] = if cfg!(windows) {
    &["shaderc_shared.dll"]
} else if cfg!(target_os = "macos") {
    &["libshaderc_shared.dylib", "libshaderc_shared.1.dylib"]
} else {
    &["libshaderc_shared.so", "libshaderc_shared.so.1"]
};

/// The runtime identifier whose native folder a framework-dependent build keeps Shaderc in.
const RID: &str = match (cfg!(windows), cfg!(target_os = "macos"), cfg!(target_arch = "aarch64"), cfg!(target_arch = "arm")) {
    (true, _, true, _) => "win-arm64",
    (true, _, _, _) => "win-x64",
    (_, true, true, _) => "osx-arm64",
    (_, true, _, _) => "osx-x64",
    (_, _, true, _) => "linux-arm64",
    (_, _, _, true) => "linux-arm",
    _ => "linux-x64",
};

/// The file a loaded function came from, which the system's loader opened by name.
#[cfg(unix)]
fn file_of(function: *const c_void) -> Option<String> {
    let mut info: libc::Dl_info = unsafe { std::mem::zeroed() };
    if unsafe { libc::dladdr(function, &mut info) } == 0 || info.dli_fname.is_null() {
        return None;
    }
    Some(unsafe { CStr::from_ptr(info.dli_fname) }.to_string_lossy().into_owned())
}

#[cfg(not(unix))]
fn file_of(_function: *const c_void) -> Option<String> {
    None
}

/// The library file's SHA-256 in capitals, or its absence said, when no file is named or it cannot be read.
fn digest(path: Option<&str>) -> String {
    match path {
        None => "unfound".to_string(),
        Some(path) => match std::fs::read(path) {
            Ok(bytes) => Sha256::digest(&bytes).iter().map(|b| format!("{b:02X}")).collect(),
            Err(error) => format!("unread ({:?})", error.kind()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stage_names_and_the_options_use_performance_optimization() {
        assert_eq!((Stage::Vertex.name(), Stage::Fragment.name()), ("Vertex", "Fragment"));
        assert_eq!(VULKAN_1_1, 4_198_400);
        assert_eq!(OPTIMIZATION_PERFORMANCE, 2);
        assert_eq!(OPTIONS, "optimization=Performance target=Vulkan/Vulkan11 language=Glsl entry=main defines=none");
        assert_eq!(NAMES.len(), 15);
    }

    #[test]
    fn shaderc_missing_beside_the_program_is_the_systems_named_by_its_file_or_the_places_tried_are_said() {
        match Shaderc::find("/nowhere/at/all") {
            Ok(shaderc) => {
                assert!(shaderc.identity().starts_with("shaderc "), "{}", shaderc.identity());
                assert!(cfg!(not(unix)) || !shaderc.identity().contains("unfound"), "{}", shaderc.identity());
                let frag = "#version 450\nlayout(std140, set=0, binding=0) uniform UBO { mat4 MVP; float gain; };\nlayout(push_constant) uniform Push { vec4 SourceSize; };\nlayout(set=0, binding=1) uniform sampler2D Source;\nlayout(location=0) in vec2 vTex;\nlayout(location=0) out vec4 FragColor;\nvoid main() { FragColor = texture(Source, vTex) * gain * SourceSize.x + MVP[0]; }\n";
                let first = shaderc.compile(frag, Stage::Fragment, "a.slang").unwrap();
                let second = shaderc.compile(frag, Stage::Fragment, "a.slang").unwrap();
                assert_eq!(first, second, "thread-local compiler reuse produces identical SPIR-V");
                let reflected = crate::reflection::read(&first).unwrap();
                assert!(reflected.uniforms.as_ref().is_some_and(|u| u.find("MVP").is_some() && u.find("gain").is_some()), "{reflected:?}");
                assert!(reflected.push_constants.as_ref().is_some_and(|p| p.find("SourceSize").is_some()), "{reflected:?}");
                assert!(reflected.samplers.iter().any(|s| s.name == "Source" && s.binding == 1), "{reflected:?}");
            }
            Err(words) => assert!(words.contains(&format!("/nowhere/at/all/{FILE_NAME} is not there")), "{words}"),
        }
    }

    #[test]
    fn a_library_that_cannot_be_named_or_read_says_so_in_the_identity() {
        assert_eq!(digest(None), "unfound");
        assert!(digest(Some("/nowhere/at/all/libshaderc_shared.so")).starts_with("unread ("));
    }
}
