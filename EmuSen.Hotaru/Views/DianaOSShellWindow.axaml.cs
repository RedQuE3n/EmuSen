using System;
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

namespace EmuSen.Hotaru.Views
{
    // The shell a windowed launch gets in place of the terminal - see EmuSen_Frontend_Driver.md §3c.
    public partial class DianaOSShellWindow : Window
    {
        private readonly Func<string, Window>? _launchRom;
        private readonly Func<string, string?>? _resolveCoreCommand;

        // Null until a ROM launches; from then on every line goes to the running core instead.
        private ILiveShell? _live;

        private DianaOSInterpreter _shell;

        private int _historyIndex = -1;
        private string _pendingInputText = "";

        // Design-time/XAML-previewer constructor only, matching Hotaru's other windows.
        public DianaOSShellWindow() : this(null, null, null) { }

        public DianaOSShellWindow(
            Func<string, Window>? launchRom,
            Func<string, string?>? resolveCoreCommand,
            IEnumerable<string>? supportedCores)
        {
            InitializeComponent();
            _launchRom = launchRom;
            _resolveCoreCommand = resolveCoreCommand;
            _shell = BuildInterpreter();

            AppendLine(_shell.GetWelcomeBanner(supportedCores ?? Array.Empty<string>()));
            AppendLine("--- DianaOS (type 'help', 'core <name> <path>' to launch a game, 'shutdown' to quit) ---");

            InputBox.KeyDown += OnInputKeyDown;
            Opened += (_, _) => InputBox.Focus();
        }

        private DianaOSInterpreter BuildInterpreter() =>
            DianaOSInterpreter.CreateDefault(null,
                new IDianaOSCommand[] { new ClearShellWindowCommand(() => OutputText.Text = string.Empty) },
                supportedCheatSystems: () => EmuSen.Cores.CoreCatalog.SupportedCheatSystems);

        // Reports a ROM that could not be loaded back into the window that asked for it.
        public void ReportLaunchFailure(string message)
        {
            AppendLine(message);
            InputBox.Focus();
        }

        // Hands this window over to the running core's own interpreter - see EmuSen_Frontend_Driver.md §3c.
        public void AttachLiveShell(ILiveShell live)
        {
            _live = live;
            _historyIndex = -1;
            live.Output += OnLiveOutput;
            Closed += (_, _) => live.Output -= OnLiveOutput;
            AppendLine("--- Game started. This shell is now attached to the running core. ---");
            SyncPrompt();
        }

        private void OnLiveOutput(string output)
        {
            // Raised from the emulation and console-reader threads, never this one.
            Dispatcher.UIThread.Post(() =>
            {
                if (output.Length > 0) AppendLine(output);
                SyncPrompt();
            });
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

        private string[] HistoryEntries() =>
            _live is { } live ? live.SnapshotHistory() : _shell.History.Entries.ToArray();

        // Same recall algorithm ConsoleLineReader uses, driven by TextBox key events.
        private void RecallHistory(bool older)
        {
            string[] entries = HistoryEntries();
            if (entries.Length == 0) return;

            if (older)
            {
                if (_historyIndex == -1)
                {
                    _pendingInputText = InputBox.Text ?? "";
                    _historyIndex = entries.Length - 1;
                }
                else if (_historyIndex > 0)
                {
                    _historyIndex--;
                }
            }
            else
            {
                if (_historyIndex == -1) return;
                if (_historyIndex < entries.Length - 1)
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
            AppendLine(CurrentPrompt() + line);

            if (_live is { } live)
            {
                live.Submit(line);
                SyncPrompt();
                return;
            }

            string trimmed = line.Trim();
            if (!_shell.IsAwaitingMoreInput && trimmed.StartsWith("core ", StringComparison.OrdinalIgnoreCase))
            {
                LaunchFromCoreCommand(trimmed);
                return;
            }

            (_, string output, HostAction? action) = _shell.Submit(line);
            if (output.Length > 0) AppendLine(output);
            SyncPrompt();
            if (action is HostAction.Shutdown) Close();
        }

        // `core <name> <path>` is resolved by Program, not the interpreter - see EmuSen_Frontend_Driver.md §3.
        private void LaunchFromCoreCommand(string trimmed)
        {
            if (_resolveCoreCommand is null || _launchRom is null) return;

            string? romPath = _resolveCoreCommand(trimmed);
            if (romPath is null) return; // the resolver already appended why

            // A ROM that will not load must land back at the prompt, not take the window down with it.
            try
            {
                Window game = _launchRom(romPath);
                game.Show();
            }
            catch (Exception ex)
            {
                ReportLaunchFailure($"[CPU HALT] {ex.Message}");
            }
        }

        private string CurrentPrompt() =>
            _live is { IsAwaitingMoreInput: true } || (_live is null && _shell.IsAwaitingMoreInput) ? "> " : "DianaOS #: ";

        private void SyncPrompt() => PromptText.Text = CurrentPrompt();

        public void AppendLine(string text)
        {
            OutputText.Text = string.IsNullOrEmpty(OutputText.Text) ? text : OutputText.Text + "\n" + text;
            OutputScroll.ScrollToEnd();
        }
    }

    // The live interpreter a running GameWindow lends to the shell window - see EmuSen_Frontend_Driver.md §3c.
    public interface ILiveShell
    {
        bool IsAwaitingMoreInput { get; }
        string[] SnapshotHistory();
        void Submit(string line);
        event Action<string> Output;
    }

    // Replaces ClearCommand's Console.Clear(), which does nothing to a TextBox.
    public class ClearShellWindowCommand : IDianaOSCommand
    {
        private readonly Action _clearOutput;

        public ClearShellWindowCommand(Action clearOutput) => _clearOutput = clearOutput;

        public string Name => "clear";
        public bool IsReadOnly => true;
        public string Usage => "  clear                         clear this shell window's output";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            _clearOutput();
            return DianaOSResult.Ok(string.Empty);
        }
    }
}
