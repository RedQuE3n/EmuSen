using System;
using EmuSen.Galaxia.Text;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.DianaSh.Ast;
using EmuSen.DianaSh.DianaSh.Bin.Commands;
using EmuSen.DianaSh.DianaSh.Bin.Commands.Unix;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Session;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.DianaSh.DianaSh.Dev;
using System.Text.Json;
using EmuSen.DianaSh.DianaSh.Bin.Native;

namespace EmuSen.DianaSh.DianaSh.Bin
{
    // The shell's facade: the C# interpreter below, or the Rust one under EMUSEN_DIANASH_NATIVE=1 - see EmuSen_DianaSh.md §13.3 and §13.7.
    public class DianaShInterpreter
    {
        private readonly Managed? _managed;
        private readonly NativeShell? _shell;
        private readonly uint _session;

        // The session service a frontend on Crystal's API handed this shell, whose run events answer Resume and Step.
        private ServiceTable? _service;

        public CommandHistory History { get; }

        // Every registered command's own Name - see EmuSen_Debugging_Tools_Reference_v5.md §3.18 on why this exists.
        public IReadOnlyList<string> CommandNames => _managed?.CommandNames ?? Native.Commands(_session).Select(c => c.Name).ToList();

        // Unix-flavour identity, not access control - see `man whoami` and `man su`.
        public string CurrentUser
        {
            get => _managed?.CurrentUser ?? Native.User(_session);
            set
            {
                if (_managed is not null) _managed.CurrentUser = value;
                else Native.SetUser(_session, value);
            }
        }

        // Lets a frontend show a bash-style continuation prompt - see §3.17.
        public bool IsAwaitingMoreInput => _managed?.IsAwaitingMoreInput ?? Native.Awaiting(_session);

        // Both sides share one CommandHistory rather than each getting its own.
        public DianaShInterpreter(IDebugTarget? target, IEnumerable<IDianaShCommand> commands, CommandHistory? history = null)
        {
            if (!DianaShNative.Active)
            {
                _managed = new Managed(target, commands, history);
                History = _managed.History;
                return;
            }
            List<IDianaShCommand> given = commands.ToList();
            _shell = new NativeShell(keepsSessions: false);
            _shell.SetSessionService(0, new SessionService(target));
            _shell.AddHost(0, HostManifest(_shell, 0, given, given), new HostCommandFactory(target, () => given, given));
            History = new CommandHistory(() => Native.History(_session));
        }

        private DianaShInterpreter(Managed managed)
        {
            _managed = managed;
            History = managed.History;
        }

        private DianaShInterpreter(NativeShell shell, uint session)
        {
            _shell = shell;
            _session = session;
            History = new CommandHistory(() => Native.History(_session));
        }

        private NativeShell Native => _shell!;

        // A session the Rust shell made itself, by tmux new, as its manager first meets it.
        internal static DianaShInterpreter ForNativeSession(NativeShell shell, uint session) => new(shell, session);

        // The Rust shell and session behind this facade, or none for the C# one.
        internal NativeShell? NativeShellOf => _shell;

        internal uint NativeSessionId => _session;

        // The one-stop registry every frontend wants; an extra command REPLACES a same-named default - see §3.17.
        public static DianaShInterpreter CreateDefault(
            IDebugTarget? target,
            IEnumerable<IDianaShCommand>? extraCommands = null,
            ICheatCodeCodec? cheatAutoDetectCodec = null,
            ICheatCodeCodec? cheatExplicitCodec = null,
            ICpuTraceSwitch? cpuTraceSwitch = null,
            DianaShSessionManager? sessions = null,
            Func<IReadOnlyCollection<string>>? supportedCheatSystems = null)
        {
            DianaShSandbox.EnsureInitialWorkingDirectory();
            if (DianaShNative.Active) return CreateNative(target, extraCommands, cheatAutoDetectCodec, cheatExplicitCodec, cpuTraceSwitch, sessions, supportedCheatSystems);
            return CreateManaged(target, extraCommands, cheatAutoDetectCodec, cheatExplicitCodec, cpuTraceSwitch, sessions, supportedCheatSystems, null);
        }

        // A shell with no game, as a launch prompt makes one before the frontend API has a session to hand it - see EmuSen_Backend.md §14.11.
        public static DianaShInterpreter CreateWithoutSession(IEnumerable<IDianaShCommand>? extraCommands = null, Func<IReadOnlyCollection<string>>? supportedCheatSystems = null) =>
            CreateDefault(null, extraCommands, supportedCheatSystems: supportedCheatSystems);

        // The C# interpreter; over a session's service, each shell `tmux new` makes is over it too.
        private static DianaShInterpreter CreateManaged(
            IDebugTarget? target,
            IEnumerable<IDianaShCommand>? extraCommands,
            ICheatCodeCodec? cheatAutoDetectCodec,
            ICheatCodeCodec? cheatExplicitCodec,
            ICpuTraceSwitch? cpuTraceSwitch,
            DianaShSessionManager? sessions,
            Func<IReadOnlyCollection<string>>? supportedCheatSystems,
            SessionService? service)
        {
            var history = new CommandHistory();

            // whoami/su/useradd need this interpreter's own CurrentUser, which Execute cannot reach.
            DianaShInterpreter? self = null;
            Func<DianaShInterpreter> selfAccessor = () => self!;
            Func<DianaShInterpreter>? newSession = sessions is null ? null : () => CreateManaged(target, extraCommands, cheatAutoDetectCodec, cheatExplicitCodec, cpuTraceSwitch, sessions, supportedCheatSystems, service);
            List<IDianaShCommand> commands = DefaultCommands(target, extraCommands, cheatAutoDetectCodec, cheatExplicitCodec, cpuTraceSwitch, sessions, supportedCheatSystems, selfAccessor, history, newSession);
            self = new DianaShInterpreter(new Managed(target, commands, history)) { _service = service is null ? null : new ServiceTable(service) };
            return self;
        }

        // Every command in the order `help` lists them, a host's in place of a default of its name - see §3.17.
        private static List<IDianaShCommand> DefaultCommands(
            IDebugTarget? target,
            IEnumerable<IDianaShCommand>? extraCommands,
            ICheatCodeCodec? cheatAutoDetectCodec,
            ICheatCodeCodec? cheatExplicitCodec,
            ICpuTraceSwitch? cpuTraceSwitch,
            DianaShSessionManager? sessions,
            Func<IReadOnlyCollection<string>>? supportedCheatSystems,
            Func<DianaShInterpreter> selfAccessor,
            CommandHistory history,
            Func<DianaShInterpreter>? newSession = null)
        {
            // snapshot/diff share one store, constructed here with every other command's state.
            var snapshotStore = new SnapshotStore();

            var commands = new List<IDianaShCommand>
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
                new MemcardCommand(),
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
                new LogCommand(() => DebugLogging.MasterEnabled, on => DebugLogging.MasterEnabled = on),
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
                new TmuxCommand(sessions, sessions is null ? null : newSession ?? (() => CreateDefault(target, extraCommands, cheatAutoDetectCodec, cheatExplicitCodec, cpuTraceSwitch, sessions, supportedCheatSystems))),
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

            // Listed after a host's commands, where `help` has always put it - see EmuSen_DianaSh.md §13.5.
            var trailing = new List<IDianaShCommand> { new SummaryCommand() };

            if (extraCommands is not null)
            {
                foreach (IDianaShCommand extra in extraCommands)
                {
                    commands.RemoveAll(c => string.Equals(c.Name, extra.Name, StringComparison.OrdinalIgnoreCase));
                    trailing.RemoveAll(c => string.Equals(c.Name, extra.Name, StringComparison.OrdinalIgnoreCase));
                    commands.Add(extra);
                }
            }
            commands.AddRange(trailing);
            return commands;
        }

        // The Rust shell, with every C# command it does not answer itself registered as the host's - see EmuSen_DianaSh.md §13.7.
        // A shell over a session's emusen.session@1 table, as a frontend on Crystal's API hands it one; its commands reach the session through that table - see EmuSen_Backend.md §14.8.
        // With sessions, `tmux` makes more shells over the same table; Resume and Step come back from the session's own run events - see EmuSen_Backend.md §14.11.
        public static unsafe DianaShInterpreter CreateForService(nint sessionService, IEnumerable<IDianaShCommand>? extraCommands = null, DianaShSessionManager? sessions = null)
        {
            SessionService? provider = SessionService.Of((EmuSenSessionV1*)sessionService);
            if (provider is null) return CreateForForeignService(sessionService, extraCommands, sessions);
            SessionServiceOptions o = provider.Options;
            DianaShSandbox.EnsureInitialWorkingDirectory();
            DianaShInterpreter shell = DianaShNative.Active
                ? CreateNative(provider.DebugTarget, extraCommands, o.AutoDetectCodec, o.ExplicitCodec, o.Trace, sessions, o.CheatSystems, provider)
                : CreateManaged(provider.DebugTarget, extraCommands, o.AutoDetectCodec, o.ExplicitCodec, o.Trace, sessions, o.CheatSystems, provider);
            shell._service = new ServiceTable(provider);
            return shell;
        }

        // A table the platform library's Crystal made: its modules call it, and no C# command has a target - see EmuSen_Backend.md §29.25.
        private static DianaShInterpreter CreateForForeignService(nint sessionService, IEnumerable<IDianaShCommand>? extraCommands, DianaShSessionManager? sessions)
        {
            if (sessionService == 0) throw new ArgumentNullException(nameof(sessionService));
            if (!DianaShNative.Active) throw new ArgumentException("the C# shell's commands need this process's provider; a table the platform library made needs the Rust shell", nameof(sessionService));
            DianaShSandbox.EnsureInitialWorkingDirectory();
            var table = new ServiceTable(sessionService);
            DianaShInterpreter shell = CreateNative(null, extraCommands, null, null, null, sessions, null, null, table);
            shell._service = table;
            return shell;
        }

        private static DianaShInterpreter CreateNative(
            IDebugTarget? target,
            IEnumerable<IDianaShCommand>? extraCommands,
            ICheatCodeCodec? cheatAutoDetectCodec,
            ICheatCodeCodec? cheatExplicitCodec,
            ICpuTraceSwitch? cpuTraceSwitch,
            DianaShSessionManager? sessions,
            Func<IReadOnlyCollection<string>>? supportedCheatSystems,
            SessionService? provider = null,
            ServiceTable? foreign = null)
        {
            NativeShell shell = sessions is null ? new NativeShell(keepsSessions: false) : sessions.Native ??= new NativeShell(keepsSessions: true);
            uint session = sessions is null ? 0 : shell.NewSession();
            List<IDianaShCommand> extras = extraCommands?.ToList() ?? new List<IDianaShCommand>();
            // The commands that reach the shell's own state are the Rust shell's builtins, so none here is given the manager or the facade.
            IReadOnlyList<IDianaShCommand> Build() => DefaultCommands(target, extras, cheatAutoDetectCodec, cheatExplicitCodec, cpuTraceSwitch, null, supportedCheatSystems, Unreachable, new CommandHistory());
            IReadOnlyList<IDianaShCommand> prototype = Build();
            // The session's own provider over its target, which the Rust command modules call - see EmuSen_DianaSh.md §13.11.
            if (foreign is not null) shell.SetSessionService(session, foreign);
            else shell.SetSessionService(session, provider ?? new SessionService(target, new SessionServiceOptions
            {
                AutoDetectCodec = cheatAutoDetectCodec,
                ExplicitCodec = cheatExplicitCodec,
                Trace = cpuTraceSwitch,
                CheatSystems = supportedCheatSystems,
                CheatDatabaseFolder = () => global::EmuSen.Galaxia.Library.SettingFolders.Cheats(global::EmuSen.Galaxia.Models.AppSettings.Load()),
            }), owned: provider is null);
            shell.AddHost(session, HostManifest(shell, session, prototype, extras), new HostCommandFactory(target, Build, prototype));
            var facade = new DianaShInterpreter(shell, session);
            shell.Remember(session, facade);
            return facade;
        }

        private static DianaShInterpreter Unreachable() => throw new InvalidOperationException("a command the Rust shell answers itself was run from C#");

        // The host's manifest: each C# command but those the shell answers itself or by a module, a host's own always, and the listing's order - see EmuSen_DianaSh.md §13.7.
        private static string HostManifest(NativeShell shell, uint session, IReadOnlyList<IDianaShCommand> commands, IEnumerable<IDianaShCommand> extras)
        {
            var builtins = new HashSet<string>(shell.Commands(session).Where(c => c.Kind is "builtin" or "module" or "program").Select(c => c.Name), StringComparer.OrdinalIgnoreCase);
            var own = new HashSet<string>(extras.Select(e => e.Name), StringComparer.OrdinalIgnoreCase);
            using var buffer = new System.IO.MemoryStream();
            using (var json = new Utf8JsonWriter(buffer))
            {
                json.WriteStartObject();
                json.WriteNumber("manifest", 1);
                json.WriteString("id", "emusen.host");
                json.WriteString("version", Version);
                json.WriteString("abi", "1.0");
                json.WriteString("kind", "host");
                json.WriteStartArray("commands");
                foreach (IDianaShCommand c in commands.Where(c => own.Contains(c.Name) || !builtins.Contains(c.Name)))
                {
                    json.WriteStartObject();
                    json.WriteString("name", c.Name);
                    json.WriteString("usage", c.Usage);
                    json.WriteBoolean("read_only", c.IsReadOnly);
                    json.WriteEndObject();
                }
                json.WriteEndArray();
                json.WriteStartObject("aliases");
                json.WriteString("c", "resume");
                json.WriteString("s", "step");
                json.WriteString("hexdump", "xxd");
                json.WriteEndObject();
                // A prefix is claimed with the command that makes its jobs, so a module's bp and watch list their own.
                json.WriteStartArray("jobs");
                foreach (string prefix in new[] { "bp", "watch" })
                    if (!builtins.Contains(prefix) || own.Contains(prefix)) json.WriteStringValue(prefix);
                json.WriteEndArray();
                json.WriteStartArray("order");
                foreach (IDianaShCommand c in commands) json.WriteStringValue(c.Name);
                json.WriteEndArray();
                json.WriteStartArray("notes");
                json.WriteStringValue("Addresses/values are hex; an optional 0x or $ prefix is fine either way.");
                json.WriteEndArray();
                json.WriteEndObject();
            }
            return Encoding.UTF8.GetString(buffer.ToArray());
        }

        public string Execute(string commandLine) => Submit(commandLine).Output;

        // Buffers raw text across calls whenever the parser says "not finished yet" - see §3.17.
        public (bool NeedsMoreInput, string Output, HostAction? Action) Submit(string rawLine)
        {
            // A run event the frontend raised itself, outside a line, is not this line's.
            _service?.DrainRunRequests();
            var result = SubmitToShell(rawLine);
            return _service is null ? result : FromRunEvents(result);
        }

        // Over a session's service a C# command's resume or step is asked of run control, as the Rust shell's modules ask it; either way the session's run events answer - see EmuSen_Backend.md §14.11.
        private unsafe (bool NeedsMoreInput, string Output, HostAction? Action) FromRunEvents((bool NeedsMoreInput, string Output, HostAction? Action) result)
        {
            if (_managed is null) return result;
            if (result.Action is HostAction.Resume or HostAction.Step)
            {
                EmuSenSessionV1* table = _service!.Table;
                table->run_control(table->ctx, 0, SessionAbi.RunResume, 0, 0);
            }
            string? run = null;
            foreach (string op in _service!.DrainRunRequests()) run = op;
            if (result.Action is not null || run is null) return result;
            return (result.NeedsMoreInput, result.Output, run == "step" ? new HostAction.Step() : new HostAction.Resume());
        }

        private (bool NeedsMoreInput, string Output, HostAction? Action) SubmitToShell(string rawLine)
        {
            if (_managed is not null) return _managed.Submit(rawLine);
            var (status, output, requests) = Native.Submit(_session, rawLine ?? "");
            HostAction? action = null;
            foreach (NativeRequest request in requests)
            {
                action = request.Kind switch
                {
                    "exit" => new HostAction.Shutdown(),
                    "session" => new HostAction.SwitchSession(request.Name),
                    "continue" => new HostAction.Resume(),
                    "host" => HostActions.Take(request.Value),
                    _ => action,
                };
            }
            // Run control's requests, made by a Rust module through the session service, stand for Step and Resume - see EmuSen_DianaSh.md §13.11.
            string? run = Native.DrainRunRequest();
            action ??= run switch
            {
                "step" => new HostAction.Step(),
                "resume" => new HostAction.Resume(),
                _ => null,
            };
            return (status == DianaShNative.NeedsMore, output, action);
        }

        // Answers whether a line is safe to run off the caller's thread, without running it - see §3.17.
        public bool TryGetReadOnlyFastPath(string rawLine, out string trimmed)
        {
            if (_managed is not null) return _managed.TryGetReadOnlyFastPath(rawLine, out trimmed);
            trimmed = (rawLine ?? "").Trim();
            return trimmed.Length > 0 && Native.IsReadOnly(_session, trimmed);
        }

        // The last line's exit status, as `$?` reads it - for the golden corpus, see EmuSen_DianaSh.md §13.4.
        internal string LastExitStatus => _managed?.LastExitStatus ?? Native.LastStatus(_session).ToString();

        // Every command, in the order `help` lists them - for the golden corpus, see EmuSen_DianaSh.md §13.4.
        internal IReadOnlyList<IDianaShCommand> RegisteredCommands =>
            _managed?.RegisteredCommands ?? Native.Commands(_session).Select(c => (IDianaShCommand)new ListedCommand(c.Name, c.Usage, c.ReadOnly)).ToList();

        // A command as the Rust shell lists it, for what reads the listing and never runs it.
        private sealed record ListedCommand(string Name, string Usage, bool IsReadOnly) : IDianaShCommand
        {
            public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin) => throw new InvalidOperationException($"{Name} is listed, not run, here");
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
                CenterInBanner($"Welcome to DianaSh v{Version}"),
                CenterInBanner("A Unix shell centered around emulation."),
                CenterInBanner("DianaSh is also a functional cross-platform Bash-like shell"),
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

        // Today's C# shell, unchanged but for its name, which the facade runs unless the Rust one is chosen - see EmuSen_DianaSh.md §13.3.
        private sealed class Managed
        {
            private static readonly TimeSpan MaxExecutionTime = TimeSpan.FromSeconds(10);

            private readonly IDebugTarget? _target;
            private readonly CommandTable _table;
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
            public IReadOnlyList<string> CommandNames => _table.Ordered.Select(c => c.Name).ToList();

            // Unix-flavour identity, not access control - see `man whoami` and `man su`.
            public string CurrentUser { get; set; } = "root";

            // Lets a frontend show a bash-style continuation prompt - see §3.17.
            public bool IsAwaitingMoreInput => _pendingInput.Length > 0;


            // Both sides share one CommandHistory rather than each getting its own.
            internal Managed(IDebugTarget? target, IEnumerable<IDianaShCommand> commands, CommandHistory? history = null)
                : this(target, new CommandTable(new List<IDianaShCommand>(commands)), new Dictionary<string, string>(), history ?? new CommandHistory(), isRoot: true)
            {
            }

            private Managed(
                IDebugTarget? target,
                CommandTable table,
                Dictionary<string, string> variables,
                CommandHistory history,
                bool isRoot)
            {
                _target = target;
                _table = table;
                _variables = variables;
                History = history;
                _isRoot = isRoot;
                if (!_variables.ContainsKey("?")) _variables["?"] = "0";
            }

            // A $(...) substitution runs in a real subshell, so assignments do not leak out - see §3.17.
            private Managed CreateSubshell()
            {
                var sub = new Managed(_target, _table, new Dictionary<string, string>(_variables), new CommandHistory(), isRoot: false);
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

                return _table.IsReadOnly(nameWord.Parts[0].Text);
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
                    if (!DianaShSandbox.TryResolve(target, out string resolved))
                    {
                        throw new IOException($"'{target}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
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

                DianaShResult result = Dispatch(name, args.ToArray(), effectiveStdin);

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

            // These five need interpreter-internal state that IDianaShCommand does not expose - see §3.17.
            private DianaShResult Dispatch(string name, string[] args, string? stdin, bool asItself = false)
            {
                string cmd = name.ToLowerInvariant();

                // Two separate jobs, not one aliased to the other - see §3.17.
                if (cmd == "help") return Help();
                if (cmd == "man") return Man(args);

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

                // Which kind answers a name, a command run as itself, and a player's module approved - see `man which`, `man command`, `man trust`.
                if (cmd == "which") return Which(args);
                if (cmd == "command") return RunAsItself(args, stdin);
                if (cmd == "trust") return Trust(args);

                // Mapped then dropped through to the ordinary lookup; deliberately not `continue` - see §3.17a.
                if (!asItself) cmd = CommandTable.Canonical(cmd);

                if (!_table.TryFind(cmd, out IDianaShCommand? command))
                {
                    return DianaShResult.Fail(
                        $"Unknown command '{cmd}'.{Suggestion.Hint(cmd, _table.Names)} Type 'help' for a list.");
                }

                try
                {
                    return command.Execute(_target, args, stdin);
                }
                catch (Exception ex)
                {
                    // A malformed command must read as failure to `&&`/`if`, never take the console down.
                    return DianaShResult.Fail($"Error: {ex.Message}");
                }
            }

            // The names the shell answers itself: the language's, and the builtins of EmuSen_DianaSh.md §4.2 - see `man which`.
            private static readonly HashSet<string> ShellBuiltins = new(StringComparer.Ordinal)
            {
                "help", "man", "export", "unset", "source", ".", "which", "command", "trust",
                "echo", "cd", "pwd", "tmux", "ps", "kill", "whoami", "who", "su", "useradd", "userdel", "passwd",
                "true", "false", "test", "[", "history", "shutdown",
            };

            private DianaShResult Which(string[] args)
            {
                if (args.Length < 2) return DianaShResult.Fail("which: usage: which <name>...");
                var lines = new List<string>();
                int status = 0;
                foreach (string name in args.Skip(1))
                {
                    string lower = name.ToLowerInvariant();
                    if (ShellBuiltins.Contains(lower)) lines.Add($"{name}: shell builtin");
                    else if (CommandTable.AliasOf(lower) is { } target) lines.Add($"{name}: alias for {target}");
                    else if (_table.TryFind(lower, out _)) lines.Add($"{name}: command");
                    else { lines.Add($"which: no {name}"); status = 1; }
                }
                return new DianaShResult(string.Join('\n', lines), status);
            }

            private DianaShResult RunAsItself(string[] args, string? stdin)
            {
                if (args.Length < 2) return DianaShResult.Fail("command: usage: command [-v] <name> [args...]");
                if (args[1] != "-v") return Dispatch(args[1], args[1..], stdin, asItself: true);
                if (args.Length < 3) return DianaShResult.Fail("command: usage: command [-v] <name> [args...]");
                var found = args.Skip(2).Where(n => ShellBuiltins.Contains(n.ToLowerInvariant()) || CommandTable.AliasOf(n.ToLowerInvariant()) is not null || _table.TryFind(n.ToLowerInvariant(), out _)).ToList();
                return new DianaShResult(string.Join('\n', found), found.Count == args.Length - 2 ? 0 : 1);
            }

            private DianaShResult Trust(string[] args)
            {
                if (args.Length < 2) return DianaShResult.Fail("trust: usage: trust <path>");
                if (_sourceDepth > 0 || !_isRoot) return DianaShResult.Fail("trust: a module is approved at the prompt, not by a script");
                if (!DianaShSandbox.TryResolve(args[1], out string resolved))
                    return DianaShResult.Fail($"trust: '{args[1]}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
                if (!File.Exists(resolved)) return DianaShResult.Fail($"trust: no such file: {args[1]}");
                string hash = Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(resolved))).ToLowerInvariant();
                string store = Path.Combine(EmuSen.Galaxia.ConfigStore.Directory, "dianash");
                Directory.CreateDirectory(store);
                File.AppendAllText(Path.Combine(store, "trusted"), $"{hash}  {resolved}\n");
                return $"trust: approved {args[1]} (sha256 {hash})";
            }

            // Feeds lines back through this interpreter's own SubmitCore - see §3.17.
            private DianaShResult RunScript(string[] args)
            {
                if (args.Length < 2) return DianaShResult.Fail($"{args[0]}: usage: {args[0]} <path>");

                if (_sourceDepth >= MaxSourceDepth)
                {
                    return DianaShResult.Fail($"{args[0]}: too many nested source calls (>{MaxSourceDepth}) - probably a self-referential script.");
                }

                string requested = args[1];
                if (!DianaShSandbox.TryResolve(requested, out string resolved))
                {
                    return DianaShResult.Fail($"{args[0]}: '{requested}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
                }
                if (!File.Exists(resolved))
                {
                    return DianaShResult.Fail($"{args[0]}: no such file: {requested}");
                }

                string[] lines;
                try { lines = File.ReadAllLines(resolved); }
                catch (Exception ex) { return DianaShResult.Fail($"{args[0]}: {ex.Message}"); }

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
                return new DianaShResult(result, 0, scriptAction);
            }

            // Falls back to Usage for a command with no page yet - see §3.17.
            private DianaShResult Man(string[] args)
            {
                if (args.Length < 2) return Help();

                string target = args[1].ToLowerInvariant();
                string? page = ManPages.Lookup(target);
                if (page != null) return page;

                if (_table.TryFind(target, out IDianaShCommand? command))
                {
                    return $"{target}\n\n{command.Usage}\n\n(No detailed manual page yet for this command - showing its one-line usage above.)";
                }

                return DianaShResult.Fail($"No manual entry for '{target}'. Type 'help' for a list of commands.");
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
                lines.AddRange(_table.Ordered.Select(c => c.Usage));
                lines.Add("  export NAME[=value]           set a shell variable (no real subprocess env to export TO -");
                lines.Add("                                kept for script compatibility with real bash habits)");
                lines.Add("  unset NAME                    remove a shell variable");
                lines.Add("  source <path> (alias: .)      run a script file's lines in this same shell (variables/history scope)");
                lines.Add("  which <name>...               say what answers each name: an alias, a shell builtin or a command");
                lines.Add("  command [-v] <name> [args]    run <name> as itself, not as an alias; -v prints the names that would run");
                lines.Add("  trust <path>                  approve a module file you added, by its hash (typed at the prompt only)");
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

            // The last line's exit status, as `$?` reads it - for the golden corpus, see EmuSen_DianaSh.md §13.4.
            internal string LastExitStatus => GetVariable("?");

            // Every command, in the order `help` lists them - for the golden corpus, see EmuSen_DianaSh.md §13.4.
            internal IReadOnlyList<IDianaShCommand> RegisteredCommands => _table.Ordered;

            internal void SetVariable(string name, string value) => _variables[name] = value;

            internal bool UnsetVariable(string name) => _variables.Remove(name);

            // A subshell, so assignments inside do not leak out - see §3.17.
            private string RunCommandSubstitution(string source)
            {
                Managed sub = CreateSubshell();
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
}
