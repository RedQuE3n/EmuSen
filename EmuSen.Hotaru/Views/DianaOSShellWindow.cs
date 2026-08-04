using System;
using System.Collections.Generic;
using Avalonia.Controls;
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

namespace EmuSen.Hotaru.Views
{
    // The shell a windowed launch gets in place of the terminal - see EmuSen_Frontend_Driver.md §3c.
    public class DianaOSShellWindow : ToolWindow
    {
        private readonly Func<string, Window>? _launchRom;
        private readonly Func<string, string?>? _resolveCoreCommand;

        private readonly ConsolePane _console = new() { Prompt = "DianaOS #: " };

        // Null until a ROM launches; from then on every line goes to the running core instead.
        private ILiveShell? _live;

        private DianaOSInterpreter _shell;

        public DianaOSShellWindow() : this(null, null, null) { }

        public DianaOSShellWindow(
            Func<string, Window>? launchRom,
            Func<string, string?>? resolveCoreCommand,
            IEnumerable<string>? supportedCores)
        {
            _launchRom = launchRom;
            _resolveCoreCommand = resolveCoreCommand;
            _shell = BuildInterpreter();

            Title = "DianaOS";
            Width = 900;
            Height = 560;
            Background = LunaPalette.Surface;
            this.MinSize(420, 240);

            Content = _console.Margin(8);

            // The one place the two frontends' console windows genuinely differ: this one follows a live core once a game attaches.
            _console.HistorySource = HistoryEntries;
            _console.Submitted += Submit;

            _console.AppendLine(_shell.GetWelcomeBanner(supportedCores ?? Array.Empty<string>()));
            _console.AppendLine("--- DianaOS (type 'help', 'core <name> <path>' to launch a game, 'shutdown' to quit) ---");

            Opened += (_, _) => _console.FocusInput();
        }

        private DianaOSInterpreter BuildInterpreter() =>
            DianaOSInterpreter.CreateDefault(null,
                new IDianaOSCommand[] { new ClearShellWindowCommand(() => _console.Clear()) },
                supportedCheatSystems: () => EmuSen.Cores.CoreCatalog.SupportedCheatSystems);

        // Reports a ROM that could not be loaded back into the window that asked for it.
        public void ReportLaunchFailure(string message)
        {
            _console.AppendLine(message);
            _console.FocusInput();
        }

        // Hands this window over to the running core's own interpreter - see EmuSen_Frontend_Driver.md §3c.
        public void AttachLiveShell(ILiveShell live)
        {
            _live = live;
            _console.ResetHistoryRecall();
            live.Output += OnLiveOutput;
            Closed += (_, _) => live.Output -= OnLiveOutput;
            _console.AppendLine("--- Game started. This shell is now attached to the running core. ---");
            SyncPrompt();
        }

        private void OnLiveOutput(string output)
        {
            // Raised from the emulation and console-reader threads, never this one.
            Dispatcher.UIThread.Post(() =>
            {
                if (output.Length > 0) _console.AppendLine(output);
                SyncPrompt();
            });
        }

        private IReadOnlyList<string> HistoryEntries() =>
            _live is { } live ? live.SnapshotHistory() : _shell.History.Entries;

        private void Submit(string line)
        {
            _console.AppendLine(CurrentPrompt() + line);

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
            if (output.Length > 0) _console.AppendLine(output);
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

        private void SyncPrompt() => _console.Prompt = CurrentPrompt();

        public void AppendLine(string text) => _console.AppendLine(text);
    }

    // The live interpreter a running GameWindow lends to the shell window - see EmuSen_Frontend_Driver.md §3c.
    public interface ILiveShell
    {
        bool IsAwaitingMoreInput { get; }
        string[] SnapshotHistory();
        void Submit(string line);
        event Action<string> Output;
    }

    // Replaces ClearCommand's Console.Clear(), which does nothing to a widget.
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
