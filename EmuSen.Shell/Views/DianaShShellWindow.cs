using System;
using System.Collections.Generic;
using Avalonia.Controls;
using Avalonia.Threading;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.Hotaru;
using EmuSen.Hotaru.Views;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Theme;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Shell.Views
{
    // The shell a windowed launch gets in place of the terminal - see EmuSen_Frontend_Driver.md §3c.
    public class DianaShShellWindow : ToolWindow
    {
        private readonly Func<string, Window>? _launchRom;
        private readonly Func<string, string?>? _resolveCoreCommand;

        private readonly ConsolePane _console = new() { Prompt = "DianaSh #: " };

        // Null until a ROM launches; from then on every line goes to the running core instead.
        private ILiveShell? _live;

        private DianaShInterpreter _shell;

        public DianaShShellWindow() : this(null, null, null) { }

        public DianaShShellWindow(
            Func<string, Window>? launchRom,
            Func<string, string?>? resolveCoreCommand,
            IEnumerable<string>? supportedCores)
        {
            _launchRom = launchRom;
            _resolveCoreCommand = resolveCoreCommand;
            _shell = BuildInterpreter();

            Title = "DianaSh";
            Width = 900;
            Height = 560;
            this.MinSize(420, 240);

            Content = _console.Margin(8);

            // The one place the two frontends' console windows genuinely differ: this one follows a live core once a game attaches.
            _console.HistorySource = HistoryEntries;
            _console.Submitted += Submit;

            _console.AppendLine(_shell.GetWelcomeBanner(supportedCores ?? Array.Empty<string>()));
            _console.AppendLine("--- DianaSh (type 'help', 'hotaru <rom>' or 'core <name> <path>' to launch a game, 'shutdown' to quit) ---");

            Opened += (_, _) => _console.FocusInput();
        }

        private DianaShInterpreter BuildInterpreter() =>
            DianaShInterpreter.CreateDefault(null,
                new IDianaShCommand[] { new ClearShellWindowCommand(() => _console.Clear()), new HotaruCommand(_launchRom is null ? null : OpenGame) },
                supportedCheatSystems: () => EmuSen.Cores.CoreCatalog.SupportedCheatSystems);

        // `hotaru`'s window, from this window's own prompt; a ROM that will not load comes back as the command's failure.
        private void OpenGame(string romPath, HotaruOutput output) => _launchRom!(romPath).Show();

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
            _live is { IsAwaitingMoreInput: true } || (_live is null && _shell.IsAwaitingMoreInput) ? "> " : "DianaSh #: ";

        private void SyncPrompt() => _console.Prompt = CurrentPrompt();

        public void AppendLine(string text) => _console.AppendLine(text);
    }

    // Replaces ClearCommand's Console.Clear(), which does nothing to a widget.
    public class ClearShellWindowCommand : IDianaShCommand
    {
        private readonly Action _clearOutput;

        public ClearShellWindowCommand(Action clearOutput) => _clearOutput = clearOutput;

        public string Name => "clear";
        public bool IsReadOnly => true;
        public string Usage => "  clear                         clear this shell window's output";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            _clearOutput();
            return DianaShResult.Ok(string.Empty);
        }
    }
}
