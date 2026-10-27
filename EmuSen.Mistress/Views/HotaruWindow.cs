using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Input.Platform;
using Avalonia.Threading;
using EmuSen.Mistress.Input;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Client;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Theme;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // Hotaru, the shell's window: a terminal onto the DianaShInterpreter with a toolbar for what a pad cannot type - see EmuSen_HotaruWindow.md and EmuSen_Debugging_Tools_Reference_v5.md §3.17.
    public class HotaruWindow : ToolWindow, IPadDriven
    {
        // See `man tmux`.
        private readonly DianaShSessionManager _sessions = new();

        // One scheduler per session - see `man tmux`. ConcurrentDictionary since MainWindow's emulation thread reads it while the UI thread edits it.
        private readonly ConcurrentDictionary<DianaShSession, DianaShInterpreterScheduler> _schedulers = new();

        private readonly ConsolePane _console = new() { Prompt = "DianaSh #: " };

        private DianaShInterpreter _shell => _sessions.Current!.Interpreter;
        private DianaShInterpreterScheduler _scheduler => _schedulers[_sessions.Current!];

        // Display text for the welcome banner, from the runtime's shelves rather than a second copy.
        private static string[] SupportedCores => EmuSen.Mistress.Library.RuntimeCatalog.Consoles.Select(c => c.DisplayName).ToArray();

        // Mistress-only builtins that DianaShInterpreter's CreateDefault does not know about - see EmulationControlCommands.cs.
        private readonly IEnumerable<IDianaShCommand> _extraCommands;

        // The running game's session, whose service table the shell is made over; null with no game - see EmuSen_Backend.md §14.13.
        private CrystalSession? _session;

        // The newest non-empty output, which the toolbar's copy puts on the clipboard.
        private string _lastOutput = "";

        public const string ToolbarName = "HotaruToolbar";

        public HotaruWindow() : this(null, null) { }

        public HotaruWindow(CrystalSession? session, IEnumerable<IDianaShCommand>? extraCommands = null)
        {
            _extraCommands = Combine(extraCommands);
            _session = session;

            Title = "Shell";
            Width = 860;
            Height = 540;
            this.MinSize(420, 240);

            var toolbar = new ToolBar { Name = ToolbarName };
            toolbar.SetActions(
                new LunaAction("History Back", HistoryBack),
                new LunaAction("History Forward", HistoryForward),
                new LunaAction("Complete", Complete),
                new LunaAction("Clear", () => _console.Clear()),
                new LunaAction("Copy Output", CopyLastOutput));
            var hint = new HintText { Name = "HotaruHint", Text = "L1 / R1 recall history · L2 / R2 complete · B clears the line, then closes" };
            DockPanel.SetDock(toolbar, Dock.Top);
            DockPanel.SetDock(hint, Dock.Bottom);
            Content = new DockPanel { Children = { toolbar, hint, _console.Margin(8) } };

            var initial = new DianaShSession("main", BuildInterpreter());
            _sessions.RegisterInitial(initial);
            _schedulers[initial] = new DianaShInterpreterScheduler();

            _console.HistorySource = () => _shell.History.Entries;
            _console.Submitted += Submit;

            // Printed once, right here - opening this window IS launching this frontend's shell.
            _console.AppendLine(_shell.GetWelcomeBanner(SupportedCores));
            if (_refusal is not null) _console.AppendLine($"[ERROR] {_refusal}");
            else if (session is null) _console.AppendLine("No ROM loaded yet - commands needing a real target will report so until one is.");

            Opened += (_, _) => _console.FocusInput();
        }

        public void HistoryBack() => _console.HistoryBack();

        public void HistoryForward() => _console.HistoryForward();

        // Completes the first word to the longest prefix the command names share, with a space when one command remains.
        public void Complete()
        {
            string typed = _console.InputText;
            if (typed.Length == 0 || typed.Any(char.IsWhiteSpace)) return;
            List<string> matches = _shell.CommandNames.Where(n => n.StartsWith(typed, System.StringComparison.Ordinal)).ToList();
            if (matches.Count == 0) return;
            string common = matches.Aggregate((a, b) => a.Substring(0, CommonLength(a, b)));
            _console.Insert(common.Substring(typed.Length) + (matches.Count == 1 ? " " : ""));
        }

        private static int CommonLength(string a, string b)
        {
            int n = 0;
            while (n < a.Length && n < b.Length && a[n] == b[n]) n++;
            return n;
        }

        // The last output put on the clipboard; nothing when there has been none.
        public void CopyLastOutput()
        {
            if (_lastOutput.Length == 0) return;
            _ = GetTopLevel(this)?.Clipboard?.SetTextAsync(_lastOutput);
        }

        // Shoulders step through history, a trigger completes, and B with a line typed clears it before it closes the window.
        public bool OnPad(UiButton button)
        {
            switch (button)
            {
                case UiButton.PageUp: HistoryBack(); return true;
                case UiButton.PageDown: HistoryForward(); return true;
                case UiButton.First or UiButton.Last: Complete(); return true;
                case UiButton.Back when _console.InputText.Length > 0: _console.SetInput(""); return true;
                default: return false;
            }
        }

        // Adds this window's own `clear` override onto whatever MainWindow passed in - only this window can reach its own output.
        private IEnumerable<IDianaShCommand> Combine(IEnumerable<IDianaShCommand>? extraCommands)
        {
            var combined = new List<IDianaShCommand>(extraCommands ?? System.Array.Empty<IDianaShCommand>());
            combined.Add(new ClearConsoleWindowCommand(() => _console.Clear()));
            return combined;
        }

        // Called by MainWindow whenever the loaded ROM changes - a fresh interpreter, since it is immutable after construction.
        public void UpdateTarget(CrystalSession? session, string? romDisplayName)
        {
            _session = session;
            _sessions.RebuildAll(BuildInterpreter);
            SyncSchedulersToSessions();
            _console.ResetHistoryRecall();
            _console.AppendLine(romDisplayName != null ? $"--- ROM changed: {romDisplayName} ---" : "--- ROM unloaded ---");
            if (_refusal is not null) _console.AppendLine($"[ERROR] {_refusal}");
        }

        // The words the shell refused the running game's table in, said at the prompt; null where the shell is over the table - see EmuSen_Backend.md §29.29.
        private string? _refusal;

        // Over the running game's table where this process's shell can go over it, else a shell with no game that says why.
        private DianaShInterpreter BuildInterpreter()
        {
            _refusal = null;
            if (_session is { Closed: false } session)
            {
                nint table = session.ServiceTable();
                try
                {
                    DianaShInterpreter.CheckService(table);
                    return DianaShInterpreter.CreateForService(table, _extraCommands, _sessions);
                }
                catch (DianaShTableRefusedException refused)
                {
                    _refusal = refused.Message;
                }
            }
            return DianaShInterpreter.CreateWithoutSession(_extraCommands, () => EmuSen.Mistress.Library.RuntimeCatalog.SupportedCheatSystems);
        }

        // Syncs _schedulers to whatever sessions currently exist - see `man tmux`.
        private void SyncSchedulersToSessions()
        {
            var live = new HashSet<DianaShSession>(_sessions.Sessions);
            foreach (DianaShSession stale in _schedulers.Keys.Where(s => !live.Contains(s)).ToList())
            {
                _schedulers.TryRemove(stale, out _);
            }
            foreach (DianaShSession session in _sessions.Sessions)
            {
                if (!_schedulers.ContainsKey(session)) _schedulers[session] = new DianaShInterpreterScheduler();
            }
        }

        private void Submit(string line)
        {
            string enteredPrompt = _shell.IsAwaitingMoreInput ? "> " : "DianaSh #: ";
            _console.AppendLine(enteredPrompt + line);

            // A read-only line runs inline against a published snapshot; anything else queues for the emulation thread - see EmuSen_Debugging_Tools_Reference_v5.md §3.66.2.
            var result = _scheduler.SubmitFromAnyThread(_shell, line);
            if (result is { } r) ApplySubmitResult(r.NeedsMoreInput, r.Output, r.Action);
        }

        // SwitchSession is the one HostAction this window itself must react to - see `man tmux`.
        private void ApplySubmitResult(bool needsMore, string output, HostAction? action)
        {
            if (action is HostAction.SwitchSession) SyncSchedulersToSessions();
            _console.Prompt = _shell.IsAwaitingMoreInput ? "> " : "DianaSh #: ";
            if (needsMore) return;
            if (output.Length > 0) { _lastOutput = output; _console.AppendLine(output); }
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

    // Replaces DianaSh's own ClearCommand, whose Console.Clear() does nothing against a widget - see EmuSen_Debugging_Tools_Reference_v5.md §3.17.
    public class ClearConsoleWindowCommand : HostCommand
    {
        private readonly System.Action _clearOutput;

        public ClearConsoleWindowCommand(System.Action clearOutput)
        {
            _clearOutput = clearOutput;
        }

        public override string Name => "clear";
        public override bool IsReadOnly => true;
        public override string Usage => "  clear                         clear this console window's output";

        protected override DianaShResult Run(string[] args, string? stdin)
        {
            _clearOutput();
            return DianaShResult.Ok(string.Empty);
        }
    }
}
