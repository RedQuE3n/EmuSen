using System;
using EmuSen.Galaxia.Text;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.DianaOS.Ast;
using EmuSen.DianaOS.DianaOS.Bin.Commands;
using EmuSen.DianaOS.DianaOS.Bin.Commands.Unix;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin
{
    // A scoped-down bash-alike, and what it deliberately leaves out - see EmuSen_Debugging_Tools_Reference_v5.md §3.17.
    public class DianaOSInterpreter
    {
        private static readonly TimeSpan MaxExecutionTime = TimeSpan.FromSeconds(10);

        private readonly IDebugTarget? _target;
        private readonly IReadOnlyList<IDianaOSCommand> _orderedCommands;
        private readonly Dictionary<string, IDianaOSCommand> _commands;
        private readonly Dictionary<string, string> _variables;
        private readonly bool _isRoot;

        // Shared with any $(...) subshell, so a nested runaway cannot get a fresh budget - see §3.17.
        private Stopwatch? _stopwatch;

        // Holds raw text across Submit() calls while a block is still open - see §3.17.
        private string _pendingInput = "";

        // Recursion through the C# stack has no other circuit breaker - see §3.17.
        private int _sourceDepth;
        private const int MaxSourceDepth = 20;

        // Last one wins, read once at the end of SubmitCore - see §3.17a.
        private HostAction? _pendingHostAction;

        public CommandHistory History { get; }

        // Every registered command's own Name - see EmuSen_Debugging_Tools_Reference_v5.md §3.18 on why this exists.
        public IReadOnlyList<string> CommandNames => _orderedCommands.Select(c => c.Name).ToList();

        // Unix-flavour identity, not access control - see `man whoami` and `man su`.
        public string CurrentUser { get; set; } = "root";

        // Lets a frontend show a bash-style continuation prompt - see §3.17.
        public bool IsAwaitingMoreInput => _pendingInput.Length > 0;

        // Both sides share one CommandHistory rather than each getting its own.
        public DianaOSInterpreter(IDebugTarget? target, IEnumerable<IDianaOSCommand> commands, CommandHistory? history = null)
            : this(target, new List<IDianaOSCommand>(commands), null, new Dictionary<string, string>(), history ?? new CommandHistory(), isRoot: true)
        {
        }

        // The one-stop registry every frontend wants; an extra command REPLACES a same-named default - see §3.17.
        public static DianaOSInterpreter CreateDefault(
            IDebugTarget? target,
            IEnumerable<IDianaOSCommand>? extraCommands = null,
            ICheatCodeCodec? cheatAutoDetectCodec = null,
            ICheatCodeCodec? cheatExplicitCodec = null,
            ICpuTraceSwitch? cpuTraceSwitch = null,
            DianaOSSessionManager? sessions = null,
            Func<IReadOnlyCollection<string>>? supportedCheatSystems = null)
        {
            DianaOSSandbox.EnsureInitialWorkingDirectory();

            // snapshot/diff share one store, constructed here with every other command's state.
            var snapshotStore = new SnapshotStore();
            var history = new CommandHistory();

            // whoami/su/useradd need this interpreter's own CurrentUser, which Execute cannot reach.
            DianaOSInterpreter? self = null;
            Func<DianaOSInterpreter> selfAccessor = () => self!;

            var commands = new List<IDianaOSCommand>
            {
                new SpacesCommand(),
                new MemCommand(),
                new WriteCommand(),
                new RegsCommand(),
                new CopHistCommand(),
                new SpritesCommand(),
                new PalCommand(),
                new ChannelsCommand(),
                new MuteCommand(),
                new WatchCommand(),
                new BreakCommand(),
                new RunToCommand(),
                new BtCommand(),
                new EvalCommand(),
                new LabelCommand(),
                new CountersCommand(),
                new FreezeCommand(),
                new ProfileCommand(),
                new CpusCommand(),
                new CopFlowCommand(),
                new CovCommand(),
                new SetRegCommand(),
                new VectorsCommand(),
                new AddrCommand(),
                new DmaCommand(),
                new FrameLogCommand(),
                new CheatCommand(cheatAutoDetectCodec, cheatExplicitCodec, supportedCheatSystems),
                new SearchCommand(),
                new MemFindCommand(),
                new SnapshotCommand(snapshotStore),
                new DiffCommand(snapshotStore),
                new DumpCommand(),
                new LoadCommand(),
                new TileCommand(),
                new TilemapCommand(),
                new DisasmCommand(),
                new TraceCommand(cpuTraceSwitch),
                new CallersCommand(),
                new WritersCommand(),
                new ReadersCommand(),
                new LogCommand(),
                new EchoCommand(),
                new SedCommand(),
                new GrepCommand(),
                new WcCommand(),
                new SortCommand(),
                new UniqCommand(),
                new AwkCommand(),
                new LsCommand(),
                new CdCommand(selfAccessor),
                new MvCommand(),
                new CpCommand(),
                new RmCommand(),
                new PwdCommand(),
                new CatCommand(),
                new HeadCommand(),
                new TailCommand(),
                new TouchCommand(),
                new Commands.Unix.FindCommand(),
                new XxdCommand(),
                new NanoCommand(),
                new TmuxCommand(sessions, sessions is null ? null : () => CreateDefault(target, extraCommands, cheatAutoDetectCodec, cheatExplicitCodec, cpuTraceSwitch, sessions, supportedCheatSystems)),
                new PsCommand(sessions),
                new KillCommand(sessions),
                new WhoamiCommand(selfAccessor),
                new WhoCommand(selfAccessor, sessions),
                new SuCommand(selfAccessor),
                new UseraddCommand(selfAccessor),
                new UserdelCommand(selfAccessor, sessions),
                new PasswdCommand(selfAccessor),
                new CoretopCommand(),
                new VstopCommand(),
                new ClearCommand(),
                new TrueCommand(),
                new FalseCommand(),
                new TestCommand(),
                new BracketCommand(),
                new HistoryCommand(history),
                new ResumeCommand(),
                new ShutdownCommand(),
                new StepCommand(),
            };

            if (extraCommands is not null)
            {
                foreach (IDianaOSCommand extra in extraCommands)
                {
                    commands.RemoveAll(c => string.Equals(c.Name, extra.Name, StringComparison.OrdinalIgnoreCase));
                    commands.Add(extra);
                }
            }

            var interpreter = new DianaOSInterpreter(target, commands, history);
            self = interpreter;
            return interpreter;
        }

        private DianaOSInterpreter(
            IDebugTarget? target,
            IReadOnlyList<IDianaOSCommand> orderedCommands,
            Dictionary<string, IDianaOSCommand>? commandsDict,
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

        // A $(...) substitution runs in a real subshell, so assignments do not leak out - see §3.17.
        private DianaOSInterpreter CreateSubshell()
        {
            var sub = new DianaOSInterpreter(_target, _orderedCommands, _commands, new Dictionary<string, string>(_variables), new CommandHistory(), isRoot: false);
            sub._stopwatch = _stopwatch;
            sub.CurrentUser = CurrentUser;
            return sub;
        }

        // Exceptions are how a tree-walker unwinds to the nearest loop - see §3.17a.
        private class BreakSignal : Exception { }
        private class ContinueSignal : Exception { }

        public string Execute(string commandLine) => Submit(commandLine).Output;

        // Buffers raw text across calls whenever the parser says "not finished yet" - see §3.17.
        public (bool NeedsMoreInput, string Output, HostAction? Action) Submit(string rawLine) => SubmitCore(rawLine, interactive: true);

        // Answers whether a line is safe to run off the caller's thread, without running it - see §3.17.
        public bool TryGetReadOnlyFastPath(string rawLine, out string trimmed)
        {
            trimmed = (rawLine ?? "").Trim();
            if (trimmed.Length == 0) return false;

            // Neither can be classified without actually running Submit.
            if (_pendingInput.Length > 0) return false;
            if (trimmed.StartsWith('!')) return false;

            LexResult lexResult;
            try { lexResult = Lexer.Tokenize(trimmed); }
            catch { return false; }
            if (lexResult.NeedsMoreInput) return false;

            StatementList script;
            try { script = Parser.Parse(lexResult.Tokens); }
            catch { return false; }

            if (script.Statements.Count != 1) return false;
            if (script.Statements[0] is not AndOrList andOr) return false;
            if (andOr.Rest.Count != 0) return false;

            Pipeline pipeline = andOr.First;
            if (pipeline.Negate || pipeline.Stages.Count != 1) return false;

            SimpleCommand simple = pipeline.Stages[0];
            if (simple.Assignments.Count != 0 || simple.Redirections.Count != 0 || simple.Words.Count == 0) return false;

            Word nameWord = simple.Words[0];
            if (nameWord.Parts.Count != 1 || nameWord.Parts[0].Kind != PartKind.Literal) return false;

            // The same alias normalisation Dispatch applies before its own lookup - see §3.17a.
            string cmd = nameWord.Parts[0].Text.ToLowerInvariant();
            if (cmd == "c") cmd = "resume";
            else if (cmd == "quit") cmd = "shutdown";
            else if (cmd == "s") cmd = "step";
            else if (cmd == "jobs") cmd = "ps";
            else if (cmd == "hexdump") cmd = "xxd";

            return _commands.TryGetValue(cmd, out IDianaOSCommand? command) && command.IsReadOnly;
        }

        // A sourced script shares this scope but not history or `!` expansion - see §3.17.
        private (bool NeedsMoreInput, string Output, HostAction? Action) SubmitCore(string rawLine, bool interactive)
        {
            string line = rawLine ?? "";

            // History recall only makes sense at the start of a new command, never mid-block.
            if (interactive && _pendingInput.Length == 0 && line.TrimStart().StartsWith('!'))
            {
                try { line = ExpandHistoryReference(line.Trim()); }
                catch (Exception ex) { return (false, $"Error: {ex.Message}", null); }
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
                return (false, $"Error: {ex.Message}", null);
            }

            if (lexResult.NeedsMoreInput)
            {
                _pendingInput = combined;
                return (true, "", null);
            }

            StatementList script;
            try
            {
                script = Parser.Parse(lexResult.Tokens);
            }
            catch (ShellIncompleteException)
            {
                _pendingInput = combined;
                return (true, "", null);
            }
            catch (ShellSyntaxException ex)
            {
                _pendingInput = "";
                return (false, $"Syntax error: {ex.Message}", null);
            }

            _pendingInput = "";
            string finalText = combined.Trim();
            if (finalText.Length == 0) return (false, "", null);

            if (interactive) History.Add(finalText);
            if (_isRoot) _stopwatch = Stopwatch.StartNew();

            _pendingHostAction = null;
            var sb = new StringBuilder();
            try
            {
                ExecuteStatementList(script, sb);
            }
            catch (BreakSignal) { /* bare 'break' outside any loop - bash silently no-ops this */ }
            catch (ContinueSignal)
            {
                // Bare `continue` can only reach here from outside a loop, where it means resume - see §3.17a.
                _pendingHostAction = new HostAction.Resume();
            }
            catch (Exception ex)
            {
                if (sb.Length > 0 && sb[^1] != '\n') sb.Append('\n');
                sb.Append($"Error: {ex.Message}");
            }

            string result = sb.ToString();
            if (result.EndsWith('\n')) result = result[..^1];
            return (false, result, _pendingHostAction);
        }

        // Expanded before the line is recorded, so history shows what actually ran.
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
                string target = ExpandWordSingle(r.Target);
                if (!DianaOSSandbox.TryResolve(target, out string resolved))
                {
                    throw new IOException($"'{target}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
                }

                switch (r.Kind)
                {
                    case RedirectKind.Input:
                        effectiveStdin = File.ReadAllText(resolved);
                        break;
                    case RedirectKind.Truncate:
                        outputRedirectPath = resolved;
                        append = false;
                        break;
                    case RedirectKind.Append:
                        outputRedirectPath = resolved;
                        append = true;
                        break;
                }
            }

            DianaOSResult result = Dispatch(name, args.ToArray(), effectiveStdin);

            // A field rather than six changed signatures - see §3.17a.
            if (result.Action is not null) _pendingHostAction = result.Action;

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

        // These four need interpreter-internal state that IDianaOSCommand does not expose - see §3.17.
        private DianaOSResult Dispatch(string name, string[] args, string? stdin)
        {
            string cmd = name.ToLowerInvariant();

            // Two separate jobs, not one aliased to the other - see §3.17.
            if (cmd == "help") return Help();
            if (cmd == "man") return Man(args);

            if (cmd == "summary") return _target?.GetSummaryText() ?? "No target attached.";

            // There are no subprocesses to export to; kept so bash habits still do something sane - see §3.17.
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

            // Runs in THIS interpreter's scope, which is why it cannot be an ordinary command - see §3.17.
            if (cmd == "source" || cmd == ".") return RunScript(args);

            // Mapped then dropped through to the ordinary lookup; deliberately not `continue` - see §3.17a.
            if (cmd == "c") cmd = "resume";
            else if (cmd == "quit") cmd = "shutdown";
            else if (cmd == "s") cmd = "step";
            else if (cmd == "jobs") cmd = "ps";
            else if (cmd == "hexdump") cmd = "xxd";

            if (!_commands.TryGetValue(cmd, out IDianaOSCommand? command))
            {
                return DianaOSResult.Fail(
                    $"Unknown command '{cmd}'.{Suggestion.Hint(cmd, _commands.Keys)} Type 'help' for a list.");
            }

            try
            {
                return command.Execute(_target, args, stdin);
            }
            catch (Exception ex)
            {
                // A malformed command must read as failure to `&&`/`if`, never take the console down.
                return DianaOSResult.Fail($"Error: {ex.Message}");
            }
        }

        // Feeds lines back through this interpreter's own SubmitCore - see §3.17.
        private DianaOSResult RunScript(string[] args)
        {
            if (args.Length < 2) return DianaOSResult.Fail($"{args[0]}: usage: {args[0]} <path>");

            if (_sourceDepth >= MaxSourceDepth)
            {
                return DianaOSResult.Fail($"{args[0]}: too many nested source calls (>{MaxSourceDepth}) - probably a self-referential script.");
            }

            string requested = args[1];
            if (!DianaOSSandbox.TryResolve(requested, out string resolved))
            {
                return DianaOSResult.Fail($"{args[0]}: '{requested}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }
            if (!File.Exists(resolved))
            {
                return DianaOSResult.Fail($"{args[0]}: no such file: {requested}");
            }

            string[] lines;
            try { lines = File.ReadAllLines(resolved); }
            catch (Exception ex) { return DianaOSResult.Fail($"{args[0]}: {ex.Message}"); }

            var sb = new StringBuilder();
            HostAction? scriptAction = null;
            _sourceDepth++;
            try
            {
                foreach (string rawLine in lines)
                {
                    string trimmed = rawLine.TrimStart();
                    if (trimmed.Length == 0 || trimmed.StartsWith('#')) continue;

                    (bool needsMore, string output, HostAction? action) = SubmitCore(rawLine, interactive: false);
                    if (needsMore) continue; // still buffering a multi-line if/for/while block
                    if (output.Length > 0) { sb.Append(output); sb.Append('\n'); }

                    // A sourced script may trigger a host action, as bash's `exit` does; stop the script there.
                    if (action is not null) { scriptAction = action; break; }
                }
            }
            finally
            {
                _sourceDepth--;
            }

            // Or a script ending mid-block would silently swallow whatever is typed next - see §3.17.
            if (IsAwaitingMoreInput)
            {
                _pendingInput = "";
                sb.Append($"{args[0]}: unexpected end of file - unterminated block in {requested}");
            }

            string result = sb.ToString();
            if (result.EndsWith('\n')) result = result[..^1];
            return new DianaOSResult(result, 0, scriptAction);
        }

        // Falls back to Usage for a command with no page yet - see §3.17.
        private DianaOSResult Man(string[] args)
        {
            if (args.Length < 2) return Help();

            string target = args[1].ToLowerInvariant();
            string? page = ManPages.Lookup(target);
            if (page != null) return page;

            if (_commands.TryGetValue(target, out IDianaOSCommand? command))
            {
                return $"{target}\n\n{command.Usage}\n\n(No detailed manual page yet for this command - showing its one-line usage above.)";
            }

            return DianaOSResult.Fail($"No manual entry for '{target}'. Type 'help' for a list of commands.");
        }

        // Always this listing, never a single command's page - see §3.17.
        private string Help()
        {
            var lines = new List<string>
            {
                "Available commands:",
                "  help                          this text - every command, one line each, with a brief explanation",
                "  man [command]                 with no argument, same as 'help'; with a command name, its full manual page",
            };
            lines.AddRange(_orderedCommands.Select(c => c.Usage));
            lines.Add("  summary                       free-text state dump (whatever isn't structured above yet)");
            lines.Add("  export NAME[=value]           set a shell variable (no real subprocess env to export TO -");
            lines.Add("                                kept for script compatibility with real bash habits)");
            lines.Add("  unset NAME                    remove a shell variable");
            lines.Add("  source <path> (alias: .)      run a script file's lines in this same shell (variables/history scope)");
            lines.Add("  if/then/elif/else/fi, for/in/do/done, while|until/do/done, break, continue");
            lines.Add("                                control flow - see the reference doc for the full grammar");
            lines.Add("");
            lines.Add("Addresses/values are hex; an optional 0x or $ prefix is fine either way.");
            lines.Add("Shell features: quoting ('...'/\"...\"), $VAR/${VAR}/$? , $(command substitution),");
            lines.Add("pipes (|), redirection (> >> <), sequencing (; && ||), if/for/while - see");
            lines.Add("EmuSen_Debugging_Tools_Reference_v5.md for the full writeup and what's NOT supported.");
            return string.Join('\n', lines);
        }

        // Bump alongside a release-worthy milestone.
        public const string Version = "0.1a";

        // Printed once per shell launch, not per command or per F4 - see §3.17.
        public string GetWelcomeBanner(IEnumerable<string> supportedCores)
        {
            const string border = "+------------------------------------------------------------------------+";
            var lines = new List<string>
            {
                border,
                CenterInBanner($"Welcome to DianaOS v{Version}"),
                CenterInBanner("A virtualized micro operating system centered around emulation."),
                CenterInBanner("DianaOS is also a functional cross-platform Bash-like shell"),
                CenterInBanner("that accepts scripting."),
                border,
                "",
                "Supported cores:",
            };
            foreach (string core in supportedCores) lines.Add("  - " + core);
            lines.Add("");
            lines.Add("Features:");
            lines.Add("  - Pipes, redirection, $(...) command substitution, shell variables");
            lines.Add("  - Control flow: if/for/while, break/continue");
            lines.Add("  - Scripting: source <path> to run scripts, nano <path> to edit them");
            lines.Add("  - Live hardware inspection: regs, mem, watch, bp, disasm, trace");
            lines.Add("  - Cheats (Action Replay / Game Genie), snapshot/diff, dump/load");
            lines.Add("  - coretop: live htop-style hardware dashboard (-w opens a window)");
            lines.Add("  - vstop: the same, for the .NET runtime underneath (works with no core loaded)");
            lines.Add("  - feed / feed -w: watch gameplay without losing the shell");
            lines.Add("  - clear, nano, history, and the rest of a real shell's toolkit");
            lines.Add("");
            lines.Add("Type \"help\" for a list of commands. Man is supported for each.");
            return string.Join('\n', lines);
        }

        // Matches the border's own 72-column interior; fixed text, so a constant beats measuring.
        private static string CenterInBanner(string text)
        {
            const int innerWidth = 72;
            int pad = innerWidth - text.Length;
            int left = pad / 2;
            int right = pad - left;
            return "|" + new string(' ', left) + text + new string(' ', right) + "|";
        }

        // --- Word expansion ---

        // Quoted spans survive word-splitting; an unquoted word expanding to nothing vanishes - see §3.17a.
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

        // Where bash suppresses word-splitting regardless of quoting - see §3.17a.
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

        // A subshell, so assignments inside do not leak out - see §3.17.
        private string RunCommandSubstitution(string source)
        {
            DianaOSInterpreter sub = CreateSubshell();
            // A subshell's own host action is isolated by construction; the discard is only arity.
            (bool needsMore, string output, _) = sub.Submit(source);
            if (needsMore)
            {
                throw new ShellSyntaxException("Command substitution \"$(...)\" isn't syntactically complete (e.g. a missing 'fi'/'done') - finish the block outside the substitution instead.");
            }
            return output.TrimEnd('\n');
        }
    }
}
