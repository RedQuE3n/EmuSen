//! The golden corpus (`EmuSen.WiseMan/DianaSh/Golden/`, EmuSen_DianaSh.md §13.4) run through this crate alone:
//! every case recorded with no target whose lines need nothing but what the Rust shell has. A case is passed over
//! when the shell answers one of its lines "Unknown command" where the recording does not, since it then needs a
//! command a host provides; WiseMan runs those through the bridge. The rest must match byte for byte, with their
//! statuses, their continuations and their requests as the C# host actions print them.

use emusen_dianash::shell::{Config, Request, Shell, Submitted};
use std::path::{Path, PathBuf};

#[derive(Default, Debug)]
struct Step {
    input: String,
    output: Option<Vec<u8>>,
    pattern: Option<String>,
    status: i32,
    more: bool,
    action: String,
}

#[derive(Default, Debug)]
struct Case {
    title: String,
    target: String,
    no_sessions: bool,
    files: Vec<(String, Vec<u8>)>,
    steps: Vec<Step>,
}

fn parse(bytes: &[u8]) -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();
    let mut at = 0;
    let line = |at: &mut usize| -> String {
        let end = bytes[*at..].iter().position(|b| *b == b'\n').map_or(bytes.len(), |p| *at + p);
        let text = String::from_utf8_lossy(&bytes[*at..end]).into_owned();
        *at = (end + 1).min(bytes.len());
        text
    };
    let block = |at: &mut usize, count: usize| -> Vec<u8> {
        let block = bytes[*at..*at + count].to_vec();
        *at += count;
        if *at < bytes.len() && bytes[*at] == b'\n' {
            *at += 1;
        }
        block
    };
    while at < bytes.len() {
        let text = line(&mut at);
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        let (word, rest) = text.split_once(' ').unwrap_or((&text, ""));
        let case = cases.last_mut();
        match word {
            "case" => cases.push(Case { title: rest.to_string(), target: "none".to_string(), ..Case::default() }),
            "target" => case.unwrap().target = rest.to_string(),
            "sessions" => case.unwrap().no_sessions = rest == "none",
            "file" => {
                let (path, size) = rest.rsplit_once(' ').unwrap();
                let bytes = block(&mut at, size.parse().unwrap());
                case.unwrap().files.push((path.to_string(), bytes));
            }
            "$" => case.unwrap().steps.push(Step { input: rest.to_string(), ..Step::default() }),
            "out" => case.unwrap().steps.last_mut().unwrap().output = Some(block(&mut at, rest.parse().unwrap())),
            "match" => case.unwrap().steps.last_mut().unwrap().pattern = Some(String::from_utf8(block(&mut at, rest.parse().unwrap())).unwrap()),
            "status" => {
                let step = case.unwrap().steps.last_mut().unwrap();
                let mut parts = rest.split(' ');
                step.status = parts.next().unwrap().parse().unwrap();
                step.more = parts.next() == Some("more");
            }
            "action" => case.unwrap().steps.last_mut().unwrap().action = rest.to_string(),
            _ => {}
        }
    }
    cases
}

/// A shell with the modules a program links, as the platform library and `dianash` build it.
fn shell_with_modules(config: Config) -> Shell {
    let mut shell = Shell::new(config);
    shell.add_module(&dianash_text::MODULE).unwrap();
    shell
}

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../EmuSen.WiseMan/DianaSh/Golden")
}

/// A host action as the C# record prints it.
fn action(request: Option<Request>) -> String {
    match request {
        None => "none".to_string(),
        Some(Request::Exit) => "Shutdown { }".to_string(),
        Some(Request::Continue) => "Resume { }".to_string(),
        Some(Request::Session { name, .. }) => format!("SwitchSession {{ Name = {name} }}"),
        Some(Request::Host(value)) => format!("host {value}"),
    }
}

enum Outcome {
    Passed,
    NeedsHost(String),
    Failed(String),
}

/// Whether a line runs a command only a host has (a word in command position: first, or after `|`, `;`, `&&`, `||`,
/// `$(`, `if`, `then`, `do` and the like), or what it printed was a host's (a suggestion of one, no ROM loaded).
fn names_a_host_command(input: &str, recorded: &str, host: &[String]) -> bool {
    let keywords = ["if", "then", "else", "elif", "do", "while", "until", "!"];
    let mut command_position = true;
    let spaced = input.replace("$(", " ; ").replace(['|', ';', '&', '(', ')', '`'], " ; ");
    for word in spaced.split_whitespace() {
        if word == ";" {
            command_position = true;
            continue;
        }
        let bare = word.trim_matches(|c| c == '"' || c == '\'');
        if command_position && host.iter().any(|h| h == &bare.to_lowercase()) {
            return true;
        }
        command_position = keywords.contains(&bare);
    }
    recorded.contains("No ROM loaded") || host.iter().any(|h| recorded.contains(&format!("'{h}'")))
}

/// One case in a fresh home, as `GoldenRunner` runs it: the skeleton's folders, the case's files, the home as `/` and as the working directory.
fn run(case: &Case, temp: &Path, host: &[String]) -> Outcome {
    let home = temp.join("home");
    let _ = std::fs::remove_dir_all(&home);
    for dir in ["Logs", "Saves/Save States", "Firmware", "Cheats", "Games", "etc/EmuSen", "tmp"] {
        std::fs::create_dir_all(home.join(dir)).unwrap();
    }
    for (path, bytes) in &case.files {
        let full = home.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, bytes).unwrap();
    }
    std::env::set_current_dir(&home).unwrap();
    let root = home.to_string_lossy().into_owned();
    let normalise = |text: &str| text.replace(&root, "<ROOT>").replace(&*temp.to_string_lossy(), "<TEMP>");
    let mut shell = shell_with_modules(Config { sessions: !case.no_sessions, root: Some(root.clone()), ..Config::default() });
    if !case.no_sessions {
        let id = shell.session_new().unwrap();
        shell.session_attach(id, Some("main")).unwrap();
    }
    for (i, step) in case.steps.iter().enumerate() {
        let submitted = shell.submit(0, &step.input).unwrap();
        let output = normalise(&shell.take_output());
        let request = shell.drain_requests().pop();
        let status = shell.last_status(0).unwrap();
        let recorded = step.output.as_deref().map(|o| String::from_utf8_lossy(o).into_owned());
        if output.contains("Unknown command '") && !recorded.as_deref().unwrap_or("").contains("Unknown command '") {
            return Outcome::NeedsHost(format!("{} answers Unknown command", step.input));
        }
        let where_ = format!("'{}', line {} ($ {})", case.title, i + 1, step.input);
        if let Some(recorded) = &recorded {
            if &output != recorded && names_a_host_command(&step.input, recorded, host) {
                return Outcome::NeedsHost(format!("{} runs a host's command", step.input));
            }
            if &output != recorded {
                return Outcome::Failed(format!("{where_}: printed\n{output}\n-- recorded\n{recorded}"));
            }
        }
        if let Some(pattern) = &step.pattern {
            // The recorder's patterns are .NET's, matched with Singleline; the ones it writes are plain enough to read alike here.
            let regex = regex::Regex::new(&format!("(?s){pattern}")).unwrap();
            if !regex.is_match(&output) {
                if names_a_host_command(&step.input, "", host) {
                    return Outcome::NeedsHost(format!("{} runs a host's command", step.input));
                }
                return Outcome::Failed(format!("'{}', line {} ($ {}): printed\n{output}\n-- does not match\n{pattern}", case.title, i + 1, step.input));
            }
        }
        if status != step.status || (submitted == Submitted::NeedsMore) != step.more || action(request.clone()) != step.action {
            return Outcome::Failed(format!("{where_}: status {status} more {:?} action {} - recorded {} {} {}", submitted, action(request), step.status, step.more, step.action));
        }
    }
    Outcome::Passed
}

#[test]
fn every_case_the_rust_shell_can_run_alone_prints_its_recording() {
    let temp = std::env::temp_dir().join(format!("dianash-golden-{}", std::process::id()));
    std::fs::create_dir_all(&temp).unwrap();
    let mut files: Vec<PathBuf> = std::fs::read_dir(corpus()).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "golden")).collect();
    files.sort();
    // Every command the corpus records that the Rust shell does not answer itself: a host's, through the bridge.
    let own: Vec<String> = shell_with_modules(Config::default()).own_names();
    let host: Vec<String> = files
        .iter()
        .map(|f| f.file_stem().unwrap().to_string_lossy().into_owned())
        .filter(|name| !own.contains(name) && !["help", "man", "export", "unset", "source", "shell", "bracket"].contains(&name.as_str()))
        .collect();
    let (mut passed, mut needs_host, mut failures) = (0, 0, Vec::new());
    for file in files {
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        // help and man list and read what a host ships beside the program; WiseMan runs them.
        if name == "help.golden" || name == "man.golden" {
            continue;
        }
        for case in parse(&std::fs::read(&file).unwrap()).iter().filter(|c| c.target == "none") {
            match run(case, &temp, &host) {
                Outcome::Passed => passed += 1,
                Outcome::NeedsHost(why) => {
                    needs_host += 1;
                    if std::env::var_os("DIANASH_GOLDEN_VERBOSE").is_some() {
                        eprintln!("needs a host: {name} '{}': {why}", case.title);
                    }
                }
                Outcome::Failed(why) => failures.push(format!("{name}: {why}")),
            }
        }
    }
    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    let _ = std::fs::remove_dir_all(&temp);
    eprintln!("golden cases run by the Rust shell alone: {passed} passed, {needs_host} need a host's command, {} failed", failures.len());
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    assert!(passed > 0);
}
