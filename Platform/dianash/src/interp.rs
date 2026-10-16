//! The interpreter: a line lexed, parsed and run, with expansion, pipes, redirection, control flow, `source` and
//! `$(...)`. Written as the C# `DianaShInterpreter` is, member for member, because the golden corpus holds what it
//! prints byte for byte, quirks included (EmuSen_DianaSh.md §13.4's findings 1 to 5). See §4.1 and §13.7.

use crate::ast::*;
use crate::builtins;
use crate::dotnet;
use crate::host;
use crate::lexer::{self, Lexed};
use crate::parser::{self, ParseError};
use crate::shell::{History, Request, World};
use crate::table::{Kind, Table};
use emusen_galaxia::suggestion;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

/// How a statement left: a `break` or `continue` on its way to the nearest loop, or a failure on its way to the line.
pub(crate) enum Flow {
    Break,
    Continue,
    Error(String),
}

pub(crate) type Exec<T> = Result<T, Flow>;

/// A command's answer: its standard output, its standard error, its status, and a request for the host it raised.
pub(crate) struct Outcome {
    pub output: String,
    pub errors: String,
    pub status: i32,
    pub request: Option<Request>,
}

impl Outcome {
    pub fn ok(output: impl Into<String>) -> Outcome {
        Outcome { output: output.into(), errors: String::new(), status: 0, request: None }
    }

    pub fn fail(output: impl Into<String>) -> Outcome {
        Outcome { output: output.into(), errors: String::new(), status: 1, request: None }
    }

    pub fn with(output: impl Into<String>, request: Request) -> Outcome {
        Outcome { output: output.into(), errors: String::new(), status: 0, request: Some(request) }
    }
}

const MAX_SOURCE_DEPTH: u32 = 20;

pub(crate) struct Interp {
    /// The session this interpreter is, or for a subshell the one it was made in: whose account and history its commands see.
    pub session: u32,
    variables: HashMap<String, String>,
    exported: HashSet<String>,
    /// A subshell's own history, which `!` reads and nothing keeps; the session's for the session's own interpreter.
    own_history: Option<History>,
    /// Raw text held across submits while a block is open.
    pending_input: String,
    source_depth: u32,
    /// The last request a command of this line raised; read once at the end.
    pending_request: Option<Request>,
    /// Shared with a `$(...)` subshell, so a nested runaway gets no fresh budget.
    started: Option<Instant>,
    is_root: bool,
}

impl Interp {
    pub fn new(session: u32) -> Interp {
        let mut variables = HashMap::new();
        variables.insert("?".to_string(), "0".to_string());
        Interp { session, variables, exported: HashSet::new(), own_history: None, pending_input: String::new(), source_depth: 0, pending_request: None, started: None, is_root: true }
    }

    fn subshell(&self) -> Interp {
        Interp {
            session: self.session,
            variables: self.variables.clone(),
            exported: self.exported.clone(),
            own_history: Some(History::default()),
            pending_input: String::new(),
            source_depth: 0,
            pending_request: None,
            started: self.started,
            is_root: false,
        }
    }

    pub fn awaiting(&self) -> bool {
        !self.pending_input.is_empty()
    }

    pub fn last_status(&self) -> i32 {
        self.variable("?").parse().unwrap_or(0)
    }

    pub fn variable(&self, name: &str) -> &str {
        self.variables.get(name).map_or("", String::as_str)
    }

    /// The exported variables, as a call's environment gives them.
    pub fn exported_variables(&self) -> HashMap<String, String> {
        self.exported.iter().map(|name| (name.clone(), self.variable(name).to_string())).collect()
    }

    /// When this line's budget began.
    pub fn started(&self) -> Option<Instant> {
        self.started
    }

    fn history<'a>(&'a mut self, world: &'a mut World) -> Option<&'a mut History> {
        match &mut self.own_history {
            Some(history) => Some(history),
            None => world.session_mut(self.session).map(|s| &mut s.history),
        }
    }

    fn table(&self, world: &World) -> Arc<Table> {
        world.session(self.session).map(|s| s.table.clone()).unwrap_or_else(Table::builtins)
    }

    /// `TryGetReadOnlyFastPath`: one simple command, no assignment or redirection, a literal name the table marks read-only.
    pub fn is_read_only(&self, table: &Table, raw: &str) -> bool {
        let trimmed = dotnet::trim(raw);
        if trimmed.is_empty() || !self.pending_input.is_empty() || trimmed.starts_with('!') {
            return false;
        }
        let Ok(Lexed::Tokens(tokens)) = lexer::tokenize(trimmed) else { return false };
        let Ok(script) = parser::parse(&tokens) else { return false };
        let [Node::AndOr(and_or)] = script.statements.as_slice() else { return false };
        if !and_or.rest.is_empty() || and_or.first.negate || and_or.first.stages.len() != 1 {
            return false;
        }
        let simple = &and_or.first.stages[0];
        if !simple.assignments.is_empty() || !simple.redirections.is_empty() || simple.words.is_empty() {
            return false;
        }
        match simple.words[0].parts.as_slice() {
            [part] if part.kind == PartKind::Literal => table.is_read_only(&part.text),
            _ => false,
        }
    }

    /// `SubmitCore`: a sourced line shares this scope but not history or `!` expansion.
    pub fn submit_core(&mut self, world: &mut World, raw: &str, interactive: bool) -> (bool, String, Option<Request>) {
        let mut line = raw.to_string();
        if interactive && self.pending_input.is_empty() && dotnet::trim_start(&line).starts_with('!') {
            match self.expand_history(world, dotnet::trim(&line)) {
                Ok(expanded) => line = expanded,
                Err(words) => return (false, format!("Error: {words}"), None),
            }
        }
        let combined = if self.pending_input.is_empty() { line } else { format!("{}\n{}", self.pending_input, line) };
        let tokens = match lexer::tokenize(&combined) {
            Ok(Lexed::Tokens(tokens)) => tokens,
            Ok(Lexed::NeedsMoreInput) => {
                self.pending_input = combined;
                return (true, String::new(), None);
            }
            Err(words) => {
                self.pending_input.clear();
                return (false, format!("Error: {words}"), None);
            }
        };
        let script = match parser::parse(&tokens) {
            Ok(script) => script,
            Err(ParseError::Incomplete(_)) => {
                self.pending_input = combined;
                return (true, String::new(), None);
            }
            Err(ParseError::Syntax(words)) => {
                self.pending_input.clear();
                return (false, format!("Syntax error: {words}"), None);
            }
        };
        self.pending_input.clear();
        let final_text = dotnet::trim(&combined).to_string();
        if final_text.is_empty() {
            return (false, String::new(), None);
        }
        if interactive {
            if let Some(history) = self.history(world) {
                history.add(&final_text);
            }
        }
        if self.is_root {
            self.started = Some(Instant::now());
        }
        self.pending_request = None;
        let mut output = String::new();
        match self.statement_list(world, &script, &mut output) {
            Ok(_) | Err(Flow::Break) => {}
            // A bare `continue` reaches here only from outside a loop: the host decides what it means (§13.4 finding 4).
            Err(Flow::Continue) => self.pending_request = Some(Request::Continue),
            Err(Flow::Error(words)) => {
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str("Error: ");
                output.push_str(&words);
            }
        }
        if output.ends_with('\n') {
            output.pop();
        }
        (false, output, self.pending_request.clone())
    }

    /// `!!` and `!N`, expanded before the line is recorded so history shows what ran.
    fn expand_history(&mut self, world: &mut World, trimmed: &str) -> Result<String, String> {
        let entries: Vec<String> = self.history(world).map(|h| h.entries().to_vec()).unwrap_or_default();
        if trimmed == "!!" {
            return entries.last().cloned().ok_or_else(|| "History is empty - nothing to repeat.".to_string());
        }
        if trimmed.chars().count() > 1 {
            if let Some(index) = dotnet::parse_i32(&trimmed[1..]) {
                if index < 1 || index as usize > entries.len() {
                    return Err(format!("No history entry {index}."));
                }
                return Ok(entries[index as usize - 1].clone());
            }
        }
        Ok(trimmed.to_string())
    }

    fn statement_list(&mut self, world: &mut World, list: &StatementList, output: &mut String) -> Exec<i32> {
        let mut exit = 0;
        for statement in &list.statements {
            exit = self.statement(world, statement, output)?;
        }
        Ok(exit)
    }

    fn statement(&mut self, world: &mut World, node: &Node, output: &mut String) -> Exec<i32> {
        match node {
            Node::AndOr(list) => self.and_or(world, list, output),
            Node::If { branches, else_body } => {
                for (condition, body) in branches {
                    if self.statement_list(world, condition, output)? == 0 {
                        return self.statement_list(world, body, output);
                    }
                }
                match else_body {
                    Some(body) => self.statement_list(world, body, output),
                    None => Ok(0),
                }
            }
            Node::For { var, items, body } => {
                let mut exit = 0;
                let mut values = Vec::new();
                for word in items {
                    values.extend(self.expand_word(world, word)?);
                }
                for value in values {
                    self.check_timeout(world)?;
                    self.variables.insert(var.clone(), value);
                    match self.statement_list(world, body, output) {
                        Ok(code) => exit = code,
                        Err(Flow::Continue) => continue,
                        Err(Flow::Break) => break,
                        Err(e) => return Err(e),
                    }
                }
                Ok(exit)
            }
            Node::While { condition, body, until } => {
                let mut exit = 0;
                loop {
                    self.check_timeout(world)?;
                    let code = self.statement_list(world, condition, output)?;
                    let run = if *until { code != 0 } else { code == 0 };
                    if !run {
                        break;
                    }
                    match self.statement_list(world, body, output) {
                        Ok(code) => exit = code,
                        Err(Flow::Continue) => continue,
                        Err(Flow::Break) => break,
                        Err(e) => return Err(e),
                    }
                }
                Ok(exit)
            }
            Node::Break => Err(Flow::Break),
            Node::Continue => Err(Flow::Continue),
        }
    }

    fn and_or(&mut self, world: &mut World, list: &AndOrList, output: &mut String) -> Exec<i32> {
        let mut exit = self.pipeline(world, &list.first, output)?;
        for (kind, pipeline) in &list.rest {
            if (*kind == AndOr::And && exit != 0) || (*kind == AndOr::Or && exit == 0) {
                continue;
            }
            exit = self.pipeline(world, pipeline, output)?;
        }
        Ok(exit)
    }

    pub(crate) fn check_timeout(&self, world: &World) -> Exec<()> {
        if world.cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(Flow::Error("Interrupted.".to_string()));
        }
        if self.started.is_some_and(|started| started.elapsed() > world.config.budget) {
            return Err(Flow::Error(format!(
                "Loop exceeded the {}-second safety timeout - aborting rather than hanging the console.",
                world.config.budget.as_secs_f64().round()
            )));
        }
        Ok(())
    }

    fn pipeline(&mut self, world: &mut World, pipeline: &Pipeline, output: &mut String) -> Exec<i32> {
        let mut stdin: Option<String> = None;
        let mut stage_output = String::new();
        let mut code = 0;
        for stage in &pipeline.stages {
            let errors;
            (stage_output, errors, code) = self.simple_command(world, stage, stdin.take())?;
            // Standard error goes where the output goes, as the stage ends; no pipe and no redirection takes it (§4.3, Q10).
            if !errors.is_empty() {
                output.push_str(&errors);
                if !errors.ends_with('\n') {
                    output.push('\n');
                }
            }
            stdin = Some(stage_output.clone());
        }
        if pipeline.negate {
            code = if code == 0 { 1 } else { 0 };
        }
        self.variables.insert("?".to_string(), code.to_string());
        if !stage_output.is_empty() {
            output.push_str(&stage_output);
            output.push('\n');
        }
        Ok(code)
    }

    fn simple_command(&mut self, world: &mut World, command: &SimpleCommand, stdin: Option<String>) -> Exec<(String, String, i32)> {
        for assignment in &command.assignments {
            let value = self.expand_word_single(world, &assignment.value)?;
            self.variables.insert(assignment.name.clone(), value);
        }
        if command.words.is_empty() {
            return Ok((String::new(), String::new(), 0));
        }
        let mut args = Vec::new();
        for word in &command.words {
            args.extend(self.expand_word(world, word)?);
        }
        if args.is_empty() {
            return Ok((String::new(), String::new(), 0));
        }
        let mut stdin = stdin;
        let mut redirect: Option<(String, bool)> = None;
        for redirection in &command.redirections {
            let target = self.expand_word_single(world, &redirection.target)?;
            let (inside, resolved) = world.sandbox.resolve(&target);
            if !inside {
                return Err(Flow::Error(world.sandbox.outside(&target)));
            }
            match redirection.kind {
                RedirectKind::Input => stdin = Some(dotnet::read_all_text(&resolved).map_err(Flow::Error)?),
                RedirectKind::Truncate => redirect = Some((resolved, false)),
                RedirectKind::Append => redirect = Some((resolved, true)),
            }
        }
        let name = args[0].clone();
        let outcome = self.dispatch(world, &name, &args, stdin)?;
        if outcome.request.is_some() {
            self.pending_request = outcome.request;
        }
        let mut visible = outcome.output;
        if let Some((path, append)) = redirect {
            let content = if visible.is_empty() { String::new() } else { format!("{visible}\n") };
            dotnet::write_all_text(&path, &content, append).map_err(Flow::Error)?;
            visible.clear();
        }
        Ok((visible, outcome.errors, outcome.status))
    }

    /// The language's own five run here, then the table: a builtin of the shell's, or a host's command.
    fn dispatch(&mut self, world: &mut World, name: &str, args: &[String], stdin: Option<String>) -> Exec<Outcome> {
        let lower = dotnet::to_lower(name);
        let table = self.table(world);
        match lower.as_str() {
            "help" => return Ok(Outcome::ok(builtins::help(&table))),
            "man" => return Ok(builtins::man(&table, args)),
            "export" => {
                for arg in &args[1..] {
                    match arg.find('=') {
                        Some(eq) if eq > 0 => {
                            self.variables.insert(arg[..eq].to_string(), arg[eq + 1..].to_string());
                            self.exported.insert(arg[..eq].to_string());
                        }
                        _ => {
                            self.variables.entry(arg.clone()).or_default();
                            self.exported.insert(arg.clone());
                        }
                    }
                }
                return Ok(Outcome::ok(""));
            }
            "unset" => {
                for arg in &args[1..] {
                    self.variables.remove(arg);
                    self.exported.remove(arg);
                }
                return Ok(Outcome::ok(""));
            }
            "source" | "." => return Ok(self.run_script(world, args)),
            _ => {}
        }
        let canonical = table.canonical(&lower);
        let Some(entry) = table.find(&canonical) else {
            let names: Vec<&str> = table.entries().iter().map(|e| e.name.as_str()).collect();
            return Ok(Outcome::fail(format!("Unknown command '{canonical}'.{} Type 'help' for a list.", suggestion::hint(&canonical, &names, 2))));
        };
        let outcome = match entry.kind {
            Kind::Builtin(builtin) => builtins::run(builtin, self, world, args, stdin.as_deref()),
            Kind::Host(registration) => host::run(self, world, registration, &canonical, args, stdin.as_deref()),
            Kind::Module(index) => host::run_module(self, world, index, &canonical, args, stdin.as_deref()),
        };
        // A malformed command must read as failure to `&&` and `if`, never take the console down.
        Ok(outcome.unwrap_or_else(|words| Outcome::fail(format!("Error: {words}"))))
    }

    /// `RunScript`: a file's lines fed back through this interpreter's own submit.
    fn run_script(&mut self, world: &mut World, args: &[String]) -> Outcome {
        let name = &args[0];
        if args.len() < 2 {
            return Outcome::fail(format!("{name}: usage: {name} <path>"));
        }
        if self.source_depth >= MAX_SOURCE_DEPTH {
            return Outcome::fail(format!("{name}: too many nested source calls (>{MAX_SOURCE_DEPTH}) - probably a self-referential script."));
        }
        let requested = &args[1];
        let (inside, resolved) = world.sandbox.resolve(requested);
        if !inside {
            return Outcome::fail(format!("{name}: {}", world.sandbox.outside(requested)));
        }
        if !emusen_galaxia::tree::file_exists(&resolved) {
            return Outcome::fail(format!("{name}: no such file: {requested}"));
        }
        let lines = match dotnet::read_all_lines(&resolved) {
            Ok(lines) => lines,
            Err(words) => return Outcome::fail(format!("{name}: {words}")),
        };
        let mut out = String::new();
        let mut request = None;
        self.source_depth += 1;
        for raw in &lines {
            let trimmed = dotnet::trim_start(raw);
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let (needs_more, output, action) = self.submit_core(world, raw, false);
            if needs_more {
                continue;
            }
            if !output.is_empty() {
                out.push_str(&output);
                out.push('\n');
            }
            // A sourced script may raise a request, as bash's `exit` does; the script stops there.
            if action.is_some() {
                request = action;
                break;
            }
        }
        self.source_depth -= 1;
        if self.awaiting() {
            self.pending_input.clear();
            out.push_str(&format!("{name}: unexpected end of file - unterminated block in {requested}"));
        }
        if out.ends_with('\n') {
            out.pop();
        }
        Outcome { output: out, errors: String::new(), status: 0, request }
    }

    /// Quoted spans survive splitting; an unquoted word that expands to nothing vanishes.
    fn expand_word(&mut self, world: &mut World, word: &Word) -> Exec<Vec<String>> {
        if word.parts.is_empty() {
            return Ok(vec![String::new()]);
        }
        let mut text: Vec<(char, bool)> = Vec::new();
        let mut unquoted = false;
        for part in &word.parts {
            let value = self.expand_part(world, part)?;
            if !part.quoted {
                unquoted = true;
            }
            text.extend(value.chars().map(|c| (c, part.quoted)));
        }
        if !unquoted {
            return Ok(vec![text.into_iter().map(|(c, _)| c).collect()]);
        }
        let mut results = Vec::new();
        let mut current = String::new();
        for (c, quoted) in text {
            if matches!(c, ' ' | '\t' | '\n') && !quoted {
                if !current.is_empty() {
                    results.push(std::mem::take(&mut current));
                }
            } else {
                current.push(c);
            }
        }
        if !current.is_empty() {
            results.push(current);
        }
        Ok(results)
    }

    /// Where bash suppresses splitting whatever the quoting.
    fn expand_word_single(&mut self, world: &mut World, word: &Word) -> Exec<String> {
        let mut out = String::new();
        for part in &word.parts {
            out.push_str(&self.expand_part(world, part)?);
        }
        Ok(out)
    }

    fn expand_part(&mut self, world: &mut World, part: &WordPart) -> Exec<String> {
        match part.kind {
            PartKind::Literal => Ok(part.text.clone()),
            PartKind::Variable => Ok(self.variable(&part.text).to_string()),
            PartKind::CommandSubstitution => self.substitute(world, &part.text),
        }
    }

    /// A subshell, so assignments inside do not leak out; its request is its own and is dropped.
    fn substitute(&mut self, world: &mut World, source: &str) -> Exec<String> {
        let mut sub = self.subshell();
        let (needs_more, output, _) = sub.submit_core(world, source, true);
        if needs_more {
            return Err(Flow::Error(
                "Command substitution \"$(...)\" isn't syntactically complete (e.g. a missing 'fi'/'done') - finish the block outside the substitution instead.".to_string(),
            ));
        }
        Ok(output.trim_end_matches('\n').to_string())
    }
}
