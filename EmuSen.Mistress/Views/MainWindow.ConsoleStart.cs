using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // A console started with nothing in it, and a disc put in or taken out afterwards - see Diamond_Shell.md §1.5.
    public partial class MainWindow
    {
        public const string StartConsoleMenu = "Start a Console";

        // The console of a session started with no game, null for a game's.
        private string? _consoleSession;

        // The words every entry uses, so the desktop, the pad menu and big picture say the same.
        internal static string StartLabel(string console) => $"Start {console}";
        internal static string StartWithNoDiscLabel(string console) => $"Start {console} With No Disc";

        // The consoles an engine of this build starts empty, read from the engines' info.
        private static IReadOnlyList<string> StartableConsoles => CoreCatalog.ConsolesThatStartEmpty.Select(c => c.Console).ToArray();

        // The console a library shelf shows, where it can be started empty; null otherwise.
        private static string? StartableConsoleOfShelf(string? shelf) =>
            CoreCatalog.ShelfByName(shelf)?.Core.Console is string console && CoreCatalog.EmptyStartFor(console) is not null ? console : null;

        // Run from the menus: no game to ask about, so no firmware or resume question comes first.
        private void StartConsole(string console) => StartConsole(console, reset: false);

        private void StartConsole(string console, bool reset)
        {
            ShutDownCurrentSession(leaving: !reset);
            string name = ConsoleSession.NameOf(console);
            int restoredCheats = DropCheatsFromAnotherGame(name);
            try
            {
                _session = new EmulatorSession { Cheats = _cheats, Engine = CoreCatalog.EngineChosen(console, _graphics.Value(console, CoreCatalog.EngineKey)), DeviceLender = LendDevice };
                StartLogging(console);
                _session.StartConsole(console);
                _activeConsole = console;
                ApplyConsoleSettings(_session, _activeConsole);
                ApplyScreenFilter(_activeConsole);
                WireSession(_session, name, restoredCheats, newGame: !reset);
                _currentRomPath = null;
                _consoleSession = console;
                _currentDisplayName = name;
                RunSession(_session);
            }
            catch (Exception ex)
            {
                _session = null;
                ShowLibrary();
                ErrorLog.Error("launch", $"Failed to start {console}", ex);
                StatusText.Text = $"Failed to start {console}: {ex.Message}";
            }
        }

        // File > Start Console, one item a console; disabled where none starts empty.
        private LunaAction StartConsoleMenuItems()
        {
            var items = StartableConsoles.Select(c => new LunaAction(StartLabel(c), () => StartConsole(c))).ToArray();
            return new LunaAction("Start _Console") { Submenu = new LunaMenu("Start _Console", items), IsEnabled = items.Length > 0 };
        }

        // Emulation > Insert Disc and Eject Disc, for a running machine with a drive.
        private LunaAction? _insertDisc, _ejectDisc;

        private LunaAction[] DriveMenuItems()
        {
            _insertDisc ??= new LunaAction("_Insert Disc...", () => _ = InsertDiscAsync());
            _ejectDisc ??= new LunaAction("E_ject Disc", EjectDisc);
            return [_insertDisc, _ejectDisc];
        }

        // The two items enabled where the running machine has a drive, Eject where a disc is in it.
        private void SyncDriveMenu()
        {
            if (_insertDisc is null || _ejectDisc is null) return;
            _insertDisc.IsEnabled = DriveEngine is not null;
            _ejectDisc.IsEnabled = DriveEngine is { DiscIndex: >= 0 };
        }

        // The library toolbar's button, shown while the sidebar shows a console that starts empty.
        private void UpdateStartConsoleButton()
        {
            string? console = StartableConsoleOfShelf(SelectedConsole);
            StartConsoleButton.IsVisible = console is not null;
            StartConsoleButton.Content = console is null ? "" : StartLabel(console);
        }

        private void OnStartConsoleButton(object? sender, Avalonia.Interactivity.RoutedEventArgs e)
        {
            if (StartableConsoleOfShelf(SelectedConsole) is string console) StartConsole(console);
        }

        // The pad menu's Library > Start a Console, a row a console.
        private List<PadMenuEntry> StartConsoleRows() => StartableConsoles.Select(c => new PadMenuEntry(() => StartLabel(c), () => StartConsole(c))).ToList();

        // The themed gamelist's console where it starts empty: first in Select's menu, as on the desktop's toolbar.
        private void AddStartConsoleOption(List<GameOption> options)
        {
            string? esde = _themed?.SelectedSystem?.System.Name;
            string? console = CoreCatalog.ShelvesInReleaseOrder.FirstOrDefault(s => s.EsdeSystem == esde)?.Core.Console;
            if (console is null || CoreCatalog.EmptyStartFor(console) is null) return;
            options.Insert(0, new GameOption(StartWithNoDiscLabel(console), () => StartConsole(console)));
        }

        // The running engine where its machine has a drive; null otherwise.
        private CoreEngine? DriveEngine => _session?.Core is CoreEngine { CanChangeDisc: true } engine ? engine : null;

        // The console's games in the library, as the chooser lists them.
        private IReadOnlyList<(string Title, string Path)> DiscsFor(string console)
        {
            var shelves = CoreCatalog.ShelvesInReleaseOrder.Where(s => s.Core.Console == console).Select(s => s.Name).ToHashSet(StringComparer.OrdinalIgnoreCase);
            return _allScan.Entries.Where(e => e.Shelf is not null && shelves.Contains(e.Shelf) && Listed(e)).Select(e => (DisplayTitle(e), e.FullPath)).OrderBy(d => d.Item1, StringComparer.CurrentCultureIgnoreCase).ToArray();
        }

        // The chooser, paused under it; a game chosen goes in, Browse asks for any file on the desktop.
        private async Task InsertDiscAsync()
        {
            if (DriveEngine is null) return;
            bool wasRunning = !IsPaused;
            PauseEmulation();
            try
            {
                string? chosen = await PresentInsertDisc(_activeConsole, DiscsFor(_activeConsole), browse: !_bigScreen);
                if (chosen == InsertDiscWindow.Browse)
                {
                    string? startIn = Directory.Exists(_appSettings.RomDirectory) ? _appSettings.RomDirectory : null;
                    var types = new[] { new FilePickerFileType("Disc images") { Patterns = new[] { "*.cue", "*.chd", "*.iso", "*.bin", "*.m3u", "*.zip" } }, new FilePickerFileType("All files") { Patterns = new[] { "*" } } };
                    chosen = await Dialogs.PickFileAsync(this, "Insert Disc", types, startIn) is { } file ? file.Path : null;
                }
                if (chosen is not null) InsertDisc(chosen);
            }
            finally
            {
                if (wasRunning) ResumeEmulation();
            }
        }

        // The chooser as a sheet in a big-screen session, a LunaP window on the desktop; the path chosen, Browse, or null.
        internal Task<string?> PresentInsertDisc(string console, IReadOnlyList<(string Title, string Path)> discs, bool browse) =>
            SheetLayer.ShowDialog<string?>(new InsertDiscWindow(console, discs, browse), this);

        // On the emulation thread, as Change Disc is, so no frame runs half through the change.
        private void InsertDisc(string path) => OnDrive(engine => engine.InsertDisc(path), $"{Path.GetFileNameWithoutExtension(path)} is in", "Insert Disc");

        private void EjectDisc() => OnDrive(engine => engine.EjectDisc(), "The disc is out", "Eject Disc");

        private void OnDrive(Action<CoreEngine> act, string done, string what)
        {
            if (DriveEngine is null) return;
            RequestOnEmulationThread(_ =>
            {
                string status;
                try
                {
                    if (_session?.Core is CoreEngine engine) act(engine);
                    status = done;
                }
                catch (Exception ex) when (ex is InvalidOperationException or IOException or InvalidDataException or NotSupportedException)
                {
                    status = $"{what} failed: {ex.Message}";
                }
                Dispatcher.UIThread.Post(() => { StatusText.Text = status; Notify(status); SyncMenuState(); });
            });
        }

        // The game menu's rows for a machine with a drive.
        private IEnumerable<PadMenuEntry> DriveRows()
        {
            if (DriveEngine is not { } engine) yield break;
            yield return new PadMenuEntry(() => "Insert Disc...", () => _ = InsertDiscAsync()) { Opens = true };
            if (engine.DiscIndex >= 0) yield return new PadMenuEntry(() => "Eject Disc", EjectDisc);
        }
    }
}
