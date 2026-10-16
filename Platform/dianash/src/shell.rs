//! A shell: its sessions, each an interpreter with its own variables, history and account, the accounts they share,
//! the host's commands, and what a submitted line printed and asked of the host. The host API of `dianash.h` is a
//! C layer over this. See EmuSen_DianaSh.md §4.1 and §10.

use crate::abi::{self, dianash_host_commands, status, HostJobsFn, HostKillFn, HostRunFn, ReleaseFn};
use crate::accounts::Accounts;
use crate::dotnet;
use crate::interp::Interp;
use crate::sandbox::Sandbox;
use crate::table::{Entry, Kind, Table};
use dianash_module::Module;
use emusen_galaxia::json::{self, Value};
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// How a host builds a shell.
#[derive(Clone, Debug)]
pub struct Config {
    /// Whether the shell keeps named sessions (`tmux`), or is one session with no manager, as a host without `tmux` builds it.
    pub sessions: bool,
    /// Whether the shell runs at a terminal, which a console window is not.
    pub interactive: bool,
    /// The sandbox's root; none is Galaxia's home.
    pub root: Option<String>,
    /// How long a line's loops may run before they are stopped.
    pub budget: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config { sessions: true, interactive: false, root: None, budget: Duration::from_secs(10) }
    }
}

/// What only the host can do, raised by a line: the last one a line raised is the one it gives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// `shutdown`, `quit`.
    Exit,
    /// The current session changed; `name` as the command gave it.
    Session { id: u32, name: String },
    /// A bare `continue` reached the prompt, outside any loop.
    Continue,
    /// A host's command raised a request of the host's own.
    Host(u64),
}

/// A line's history, as readline keeps it: verbatim, not deduplicated, capped.
#[derive(Clone, Debug, Default)]
pub struct History {
    entries: Vec<String>,
}

impl History {
    const MAX: usize = 500;

    pub fn add(&mut self, line: &str) {
        if line.is_empty() {
            return;
        }
        self.entries.push(line.to_string());
        if self.entries.len() > Self::MAX {
            self.entries.remove(0);
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}

/// One pointer of state a host registration keeps per session, and how the shell releases it.
pub(crate) struct Slot {
    pub state: *mut c_void,
    pub release: Option<ReleaseFn>,
}

impl Drop for Slot {
    fn drop(&mut self) {
        if let Some(release) = self.release {
            if !self.state.is_null() {
                unsafe { release(self.state) };
            }
        }
    }
}

pub(crate) struct Session {
    pub id: u32,
    /// None until a host lists it under a name.
    pub name: Option<String>,
    pub user: String,
    pub history: History,
    pub table: Arc<Table>,
    pub slots: HashMap<usize, Slot>,
}

/// A host's registration: the functions that serve its commands, and the context they are given.
pub(crate) struct Host {
    pub ctx: *mut c_void,
    pub run: HostRunFn,
    pub jobs: Option<HostJobsFn>,
    pub kill: Option<HostKillFn>,
}

/// Everything a line may read or change but the interpreter running it.
pub(crate) struct World {
    pub config: Config,
    pub sandbox: Sandbox,
    pub sessions: Vec<Session>,
    pub current: Option<u32>,
    pub next_id: u32,
    /// `tmux new`'s next default name, `session-N`.
    pub next_default: u32,
    pub accounts: Accounts,
    pub hosts: Vec<Host>,
    pub requests: Vec<Request>,
    pub cancel: Arc<AtomicBool>,
    /// The modules linked into the program, by index.
    pub modules: Vec<&'static dyn Module>,
    /// The table a new session starts from: the builtins and the linked modules' commands.
    pub base: Arc<Table>,
    /// The host's services, copied into every call, never read (§9.8); the names they point at are `service_names`.
    pub services: Vec<abi::dianash_service>,
    pub service_names: Vec<Box<[u8]>>,
}

impl World {
    pub fn session(&self, id: u32) -> Option<&Session> {
        self.sessions.iter().find(|s| s.id == id)
    }

    pub fn session_mut(&mut self, id: u32) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|s| s.id == id)
    }

    /// The sessions a host has listed, in order.
    pub fn listed(&self) -> impl Iterator<Item = &Session> {
        self.sessions.iter().filter(|s| s.name.is_some())
    }

    pub fn find_listed(&self, name: &str) -> Option<u32> {
        self.listed().find(|s| s.name.as_deref().is_some_and(|n| dotnet::same(n, name))).map(|s| s.id)
    }

    pub fn current_listed(&self) -> Option<&Session> {
        self.current.and_then(|id| self.listed().find(|s| s.id == id))
    }

    /// A new session, unlisted, with its own variables, history and account.
    pub fn new_session(&mut self, table: Arc<Table>, user: String) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.sessions.push(Session { id, name: None, user, history: History::default(), table, slots: HashMap::new() });
        id
    }

    /// `NextDefaultName`: `session-N`, skipping a name in use.
    pub fn next_default_name(&mut self) -> String {
        loop {
            let name = format!("session-{}", self.next_default);
            self.next_default += 1;
            if self.find_listed(&name).is_none() {
                return name;
            }
        }
    }

    /// Lists a session under a name, or the next default one, and makes it current.
    pub fn attach(&mut self, id: u32, name: Option<&str>) -> Result<String, (i32, String)> {
        let name = match name {
            Some(name) => name.to_string(),
            None => self.next_default_name(),
        };
        if self.find_listed(&name).is_some() {
            return Err((status::EXISTS, format!("a session named '{name}' already exists")));
        }
        let session = self.sessions.iter_mut().find(|s| s.id == id).ok_or((status::NO_SUCH_SESSION, format!("no session {id}")))?;
        session.name = Some(name.clone());
        self.current = Some(id);
        Ok(name)
    }

    /// `TryKill`: never the last listed session; a current one hands over to the first that remains. A listed session
    /// killed is only unlisted, since a host may still hold it and the C# interpreter it stands for went on answering;
    /// an unlisted one ends.
    pub fn kill(&mut self, id: u32) -> Result<(), (i32, String)> {
        let at = self.sessions.iter().position(|s| s.id == id).ok_or((status::NO_SUCH_SESSION, format!("no session {id}")))?;
        if self.sessions[at].name.is_none() {
            self.sessions.remove(at);
            return Ok(());
        }
        if self.listed().count() == 1 {
            return Err((status::LAST_SESSION, "can't kill the only remaining session".to_string()));
        }
        self.sessions[at].name = None;
        if self.current == Some(id) {
            let first = self.listed().next().map(|s| s.id);
            self.current = first;
        }
        Ok(())
    }
}

/// A shell, as a host holds it.
pub struct Shell {
    pub(crate) world: World,
    pub(crate) interps: HashMap<u32, Interp>,
    /// What the last submit printed.
    output: String,
    /// Whether a line is running, for a host that calls in again from inside one of its commands.
    busy: bool,
}

/// What `submit` answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Submitted {
    Done,
    /// A block, a quote or a `$(...)` is still open.
    NeedsMore,
    /// The line asked the shell to end.
    Exit,
}

impl Shell {
    pub fn new(config: Config) -> Shell {
        let sandbox = Sandbox { root: config.root.clone() };
        let mut world = World {
            config,
            sandbox,
            sessions: Vec::new(),
            current: None,
            next_id: 1,
            next_default: 1,
            accounts: Accounts::default(),
            hosts: Vec::new(),
            requests: Vec::new(),
            cancel: Arc::new(AtomicBool::new(false)),
            services: Vec::new(),
            service_names: Vec::new(),
            modules: Vec::new(),
            base: Table::builtins(),
        };
        let mut interps = HashMap::new();
        if !world.config.sessions {
            // A shell with no manager is one session, current and never listed.
            let id = world.new_session(world.base.clone(), "root".to_string());
            world.current = Some(id);
            interps.insert(id, Interp::new(id));
        }
        Shell { world, interps, output: String::new(), busy: false }
    }

    /// The session a host means: its id, or 0 for the current one.
    fn resolve(&self, session: u32) -> Option<u32> {
        let id = if session == 0 { self.world.current? } else { session };
        self.world.sessions.iter().any(|s| s.id == id).then_some(id)
    }

    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        self.world.cancel.clone()
    }

    pub fn is_busy(&self) -> bool {
        self.busy
    }

    /// Runs a line in a session, gathering what it printed for `take_output` and what it asked for `drain_requests`.
    pub fn submit(&mut self, session: u32, line: &str) -> Result<Submitted, i32> {
        let id = self.resolve(session).ok_or(status::NO_SUCH_SESSION)?;
        let mut interp = self.interps.remove(&id).unwrap_or_else(|| Interp::new(id));
        self.world.cancel.store(false, Ordering::Relaxed);
        self.busy = true;
        let (needs_more, output, request) = interp.submit_core(&mut self.world, line, true);
        self.busy = false;
        if self.world.sessions.iter().any(|s| s.id == id) {
            self.interps.insert(id, interp);
        }
        self.interps.retain(|id, _| self.world.sessions.iter().any(|s| s.id == *id));
        self.output = output;
        let exit = request == Some(Request::Exit);
        if let Some(request) = request {
            self.world.requests.push(request);
        }
        Ok(if needs_more {
            Submitted::NeedsMore
        } else if exit {
            Submitted::Exit
        } else {
            Submitted::Done
        })
    }

    pub fn take_output(&mut self) -> String {
        std::mem::take(&mut self.output)
    }

    pub fn output(&self) -> &str {
        &self.output
    }

    pub fn drain_requests(&mut self) -> Vec<Request> {
        std::mem::take(&mut self.world.requests)
    }

    /// Puts drained requests back in front, for a host whose buffer did not hold them.
    pub fn restore_requests(&mut self, mut requests: Vec<Request>) {
        requests.append(&mut self.world.requests);
        self.world.requests = requests;
    }

    /// The requests as a host reads them: a JSON array of `{"kind", ...}` objects.
    pub fn requests_json(requests: &[Request]) -> String {
        let items: Vec<String> = requests
            .iter()
            .map(|r| match r {
                Request::Exit => r#"{"kind":"exit"}"#.to_string(),
                Request::Continue => r#"{"kind":"continue"}"#.to_string(),
                Request::Session { id, name } => format!(r#"{{"kind":"session","id":{id},"name":{}}}"#, json_string(name)),
                Request::Host(value) => format!(r#"{{"kind":"host","value":{value}}}"#),
            })
            .collect();
        format!("[{}]", items.join(","))
    }

    fn interp(&mut self, id: u32) -> &mut Interp {
        self.interps.entry(id).or_insert_with(|| Interp::new(id))
    }

    /// `$?` of a session's last line.
    pub fn last_status(&mut self, session: u32) -> Result<i32, i32> {
        let id = self.resolve(session).ok_or(status::NO_SUCH_SESSION)?;
        Ok(self.interp(id).last_status())
    }

    /// Whether a line is a read-only command that may run on a thread that must not wait (§6.5).
    pub fn is_read_only(&mut self, session: u32, line: &str) -> Result<bool, i32> {
        let id = self.resolve(session).ok_or(status::NO_SUCH_SESSION)?;
        let table = self.world.session(id).map(|s| s.table.clone()).ok_or(status::NO_SUCH_SESSION)?;
        Ok(self.interp(id).is_read_only(&table, line))
    }

    pub fn awaiting(&mut self, session: u32) -> Result<bool, i32> {
        let id = self.resolve(session).ok_or(status::NO_SUCH_SESSION)?;
        Ok(self.interp(id).awaiting())
    }

    /// The prompt this shell's own program shows: `> ` inside an open block, the session's name when there are several.
    pub fn prompt(&mut self, session: u32) -> Result<String, i32> {
        let id = self.resolve(session).ok_or(status::NO_SUCH_SESSION)?;
        if self.interp(id).awaiting() {
            return Ok("> ".to_string());
        }
        let listed = self.world.listed().count();
        let name = self.world.session(id).and_then(|s| s.name.clone());
        Ok(match name {
            Some(name) if listed > 1 => format!("DianaSh [{name}] #: "),
            _ => "DianaSh #: ".to_string(),
        })
    }

    pub fn history(&self, session: u32) -> Result<Vec<String>, i32> {
        let id = self.resolve(session).ok_or(status::NO_SUCH_SESSION)?;
        Ok(self.world.session(id).map(|s| s.history.entries().to_vec()).unwrap_or_default())
    }

    pub fn user(&self, session: u32) -> Result<String, i32> {
        let id = self.resolve(session).ok_or(status::NO_SUCH_SESSION)?;
        Ok(self.world.session(id).map(|s| s.user.clone()).unwrap_or_default())
    }

    /// Sets a session's account as the C# property does: with no check that it exists.
    pub fn set_user(&mut self, session: u32, user: &str) -> Result<(), i32> {
        let id = self.resolve(session).ok_or(status::NO_SUCH_SESSION)?;
        if let Some(s) = self.world.session_mut(id) {
            s.user = user.to_string();
        }
        Ok(())
    }

    /// Every command a session has, in `help`'s order: `[{"name","kind","usage","read_only"}]`.
    pub fn commands_json(&self, session: u32) -> Result<String, i32> {
        let id = self.resolve(session).ok_or(status::NO_SUCH_SESSION)?;
        let table = self.world.session(id).map(|s| s.table.clone()).ok_or(status::NO_SUCH_SESSION)?;
        let items: Vec<String> = table
            .entries()
            .iter()
            .map(|e| {
                let kind = match e.kind {
                    Kind::Builtin(_) => "builtin",
                    Kind::Host(_) => "host",
                    Kind::Module(_) => "module",
                };
                format!(r#"{{"name":{},"kind":"{kind}","usage":{},"read_only":{}}}"#, json_string(&e.name), json_string(&e.usage), e.read_only)
            })
            .collect();
        Ok(format!("[{}]", items.join(",")))
    }

    /// The names the shell answers itself, its builtins' and its modules', in `help`'s order.
    pub fn own_names(&self) -> Vec<String> {
        self.world.base.entries().iter().map(|e| e.name.clone()).collect()
    }

    /// A module linked into the program: its commands, from its manifest, join every session's table, after what is there.
    pub fn add_module(&mut self, module: &'static dyn Module) -> Result<(), String> {
        let manifest = Manifest::read(module.manifest())?;
        let index = self.world.modules.len();
        self.world.modules.push(module);
        let add = |table: &mut Table| {
            for command in &manifest.commands {
                table.add(Entry { name: command.name.clone(), usage: command.usage.clone(), read_only: command.read_only, kind: Kind::Module(index) });
            }
            for (alias, command) in &manifest.aliases {
                table.alias(alias, command);
            }
        };
        add(Arc::make_mut(&mut self.world.base));
        for session in &mut self.world.sessions {
            add(Arc::make_mut(&mut session.table));
        }
        Ok(())
    }

    /// The listed sessions: `[{"id","name","user","current"}]`.
    pub fn sessions_json(&self) -> String {
        let items: Vec<String> = self
            .world
            .listed()
            .map(|s| {
                format!(
                    r#"{{"id":{},"name":{},"user":{},"current":{}}}"#,
                    s.id,
                    json_string(s.name.as_deref().unwrap_or("")),
                    json_string(&s.user),
                    self.world.current == Some(s.id)
                )
            })
            .collect();
        format!("[{}]", items.join(","))
    }

    /// A new, unlisted session, with the shell's own commands only until a host adds its own.
    pub fn session_new(&mut self) -> Result<u32, i32> {
        if !self.world.config.sessions {
            return Err(status::NOT_SUPPORTED);
        }
        let id = self.world.new_session(self.world.base.clone(), "root".to_string());
        self.interps.insert(id, Interp::new(id));
        Ok(id)
    }

    pub fn session_attach(&mut self, id: u32, name: Option<&str>) -> Result<String, (i32, String)> {
        if !self.world.config.sessions {
            return Err((status::NOT_SUPPORTED, "this shell keeps no sessions".to_string()));
        }
        self.world.attach(id, name)
    }

    pub fn session_switch(&mut self, id: u32) -> Result<(), i32> {
        if !self.world.listed().any(|s| s.id == id) {
            return Err(status::NO_SUCH_SESSION);
        }
        self.world.current = Some(id);
        Ok(())
    }

    /// Kills a session: a listed one as `tmux kill` does, which unlists it; an unlisted one ends.
    pub fn session_kill(&mut self, id: u32) -> Result<(), (i32, String)> {
        self.world.kill(id)?;
        self.interps.retain(|id, _| self.world.sessions.iter().any(|s| s.id == *id));
        Ok(())
    }

    /// The new session takes the old one's name, place and currency, and the old one ends: a host's rebuild.
    pub fn session_replace(&mut self, old: u32, new: u32) -> Result<(), i32> {
        let old_at = self.world.sessions.iter().position(|s| s.id == old).ok_or(status::NO_SUCH_SESSION)?;
        let new_at = self.world.sessions.iter().position(|s| s.id == new && s.name.is_none()).ok_or(status::NO_SUCH_SESSION)?;
        let mut replacement = self.world.sessions.remove(new_at);
        let old_at = if new_at < old_at { old_at - 1 } else { old_at };
        replacement.name = self.world.sessions[old_at].name.take();
        let _ = std::mem::replace(&mut self.world.sessions[old_at], replacement);
        if self.world.current == Some(old) {
            self.world.current = Some(new);
        }
        self.interps.remove(&old);
        Ok(())
    }

    /// The services the host provides, kept for every call until the next set (§10.3).
    pub fn set_services(&mut self, services: &[abi::dianash_service]) -> Result<(), i32> {
        let mut names = Vec::new();
        let mut kept = Vec::new();
        for service in services {
            let name: Box<[u8]> = unsafe { service.name.slice() }.ok_or(status::NULL)?.into();
            kept.push(abi::dianash_service { size: std::mem::size_of::<abi::dianash_service>() as u32, name: abi::dianash_bytes::of(&name), major: service.major, minor: service.minor, table: service.table });
            names.push(name);
        }
        self.world.services = kept;
        self.world.service_names = names;
        Ok(())
    }

    /// A host's commands, added to a session's table: each in the manifest's `commands`, its `aliases`, its `jobs` claims and its `order`.
    pub fn add_commands(&mut self, session: u32, commands: &dianash_host_commands) -> Result<(), (i32, String)> {
        let id = self.resolve(session).ok_or((status::NO_SUCH_SESSION, format!("no session {session}")))?;
        let run = commands.run.ok_or((status::NULL, "a host's commands need a run function".to_string()))?;
        let manifest = unsafe { commands.manifest.slice() }.ok_or((status::NULL, "the manifest is null".to_string()))?;
        let manifest = std::str::from_utf8(manifest).map_err(|_| (status::BAD_STRING, "the manifest is not UTF-8".to_string()))?;
        let parsed = Manifest::read(manifest).map_err(|words| (status::BAD_MANIFEST, words))?;
        let registration = self.world.hosts.len();
        self.world.hosts.push(Host { ctx: commands.ctx, run, jobs: commands.jobs, kill: commands.kill });
        let session = self.world.session_mut(id).ok_or((status::NO_SUCH_SESSION, format!("no session {id}")))?;
        let table = Arc::make_mut(&mut session.table);
        for command in parsed.commands {
            table.add(Entry { name: command.name, usage: command.usage, read_only: command.read_only, kind: Kind::Host(registration) });
        }
        for (alias, command) in parsed.aliases {
            table.alias(&alias, &command);
        }
        for prefix in parsed.jobs {
            table.claim(&prefix, registration);
        }
        for line in parsed.notes {
            table.note(&line);
        }
        if !parsed.order.is_empty() {
            table.order(&parsed.order);
        }
        Ok(())
    }
}

/// The JSON string of a text.
pub(crate) fn json_string(text: &str) -> String {
    json::quoted(text)
}

/// What the shell reads of a manifest (§5.3); a field it does not know is ignored.
#[derive(Debug, Default)]
pub(crate) struct Manifest {
    pub commands: Vec<ManifestCommand>,
    pub aliases: Vec<(String, String)>,
    pub jobs: Vec<String>,
    pub order: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct ManifestCommand {
    pub name: String,
    pub usage: String,
    pub read_only: bool,
}

impl Manifest {
    pub fn read(text: &str) -> Result<Manifest, String> {
        let value = json::parse(text).map_err(|e| format!("the manifest is not JSON: {e:?}"))?;
        if !matches!(value.get("manifest"), Some(Value::Number(format)) if format == "1") {
            return Err("the manifest's format is not version 1".to_string());
        }
        let mut manifest = Manifest::default();
        if let Some(Value::Array(commands)) = value.get("commands") {
            for command in commands {
                let name = command.get("name").and_then(Value::as_str).ok_or("a command has no name")?.to_string();
                let usage = command.get("usage").and_then(Value::as_str).unwrap_or("").to_string();
                let read_only = matches!(command.get("read_only"), Some(Value::Bool(true)));
                manifest.commands.push(ManifestCommand { name, usage, read_only });
            }
        }
        if let Some(Value::Object(aliases)) = value.get("aliases") {
            for (alias, command) in aliases {
                if let Some(command) = command.as_str() {
                    manifest.aliases.push((alias.text.clone(), command.to_string()));
                }
            }
        }
        if let Some(Value::Array(jobs)) = value.get("jobs") {
            manifest.jobs = jobs.iter().filter_map(Value::as_str).map(str::to_string).collect();
        }
        if let Some(Value::Array(order)) = value.get("order") {
            manifest.order = order.iter().filter_map(Value::as_str).map(str::to_string).collect();
        }
        if let Some(Value::Array(notes)) = value.get("notes") {
            manifest.notes = notes.iter().filter_map(Value::as_str).map(str::to_string).collect();
        }
        Ok(manifest)
    }
}

/// The version the banner shows, the C# shell's `DianaShInterpreter.Version` until that is retired (S7).
pub const DISPLAY_VERSION: &str = "0.1a";

/// The shell's own banner, which names no core: a host adds its own lines after it (§10.4).
pub fn banner() -> String {
    const BORDER: &str = "+------------------------------------------------------------------------+";
    let centre = |text: &str| {
        let pad = 72usize.saturating_sub(emusen_galaxia::dotnet_text::utf16_len(text));
        format!("|{}{text}{}|", " ".repeat(pad / 2), " ".repeat(pad - pad / 2))
    };
    [
        BORDER.to_string(),
        centre(&format!("Welcome to DianaSh v{DISPLAY_VERSION}")),
        centre("A standalone, cross-platform Unix shell"),
        centre("that accepts scripting."),
        BORDER.to_string(),
        String::new(),
        "Features:".to_string(),
        "  - Pipes, redirection, $(...) command substitution, shell variables".to_string(),
        "  - Control flow: if/for/while, break/continue".to_string(),
        "  - Scripting: source <path> to run scripts".to_string(),
        "  - Sessions (tmux), accounts (su), history (!N, !!)".to_string(),
        String::new(),
        "Type \"help\" for a list of commands. Man is supported for each.".to_string(),
    ]
    .join("\n")
}

/// The calling convention's own numbers, for the host API's version text.
pub fn version_text() -> String {
    format!("DianaSh {} (host API {}.{}, module ABI {}.{})", env!("CARGO_PKG_VERSION"), crate::HOST_ABI_MAJOR, crate::HOST_ABI_MINOR, abi::MODULE_ABI_MAJOR, abi::MODULE_ABI_MINOR)
}
