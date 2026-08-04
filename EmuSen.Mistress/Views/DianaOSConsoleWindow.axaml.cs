using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Threading;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Mistress.Views
{
    // A real terminal-style window onto the same EmuSen.DianaOS.DianaOS.Bin.DianaOSInterpreter
    // the F4 console prompt (EmuSen.Hotaru) and EmuSen.Pharaoh's
    // --commands scripting use - variables, $(...) command substitution,
    // pipes, redirection, if/for/while, the whole thing (see
    // EmuSen_Debugging_Tools_Reference_v5.md §3.17), just reachable from
    // the Avalonia GUI now.
    //
    // Deliberately NOT built on ConsoleLineReader (EmuSen.DianaOS's own
    // Console.ReadKey-based line editor) - that class exists specifically
    // for a raw terminal with no widget of its own to lean on. A real
    // Avalonia TextBox already owns its own text editing (cursor movement,
    // selection, IME, etc.) - this window only adds Up/Down-arrow history
    // recall on top via the TextBox's own KeyDown event, reading directly
    // from DianaOSInterpreter.History, exactly the division of labor that
    // class's own doc comment describes for a future GUI debug window.
    public partial class DianaOSConsoleWindow : Window
    {
        // See `man tmux`.
        private readonly DianaOSSessionManager _sessions = new();

        // One scheduler per session - see `man tmux`. ConcurrentDictionary
        // since MainWindow's _emuThread reads _scheduler (via DrainPendingFromEmulationThread)
        // while the UI thread can add/remove entries (SyncSchedulersToSessions,
        // off the back of a `tmux` HostAction).
        private readonly ConcurrentDictionary<DianaOSSession, DianaOSInterpreterScheduler> _schedulers = new();

        private DianaOSInterpreter _shell => _sessions.Current!.Interpreter;
        private DianaOSInterpreterScheduler _scheduler => _schedulers[_sessions.Current!];

        // Display text for the welcome banner, from the catalog rather than a second copy.
        private static readonly string[] SupportedCores =
            EmuSen.Cores.CoreCatalog.Cores.Select(c => c.DisplayName).ToArray();

        // Mistress-only builtins (pause/resume today) that DianaOSInterpreter's
        // own CreateDefault doesn't know about and shouldn't - see
        // EmulationControlCommands.cs's own comment. Held onto here (rather
        // than only passed once, at construction) because UpdateTarget
        // below rebuilds _shell from scratch on every ROM change and needs
        // to re-add them each time too.
        private readonly IEnumerable<IDianaOSCommand> _extraCommands;

        // -1 = "not currently recalling, editing whatever's live in the
        // box." Same recall algorithm ConsoleLineReader uses, just driven
        // by TextBox key events instead of raw console ones.
        private int _historyIndex = -1;
        private string _pendingInputText = "";

        // Design-time/XAML-previewer constructor only, matching every
        // other parameterized window in this project (InputSettingsWindow,
        // RomBrowserWindow, PreferencesWindow) - real code always uses the
        // one below with an actual (possibly null) target.
        public DianaOSConsoleWindow() : this(null, null) { }

        private IDebugTarget? _target;

        // From the loaded core's bundle, so this window names no core - see EmuSen_Multicore.md §4.
        private ICheatCodeCodec? _cheatAutoDetectCodec;
        private ICheatCodeCodec? _cheatExplicitCodec;
        private ICpuTraceSwitch? _cpuTraceSwitch;

        public DianaOSConsoleWindow(IDebugTarget? target, IEnumerable<IDianaOSCommand>? extraCommands = null,
            ICheatCodeCodec? cheatAutoDetectCodec = null, ICheatCodeCodec? cheatExplicitCodec = null,
            ICpuTraceSwitch? cpuTraceSwitch = null)
        {
            InitializeComponent();
            _extraCommands = Combine(extraCommands);
            _target = target;
            _cheatAutoDetectCodec = cheatAutoDetectCodec;
            _cheatExplicitCodec = cheatExplicitCodec;
            _cpuTraceSwitch = cpuTraceSwitch;
            var initial = new DianaOSSession("main", BuildInterpreter());
            _sessions.RegisterInitial(initial);
            _schedulers[initial] = new DianaOSInterpreterScheduler();

            // Printed once, right here - opening this window IS
            // "launching" this frontend's shell, the same one-time event
            // EmuSen.Hotaru's RunStandaloneShell prints its own copy of
            // this banner for. NOT re-printed by UpdateTarget below (a
            // ROM (re)load) - that's reopening/retargeting an
            // already-launched shell, not a fresh launch.
            AppendLine(_shell.GetWelcomeBanner(SupportedCores));
            if (target is null) AppendLine("No ROM loaded yet - commands needing a real target will report so until one is.");

            InputBox.KeyDown += OnInputKeyDown;
            Opened += (_, _) => InputBox.Focus();
        }

        // Adds this window's own `clear` override (below) onto whatever
        // MainWindow passed in (pause/resume/coretop/feed) - the caller
        // has no way to know about OutputText, so this window has to be
        // the one to supply a `clear` that actually does something here,
        // same as it's the one place that could ever plug in a
        // TextBox-clearing delegate.
        private System.Collections.Generic.IEnumerable<IDianaOSCommand> Combine(IEnumerable<IDianaOSCommand>? extraCommands)
        {
            var combined = new List<IDianaOSCommand>(extraCommands ?? System.Array.Empty<IDianaOSCommand>());
            combined.Add(new ClearConsoleWindowCommand(() => OutputText.Text = string.Empty));
            return combined;
        }

        // Called by MainWindow whenever the loaded ROM changes, including
        // the very first load (this window can be opened before any ROM
        // is loaded at all). A fresh DianaOSInterpreter is built rather than
        // trying to mutate the existing one's target - it's intentionally
        // immutable after construction (see DianaOSInterpreter's own
        // comment on why $(...) subshells clone rather than share
        // mutable state) - so this does mean history/variables reset
        // across a ROM swap. Accepted: the alternative is a shell command
        // silently running against a Cpu/Bus/Renderer from a core that
        // already stopped running the moment a new ROM loaded. pause/resume
        // are re-added here too since they're carried by the rebuilt
        // DianaOSInterpreter itself, not this window - they aren't
        // target-specific to begin with (pausing works with or without a
        // ROM loaded), but rebuilding the interpreter loses them just the
        // same as any other command unless they're passed again.
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
            _historyIndex = -1;
            AppendLine(romDisplayName != null ? $"--- ROM changed: {romDisplayName} ---" : "--- ROM unloaded ---");
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

        private void OnInputKeyDown(object? sender, KeyEventArgs e)
        {
            switch (e.Key)
            {
                case Key.Enter:
                    e.Handled = true;
                    string line = InputBox.Text ?? "";
                    InputBox.Text = "";
                    _historyIndex = -1;
                    Submit(line);
                    break;

                case Key.Up:
                    e.Handled = true;
                    RecallHistory(older: true);
                    break;

                case Key.Down:
                    e.Handled = true;
                    RecallHistory(older: false);
                    break;
            }
        }

        private void RecallHistory(bool older)
        {
            var entries = _shell.History.Entries;
            if (entries.Count == 0) return;

            if (older)
            {
                if (_historyIndex == -1)
                {
                    _pendingInputText = InputBox.Text ?? "";
                    _historyIndex = entries.Count - 1;
                }
                else if (_historyIndex > 0)
                {
                    _historyIndex--;
                }
            }
            else
            {
                if (_historyIndex == -1) return; // already on the fresh line
                if (_historyIndex < entries.Count - 1)
                {
                    _historyIndex++;
                }
                else
                {
                    _historyIndex = -1;
                    SetInputText(_pendingInputText);
                    return;
                }
            }

            SetInputText(entries[_historyIndex]);
        }

        private void SetInputText(string text)
        {
            InputBox.Text = text;
            InputBox.CaretIndex = text.Length;
        }

        private void Submit(string line)
        {
            string enteredPrompt = _shell.IsAwaitingMoreInput ? "> " : "DianaOS #: ";
            AppendLine(enteredPrompt + line);

            // A read-only line runs immediately, right here, safe because
            // it's reading a lock-free published snapshot (see
            // EmuSen.Cauldron.IRealtimeProvider) rather than touching live
            // core state directly. Anything else queues instead of running
            // inline on this (the UI) thread - MainWindow's own emulation
            // thread drains it once per frame via
            // DrainPendingFromEmulationThread below, the same "never race
            // RunFrame()" contract EmuSen.Hotaru's GameWindow already
            // established for its own console. That does mean a mutating
            // command's result now appears up to one frame later instead
            // of instantly - imperceptible at 60fps, and the price of this
            // window no longer racing the emulation thread at all.
            var result = _scheduler.SubmitFromAnyThread(_shell, line);
            if (result is { } r) ApplySubmitResult(r.NeedsMoreInput, r.Output, r.Action);
            // else: queued - ApplySubmitResult runs later, from
            // DrainPendingFromEmulationThread, once this line actually executes.
        }

        // Every HostAction except SwitchSession is still discarded here -
        // Mistress keeps its own pause/resume on the already-proven
        // PauseCommand/ResumeCommand delegate pattern (see
        // EmulationControlCommands.cs), and SwitchSession is the one
        // action this window itself must react to (see `man tmux`).
        private void ApplySubmitResult(bool needsMore, string output, HostAction? action)
        {
            if (action is HostAction.SwitchSession) SyncSchedulersToSessions();
            PromptText.Text = _shell.IsAwaitingMoreInput ? "> " : "DianaOS #: ";
            if (needsMore) return;
            if (output.Length > 0) AppendLine(output);
        }

        // Called once per frame from MainWindow's own emulation thread -
        // runs every command this window queued (typed while not
        // fast-path-eligible, see DianaOSInterpreterScheduler's own
        // comment) against the live core, then marshals each result back
        // to this window's UI thread. Safe to call even when nothing's
        // queued (a no-op loop). Must only be called from the thread that
        // actually owns the core, same contract as
        // DianaOSInterpreterScheduler.DrainPending itself.
        public void DrainPendingFromEmulationThread()
        {
            foreach (var (needsMore, output, action) in _scheduler.DrainPending(_shell))
            {
                // Captured per-iteration - Dispatcher.UIThread.Post queues
                // the closure to run later, it doesn't run it inline, so
                // each one needs its own copy rather than sharing the loop
                // variable.
                bool capturedNeedsMore = needsMore;
                string capturedOutput = output;
                HostAction? capturedAction = action;
                Dispatcher.UIThread.Post(() => ApplySubmitResult(capturedNeedsMore, capturedOutput, capturedAction));
            }
        }

        private void AppendLine(string text)
        {
            OutputText.Text = string.IsNullOrEmpty(OutputText.Text) ? text : OutputText.Text + "\n" + text;
            OutputScroll.ScrollToEnd();
        }
    }

    // Replaces (not adds alongside - same extraCommands override-by-name
    // mechanism `coretop`/`pause`/`resume`/`feed` already use here)
    // EmuSen.DianaOS.DianaOS.Bin.Commands.ClearCommand's `Console.Clear()`, which does
    // nothing useful against this window's own console - a TextBox, not a
    // real terminal, same reason `coretop`/`feed` needed replacements
    // rather than just refusing outright. Local to this file rather than
    // EmulationControlCommands.cs (which holds the commands that act on
    // MainWindow's own emulation thread/session) since this one only ever
    // needs a single window's own OutputText, supplied as a plain
    // delegate from Combine, above.
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
