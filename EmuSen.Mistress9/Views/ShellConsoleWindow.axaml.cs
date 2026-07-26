using Avalonia.Controls;
using Avalonia.Input;
using EmuSen.Shell;

namespace EmuSen.Mistress9.Views
{
    // A real terminal-style window onto the same EmuSen.Shell.ShellInterpreter
    // the F4 console prompt (EmuSen.Hotaru) and EmuSen.Pharaoh90's
    // --commands scripting use - variables, $(...) command substitution,
    // pipes, redirection, if/for/while, the whole thing (see
    // EmuSen_Debugging_Tools_Reference_v5.md §3.17), just reachable from
    // the Avalonia GUI now.
    //
    // Deliberately NOT built on ConsoleLineReader (EmuSen.Shell's own
    // Console.ReadKey-based line editor) - that class exists specifically
    // for a raw terminal with no widget of its own to lean on. A real
    // Avalonia TextBox already owns its own text editing (cursor movement,
    // selection, IME, etc.) - this window only adds Up/Down-arrow history
    // recall on top via the TextBox's own KeyDown event, reading directly
    // from ShellInterpreter.History, exactly the division of labor that
    // class's own doc comment describes for a future GUI debug window.
    public partial class ShellConsoleWindow : Window
    {
        private ShellInterpreter _shell;

        // -1 = "not currently recalling, editing whatever's live in the
        // box." Same recall algorithm ConsoleLineReader uses, just driven
        // by TextBox key events instead of raw console ones.
        private int _historyIndex = -1;
        private string _pendingInputText = "";

        // Design-time/XAML-previewer constructor only, matching every
        // other parameterized window in this project (InputSettingsWindow,
        // RomBrowserWindow, PreferencesWindow) - real code always uses the
        // one below with an actual (possibly null) target.
        public ShellConsoleWindow() : this(null) { }

        public ShellConsoleWindow(IDebugTarget? target)
        {
            InitializeComponent();
            _shell = ShellInterpreter.CreateDefault(target);

            AppendLine("EmuSen shell console - type 'help' for a list of commands.");
            if (target is null) AppendLine("No ROM loaded yet - commands needing a real target will report so until one is.");

            InputBox.KeyDown += OnInputKeyDown;
            Opened += (_, _) => InputBox.Focus();
        }

        // Called by MainWindow whenever the loaded ROM changes, including
        // the very first load (this window can be opened before any ROM
        // is loaded at all). A fresh ShellInterpreter is built rather than
        // trying to mutate the existing one's target - it's intentionally
        // immutable after construction (see ShellInterpreter's own
        // comment on why $(...) subshells clone rather than share
        // mutable state) - so this does mean history/variables reset
        // across a ROM swap. Accepted: the alternative is a shell command
        // silently running against a Cpu/Bus/Renderer from a core that
        // already stopped running the moment a new ROM loaded.
        public void UpdateTarget(IDebugTarget? target, string? romDisplayName)
        {
            _shell = ShellInterpreter.CreateDefault(target);
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
            string enteredPrompt = _shell.IsAwaitingMoreInput ? "> " : "debug> ";
            AppendLine(enteredPrompt + line);

            (bool needsMore, string output) = _shell.Submit(line);
            PromptText.Text = _shell.IsAwaitingMoreInput ? "> " : "debug> ";
            if (needsMore) return;
            if (output.Length > 0) AppendLine(output);
        }

        private void AppendLine(string text)
        {
            OutputText.Text = string.IsNullOrEmpty(OutputText.Text) ? text : OutputText.Text + "\n" + text;
            OutputScroll.ScrollToEnd();
        }
    }
}
