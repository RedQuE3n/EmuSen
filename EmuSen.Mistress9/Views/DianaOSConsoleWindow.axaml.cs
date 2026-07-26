using System.Collections.Generic;
using Avalonia.Controls;
using Avalonia.Input;
using EmuSen.DianaOS;

namespace EmuSen.Mistress9.Views
{
    // A real terminal-style window onto the same EmuSen.DianaOS.DianaOSInterpreter
    // the F4 console prompt (EmuSen.Hotaru) and EmuSen.Pharaoh90's
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
        private DianaOSInterpreter _shell;

        // Same single-entry list PreferencesWindow.AvailableCores already
        // hardcodes for its own core-selection combo - kept as its own
        // small copy here rather than a shared reference since it's just
        // display text for the welcome banner (GetWelcomeBanner, below),
        // not anything either window actually needs to stay in sync on.
        private static readonly string[] SupportedCores = { "SNES (Venus)" };

        // Mistress9-only builtins (pause/resume today) that DianaOSInterpreter's
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

        public DianaOSConsoleWindow(IDebugTarget? target, IEnumerable<IDianaOSCommand>? extraCommands = null)
        {
            InitializeComponent();
            _extraCommands = Combine(extraCommands);
            _shell = DianaOSInterpreter.CreateDefault(target, _extraCommands);

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
        public void UpdateTarget(IDebugTarget? target, string? romDisplayName)
        {
            _shell = DianaOSInterpreter.CreateDefault(target, _extraCommands);
            _historyIndex = -1;
            AppendLine(romDisplayName != null ? $"--- ROM changed: {romDisplayName} ---" : "--- ROM unloaded ---");
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

            // Action is discarded here for now - this window doesn't react
            // to a HostAction yet (Mistress9 keeps its own pause/resume on
            // the already-proven PauseCommand/ResumeCommand delegate
            // pattern instead - see EmulationControlCommands.cs's own
            // header comment on why). Behavior-neutral: reacting to a
            // HostAction here is a deliberately separate, later decision.
            (bool needsMore, string output, _) = _shell.Submit(line);
            PromptText.Text = _shell.IsAwaitingMoreInput ? "> " : "DianaOS #: ";
            if (needsMore) return;
            if (output.Length > 0) AppendLine(output);
        }

        private void AppendLine(string text)
        {
            OutputText.Text = string.IsNullOrEmpty(OutputText.Text) ? text : OutputText.Text + "\n" + text;
            OutputScroll.ScrollToEnd();
        }
    }

    // Replaces (not adds alongside - same extraCommands override-by-name
    // mechanism `coretop`/`pause`/`resume`/`feed` already use here)
    // EmuSen.DianaOS.Commands.ClearCommand's `Console.Clear()`, which does
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
        public string Usage => "  clear                         clear this console window's output";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            _clearOutput();
            return DianaOSResult.Ok(string.Empty);
        }
    }
}
