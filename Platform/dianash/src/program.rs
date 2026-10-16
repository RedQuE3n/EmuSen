//! Commands as programs (§7): the ones the install ships beside the shell, each described by a manifest
//! `<name>.dianash.json` of kind `program` read without running anything, and run as a process when named. A program
//! is given argv, the previous stage's output or the terminal, the real directory behind the shell's working
//! directory, and the environment with every exported variable and `DIANASH_ROOT`, `DIANASH_USER` and
//! `DIANASH_SHELL`; it is given no service (§7.2). See EmuSen_DianaSh.md §7 and §13.11.

use crate::dotnet;
use crate::interp::{Interp, Outcome};
use crate::shell::World;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::json::{self, Value};
use emusen_galaxia::tree;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::Duration;

/// A shipped program: the command it answers, its usage line, and the file that runs it.
#[derive(Clone, Debug)]
pub struct Program {
    pub name: String,
    pub usage: String,
    pub read_only: bool,
    pub file: String,
}

/// The programs beside the shell, in the order of their manifests' names, read once per process.
pub fn shipped() -> &'static [Program] {
    static SHIPPED: OnceLock<Vec<Program>> = OnceLock::new();
    SHIPPED.get_or_init(|| read(&tree::shell_modules_directory()))
}

/// The manifests of kind `program` in a folder whose executable is there too.
pub fn read(folder: &str) -> Vec<Program> {
    let Ok(entries) = std::fs::read_dir(folder) else { return Vec::new() };
    let mut manifests: Vec<String> = entries.filter_map(|e| e.ok()).map(|e| e.path().to_string_lossy().into_owned()).filter(|p| p.ends_with(".dianash.json")).collect();
    manifests.sort();
    let mut programs = Vec::new();
    for manifest in manifests {
        let Ok(text) = std::fs::read_to_string(&manifest) else { continue };
        let Ok(value) = json::parse(&text) else { continue };
        if value.get("kind").and_then(Value::as_str) != Some("program") {
            continue;
        }
        let Some(file) = value.get("file").and_then(Value::as_str) else { continue };
        let file = path::combine(Style::HOST, folder, &format!("{file}{}", std::env::consts::EXE_SUFFIX));
        if !std::path::Path::new(&file).is_file() {
            continue;
        }
        if let Some(Value::Array(commands)) = value.get("commands") {
            for command in commands {
                let Some(name) = command.get("name").and_then(Value::as_str) else { continue };
                programs.push(Program {
                    name: name.to_string(),
                    usage: command.get("usage").and_then(Value::as_str).unwrap_or("").to_string(),
                    read_only: matches!(command.get("read_only"), Some(Value::Bool(true))),
                    file: file.clone(),
                });
            }
        }
    }
    programs
}

/// Text a program wrote, the line end it closed with dropped, as the shell adds its own.
fn text(bytes: Vec<u8>) -> String {
    let text = String::from_utf8_lossy(&bytes).into_owned();
    text.strip_suffix("\r\n").or_else(|| text.strip_suffix('\n')).map(str::to_string).unwrap_or(text)
}

/// Runs a program: at a terminal, alone, it owns the terminal; otherwise its input and output are the pipeline's.
pub(crate) fn run(interp: &mut Interp, world: &mut World, program: &Program, args: &[String], stdin: Option<&str>) -> Result<Outcome, String> {
    let mut command = Command::new(&program.file);
    command.args(&args[1..]);
    #[cfg(unix)]
    std::os::unix::process::CommandExt::arg0(&mut command, &args[0]);
    command.current_dir(dotnet::current_directory());
    command.envs(interp.exported_variables());
    command.env("DIANASH_ROOT", world.sandbox.root());
    command.env("DIANASH_USER", world.session(interp.session).map(|s| s.user.clone()).unwrap_or_default());
    command.env("DIANASH_SHELL", crate::shell::version_text());
    let owns_terminal = world.config.interactive && stdin.is_none();
    if owns_terminal {
        command.stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
    } else {
        command.stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
    }
    let mut child = command.spawn().map_err(|e| format!("{}: {e}", program.name))?;
    let writer = child.stdin.take().map(|mut pipe| {
        let input = stdin.unwrap_or("").to_string();
        std::thread::spawn(move || {
            let _ = pipe.write_all(input.as_bytes());
        })
    });
    let reader = |pipe: Option<Box<dyn Read + Send>>| {
        pipe.map(|mut pipe| {
            std::thread::spawn(move || {
                let mut bytes = Vec::new();
                let _ = pipe.read_to_end(&mut bytes);
                bytes
            })
        })
    };
    let out = reader(child.stdout.take().map(|p| Box::new(p) as Box<dyn Read + Send>));
    let err = reader(child.stderr.take().map(|p| Box::new(p) as Box<dyn Read + Send>));
    // A program the player stops, or one past the line's budget, is ended; one that owns the terminal takes Ctrl-C itself.
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        let over = interp.started().is_some_and(|s| s.elapsed() > world.config.budget) && !owns_terminal;
        if world.cancel.load(std::sync::atomic::Ordering::Relaxed) || over {
            let _ = child.kill();
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    if let Some(writer) = writer {
        let _ = writer.join();
    }
    let output = out.and_then(|t| t.join().ok()).map(text).unwrap_or_default();
    let errors = err.and_then(|t| t.join().ok()).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default();
    Ok(Outcome { output, errors, status: exit_status(status), request: None })
}

/// The exit status as `$?` gives it: the code, or 128 and the signal that ended it, as Unix shells give.
fn exit_status(status: std::process::ExitStatus) -> i32 {
    #[cfg(unix)]
    if let Some(signal) = std::os::unix::process::ExitStatusExt::signal(&status) {
        return 128 + signal;
    }
    status.code().unwrap_or(1)
}
