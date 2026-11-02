//! The build graph at a revision, read from its files: every Cargo crate with its workspace and path dependencies,
//! every MSBuild project with its project references and imports, and the host's edges from a crate to the project
//! that builds it in (RustCores.props, Platform.targets). One graph, so a reverse closure crosses the two languages.
//! The cargo half agrees with `cargo metadata --no-deps` (held by a test); it is read from git objects so that a branch
//! not checked out is read as it is. See EmuSen_Testing.md §3.2.

use crate::repo::{join, parent, Repo};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

#[derive(Clone, Debug, Default)]
pub struct Crate {
    pub dir: String,
    pub name: Option<String>,
    /// The workspace root's folder when this crate is a member; its own folder when it is a root.
    pub root: String,
    pub virtual_root: bool,
}

#[derive(Default)]
pub struct Graph {
    pub crates: BTreeMap<String, Crate>,
    /// MSBuild projects by path, and whether each is a test project.
    pub projects: BTreeMap<String, bool>,
    /// node -> the nodes it depends on; a node is a crate folder, a project path or an imported file's path.
    pub deps: HashMap<String, BTreeSet<String>>,
    /// node -> the nodes that depend on it.
    pub rdeps: HashMap<String, BTreeSet<String>>,
}

/// What one Cargo.toml says.
#[derive(Debug, Default, PartialEq)]
pub struct Manifest {
    pub name: Option<String>,
    pub members: Option<Vec<String>>,
    pub has_package: bool,
    pub path_deps: Vec<(String, String)>,
    pub workspace_deps: Vec<(String, String)>,
    pub inherited: Vec<String>,
}

fn quoted(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('"') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('"') else { break };
        out.push(after[..end].to_string());
        rest = &after[end + 1..];
    }
    out
}

fn value_of<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let mut search = line;
    while let Some(i) = search.find(key) {
        let before_ok = i == 0 || !search.as_bytes()[i - 1].is_ascii_alphanumeric() && search.as_bytes()[i - 1] != b'-' && search.as_bytes()[i - 1] != b'_';
        let after = search[i + key.len()..].trim_start();
        if before_ok && after.starts_with('=') {
            let v = after[1..].trim_start();
            if let Some(stripped) = v.strip_prefix('"') {
                return stripped.find('"').map(|e| &stripped[..e]);
            }
            return Some(v.split([',', '}', ' ']).next().unwrap_or(""));
        }
        search = &search[i + key.len()..];
    }
    None
}

/// The parts of a Cargo.toml the graph needs: package name, workspace members, path dependencies in any dependency
/// table (target-specific, dev and build included), workspace dependency declarations and the ones inherited.
pub fn parse_manifest(text: &str) -> Manifest {
    let mut m = Manifest::default();
    let mut section = String::new();
    let mut members: Option<String> = None;
    for raw in text.lines() {
        let line = raw.split(" #").next().unwrap_or("").trim();
        if let Some(acc) = members.as_mut() {
            acc.push_str(line);
            if line.contains(']') {
                m.members = Some(quoted(acc));
                members = None;
            }
            continue;
        }
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            section = line.trim_matches(['[', ']']).trim().to_string();
            if section == "package" {
                m.has_package = true;
            }
            continue;
        }
        let key = line.split('=').next().unwrap_or("").trim().trim_matches('"').to_string();
        if section == "package" && key == "name" {
            m.name = value_of(line, "name").map(str::to_string);
        } else if section == "workspace" && key == "members" {
            if line.contains(']') {
                m.members = Some(quoted(line.split_once('=').map_or("", |x| x.1)));
            } else {
                members = Some(line.split_once('=').map_or("", |x| x.1).to_string());
            }
        } else if section == "workspace.dependencies" {
            if let Some(p) = value_of(line, "path") {
                m.workspace_deps.push((key.clone(), p.to_string()));
            }
        } else if section.ends_with("dependencies") || section.starts_with("patch.") {
            let dep = key.split('.').next().unwrap_or("").to_string();
            if let Some(p) = value_of(line, "path") {
                m.path_deps.push((dep, p.to_string()));
            } else if value_of(line, "workspace") == Some("true") || key.ends_with(".workspace") {
                m.inherited.push(dep);
            }
        } else if let Some(dep) = section.rsplit_once("dependencies.").map(|(_, d)| d.to_string()) {
            if key == "path" {
                if let Some(p) = value_of(line, "path") {
                    m.path_deps.push((dep, p.to_string()));
                }
            } else if key == "workspace" && value_of(line, "workspace") == Some("true") {
                m.inherited.push(dep);
            }
        }
    }
    m
}

/// Every path a project file names that is a file or folder of the repository: references, imports and, for the host,
/// a crate's folder named in a property (`$(MSBuildThisFileDirectory)` read as the file's own folder).
fn msbuild_paths(file: &str, text: &str) -> Vec<String> {
    let dir = parent(file);
    let mut out = Vec::new();
    let mut take = |raw: &str| {
        let v = raw.replace("$(MSBuildThisFileDirectory)", "").replace("$(MSBuildProjectDirectory)", "");
        if v.contains("$(") || v.contains('*') || v.trim().is_empty() || v.contains(';') {
            return;
        }
        out.push(join(dir, v.trim()));
    };
    for cap in text.split('"').skip(1).step_by(2) {
        take(cap);
    }
    for piece in text.split('>').skip(1) {
        if let Some(t) = piece.split('<').next() {
            if t.contains('/') || t.contains('\\') {
                take(t);
            }
        }
    }
    out
}

fn add_edge(g: &mut Graph, from: &str, to: &str) {
    if from == to {
        return;
    }
    g.deps.entry(from.to_string()).or_default().insert(to.to_string());
    g.rdeps.entry(to.to_string()).or_default().insert(from.to_string());
}

impl Graph {
    /// The graph of `rev`, given its file list.
    pub fn read(repo: &Repo, rev: &str, files: &[String]) -> Result<Graph, String> {
        let wanted: Vec<String> = files
            .iter()
            .filter(|f| f.ends_with("Cargo.toml") && (f.as_str() == "Cargo.toml" || f.ends_with("/Cargo.toml")) || f.ends_with(".csproj") || f.ends_with(".props") || f.ends_with(".targets"))
            .cloned()
            .collect();
        let texts = repo.read_many(rev, &wanted)?;
        Ok(Graph::from_texts(&texts, files))
    }

    pub fn from_texts(texts: &HashMap<String, String>, files: &[String]) -> Graph {
        let mut g = Graph::default();
        let manifests: BTreeMap<String, Manifest> = texts.iter().filter(|(p, _)| p.ends_with("Cargo.toml")).map(|(p, t)| (parent(p).to_string(), parse_manifest(t))).collect();
        for (dir, m) in &manifests {
            g.crates.insert(dir.clone(), Crate { dir: dir.clone(), name: m.name.clone(), root: dir.clone(), virtual_root: m.members.is_some() && !m.has_package });
        }
        let dirs: BTreeSet<String> = manifests.keys().cloned().collect();
        for (root, m) in &manifests {
            let Some(members) = &m.members else { continue };
            for member in members {
                let pattern = join(root, member);
                let matched: Vec<String> = if let Some(prefix) = pattern.strip_suffix("/*") {
                    dirs.iter().filter(|d| parent(d) == prefix).cloned().collect()
                } else {
                    dirs.iter().filter(|d| **d == pattern).cloned().collect()
                };
                for d in matched {
                    if let Some(c) = g.crates.get_mut(&d) {
                        c.root = root.clone();
                    }
                }
            }
        }
        // A member depends on its workspace's manifest and lock, which the virtual root's folder stands for.
        let members: Vec<(String, String)> = g.crates.values().filter(|c| c.root != c.dir).map(|c| (c.dir.clone(), c.root.clone())).collect();
        for (dir, root) in members {
            add_edge(&mut g, &dir, &root);
        }
        for (dir, m) in &manifests {
            let root = g.crates[dir].root.clone();
            let wsdeps: HashMap<String, String> = manifests.get(&root).map(|r| r.workspace_deps.iter().map(|(n, p)| (n.clone(), join(&root, p))).collect()).unwrap_or_default();
            for (_, p) in &m.path_deps {
                let to = join(dir, p);
                if dirs.contains(&to) {
                    add_edge(&mut g, dir, &to);
                }
            }
            for n in &m.inherited {
                if let Some(to) = wsdeps.get(n) {
                    if dirs.contains(to) {
                        add_edge(&mut g, dir, to);
                    }
                }
            }
        }
        // MSBuild: references and imports, and the host's crate edges.
        let filenames: BTreeSet<&str> = files.iter().map(String::as_str).collect();
        let root_imports: Vec<String> = ["Directory.Build.props", "Directory.Build.targets"].iter().filter(|f| filenames.contains(**f)).map(|f| f.to_string()).collect();
        let msbuild: Vec<(&String, &String)> = texts.iter().filter(|(p, _)| !p.ends_with("Cargo.toml")).collect();
        for (p, t) in &msbuild {
            if p.ends_with(".csproj") {
                let test = t.contains("xunit") || t.contains("Microsoft.NET.Test.Sdk") || t.contains("<IsTestProject>true");
                g.projects.insert((*p).clone(), test);
                for imp in &root_imports {
                    add_edge(&mut g, p, imp);
                }
            }
            for named in msbuild_paths(p, t) {
                let named = named.trim_end_matches('/').to_string();
                if named.ends_with(".csproj") || named.ends_with(".props") || named.ends_with(".targets") {
                    if filenames.contains(named.as_str()) {
                        add_edge(&mut g, p, &named);
                    }
                } else if let Some(dir) = named.strip_suffix("/Cargo.toml").map(str::to_string).or(Some(named.clone())) {
                    if dirs.contains(&dir) {
                        add_edge(&mut g, p, &dir);
                    }
                }
            }
        }
        // Platform.targets builds the platform library, the workspace crate that names it.
        for (p, _) in &msbuild {
            if p.ends_with("Platform.targets") {
                let ws = parent(p);
                if let Some(c) = g.crates.values().find(|c| c.root == ws && c.name.as_deref() == Some("emusen-platform")) {
                    let d = c.dir.clone();
                    add_edge(&mut g, p, &d);
                }
            }
        }
        g
    }

    /// The node a file belongs to: its nearest crate folder or project folder, or the file itself when it is an
    /// imported MSBuild file or a project.
    pub fn owner(&self, file: &str) -> Option<String> {
        if self.rdeps.contains_key(file) || self.projects.contains_key(file) {
            return Some(file.to_string());
        }
        let mut d = parent(file);
        loop {
            if self.crates.contains_key(d) {
                return Some(d.to_string());
            }
            if let Some(p) = self.projects.keys().find(|p| parent(p) == d) {
                return Some(p.clone());
            }
            if d.is_empty() {
                return None;
            }
            d = parent(d);
        }
    }

    /// The nodes that can see a change to `start`, itself first, up to `depth` steps (none for all).
    pub fn reverse_closure(&self, start: &str, depth: Option<usize>) -> Vec<String> {
        let mut seen = BTreeSet::from([start.to_string()]);
        let mut order = vec![start.to_string()];
        let mut queue = VecDeque::from([(start.to_string(), 0usize)]);
        while let Some((n, d)) = queue.pop_front() {
            if depth.is_some_and(|max| d >= max) {
                continue;
            }
            for r in self.rdeps.get(&n).into_iter().flatten() {
                if seen.insert(r.clone()) {
                    order.push(r.clone());
                    queue.push_back((r.clone(), d + 1));
                }
            }
        }
        order
    }

    /// Every node `start` depends on, itself included.
    pub fn forward_closure(&self, start: &str) -> BTreeSet<String> {
        let mut seen = BTreeSet::from([start.to_string()]);
        let mut queue = VecDeque::from([start.to_string()]);
        while let Some(n) = queue.pop_front() {
            for d in self.deps.get(&n).into_iter().flatten() {
                if seen.insert(d.clone()) {
                    queue.push_back(d.clone());
                }
            }
        }
        seen
    }

    pub fn is_crate(&self, node: &str) -> bool {
        self.crates.contains_key(node)
    }

    pub fn is_test_project(&self, node: &str) -> bool {
        self.projects.get(node).copied().unwrap_or(false)
    }

    /// The roots: crates that are no workspace's member, a virtual workspace counting once.
    pub fn roots(&self) -> Vec<&Crate> {
        self.crates.values().filter(|c| c.root == c.dir).collect()
    }

    pub fn members_of(&self, root: &str) -> Vec<&Crate> {
        self.crates.values().filter(|c| c.root == root && (c.dir != root || !c.virtual_root)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_manifest_gives_its_name_members_and_path_dependencies() {
        let m = parse_manifest(
            "[package]\nname = \"core\"\n\n[dependencies]\nnative = { path = \"../native\", version = \"0.1\" }\nlibc = \"0.2\"\nshared.workspace = true\n\n[target.'cfg(unix)'.dependencies]\nhw = { path = \"../hw\" }\n\n[dependencies.extra]\npath = \"x\"\n\n[lib]\npath = \"src/lib.rs\"\n",
        );
        assert_eq!(m.name.as_deref(), Some("core"));
        assert_eq!(m.path_deps, vec![("native".into(), "../native".into()), ("hw".into(), "../hw".into()), ("extra".into(), "x".into())]);
        assert_eq!(m.inherited, vec!["shared".to_string()]);
        let w = parse_manifest("[workspace]\nmembers = [\"a\",\n  \"b\", # the second\n]\n[workspace.dependencies]\nshared = { path = \"a\" }\n");
        assert_eq!(w.members, Some(vec!["a".into(), "b".into()]));
        assert_eq!(w.workspace_deps, vec![("shared".into(), "a".into())]);
        assert!(!w.has_package);
    }
}
