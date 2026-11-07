using System;
using EmuSen.Galaxia.Text;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.DianaSh.DianaSh.Bin.Commands;
using EmuSen.DianaSh.DianaSh.Bin.Commands.Unix;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Session;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.DianaSh.DianaSh.Dev;
using System.Text.Json;
using EmuSen.DianaSh.DianaSh.Bin.Native;

namespace EmuSen.DianaSh.DianaSh.Bin
{
    // The shell's facade over the Rust shell in the platform library - see EmuSen_DianaSh.md §13.7 and EmuSen_Backend.md §40.7.
    public class DianaShInterpreter
    {
        private readonly NativeShell _shell;
        private readonly uint _session;

        // The session service a frontend on Crystal's API handed this shell, whose run events answer Resume and Step.
        private ServiceTable? _service;

        public CommandHistory History { get; }

        // Every registered command's own Name - see EmuSen_Debugging_Tools_Reference_v5.md §3.18 on why this exists.
        public IReadOnlyList<string> CommandNames => Native.Commands(_session).Select(c => c.Name).ToList();

        // Unix-flavour identity, not access control - see `man whoami` and `man su`.
        public string CurrentUser
        {
            get => Native.User(_session);
            set => Native.SetUser(_session, value);
        }

        // Lets a frontend show a bash-style continuation prompt - see §3.17.
        public bool IsAwaitingMoreInput => Native.Awaiting(_session);

        private DianaShInterpreter(NativeShell shell, uint session)
        {
            _shell = shell;
            _session = session;
            History = new CommandHistory(() => Native.History(_session));
        }

        private NativeShell Native => _shell;

        // A session the Rust shell made itself, by tmux new, as its manager first meets it.
        internal static DianaShInterpreter ForNativeSession(NativeShell shell, uint session) => new(shell, session);

        // The Rust shell and session behind this facade.
        internal NativeShell NativeShellOf => _shell;

        internal uint NativeSessionId => _session;

        // A shell with no game, as a launch prompt makes one before the frontend API has a session to hand it - see EmuSen_Backend.md §14.11.
        // The host's runtime for the settings service, which the shell hands its command modules - see EmuSen_DianaSh.md §9.9.
        public void UseSettings(EmuSen.DianaSh.DianaSh.Bin.Native.IShellSettings settings) => _shell.SetSettings(settings);

        // The runtime's table with no game (frontend API 1.12's SERVICE_NO_GAME) answers the cheat database; zero hands the shell no session service - see EmuSen_Backend.md §40.7.
        public static DianaShInterpreter CreateWithoutSession(nint noGameService = 0, IEnumerable<IDianaShCommand>? extraCommands = null, DianaShSessionManager? sessions = null)
        {
            DianaShSandbox.EnsureInitialWorkingDirectory();
            ServiceTable? table = noGameService == 0 ? null : new ServiceTable(noGameService);
            DianaShInterpreter shell = CreateNative(extraCommands, sessions, table);
            shell._service = table;
            return shell;
        }

        // Every command's name in the order `help` has always listed them; summary follows a host's own - see EmuSen_DianaSh.md §3.17 and §13.5.
        private static readonly string[] HelpOrder =
        {
            "spaces", "mem", "write", "regs", "cophist", "sprites", "pal", "channels", "mute", "watch", "bp", "runto", "bt", "eval", "label",
            "counters", "freeze", "profile", "cpus", "copflow", "cov", "setreg", "vectors", "addr", "dma", "framelog", "cheat", "memcard",
            "search", "memfind", "snapshot", "diff", "dump", "load", "tile", "tilemap", "disasm", "trace", "callers", "writers", "readers",
            "log", "echo", "sed", "grep", "wc", "sort", "uniq", "awk", "ls", "cd", "mv", "cp", "rm", "pwd", "cat", "head", "tail", "touch",
            "find", "xxd", "nano", "tmux", "ps", "kill", "whoami", "who", "su", "useradd", "userdel", "passwd", "coretop", "vstop", "clear",
            "true", "false", "test", "[", "history", "resume", "shutdown", "step",
        };

        // The order `help` lists them in, a host's command in place of a default of its name moved to the end, summary last.
        private static List<string> ListingOrder(IEnumerable<IDianaShCommand> extras)
        {
            var names = HelpOrder.ToList();
            var trailing = new List<string> { "summary" };
            foreach (IDianaShCommand extra in extras)
            {
                names.RemoveAll(n => string.Equals(n, extra.Name, StringComparison.OrdinalIgnoreCase));
                trailing.RemoveAll(n => string.Equals(n, extra.Name, StringComparison.OrdinalIgnoreCase));
                names.Add(extra.Name);
            }
            names.AddRange(trailing);
            return names;
        }

        // The commands with no Rust twin, which the host answers, and a host's own in place of one of their names - see EmuSen_Backend.md §40.7.
        private static List<IDianaShCommand> HostCommands(IEnumerable<IDianaShCommand> extras)
        {
            var commands = new List<IDianaShCommand> { new MemcardCommand(), new VstopCommand() };
            foreach (IDianaShCommand extra in extras)
            {
                commands.RemoveAll(c => string.Equals(c.Name, extra.Name, StringComparison.OrdinalIgnoreCase));
                commands.Add(extra);
            }
            return commands;
        }

        // The Rust shell, with every C# command it does not answer itself registered as the host's - see EmuSen_DianaSh.md §13.7.
        // A shell over a session's emusen.session@1 table, as a frontend on Crystal's API hands it one; its commands reach the session through that table - see EmuSen_Backend.md §14.8.
        // With sessions, `tmux` makes more shells over the same table; Resume and Step come back from the session's own run events - see EmuSen_Backend.md §14.11.
        public static DianaShInterpreter CreateForService(nint sessionService, IEnumerable<IDianaShCommand>? extraCommands = null, DianaShSessionManager? sessions = null)
        {
            if (sessionService == 0) throw new ArgumentNullException(nameof(sessionService));
            DianaShSandbox.EnsureInitialWorkingDirectory();
            var table = new ServiceTable(sessionService);
            DianaShInterpreter shell = CreateNative(extraCommands, sessions, table);
            shell._service = table;
            return shell;
        }

        private static DianaShInterpreter CreateNative(IEnumerable<IDianaShCommand>? extraCommands, DianaShSessionManager? sessions, ServiceTable? table)
        {
            NativeShell shell = sessions is null ? new NativeShell(keepsSessions: false) : sessions.Native ??= new NativeShell(keepsSessions: true);
            uint session = sessions is null ? 0 : shell.NewSession();
            List<IDianaShCommand> extras = extraCommands?.ToList() ?? new List<IDianaShCommand>();
            // The commands that reach the shell's own state are the Rust shell's builtins, so none here is given the manager or the facade.
            IReadOnlyList<IDianaShCommand> Build() => HostCommands(extras);
            IReadOnlyList<IDianaShCommand> prototype = Build();
            // The session's table, which the Rust command modules call; a shell with none answers every game's command with "No ROM loaded" - see EmuSen_DianaSh.md §13.11.
            if (table is not null) shell.SetSessionService(session, table);
            shell.AddHost(session, HostManifest(prototype, extras), new HostCommandFactory(Build, prototype));
            var facade = new DianaShInterpreter(shell, session);
            shell.Remember(session, facade);
            return facade;
        }

        // The host's manifest: its commands, the aliases, and the listing's order - see EmuSen_DianaSh.md §13.7.
        private static string HostManifest(IReadOnlyList<IDianaShCommand> commands, IEnumerable<IDianaShCommand> extras)
        {
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
                foreach (IDianaShCommand c in commands)
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
                // The breakpoints' and watches' jobs are the trace module's, which lists and kills its own.
                json.WriteStartArray("jobs");
                json.WriteEndArray();
                json.WriteStartArray("order");
                foreach (string name in ListingOrder(extras)) json.WriteStringValue(name);
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
            return SubmitToShell(rawLine);
        }

        private (bool NeedsMoreInput, string Output, HostAction? Action) SubmitToShell(string rawLine)
        {
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

        // Stops the running line at its next check, from any thread: it prints ^C and leaves $? at 130 - see EmuSen_DianaSh.md §10.2.
        public void Interrupt() => Native.Cancel();

        // Answers whether a line is safe to run off the caller's thread, without running it - see §3.17.
        public bool TryGetReadOnlyFastPath(string rawLine, out string trimmed)
        {
            trimmed = (rawLine ?? "").Trim();
            return trimmed.Length > 0 && Native.IsReadOnly(_session, trimmed);
        }

        // The last line's exit status, as `$?` reads it - for the golden corpus, see EmuSen_DianaSh.md §13.4.
        internal string LastExitStatus => Native.LastStatus(_session).ToString();

        // Every command, in the order `help` lists them - for the golden corpus, see EmuSen_DianaSh.md §13.4.
        internal IReadOnlyList<IDianaShCommand> RegisteredCommands =>
            Native.Commands(_session).Select(c => (IDianaShCommand)new ListedCommand(c.Name, c.Usage, c.ReadOnly)).ToList();

        // A command as the Rust shell lists it, for what reads the listing and never runs it.
        private sealed record ListedCommand(string Name, string Usage, bool IsReadOnly) : IDianaShCommand
        {
            public DianaShResult Execute(string[] args, string? stdin) => throw new InvalidOperationException($"{Name} is listed, not run, here");
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
    }
}
