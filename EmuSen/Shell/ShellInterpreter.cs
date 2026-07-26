using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.Shell.Ast;

namespace EmuSen.Shell
{
    // The general-purpose command shell this project's debug console has
    // grown into - was DebugCommandProcessor (one line in, one line of
    // text out, no state beyond per-command registries). Now a real,
    // if deliberately scoped-down, bash-alike: quoting, variables,
    // $(...) command substitution, pipes, '>' / '>>' / '<' redirection to
    // real files, ';' / '&&' / '||' sequencing, and if/for/while control
    // flow, all built on the same small hand-written Lexer/Parser this
    // namespace also contains.
    //
    // What's deliberately NOT here, because it doesn't map onto "a
    // command console embedded in a single-process emulator" the way it
    // does onto a real multi-process OS shell: background jobs ('&' - see
    // Lexer's own error message), heredocs, globbing (there's no
    // filesystem-of-interest to expand a pattern against), arithmetic
    // expansion, brace/tilde expansion, functions, arrays, and true
    // per-command-ephemeral variable scoping (`FOO=bar cmd` sets FOO
    // persistently here, not just for that one command - see Assignment's
    // own comment). Each is a real, known simplification, not a silent
    // gap - flagged here once rather than re-litigated at every call site
    // that could theoretically want one of them.
    public class ShellInterpreter
    {
        private static readonly TimeSpan MaxExecutionTime = TimeSpan.FromSeconds(10);

        private readonly IDebugTarget? _target;
        private readonly IReadOnlyList<IShellCommand> _orderedCommands;
        private readonly Dictionary<string, IShellCommand> _commands;
        private readonly Dictionary<string, string> _variables;
        private readonly bool _isRoot;

        // Shared by reference with any subshell created for a $(...)
        // command substitution (see CreateSubshell) so a runaway loop
        // nested inside a substitution still counts against the same
        // wall-clock budget as the outer command line that contains it,
        // rather than getting its own fresh allowance.
        private Stopwatch? _stopwatch;

        // Holds raw text across Submit() calls while a quote/"$(...)"/
        // if-else-fi/for-do-done block is still open - see Submit's own
        // comment for the full flow.
        private string _pendingInput = "";

        public CommandHistory History { get; }

        // True between Submit() calls while a multi-line construct (an
        // unterminated quote, "$(...)", or if/for/while block) is still
        // being typed - an interactive frontend can check this to show a
        // continuation prompt ("> ", bash-style) instead of the normal one.
        public bool IsAwaitingMoreInput => _pendingInput.Length > 0;

        // `history` lets a caller building its own command list (see
        // CreateDefault below) hand a HistoryCommand instance the exact
        // same CommandHistory object this interpreter itself records
        // into and exposes via the History property, rather than each
        // silently getting its own - the same "neither side owns it"
        // sharing CommandHistory's own comment describes.
        public ShellInterpreter(IDebugTarget? target, IEnumerable<IShellCommand> commands, CommandHistory? history = null)
            : this(target, new List<IShellCommand>(commands), null, new Dictionary<string, string>(), history ?? new CommandHistory(), isRoot: true)
        {
        }

        // Builds the standard, full command registry (every debug command
        // under EmuSen.Shell.Commands plus this namespace's own shell
        // builtins) - the one-stop constructor call every frontend
        // (EmuSen.Hotaru, EmuSen.HeadlessDebug) actually wants,
        // rather than each independently re-listing 30-odd command
        // classes and risking them drifting out of sync with each other.
        public static ShellInterpreter CreateDefault(IDebugTarget? target)
        {
            // snapshot/diff share one SnapshotStore (see that file's own
            // comment) - constructed once here, same lifetime as every
            // other command's own state.
            var snapshotStore = new EmuSen.Shell.Commands.SnapshotStore();
            var history = new CommandHistory();

            var commands = new List<IShellCommand>
            {
                new EmuSen.Shell.Commands.SpacesCommand(),
                new EmuSen.Shell.Commands.MemCommand(),
                new EmuSen.Shell.Commands.WriteCommand(),
                new EmuSen.Shell.Commands.RegsCommand(),
                new EmuSen.Shell.Commands.SpritesCommand(),
                new EmuSen.Shell.Commands.PalCommand(),
                new EmuSen.Shell.Commands.ChannelsCommand(),
                new EmuSen.Shell.Commands.MuteCommand(),
                new EmuSen.Shell.Commands.WatchCommand(),
                new EmuSen.Shell.Commands.BreakCommand(),
                new EmuSen.Shell.Commands.FrameLogCommand(),
                new EmuSen.Shell.Commands.CheatCommand(),
                new EmuSen.Shell.Commands.SearchCommand(),
                new EmuSen.Shell.Commands.SnapshotCommand(snapshotStore),
                new EmuSen.Shell.Commands.DiffCommand(snapshotStore),
                new EmuSen.Shell.Commands.DumpCommand(),
                new EmuSen.Shell.Commands.LoadCommand(),
                new EmuSen.Shell.Commands.TileCommand(),
                new EmuSen.Shell.Commands.TilemapCommand(),
                new EmuSen.Shell.Commands.DisasmCommand(),
                new EmuSen.Shell.Commands.TraceCommand(),
                new EmuSen.Shell.Commands.CallersCommand(),
                new EmuSen.Shell.Commands.WritersCommand(),
                new EmuSen.Shell.Commands.ReadersCommand(),
                new EmuSen.Shell.Commands.LogCommand(),
                new Commands.EchoCommand(),
                new Commands.SedCommand(),
                new Commands.TrueCommand(),
                new Commands.FalseCommand(),
                new Commands.TestCommand(),
                new Commands.BracketCommand(),
                new Commands.HistoryCommand(history),
            };

            return new ShellInterpreter(target, commands, history);
        }

        private ShellInterpreter(
            IDebugTarget? target,
            IReadOnlyList<IShellCommand> orderedCommands,
            Dictionary<string, IShellCommand>? commandsDict,
            Dictionary<string, string> variables,
            CommandHistory history,
            bool isRoot)
        {
            _target = target;
            _orderedCommands = orderedCommands;
            _commands = commandsDict ?? orderedCommands.ToDictionary(c => c.Name, StringComparer.OrdinalIgnoreCase);
            _variables = variables;
            History = history;
            _isRoot = isRoot;
            if (!_variables.ContainsKey("?")) _variables["?"] = "0";
        }

        // A fresh, isolated interpreter for one $(...) command substitution
        // - real bash semantics: a command substitution runs in a
        // subshell, so variable assignments made inside it do NOT leak
        // back out (a genuinely surprising-if-you-don't-know-it bash
        // behavior, reproduced deliberately here rather than by accident).
        // Shares the same command registry/target (read-only, no reason to
        // rebuild the dictionary or reconnect the same emulator session)
        // and the same wall-clock safety-timeout Stopwatch, but gets its
        // own variable-dictionary snapshot and its own throwaway history
        // (a substitution isn't something the user "typed at the prompt").
        private ShellInterpreter CreateSubshell()
        {
            var sub = new ShellInterpreter(_target, _orderedCommands, _commands, new Dictionary<string, string>(_variables), new CommandHistory(), isRoot: false);
            sub._stopwatch = _stopwatch;
            return sub;
        }

        // Internal, loop-only control-flow signals - never surface past
        // ExecuteFor/ExecuteWhile. A tree-walking interpreter using
        // exceptions for break/continue is a common, accepted pattern
        // (the same shape a real interpreter for a language with these
        // constructs almost always ends up using), not a "using
        // exceptions for control flow" antipattern smell here - there's no
        // other clean way to unwind an arbitrary number of nested
        // StatementList/If frames back to the nearest enclosing loop.
        private class BreakSignal : Exception { }
        private class ContinueSignal : Exception { }

        public string Execute(string commandLine) => Submit(commandLine).Output;

        // The core entry point. Buffers raw text across calls whenever the
        // lexer or parser reports "not finished yet" (an open quote, an
        // unbalanced "$(...)", or an if/for/while block missing its
        // closing keyword) - both an interactive frontend feeding one line
        // at a time and EmuSen.HeadlessDebug's `--commands` script reader
        // (which also feeds one raw file line per call) get correct
        // multi-line control-flow support for free from this, without
        // either caller needing to know anything about the grammar.
        public (bool NeedsMoreInput, string Output) Submit(string rawLine)
        {
            string line = rawLine ?? "";

            // '!N'/'!!' history recall only makes sense as the START of a
            // brand-new command, never mid-block - a continuation line
            // inside an open "if" typed as "!!" is just a literal word.
            if (_pendingInput.Length == 0 && line.TrimStart().StartsWith('!'))
            {
                try { line = ExpandHistoryReference(line.Trim()); }
                catch (Exception ex) { return (false, $"Error: {ex.Message}"); }
            }

            string combined = _pendingInput.Length > 0 ? _pendingInput + "\n" + line : line;

            LexResult lexResult;
            try
            {
                lexResult = Lexer.Tokenize(combined);
            }
            catch (Exception ex)
            {
                _pendingInput = "";
                return (false, $"Error: {ex.Message}");
            }

            if (lexResult.NeedsMoreInput)
            {
                _pendingInput = combined;
                return (true, "");
            }

            StatementList script;
            try
            {
                script = Parser.Parse(lexResult.Tokens);
            }
            catch (ShellIncompleteException)
            {
                _pendingInput = combined;
                return (true, "");
            }
            catch (ShellSyntaxException ex)
            {
                _pendingInput = "";
                return (false, $"Syntax error: {ex.Message}");
            }

            _pendingInput = "";
            string finalText = combined.Trim();
            if (finalText.Length == 0) return (false, "");

            History.Add(finalText);
            if (_isRoot) _stopwatch = Stopwatch.StartNew();

            var sb = new StringBuilder();
            try
            {
                ExecuteStatementList(script, sb);
            }
            catch (BreakSignal) { /* bare 'break' outside any loop - bash silently no-ops this */ }
            catch (ContinueSignal) { /* likewise for a bare 'continue' */ }
            catch (Exception ex)
            {
                if (sb.Length > 0 && sb[^1] != '\n') sb.Append('\n');
                sb.Append($"Error: {ex.Message}");
            }

            string result = sb.ToString();
            if (result.EndsWith('\n')) result = result[..^1];
            return (false, result);
        }

        // Bash-style history expansion: "!!" re-runs the last command,
        // "!N" re-runs history entry N (1-based, matching `history`'s own
        // display numbering). Expanded before this line is recorded into
        // History, so (matching real bash) history shows what actually
        // ran, not the literal "!N"/"!!" that was typed.
        private string ExpandHistoryReference(string trimmed)
        {
            if (trimmed == "!!")
            {
                if (History.Entries.Count == 0) throw new InvalidOperationException("History is empty - nothing to repeat.");
                return History.Entries[^1];
            }

            if (trimmed.Length > 1 && int.TryParse(trimmed.AsSpan(1), out int index))
            {
                if (index < 1 || index > History.Entries.Count) throw new InvalidOperationException($"No history entry {index}.");
                return History.Entries[index - 1];
            }

            return trimmed;
        }

        // --- Statement execution ---

        private int ExecuteStatementList(StatementList list, StringBuilder output)
        {
            int exit = 0;
            foreach (Node stmt in list.Statements) exit = ExecuteStatement(stmt, output);
            return exit;
        }

        private int ExecuteStatement(Node stmt, StringBuilder output)
        {
            switch (stmt)
            {
                case AndOrList andOr: return ExecuteAndOrList(andOr, output);
                case IfNode ifNode: return ExecuteIf(ifNode, output);
                case ForNode forNode: return ExecuteFor(forNode, output);
                case WhileNode whileNode: return ExecuteWhile(whileNode, output);
                case BreakNode: throw new BreakSignal();
                case ContinueNode: throw new ContinueSignal();
                default: throw new InvalidOperationException($"Unhandled statement node {stmt.GetType().Name}.");
            }
        }

        private int ExecuteAndOrList(AndOrList node, StringBuilder output)
        {
            int exit = ExecutePipeline(node.First, output);
            foreach (var (kind, pipeline) in node.Rest)
            {
                if (kind == AndOrKind.And && exit != 0) continue;
                if (kind == AndOrKind.Or && exit == 0) continue;
                exit = ExecutePipeline(pipeline, output);
            }
            return exit;
        }

        private int ExecuteIf(IfNode node, StringBuilder output)
        {
            foreach (var (condition, body) in node.Branches)
            {
                if (ExecuteStatementList(condition, output) == 0) return ExecuteStatementList(body, output);
            }
            return node.ElseBody != null ? ExecuteStatementList(node.ElseBody, output) : 0;
        }

        private int ExecuteFor(ForNode node, StringBuilder output)
        {
            int exit = 0;
            var items = new List<string>();
            foreach (Word w in node.Items) items.AddRange(ExpandWord(w));

            foreach (string item in items)
            {
                CheckTimeout();
                _variables[node.VarName] = item;
                try { exit = ExecuteStatementList(node.Body, output); }
                catch (ContinueSignal) { continue; }
                catch (BreakSignal) { break; }
            }
            return exit;
        }

        private int ExecuteWhile(WhileNode node, StringBuilder output)
        {
            int exit = 0;
            while (true)
            {
                CheckTimeout();
                int condExit = ExecuteStatementList(node.Condition, output);
                bool shouldRun = node.Until ? condExit != 0 : condExit == 0;
                if (!shouldRun) break;

                try { exit = ExecuteStatementList(node.Body, output); }
                catch (ContinueSignal) { continue; }
                catch (BreakSignal) { break; }
            }
            return exit;
        }

        private void CheckTimeout()
        {
            if (_stopwatch != null && _stopwatch.Elapsed > MaxExecutionTime)
            {
                throw new InvalidOperationException($"Loop exceeded the {MaxExecutionTime.TotalSeconds:0}-second safety timeout - aborting rather than hanging the console.");
            }
        }

        private int ExecutePipeline(Pipeline node, StringBuilder output)
        {
            string? stdin = null;
            string stageOutput = "";
            int exitCode = 0;

            for (int i = 0; i < node.Stages.Count; i++)
            {
                (stageOutput, exitCode) = ExecuteSimpleCommand(node.Stages[i], stdin);
                stdin = stageOutput;
            }

            if (node.Negate) exitCode = exitCode == 0 ? 1 : 0;
            _variables["?"] = exitCode.ToString();

            if (stageOutput.Length > 0)
            {
                output.Append(stageOutput);
                output.Append('\n');
            }
            return exitCode;
        }

        private (string Output, int ExitCode) ExecuteSimpleCommand(SimpleCommand cmd, string? stdin)
        {
            foreach (Assignment assignment in cmd.Assignments)
            {
                _variables[assignment.Name] = ExpandWordSingle(assignment.Value);
            }

            if (cmd.Words.Count == 0) return ("", 0); // bare assignment(s), no command - bash: success

            var args = new List<string>();
            foreach (Word w in cmd.Words) args.AddRange(ExpandWord(w));
            if (args.Count == 0) return ("", 0); // every word expanded away to nothing

            string name = args[0];

            string? effectiveStdin = stdin;
            string? outputRedirectPath = null;
            bool append = false;
            foreach (Redirection r in cmd.Redirections)
            {
                switch (r.Kind)
                {
                    case RedirectKind.Input:
                        effectiveStdin = File.ReadAllText(ExpandWordSingle(r.Target));
                        break;
                    case RedirectKind.Truncate:
                        outputRedirectPath = ExpandWordSingle(r.Target);
                        append = false;
                        break;
                    case RedirectKind.Append:
                        outputRedirectPath = ExpandWordSingle(r.Target);
                        append = true;
                        break;
                }
            }

            ShellResult result = Dispatch(name, args.ToArray(), effectiveStdin);

            string visibleOutput = result.Output;
            if (outputRedirectPath != null)
            {
                string content = visibleOutput.Length > 0 ? visibleOutput + "\n" : "";
                if (append) File.AppendAllText(outputRedirectPath, content);
                else File.WriteAllText(outputRedirectPath, content);
                visibleOutput = "";
            }

            return (visibleOutput, result.ExitCode);
        }

        // `help`/`summary`/`export`/`unset` stay special-cased here (not
        // registered IShellCommand instances) since each needs direct
        // access to interpreter-internal state (`_orderedCommands`, the
        // target's own GetSummaryText(), `_variables`) that IShellCommand
        // deliberately doesn't expose to ordinary commands - the same
        // reasoning `VAR=value` assignment is a parser/interpreter
        // concept, not a command, at all.
        private ShellResult Dispatch(string name, string[] args, string? stdin)
        {
            string cmd = name.ToLowerInvariant();
            if (cmd == "help") return Help();
            if (cmd == "summary") return _target?.GetSummaryText() ?? "No target attached.";

            // `export NAME[=value]` - real bash exports a variable into
            // child processes' environment; there's no such thing here
            // (no subprocesses at all), so this is just a plain assignment
            // (or a no-op declaration if no '=' is given and the name is
            // already set) - kept purely so a script written with real
            // bash habits ("export FOO=bar") still does something sane
            // instead of erroring as an unknown command.
            if (cmd == "export")
            {
                foreach (string arg in args.Skip(1))
                {
                    int eq = arg.IndexOf('=');
                    if (eq > 0) _variables[arg.Substring(0, eq)] = arg.Substring(eq + 1);
                    else if (!_variables.ContainsKey(arg)) _variables[arg] = "";
                }
                return "";
            }

            if (cmd == "unset")
            {
                foreach (string arg in args.Skip(1)) _variables.Remove(arg);
                return "";
            }

            if (!_commands.TryGetValue(cmd, out IShellCommand? command))
            {
                return ShellResult.Fail($"Unknown command '{cmd}'. Type 'help' for a list.");
            }

            try
            {
                return command.Execute(_target, args, stdin);
            }
            catch (Exception ex)
            {
                // A malformed command (bad address, out-of-range space,
                // etc.) should produce a readable error and a nonzero
                // exit status - never take down whatever's running this
                // console, and never look like success to '&&'/'if'.
                return ShellResult.Fail($"Error: {ex.Message}");
            }
        }

        private string Help()
        {
            var lines = new List<string>
            {
                "Available commands:",
                "  help                          this text",
            };
            lines.AddRange(_orderedCommands.Select(c => c.Usage));
            lines.Add("  summary                       free-text state dump (whatever isn't structured above yet)");
            lines.Add("  export NAME[=value]           set a shell variable (no real subprocess env to export TO -");
            lines.Add("                                kept for script compatibility with real bash habits)");
            lines.Add("  unset NAME                    remove a shell variable");
            lines.Add("  if/then/elif/else/fi, for/in/do/done, while|until/do/done, break, continue");
            lines.Add("                                control flow - see the reference doc for the full grammar");
            lines.Add("");
            lines.Add("Addresses/values are hex; an optional 0x or $ prefix is fine either way.");
            lines.Add("Shell features: quoting ('...'/\"...\"), $VAR/${VAR}/$? , $(command substitution),");
            lines.Add("pipes (|), redirection (> >> <), sequencing (; && ||), if/for/while - see");
            lines.Add("EmuSen_Debugging_Tools_Reference_v5.md for the full writeup and what's NOT supported.");
            return string.Join('\n', lines);
        }

        // --- Word expansion ---

        // Full bash-style expansion: substitutes $VAR/${VAR}/$?/$(...),
        // then - only for parts that were NOT inside quotes - splits the
        // result on whitespace, tracking which characters came from a
        // quoted span so "pre$X post" (X quoted) stays one argument even
        // though it contains a space, while unquoted $X (X="a b") becomes
        // two. An entirely-quoted word (including an explicit empty ""/'')
        // always produces exactly one argument, even if empty; a word
        // that's entirely unquoted and expands to nothing produces ZERO
        // arguments (the word "vanishes"), matching a very commonly
        // relied-upon bash behavior (`echo a $EMPTY b` -> "a b", not
        // "a  b" with a phantom empty argument in the middle).
        internal List<string> ExpandWord(Word w)
        {
            if (w.Parts.Count == 0) return new List<string> { "" };

            var text = new StringBuilder();
            var quotedMask = new List<bool>();
            bool hasUnquotedPart = false;

            foreach (WordPart part in w.Parts)
            {
                string value = ExpandPart(part);
                if (!part.Quoted) hasUnquotedPart = true;
                foreach (char c in value) { text.Append(c); quotedMask.Add(part.Quoted); }
            }

            if (!hasUnquotedPart) return new List<string> { text.ToString() };

            string full = text.ToString();
            var results = new List<string>();
            var current = new StringBuilder();
            for (int i = 0; i < full.Length; i++)
            {
                char c = full[i];
                bool splitHere = (c == ' ' || c == '\t' || c == '\n') && !quotedMask[i];
                if (splitHere)
                {
                    if (current.Length > 0) { results.Add(current.ToString()); current.Clear(); }
                }
                else
                {
                    current.Append(c);
                }
            }
            if (current.Length > 0) results.Add(current.ToString());
            return results;
        }

        // Used where bash itself suppresses word-splitting regardless of
        // quoting: assignment right-hand sides and redirection targets
        // ("FOO=$X" and "> $X" both keep $X's expansion as one value, even
        // unquoted and even if it contains whitespace).
        internal string ExpandWordSingle(Word w)
        {
            var sb = new StringBuilder();
            foreach (WordPart part in w.Parts) sb.Append(ExpandPart(part));
            return sb.ToString();
        }

        private string ExpandPart(WordPart part) => part.Kind switch
        {
            PartKind.Literal => part.Text,
            PartKind.Variable => GetVariable(part.Text),
            PartKind.CommandSubstitution => RunCommandSubstitution(part.Text),
            _ => "",
        };

        internal string GetVariable(string name) => _variables.TryGetValue(name, out string? v) ? v : "";

        internal void SetVariable(string name, string value) => _variables[name] = value;

        internal bool UnsetVariable(string name) => _variables.Remove(name);

        // Real bash semantics: $(...) runs in a subshell, so assignments
        // made inside it don't leak back out - see CreateSubshell's own
        // comment for why a fresh interpreter (with a cloned variable set)
        // is used rather than just re-entering this same one.
        private string RunCommandSubstitution(string source)
        {
            ShellInterpreter sub = CreateSubshell();
            (bool needsMore, string output) = sub.Submit(source);
            if (needsMore)
            {
                throw new ShellSyntaxException("Command substitution \"$(...)\" isn't syntactically complete (e.g. a missing 'fi'/'done') - finish the block outside the substitution instead.");
            }
            return output.TrimEnd('\n');
        }
    }
}
