use std::path::PathBuf;

pub fn chdman() -> String {
    ["EMUSEN_DISC_CHDMAN", "EMUSEN_CHDMAN"]
        .into_iter()
        .filter_map(|v| std::env::var(v).ok().filter(|s| !s.is_empty()))
        .chain(std::env::var_os("PATH").into_iter().flat_map(|p| std::env::split_paths(&p).filter_map(|d| d.join("chdman").is_file().then(|| d.join("chdman").to_string_lossy().into_owned())).collect::<Vec<_>>()))
        .chain(std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache/emusen/toolchains/chdman/linux-x64/chdman")).filter(|p| p.is_file()).map(|p| p.to_string_lossy().into_owned()))
        .next()
        .expect("chdman not found (set EMUSEN_DISC_CHDMAN, EMUSEN_CHDMAN, put chdman on PATH, or run Platform/library-tool/build-chdman.sh)")
}
