//! `esde-check`: EmuSen_Qt.md §4.4's corpus measure as an emusen.dev command, over the `check` module of Frontend/esde.

use std::path::PathBuf;

use emusen_esde::check::{SYSTEMS, Table, report};

const USAGE: &str = "Usage: esde-check [--diagnostics | --properties] [--systems a,b,...] <folder>...\n  Exit 1 when a theme does not load, 0 when all do, 2 when it could not run. See man esde-check.";

/// Measures the themes under each folder, each path resolved by the shell; the table as TSV and whether every theme loaded.
pub fn run(
    args: &[String],
    resolve: impl Fn(&str) -> Result<String, String>,
) -> Result<(String, i32), String> {
    let mut table = Table::Summary;
    let mut systems: Vec<String> = SYSTEMS.iter().map(|s| s.to_string()).collect();
    let mut folders = Vec::new();
    let mut rest = args.iter().skip(1);
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--diagnostics" => table = Table::Diagnostics,
            "--properties" => table = Table::Properties,
            "--systems" => {
                let list = rest
                    .next()
                    .ok_or_else(|| format!("esde-check: --systems needs a list\n{USAGE}"))?;
                systems = list
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect();
                if systems.is_empty() {
                    return Err(format!("esde-check: --systems needs a list\n{USAGE}"));
                }
            }
            "--help" => return Err(USAGE.to_owned()),
            other if other.starts_with("--") => {
                return Err(format!("esde-check: {other} is not an option\n{USAGE}"));
            }
            folder => folders.push(folder.to_owned()),
        }
    }
    if folders.is_empty() {
        return Err(format!("esde-check: a folder is needed\n{USAGE}"));
    }
    let mut paths = Vec::new();
    for given in &folders {
        let real = PathBuf::from(resolve(given)?);
        if !real.is_dir() {
            return Err(format!("esde-check: {given}: no such folder"));
        }
        paths.push(real);
    }
    let names: Vec<&str> = systems.iter().map(String::as_str).collect();
    let measured = report(&paths, &names, table);
    Ok((measured.text, i32::from(!measured.all_load)))
}

#[cfg(test)]
mod tests {
    use super::run;

    fn plain(path: &str) -> Result<String, String> {
        Ok(path.to_owned())
    }

    #[test]
    fn a_folder_is_needed_and_options_are_checked() {
        assert!(run(&["esde-check".into()], plain).is_err());
        assert!(
            run(&["esde-check".into(), "--loud".into(), ".".into()], plain)
                .unwrap_err()
                .contains("not an option")
        );
        assert!(
            run(&["esde-check".into(), "/no/such/folder".into()], plain)
                .unwrap_err()
                .contains("no such folder")
        );
    }

    #[test]
    fn a_theme_that_loads_is_a_row_and_exits_0_and_one_that_does_not_exits_1() {
        let root = std::env::temp_dir().join(format!("esde-check-{}", std::process::id()));
        let good = root.join("good");
        let bad = root.join("bad");
        std::fs::create_dir_all(&good).unwrap();
        std::fs::create_dir_all(&bad).unwrap();
        std::fs::write(good.join("capabilities.xml"), "<themeCapabilities/>").unwrap();
        std::fs::write(good.join("theme.xml"), "<theme><view name=\"gamelist\"><text name=\"t\"><text>t</text><size>1 1</size></text></view></theme>").unwrap();
        std::fs::write(bad.join("capabilities.xml"), "<themeCapabilities/>").unwrap();
        let (out, status) = run(
            &[
                "esde-check".into(),
                "--systems".into(),
                "snes".into(),
                good.display().to_string(),
            ],
            plain,
        )
        .unwrap();
        assert_eq!(status, 0, "{out}");
        assert!(
            out.lines().nth(1).unwrap().starts_with("good\tyes\t1/1\t"),
            "{out}"
        );
        let (out, status) = run(
            &[
                "esde-check".into(),
                "--systems".into(),
                "snes".into(),
                root.display().to_string(),
            ],
            plain,
        )
        .unwrap();
        assert_eq!(status, 1, "{out}");
        assert!(
            out.ends_with("# 2 themes, 1 load, 1 drawn whole, 2 use no undrawn kind\n"),
            "{out}"
        );
        let (out, _) = run(
            &[
                "esde-check".into(),
                "--properties".into(),
                "--systems".into(),
                "snes".into(),
                good.display().to_string(),
            ],
            plain,
        )
        .unwrap();
        assert!(
            out.contains("good\ttext.size\tset\n") && out.contains("good\ttext.text\tset\n"),
            "{out}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
