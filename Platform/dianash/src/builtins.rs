//! The shell's own commands: those that are the language or read and change the shell's state, which no module may
//! touch. Each prints what its C# class printed, word for word. See EmuSen_DianaSh.md §4.2 and §13.7.

use crate::dotnet;
use crate::host;
use crate::interp::{Interp, Outcome};
use crate::man;
use crate::shell::{Request, World};
use crate::table::{Kind, Table};
use emusen_galaxia::suggestion;
use emusen_galaxia::tree;

/// A command the shell answers itself; a host's command of the same name takes its place, as a frontend's always has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Builtin {
    Echo,
    Cd,
    Pwd,
    Tmux,
    Ps,
    Kill,
    Whoami,
    Who,
    Su,
    Useradd,
    Userdel,
    Passwd,
    True,
    False,
    Test,
    Bracket,
    History,
    Shutdown,
}

impl Builtin {
    /// In the order `help` has always listed them, relative to one another.
    pub const ALL: [Builtin; 18] = [
        Builtin::Echo,
        Builtin::Cd,
        Builtin::Pwd,
        Builtin::Tmux,
        Builtin::Ps,
        Builtin::Kill,
        Builtin::Whoami,
        Builtin::Who,
        Builtin::Su,
        Builtin::Useradd,
        Builtin::Userdel,
        Builtin::Passwd,
        Builtin::True,
        Builtin::False,
        Builtin::Test,
        Builtin::Bracket,
        Builtin::History,
        Builtin::Shutdown,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Builtin::Echo => "echo",
            Builtin::Cd => "cd",
            Builtin::Pwd => "pwd",
            Builtin::Tmux => "tmux",
            Builtin::Ps => "ps",
            Builtin::Kill => "kill",
            Builtin::Whoami => "whoami",
            Builtin::Who => "who",
            Builtin::Su => "su",
            Builtin::Useradd => "useradd",
            Builtin::Userdel => "userdel",
            Builtin::Passwd => "passwd",
            Builtin::True => "true",
            Builtin::False => "false",
            Builtin::Test => "test",
            Builtin::Bracket => "[",
            Builtin::History => "history",
            Builtin::Shutdown => "shutdown",
        }
    }

    pub fn usage(self) -> &'static str {
        match self {
            Builtin::Echo => "  echo <text...>                print <text> back, joined by spaces",
            Builtin::Cd => "  cd [dir]                      change directory, walled to the project root (no arg: your home directory)",
            Builtin::Pwd => "  pwd                           print the current working directory",
            Builtin::Tmux => concat!(
                "  tmux new [name]               create a session (auto-named if omitted) and switch to it\n",
                "  tmux list                     list every session, * marks the current one\n",
                "  tmux switch <name>            switch to an existing session\n",
                "  tmux kill <name>              destroy a session (can't kill the last one)"
            ),
            Builtin::Ps => "  ps | jobs                     list active breakpoints/watches/sessions as one table - see `kill`",
            Builtin::Kill => "  kill <id>                     remove a breakpoint/watch/session by its `ps` id (bp<N>, watch<N>, or a session name)",
            Builtin::Whoami => "  whoami                        print the account this shell is currently running as",
            Builtin::Who => "  who                           list every session and which account it's running as",
            Builtin::Su => "  su [name]                     switch this shell to <name> (default: root) - see `useradd`",
            Builtin::Useradd => "  useradd <name>                add a new account (root only) - see `su`",
            Builtin::Userdel => "  userdel <name>                remove an account (root only) - refuses root itself or one currently in use",
            Builtin::Passwd => concat!(
                "  passwd <newpassword>          change your own password\n",
                "  passwd <name> <newpassword>   change <name>'s password (root only)"
            ),
            Builtin::True => "  true                          do nothing, succeed (exit code 0) - for while/if conditions",
            Builtin::False => "  false                         do nothing, fail (exit code 1) - for while/if conditions",
            Builtin::Test => concat!(
                "  test EXPR / [ EXPR ]         VALUE (true if non-empty) | -z/-n VALUE | -f/-d PATH | !\n",
                "                                VALUE = / != / -eq / -ne / -lt / -le / -gt / -ge VALUE\n",
                "                                exit code only (0 = true) - meant for if/while, not for its own output"
            ),
            Builtin::Bracket => "  [ EXPR ]                      alias for 'test EXPR' - the ']' is required",
            Builtin::History => concat!(
                "  history                       list previously executed commands, numbered\n",
                "  history clear                 forget all recorded history\n",
                "  !N / !!                       re-run history entry N / the last command (typed at the prompt)"
            ),
            Builtin::Shutdown => "  shutdown | quit | exit        terminate the process",
        }
    }

    pub fn read_only(self) -> bool {
        matches!(self, Builtin::Echo | Builtin::Pwd | Builtin::Ps | Builtin::Whoami | Builtin::Who | Builtin::True | Builtin::False | Builtin::Test | Builtin::Bracket)
    }
}

/// `help`: always the listing, never one command's page.
pub fn help(table: &Table) -> String {
    let mut lines = vec![
        "Available commands:".to_string(),
        "  help                          this text - every command, one line each, with a brief explanation".to_string(),
        "  man [command]                 with no argument, same as 'help'; with a command name, its full manual page".to_string(),
    ];
    lines.extend(table.entries().iter().map(|e| e.usage.clone()));
    lines.extend(
        [
            "  export NAME[=value]           set a shell variable (no real subprocess env to export TO -",
            "                                kept for script compatibility with real bash habits)",
            "  unset NAME                    remove a shell variable",
            "  source <path> (alias: .)      run a script file's lines in this same shell (variables/history scope)",
            "  which <name>...               say what answers each name: an alias, a shell builtin or a command",
            "  command [-v] <name> [args]    run <name> as itself, not as an alias; -v prints the names that would run",
            "  trust <path>                  approve a module file you added, by its hash (typed at the prompt only)",
            "  if/then/elif/else/fi, for/in/do/done, while|until/do/done, break, continue",
            "                                control flow - see the reference doc for the full grammar",
            "",
        ]
        .map(str::to_string),
    );
    lines.extend(table.notes().iter().cloned());
    lines.extend(
        [
            "Shell features: quoting ('...'/\"...\"), $VAR/${VAR}/$? , $(command substitution),",
            "pipes (|), redirection (> >> <), sequencing (; && ||), if/for/while - see",
            "EmuSen_Debugging_Tools_Reference_v5.md for the full writeup and what's NOT supported.",
        ]
        .map(str::to_string),
    );
    lines.join("\n")
}

/// `man`: the page beside the modules, else the command's usage, else a refusal.
pub(crate) fn man(table: &Table, args: &[String]) -> Outcome {
    if args.len() < 2 {
        return Outcome::ok(help(table));
    }
    let target = dotnet::to_lower(&args[1]);
    if let Some(page) = man::lookup(&target) {
        return Outcome::ok(page);
    }
    if let Some(entry) = table.find(&target) {
        return Outcome::ok(format!("{target}\n\n{}\n\n(No detailed manual page yet for this command - showing its one-line usage above.)", entry.usage));
    }
    Outcome::fail(format!("No manual entry for '{target}'. Type 'help' for a list of commands."))
}

fn joined(args: &[String], from: usize) -> String {
    args[from..].join(" ")
}

fn user(world: &World, session: u32) -> String {
    world.session(session).map(|s| s.user.clone()).unwrap_or_else(|| "root".to_string())
}

/// Runs a builtin; an error is a failure the dispatcher words as `Error: ...`.
pub(crate) fn run(builtin: Builtin, interp: &mut Interp, world: &mut World, args: &[String], _stdin: Option<&str>) -> Result<Outcome, String> {
    let session = interp.session;
    Ok(match builtin {
        Builtin::Echo => Outcome::ok(if args.len() > 1 { joined(args, 1) } else { String::new() }),
        Builtin::True => Outcome::ok(""),
        Builtin::False => Outcome::fail(""),
        Builtin::Pwd => Outcome::ok(dotnet::current_directory()),
        Builtin::Cd => cd(world, args),
        Builtin::Test => test_outcome(evaluate(world, &args[1..])?),
        Builtin::Bracket => {
            if args.len() < 2 || args[args.len() - 1] != "]" {
                return Ok(Outcome::fail("Error: '[' must end with a matching ']'."));
            }
            test_outcome(evaluate(world, &args[1..args.len() - 1])?)
        }
        Builtin::Shutdown => Outcome::with("Shutting down.", Request::Exit),
        Builtin::History => {
            let Some(s) = world.session_mut(session) else { return Ok(Outcome::ok("No history yet.")) };
            if args.len() >= 2 && dotnet::same(&args[1], "clear") {
                s.history.clear();
                return Ok(Outcome::ok("History cleared."));
            }
            let entries = s.history.entries();
            if entries.is_empty() {
                return Ok(Outcome::ok("No history yet."));
            }
            let width = entries.len().to_string().len();
            Outcome::ok(entries.iter().enumerate().map(|(i, e)| format!("{}  {e}", dotnet::pad_left(&(i + 1).to_string(), width))).collect::<Vec<_>>().join("\n"))
        }
        Builtin::Whoami => Outcome::ok(user(world, session)),
        Builtin::Su => {
            let name = if args.len() >= 2 { joined(args, 1) } else { "root".to_string() };
            if !world.accounts.exists(&name) {
                return Ok(Outcome::fail(format!("su: no such user '{name}' - see `useradd`.")));
            }
            if let Some(s) = world.session_mut(session) {
                s.user = name.clone();
            }
            Outcome::ok(format!("Switched to '{name}'."))
        }
        Builtin::Useradd => {
            if args.len() < 2 {
                return Ok(Outcome::fail("useradd: usage: useradd <name>"));
            }
            if !dotnet::same(&user(world, session), "root") {
                return Ok(Outcome::fail("useradd: only root can add users."));
            }
            let name = joined(args, 1);
            if !world.accounts.add(&name) {
                return Ok(Outcome::fail(format!("useradd: '{name}' already exists.")));
            }
            Outcome::ok(format!("Added user '{name}'."))
        }
        Builtin::Userdel => userdel(world, session, args),
        Builtin::Passwd => match args.len() {
            2 => {
                let me = user(world, session);
                world.accounts.set_password(&me, &args[1]);
                Outcome::ok(format!("Password updated for '{me}'."))
            }
            3 => {
                if !dotnet::same(&user(world, session), "root") {
                    return Ok(Outcome::fail("passwd: only root can change another user's password."));
                }
                if !world.accounts.exists(&args[1]) {
                    return Ok(Outcome::fail(format!("passwd: no such user '{}'.", args[1])));
                }
                world.accounts.set_password(&args[1], &args[2]);
                Outcome::ok(format!("Password updated for '{}'.", args[1]))
            }
            _ => Outcome::fail("passwd: usage: passwd <newpassword> | passwd <name> <newpassword>"),
        },
        Builtin::Who => {
            if !world.config.sessions {
                return Ok(Outcome::ok(format!("  {} (this shell)", dotnet::pad_right(&user(world, session), 10))));
            }
            let current = world.current;
            Outcome::ok(
                world
                    .listed()
                    .map(|s| format!("  {} {}{}", dotnet::pad_right(&s.user, 10), s.name.as_deref().unwrap_or(""), if current == Some(s.id) { "  (current)" } else { "" }))
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        }
        Builtin::Tmux => tmux(world, session, args),
        Builtin::Ps => ps(interp, world)?,
        Builtin::Kill => kill(interp, world, args)?,
    })
}

fn test_outcome(result: bool) -> Outcome {
    if result { Outcome::ok("") } else { Outcome::fail("") }
}

fn cd(world: &mut World, args: &[String]) -> Outcome {
    let dir = if args.len() >= 2 { joined(args, 1) } else { "/".to_string() };
    let (inside, resolved) = world.sandbox.resolve(&dir);
    if !inside {
        return Outcome::fail(format!("cd: {}", world.sandbox.outside(&dir)));
    }
    if !tree::directory_exists(&resolved) {
        return Outcome::fail(format!("cd: no such directory: {dir}"));
    }
    world.shown_cwd = None;
    match std::env::set_current_dir(&resolved) {
        Ok(()) => Outcome::ok(""),
        Err(e) => Outcome::fail(format!("cd: {}", dotnet::io_message(&e, &resolved))),
    }
}

/// `TestEvaluator`: a small subset, with `!` before the arity, and compound conditions left to `&&`.
fn evaluate(world: &World, expr: &[String]) -> Result<bool, String> {
    if expr.first().is_some_and(|e| e == "!") {
        return evaluate(world, &expr[1..]).map(|v| !v);
    }
    match expr {
        [] => Ok(false),
        [value] => Ok(!value.is_empty()),
        [op, value] => match op.as_str() {
            "-z" => Ok(value.is_empty()),
            "-n" => Ok(!value.is_empty()),
            "-f" => Ok(match world.sandbox.resolve(value) {
                (true, file) => tree::file_exists(&file),
                _ => false,
            }),
            "-d" => Ok(match world.sandbox.resolve(value) {
                (true, dir) => tree::directory_exists(&dir),
                _ => false,
            }),
            _ => Err(format!("Unknown unary test operator '{op}'.")),
        },
        [lhs, op, rhs] => {
            let number = |s: &str| dotnet::parse_i64(s).ok_or_else(|| format!("'{s}' isn't a number (needed for a -eq/-ne/-lt/-le/-gt/-ge comparison)."));
            match op.as_str() {
                "=" | "==" => Ok(lhs == rhs),
                "!=" => Ok(lhs != rhs),
                "-eq" => Ok(number(lhs)? == number(rhs)?),
                "-ne" => Ok(number(lhs)? != number(rhs)?),
                "-lt" => Ok(number(lhs)? < number(rhs)?),
                "-le" => Ok(number(lhs)? <= number(rhs)?),
                "-gt" => Ok(number(lhs)? > number(rhs)?),
                "-ge" => Ok(number(lhs)? >= number(rhs)?),
                _ => Err(format!("Unknown binary test operator '{op}'.")),
            }
        }
        _ => Err("test/[ only supports: VALUE, OP VALUE, or VALUE OP VALUE - combine multiple conditions with the shell's own && / || instead.".to_string()),
    }
}

fn userdel(world: &mut World, session: u32, args: &[String]) -> Outcome {
    if args.len() < 2 {
        return Outcome::fail("userdel: usage: userdel <name>");
    }
    if !dotnet::same(&user(world, session), "root") {
        return Outcome::fail("userdel: only root can remove users.");
    }
    let name = joined(args, 1);
    if !world.accounts.exists(&name) {
        return Outcome::fail(format!("userdel: no such user '{name}'."));
    }
    if dotnet::same(&name, "root") {
        return Outcome::fail("userdel: refusing to remove 'root'.");
    }
    if world.config.sessions {
        if let Some(active) = world.listed().find(|s| dotnet::same(&s.user, &name)) {
            return Outcome::fail(format!(
                "userdel: '{name}' is currently in use by session '{}' - `su` that session to another account first.",
                active.name.as_deref().unwrap_or("")
            ));
        }
    } else if dotnet::same(&user(world, session), &name) {
        return Outcome::fail(format!("userdel: '{name}' is the account this shell is currently running as - `su` to another account first."));
    }
    world.accounts.remove(&name);
    Outcome::ok(format!("Removed user '{name}'."))
}

fn tmux(world: &mut World, session: u32, args: &[String]) -> Outcome {
    if !world.config.sessions {
        return Outcome::fail("tmux: no session manager is registered for this host.");
    }
    if args.len() < 2 {
        return Outcome::ok(Builtin::Tmux.usage());
    }
    let sub = dotnet::to_lower(&args[1]);
    match sub.as_str() {
        "new" => {
            // A new session inherits its creator's account rather than starting as root.
            let creating_user = world.current_listed().map(|s| s.user.clone()).unwrap_or_else(|| "root".to_string());
            let name = if args.len() > 2 { joined(args, 2) } else { world.next_default_name() };
            if world.find_listed(&name).is_some() {
                return Outcome::fail(format!("tmux: a session named '{name}' already exists"));
            }
            let table = world.session(session).map(|s| s.table.clone()).unwrap_or_else(Table::builtins);
            let services = world.session(session).map(|s| s.services.copy()).unwrap_or_default();
            let id = world.new_session(table, creating_user);
            if let Some(created) = world.session_mut(id) {
                created.services = services;
            }
            if let Err((_, words)) = world.attach(id, Some(&name)) {
                return Outcome::fail(format!("tmux: {words}"));
            }
            Outcome::with(format!("[tmux] Created and switched to session '{name}'."), Request::Session { id, name })
        }
        "list" => {
            let current = world.current;
            Outcome::ok(world.listed().map(|s| format!("  {} {}", if current == Some(s.id) { "*" } else { " " }, s.name.as_deref().unwrap_or(""))).collect::<Vec<_>>().join("\n"))
        }
        "switch" => {
            if args.len() < 3 {
                return Outcome::ok("Usage: tmux switch <name>");
            }
            let name = joined(args, 2);
            let Some(id) = world.find_listed(&name) else { return Outcome::fail(format!("tmux: no such session: {name}")) };
            world.current = Some(id);
            Outcome::with(format!("[tmux] Switched to session '{name}'."), Request::Session { id, name })
        }
        "kill" => {
            if args.len() < 3 {
                return Outcome::ok("Usage: tmux kill <name>");
            }
            let name = joined(args, 2);
            match kill_session(world, &name) {
                Err(words) => Outcome::fail(format!("tmux: {words}")),
                Ok(Some((id, current))) => Outcome::with(format!("[tmux] Killed '{name}'; switched to '{current}'."), Request::Session { id, name: current }),
                Ok(None) => Outcome::ok(format!("[tmux] Killed '{name}'.")),
            }
        }
        _ => {
            let subcommands = ["new", "list", "switch", "kill"];
            Outcome::ok(format!("Unknown 'tmux' subcommand '{sub}'.{} Try {}.", suggestion::hint(&sub, &subcommands, 2), subcommands.join("/")))
        }
    }
}

/// `TryKill` by name; when the current session went, the one that is current now.
fn kill_session(world: &mut World, name: &str) -> Result<Option<(u32, String)>, String> {
    let was_current = world.current_listed().is_some_and(|s| s.name.as_deref().is_some_and(|n| dotnet::same(n, name)));
    let Some(id) = world.find_listed(name) else { return Err(format!("no such session: {name}")) };
    world.kill(id).map_err(|(_, words)| words)?;
    if !was_current {
        return Ok(None);
    }
    Ok(world.current_listed().map(|s| (s.id, s.name.clone().unwrap_or_default())))
}

/// `ps`: what each host lists as jobs, then the sessions, as one table whose ids `kill` takes.
fn ps(interp: &mut Interp, world: &mut World) -> Result<Outcome, String> {
    let mut lines: Vec<String> = Vec::new();
    let table = world.session(interp.session).map(|s| s.table.clone()).unwrap_or_else(Table::builtins);
    let mut asked: Vec<Kind> = Vec::new();
    for (_, claimant) in table.claims() {
        if asked.contains(claimant) {
            continue;
        }
        asked.push(*claimant);
        let listed = host::jobs(interp, world, *claimant)?;
        lines.extend(listed.lines().map(str::to_string));
    }
    if world.config.sessions {
        let current = world.current;
        for s in world.listed() {
            let marker = if current == Some(s.id) { "(current)" } else { "" };
            lines.push(format!("  {} session     {marker}", dotnet::pad_right(s.name.as_deref().unwrap_or(""), 10)));
        }
    }
    if lines.is_empty() {
        return Ok(Outcome::ok("No active breakpoints, watches, or sessions."));
    }
    Ok(Outcome::ok(lines.join("\n")))
}

/// `kill`: the host that claims the id's prefix and takes it, else a session's name.
fn kill(interp: &mut Interp, world: &mut World, args: &[String]) -> Result<Outcome, String> {
    if args.len() < 2 {
        return Ok(Outcome::fail("kill: usage: kill <id>"));
    }
    let id = joined(args, 1);
    let table = world.session(interp.session).map(|s| s.table.clone()).unwrap_or_else(Table::builtins);
    for (prefix, claimant) in table.claims() {
        if dotnet::starts_with_ignore_case(&id, prefix) {
            if let Some(outcome) = host::kill(interp, world, *claimant, &id)? {
                return Ok(outcome);
            }
        }
    }
    if !world.config.sessions {
        return Ok(Outcome::fail(format!("kill: no session manager is registered for this host, and '{id}' isn't a bp<N>/watch<N> id.")));
    }
    Ok(match kill_session(world, &id) {
        Err(words) => Outcome::fail(format!("kill: {words}")),
        Ok(Some((session, current))) => Outcome::with(format!("[kill] Killed session '{id}'; switched to '{current}'."), Request::Session { id: session, name: current }),
        Ok(None) => Outcome::ok(format!("Session '{id}' removed.")),
    })
}
