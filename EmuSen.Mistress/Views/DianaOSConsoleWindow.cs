using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Threading;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Theme;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // A real terminal-style window onto the same DianaOSInterpreter the F4 prompt and EmuSen.Pharaoh's --commands use - see EmuSen_Debugging_Tools_Reference_v5.md §3.17.
    public class DianaOSConsoleWindow : ToolWindow
    {
        // See `man tmux`.
        private readonly DianaOSSessionManager _sessions = new();

        // One scheduler per session - see `man tmux`. ConcurrentDictionary since MainWindow's emulation thread reads it while the UI thread edits it.
        private readonly ConcurrentDictionary<DianaOSSession, DianaOSInterpreterScheduler> _schedulers = new();

        private readonly ConsolePane _console = new() { Prompt = "DianaOS #: " };

        private DianaOSInterpreter _shell => _sessions.Current!.Interpreter;
        private DianaOSInterpreterScheduler _scheduler => _schedulers[_sessions.Current!];

        // Display text for the welcome banner, from the catalog rather than a second copy.
        private static readonly string[] SupportedCores =
            EmuSen.Cores.CoreCatalog.Cores.Select(c => c.DisplayName).ToArray();

        // Mistress-only builtins that DianaOSInterpreter's CreateDefault does not know about - see EmulationControlCommands.cs.
        private readonly IEnumerable<IDianaOSCommand> _extraCommands;

        private IDebugTarget? _target;

        // From the loaded core's bundle, so this window names no core - see EmuSen_Multicore.md §4.
        private ICheatCodeCodec? _cheatAutoDetectCodec;
        private ICheatCodeCodec? _cheatExplicitCodec;
        private ICpuTraceSwitch? _cpuTraceSwitch;

        public DianaOSConsoleWindow() : this(null, null) { }

        public DianaOSConsoleWindow(IDebugTarget? target, IEnumerable<IDianaOSCommand>? extraCommands = null,
            ICheatCodeCodec? cheatAutoDetectCodec = null, ICheatCodeCodec? cheatExplicitCodec = null,
            ICpuTraceSwitch? cpuTraceSwitch = null)
        {
            _extraCommands = Combine(extraCommands);
            _target = target;
            _cheatAutoDetectCodec = cheatAutoDetectCodec;
            _cheatExplicitCodec = cheatExplicitCodec;
            _cpuTraceSwitch = cpuTraceSwitch;

            Title = "DianaOS";
            Width = 860;
            Height = 540;
            Background = LunaPalette.Surface;
            this.MinSize(420, 240);

            Content = _console.Margin(8);

            var initial = new DianaOSSession("main", BuildInterpreter());
            _sessions.RegisterInitial(initial);
            _schedulers[initial] = new DianaOSInterpreterScheduler();

            _console.HistorySource = () => _shell.History.Entries;
            _console.Submitted += Submit;

            // Printed once, right here - opening this window IS launching this frontend's shell.
            _console.AppendLine(_shell.GetWelcomeBanner(SupportedCores));
            if (target is null) _console.AppendLine("No ROM loaded yet - commands needing a real target will report so until one is.");

            Opened += (_, _) => _console.FocusInput();
        }

        // Adds this window's own `clear` override onto whatever MainWindow passed in - only this window can reach its own output.
        private IEnumerable<IDianaOSCommand> Combine(IEnumerable<IDianaOSCommand>? extraCommands)
        {
            var combined = new List<IDianaOSCommand>(extraCommands ?? System.Array.Empty<IDianaOSCommand>());
            combined.Add(new ClearConsoleWindowCommand(() => _console.Clear()));
            return combined;
        }

        // Called by MainWindow whenever the loaded ROM changes - a fresh interpreter, since it is immutable after construction.
        public void UpdateTarget(IDebugTarget? target, string? romDisplayName,
            ICheatCodeCodec? cheatAutoDetectCodec = null, ICheatCodeCodec? cheatExplicitCodec = null,
            ICpuTraceSwitch? cpuTraceSwitch = null)
        {
            _target = target;
            _cheatAutoDetectCodec = cheatAutoDetectCodec;
            _cheatExplicitCodec = cheatExplicitCodec;
            _cpuTraceSwitch = cpuTraceSwitch;
            _sessions.RebuildAll(BuildInterpreter);
            SyncSchedulersToSessions();
            _console.ResetHistoryRecall();
            _console.AppendLine(romDisplayName != null ? $"--- ROM changed: {romDisplayName} ---" : "--- ROM unloaded ---");
        }

        private DianaOSInterpreter BuildInterpreter() =>
            DianaOSInterpreter.CreateDefault(_target, _extraCommands,
                _cheatAutoDetectCodec, _cheatExplicitCodec, _cpuTraceSwitch,
                _sessions,
                () => EmuSen.Cores.CoreCatalog.SupportedCheatSystems);

        // Syncs _schedulers to whatever sessions currently exist - see `man tmux`.
        private void SyncSchedulersToSessions()
        {
            var live = new HashSet<DianaOSSession>(_sessions.Sessions);
            foreach (DianaOSSession stale in _schedulers.Keys.Where(s => !live.Contains(s)).ToList())
            {
                _schedulers.TryRemove(stale, out _);
            }
            foreach (DianaOSSession session in _sessions.Sessions)
            {
                if (!_schedulers.ContainsKey(session)) _schedulers[session] = new DianaOSInterpreterScheduler();
            }
        }

        private void Submit(string line)
        {
            string enteredPrompt = _shell.IsAwaitingMoreInput ? "> " : "DianaOS #: ";
            _console.AppendLine(enteredPrompt + line);

            // A read-only line runs inline against a published snapshot; anything else queues for the emulation thread - see EmuSen_Cauldron.md §2.
            var result = _scheduler.SubmitFromAnyThread(_shell, line);
            if (result is { } r) ApplySubmitResult(r.NeedsMoreInput, r.Output, r.Action);
        }

        // SwitchSession is the one HostAction this window itself must react to - see `man tmux`.
        private void ApplySubmitResult(bool needsMore, string output, HostAction? action)
        {
            if (action is HostAction.SwitchSession) SyncSchedulersToSessions();
            _console.Prompt = _shell.IsAwaitingMoreInput ? "> " : "DianaOS #: ";
            if (needsMore) return;
            if (output.Length > 0) _console.AppendLine(output);
        }

        // Called once per frame from MainWindow's emulation thread; must only be called from the thread that owns the core.
        public void DrainPendingFromEmulationThread()
        {
            foreach (var (needsMore, output, action) in _scheduler.DrainPending(_shell))
            {
                // Captured per-iteration - Post queues the closure rather than running it inline.
                bool capturedNeedsMore = needsMore;
                string capturedOutput = output;
                HostAction? capturedAction = action;
                Dispatcher.UIThread.Post(() => ApplySubmitResult(capturedNeedsMore, capturedOutput, capturedAction));
            }
        }
    }

    // Replaces DianaOS's own ClearCommand, whose Console.Clear() does nothing against a widget - see EmuSen_Debugging_Tools_Reference_v5.md §3.17.
    public class ClearConsoleWindowCommand : IDianaOSCommand
    {
        private readonly System.Action _clearOutput;

        public ClearConsoleWindowCommand(System.Action clearOutput)
        {
            _clearOutput = clearOutput;
        }

        public string Name => "clear";
        public bool IsReadOnly => true;
        public string Usage => "  clear                         clear this console window's output";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            _clearOutput();
            return DianaOSResult.Ok(string.Empty);
        }
    }
}
