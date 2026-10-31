//! The libretro core beside this library: found in its folder, hashed, loaded once for the process, its exports
//! resolved and its description read (EmuSen_LibretroHost.md §1.1, §1.3, §4.1).

use std::collections::HashMap;
use std::ffi::{CString, c_uint, c_void};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use emusen_libretro_sys as sys;

/// The libretro core's exports, every one `libretro.h` requires.
pub struct Api {
    pub set_environment: unsafe extern "C" fn(sys::retro_environment_t),
    pub set_video_refresh: unsafe extern "C" fn(sys::retro_video_refresh_t),
    pub set_audio_sample: unsafe extern "C" fn(sys::retro_audio_sample_t),
    pub set_audio_sample_batch: unsafe extern "C" fn(sys::retro_audio_sample_batch_t),
    pub set_input_poll: unsafe extern "C" fn(sys::retro_input_poll_t),
    pub set_input_state: unsafe extern "C" fn(sys::retro_input_state_t),
    pub init: unsafe extern "C" fn(),
    pub deinit: unsafe extern "C" fn(),
    pub api_version: unsafe extern "C" fn() -> c_uint,
    pub get_system_info: unsafe extern "C" fn(*mut sys::retro_system_info),
    pub get_system_av_info: unsafe extern "C" fn(*mut sys::retro_system_av_info),
    pub set_controller_port_device: unsafe extern "C" fn(c_uint, c_uint),
    pub reset: unsafe extern "C" fn(),
    pub run: unsafe extern "C" fn(),
    pub serialize_size: unsafe extern "C" fn() -> usize,
    pub serialize: unsafe extern "C" fn(*mut c_void, usize) -> bool,
    pub unserialize: unsafe extern "C" fn(*const c_void, usize) -> bool,
    pub cheat_reset: unsafe extern "C" fn(),
    pub cheat_set: unsafe extern "C" fn(c_uint, bool, *const std::ffi::c_char),
    pub load_game: unsafe extern "C" fn(*const sys::retro_game_info) -> bool,
    pub unload_game: unsafe extern "C" fn(),
    pub get_memory_data: unsafe extern "C" fn(c_uint) -> *mut c_void,
    pub get_memory_size: unsafe extern "C" fn(c_uint) -> usize,
}

/// What the core says of itself, and what its folder says of it.
pub struct Library {
    _handle: libloading::Library,
    pub api: Api,
    pub folder: PathBuf,
    pub stem: String,
    pub core_path: PathBuf,
    pub sha256: String,
    pub name: String,
    pub version: String,
    pub extensions: Vec<String>,
    pub need_fullpath: bool,
    /// The `.info` file's keys, `key = "value"` lines, empty where there is none.
    pub info_file: HashMap<String, String>,
    pub core_path_c: CString,
}

static LIBRARY: OnceLock<Result<Library, String>> = OnceLock::new();

/// The library, loaded at the first call; the same answer every time after.
pub fn get() -> Result<&'static Library, &'static str> {
    LIBRARY.get_or_init(load).as_ref().map_err(String::as_str)
}

/// The folder this library was loaded from.
fn own_folder() -> Result<PathBuf, String> {
    let mut info: libc::Dl_info = unsafe { std::mem::zeroed() };
    let here = own_folder as *const c_void;
    if unsafe { libc::dladdr(here, &mut info) } == 0 || info.dli_fname.is_null() {
        return Err("the libretro host cannot tell which file it was loaded from".into());
    }
    let path = PathBuf::from(unsafe { std::ffi::CStr::from_ptr(info.dli_fname) }.to_string_lossy().into_owned());
    let path = std::fs::canonicalize(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    path.parent().map(Path::to_path_buf).ok_or_else(|| format!("{} has no folder", path.display()))
}

/// The one `<stem>_libretro.<suffix>` in `folder`; none, or more than one, is refused in words.
pub fn find_core(folder: &Path) -> Result<(String, PathBuf), String> {
    let mut found = Vec::new();
    for item in std::fs::read_dir(folder).map_err(|e| format!("{}: {e}", folder.display()))? {
        let item = item.map_err(|e| e.to_string())?;
        if let Some(name) = item.file_name().to_str()
            && let Some(stem) = emusen_core_entry::stem_of_core(name)
            && emusen_core_entry::valid_stem(stem)
        {
            found.push((stem.to_owned(), item.path()));
        }
    }
    match found.len() {
        1 => Ok(found.remove(0)),
        0 => Err(format!("{} holds no libretro core", folder.display())),
        n => Err(format!("{} holds {n} libretro cores, and an entry holds one", folder.display())),
    }
}

/// `key = "value"` lines, comments and blanks skipped.
pub fn parse_info(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().trim_matches('"').to_owned()))
        .collect()
}

fn symbol<T: Copy>(lib: &libloading::Library, name: &str) -> Result<T, String> {
    unsafe { lib.get::<T>(name.as_bytes()) }.map(|s| *s).map_err(|_| format!("the core lacks {name}"))
}

fn load() -> Result<Library, String> {
    let folder = own_folder()?;
    let (stem, core_path) = find_core(&folder)?;
    let sha256 = emusen_core_entry::sha256_file(&core_path).map_err(|e| format!("{}: {e}", core_path.display()))?;
    #[cfg(unix)]
    let handle: libloading::Library = unsafe { libloading::os::unix::Library::open(Some(&core_path), libc::RTLD_NOW | libc::RTLD_LOCAL) }
        .map_err(|e| format!("{} does not load: {e}", core_path.display()))?
        .into();
    #[cfg(not(unix))]
    let handle = unsafe { libloading::Library::new(&core_path) }.map_err(|e| format!("{} does not load: {e}", core_path.display()))?;
    let l = &handle;
    let api = Api {
        set_environment: symbol(l, "retro_set_environment")?,
        set_video_refresh: symbol(l, "retro_set_video_refresh")?,
        set_audio_sample: symbol(l, "retro_set_audio_sample")?,
        set_audio_sample_batch: symbol(l, "retro_set_audio_sample_batch")?,
        set_input_poll: symbol(l, "retro_set_input_poll")?,
        set_input_state: symbol(l, "retro_set_input_state")?,
        init: symbol(l, "retro_init")?,
        deinit: symbol(l, "retro_deinit")?,
        api_version: symbol(l, "retro_api_version")?,
        get_system_info: symbol(l, "retro_get_system_info")?,
        get_system_av_info: symbol(l, "retro_get_system_av_info")?,
        set_controller_port_device: symbol(l, "retro_set_controller_port_device")?,
        reset: symbol(l, "retro_reset")?,
        run: symbol(l, "retro_run")?,
        serialize_size: symbol(l, "retro_serialize_size")?,
        serialize: symbol(l, "retro_serialize")?,
        unserialize: symbol(l, "retro_unserialize")?,
        cheat_reset: symbol(l, "retro_cheat_reset")?,
        cheat_set: symbol(l, "retro_cheat_set")?,
        load_game: symbol(l, "retro_load_game")?,
        unload_game: symbol(l, "retro_unload_game")?,
        get_memory_data: symbol(l, "retro_get_memory_data")?,
        get_memory_size: symbol(l, "retro_get_memory_size")?,
    };
    let version = unsafe { (api.api_version)() };
    if version != sys::RETRO_API_VERSION {
        return Err(format!("the core speaks libretro API {version}, this host {}", sys::RETRO_API_VERSION));
    }
    let mut info = sys::retro_system_info { library_name: std::ptr::null(), library_version: std::ptr::null(), valid_extensions: std::ptr::null(), need_fullpath: false, block_extract: false };
    unsafe { (api.get_system_info)(&mut info) };
    let text = crate::options::text;
    let extensions = text(info.valid_extensions).split('|').filter(|e| !e.is_empty()).map(|e| format!(".{}", e.to_ascii_lowercase())).collect();
    let info_path = folder.join(format!("{stem}_libretro.info"));
    let info_file = std::fs::read_to_string(&info_path).map(|t| parse_info(&t)).unwrap_or_default();
    let core_path_c = CString::new(core_path.to_string_lossy().as_bytes()).map_err(|_| "the core's path holds a NUL".to_string())?;
    Ok(Library {
        api,
        name: text(info.library_name),
        version: text(info.library_version),
        extensions,
        need_fullpath: info.need_fullpath,
        _handle: handle,
        folder,
        stem,
        core_path,
        sha256,
        info_file,
        core_path_c,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_info_file_reads_as_its_keys() {
        let m = parse_info("# Software\ndisplay_name = \"Sony - PlayStation 2 (PCEE2)\"\nsystemid = \"playstation2\"\n\nnotes = \"a|b\"\n");
        assert_eq!(m.get("systemid").map(String::as_str), Some("playstation2"));
        assert_eq!(m.get("notes").map(String::as_str), Some("a|b"));
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn a_folder_holds_exactly_one_core() {
        let dir = std::env::temp_dir().join(format!("emusen-libretro-find-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(find_core(&dir).unwrap_err().contains("no libretro core"));
        let suffix = emusen_core_entry::library_suffix();
        std::fs::write(dir.join(format!("one_libretro.{suffix}")), b"").unwrap();
        assert_eq!(find_core(&dir).unwrap().0, "one");
        std::fs::write(dir.join(format!("two_libretro.{suffix}")), b"").unwrap();
        assert!(find_core(&dir).unwrap_err().contains("2 libretro cores"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
