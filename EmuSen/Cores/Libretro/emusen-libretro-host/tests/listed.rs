// A core on the hosted list placed under the systems its list line names, not every system its .info's databases name
// (EmuSen_Cores.md §4.7): the test core laid out as picodrive, whose .info names the Genesis, the Sega CD, the Master
// System and the 32X, and whose line names the 32X alone.

use std::path::PathBuf;

use emusen_core_host::library::Library;

#[test]
fn a_listed_core_runs_only_the_systems_its_line_names() {
    let suffix = emusen_core_entry::library_suffix();
    let deps = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let entry = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("listed-{}", std::process::id())).join("libretro").join("picodrive");
    let _ = std::fs::remove_dir_all(&entry);
    std::fs::create_dir_all(&entry).unwrap();
    let host = entry.join(format!("emusen-libretro-host.{suffix}"));
    std::fs::copy(deps.join(format!("libemusen_libretro_host.{suffix}")), &host).unwrap();
    std::fs::copy(deps.join(format!("libemusen_libretro_testcore.{suffix}")), entry.join(format!("picodrive_libretro.{suffix}"))).unwrap();
    std::fs::write(
        entry.join("picodrive_libretro.info"),
        "display_name = \"PicoDrive\"\ncorename = \"PicoDrive\"\nlicense = \"MAME\"\nsystemid = \"mega_drive\"\ndatabase = \"Sega - Master System - Mark III|Sega - Mega-CD - Sega CD|Sega - Mega Drive - Genesis|Sega - 32X\"\n",
    )
    .unwrap();
    let lib = Library::load(&host).unwrap_or_else(|e| panic!("{e:?}"));
    let info = lib.info_text().unwrap();
    let systems: Vec<&str> = info.match_indices(r#""id":""#).map(|(at, _)| &info[at + 6..]).map(|s| &s[..s.find('"').unwrap()]).filter(|id| !id.starts_with("libretro-") && *id != "retropad").collect();
    assert_eq!(systems, ["32x"], "{info}");
    let _ = std::fs::remove_dir_all(entry.parent().unwrap().parent().unwrap());
}
