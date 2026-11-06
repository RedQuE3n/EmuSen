//! A stage's Vulkan GLSL to SPIR-V through Shaderc, unoptimised so the names reflection reads survive: the C#
//! `SlangCompiler`. Shaderc is the library the program ships, opened from its file or lent by a host that has it
//! loaded. See EmuSen_Serenity.md §7.3 and EmuSen_RustPlatform.md §2.8 and §16.2.

use sha2::{Digest, Sha256};
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::mem::ManuallyDrop;

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

// `shaderc_optimization_level_zero`, `shaderc_target_env_vulkan`, `shaderc_env_version_vulkan_1_1`, `shaderc_source_language_glsl`.
const OPTIMIZATION_ZERO: c_int = 0;
const TARGET_VULKAN: c_int = 0;
const VULKAN_1_1: u32 = (1 << 22) | (1 << 12);
const LANGUAGE_GLSL: c_int = 0;
const KIND_VERTEX: c_int = 0;
const KIND_FRAGMENT: c_int = 1;
const STATUS_SUCCESS: c_int = 0;
const ENTRY: &CStr = c"main";

/// Every option a compile is given, in the words the C# gives them; the cache's key carries them.
pub const OPTIONS: &str = "optimization=Zero target=Vulkan/Vulkan11 language=Glsl entry=main defines=none";

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

/// Shaderc and what its output can depend on.
pub struct Shaderc {
    f: Functions,
    _library: ManuallyDrop<libloading::Library>,
    owned: bool,
    identity: String,
}

// A compile makes its own compiler and options and shares nothing, as the C# compiles from several threads at once.
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
        Shaderc { f, _library: ManuallyDrop::new(library), owned, identity }
    }

    /// The compiler itself, as far as its output can depend on it: the library's own bytes, its SPIR-V version, this
    /// binding and the options. The C#'s names its own binding, so the two never serve each other's rows.
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// The stage's SPIR-V, or the C#'s words for a stage that does not compile, with Shaderc's own after them.
    pub fn compile(&self, source: &str, stage: Stage, name: &str) -> Result<Vec<u8>, String> {
        // The file name is handed over up to its first NUL, as the C# binding hands it.
        let file = CString::new(name.split('\0').next().unwrap_or_default()).expect("no NUL is left");
        let kind = if stage == Stage::Vertex { KIND_VERTEX } else { KIND_FRAGMENT };
        unsafe {
            let compiler = (self.f.compiler_initialize)();
            let options = (self.f.options_initialize)();
            (self.f.options_set_optimization_level)(options, OPTIMIZATION_ZERO);
            (self.f.options_set_target_env)(options, TARGET_VULKAN, VULKAN_1_1);
            (self.f.options_set_source_language)(options, LANGUAGE_GLSL);
            let result = (self.f.compile_into_spv)(compiler, source.as_ptr().cast(), source.len(), kind, file.as_ptr(), ENTRY.as_ptr(), options);
            let answer = if (self.f.result_get_compilation_status)(result) != STATUS_SUCCESS {
                let words = (self.f.result_get_error_message)(result);
                let words = if words.is_null() { String::new() } else { String::from_utf8_lossy(CStr::from_ptr(words).to_bytes()).into_owned() };
                Err(format!("{name} ({}) does not compile:\n{words}", stage.name()))
            } else {
                let bytes = (self.f.result_get_bytes)(result);
                let length = (self.f.result_get_length)(result);
                Ok(if bytes.is_null() || length == 0 { Vec::new() } else { std::slice::from_raw_parts(bytes.cast::<u8>(), length).to_vec() })
            };
            (self.f.result_release)(result);
            (self.f.options_release)(options);
            (self.f.compiler_release)(compiler);
            answer
        }
    }
}

impl Drop for Shaderc {
    fn drop(&mut self) {
        if self.owned {
            unsafe { ManuallyDrop::drop(&mut self._library) };
        }
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
    fn the_stage_names_and_the_options_are_the_csharps_words() {
        assert_eq!((Stage::Vertex.name(), Stage::Fragment.name()), ("Vertex", "Fragment"));
        assert_eq!(VULKAN_1_1, 4_198_400);
        assert_eq!(OPTIONS, "optimization=Zero target=Vulkan/Vulkan11 language=Glsl entry=main defines=none");
        assert_eq!(NAMES.len(), 14);
    }

    #[test]
    fn shaderc_missing_beside_the_program_is_the_systems_named_by_its_file_or_the_places_tried_are_said() {
        match Shaderc::find("/nowhere/at/all") {
            Ok(shaderc) => {
                assert!(shaderc.identity().starts_with("shaderc "), "{}", shaderc.identity());
                assert!(cfg!(not(unix)) || !shaderc.identity().contains("unfound"), "{}", shaderc.identity());
                assert!(shaderc.compile("#version 450\nvoid main() {}\n", Stage::Fragment, "a.slang").is_ok());
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
