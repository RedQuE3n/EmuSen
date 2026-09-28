// The source a PGO profile was trained on, included by build.rs and by src/tests/pgo_digest.rs - see Mars_Native.md §6.17.

// The `path = "..."` dependencies a Cargo.toml names, in its order.
fn path_dependencies(manifest: &str) -> Vec<String> {
    manifest
        .lines()
        .filter_map(|line| line.split_once("path = \"").and_then(|(_, rest)| rest.split_once('"')).map(|(p, _)| p.to_string()))
        .collect()
}

// FNV-1a over what the library is built from: its src/ and each path dependency's, tests left out, Cargo.toml and Cargo.lock, by path, carriage returns dropped.
fn source_digest(root: &Path) -> u64 {
    let mut named: Vec<(String, PathBuf)> = Vec::new();
    let mut add = |prefix: &str, base: &Path, lock: bool| {
        let mut files = vec![base.join("Cargo.toml")];
        if lock {
            files.push(base.join("Cargo.lock"));
        }
        collect(&base.join("src"), &mut files);
        for p in files {
            let rel = p.strip_prefix(base).unwrap_or(&p).to_string_lossy().replace('\\', "/");
            named.push((format!("{prefix}{rel}"), p));
        }
    };
    add("", root, true);
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
    for dependency in path_dependencies(&manifest) {
        add(&format!("{dependency}/"), &root.join(&dependency), false);
    }
    named.sort();
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut feed = |bytes: &[u8]| {
        for &b in bytes {
            if b != b'\r' {
                hash = (hash ^ b as u64).wrapping_mul(0x0100_0000_01b3);
            }
        }
    };
    for (name, path) in named {
        feed(name.as_bytes());
        feed(&[0]);
        feed(&fs::read(&path).unwrap_or_default());
        feed(&[0]);
    }
    hash
}

// Every file under `dir` but src/tests/ and src/tests.rs, which no library is built from.
fn collect(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let top = path.parent().and_then(Path::file_name).is_some_and(|n| n == "src");
        if top && (path.file_name().is_some_and(|n| n == "tests" || n == "tests.rs")) {
            continue;
        }
        if path.is_dir() {
            collect(&path, into);
        } else {
            into.push(path);
        }
    }
}
