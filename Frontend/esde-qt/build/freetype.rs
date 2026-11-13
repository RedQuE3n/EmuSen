//! Where the build links the system's FreeType from, libfreetype.so.6 by its run-time name, and the words when it is not
//! there to link (EmuSen_Qt.md §4.12.4). Read by build.rs and by tests/freetype_link.rs.

/// The library's run-time name, the file Qt already loads.
pub const LIBRARY: &str = "libfreetype.so.6";

/// The folders to search and a warning to print, from pkg-config's `--libs-only-L freetype2` (its output, or why it gave
/// none) and `ldconfig -p`'s listing (none where it did not run); an error naming what is missing when neither has it.
pub fn plan(pkg_config: Result<String, String>, ldconfig: Option<String>) -> Result<(Vec<String>, Option<String>), String> {
    match (pkg_config, ldconfig) {
        (Ok(libs), _) => Ok((libs.split_whitespace().filter_map(|a| a.strip_prefix("-L")).map(str::to_owned).collect(), None)),
        (Err(why), Some(listing)) if listing.contains(LIBRARY) => {
            Ok((vec![], Some(format!("pkg-config does not know freetype2 ({why}); {LIBRARY} is linked from the folders ldconfig lists"))))
        }
        (Err(why), Some(_)) => Err(format!(
            "emusen-esde-qt links the system's FreeType, {LIBRARY}, which neither pkg-config ({why}) nor ldconfig finds: install FreeType \
             (Fedora's freetype, Debian's libfreetype6) and build again"
        )),
        (Err(why), None) => Ok((vec![], Some(format!("pkg-config does not know freetype2 ({why}) and ldconfig did not run; a link error naming {LIBRARY} means FreeType is not installed")))),
    }
}
