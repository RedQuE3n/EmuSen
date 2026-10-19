using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Threading;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // A game known by its contents as well as its path, and a state that says who wrote it - see EmuSen_Settings_Reference.md §4.37.
    public partial class MainWindow
    {
        // This build's name; the SDK appends the commit to the version when it knows it.
        public static readonly string BuildName =
            typeof(MainWindow).Assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion ?? "unknown";

        // Read on the emulation thread when a state is written, so volatile; null until the hash is known.
        private volatile string? _currentRomMd5;
        private long _currentRomBytes;

        private bool _recordsClosed;
        private bool _reclaiming;

        private static (long Bytes, long Modified)? Stat(string path) => GameRecords.Stat(path);

        private static string? TryMd5(string path) => GameRecords.TryMd5(path);

        // Through the library, the whole identity on a worker; on the C#, from the cache when the file is unchanged, else hashed on a worker and written back on the UI thread.
        private void IdentifyLater(string path)
        {
            if (_records.ThroughLibrary)
            {
                Task.Run(() =>
                {
                    try { return _records.IdentifyFile(path); }
                    catch (Exception e) when (e is ObjectDisposedException or EmuSen.Crystal.Client.Library.GameLibraryException or Microsoft.Data.Sqlite.SqliteException) { return null; }
                }).ContinueWith(identifying =>
                {
                    if (identifying.Result is not { } id) return;
                    Dispatcher.UIThread.Post(() => { if (!_recordsClosed) Current(path, id.Md5, id.Bytes); });
                });
                return;
            }
            if (Stat(path) is not { } stat) return;
            if (_records.KnownHash(path, stat.Bytes, stat.Modified) is string known)
            {
                Identified(path, known, stat.Bytes);
                return;
            }
            Task.Run(() => TryMd5(path)).ContinueWith(hashing =>
            {
                if (hashing.Result is not string md5) return;
                Dispatcher.UIThread.Post(() =>
                {
                    if (_recordsClosed) return;
                    _records.StoreHash(path, stat.Bytes, stat.Modified, md5);
                    Identified(path, md5, stat.Bytes);
                });
            });
        }

        private void Identified(string path, string md5, long bytes)
        {
            _records.Identify(path, md5, bytes);
            Current(path, md5, bytes);
        }

        private void Current(string path, string md5, long bytes)
        {
            if (path != _currentRomPath) return;
            _currentRomBytes = bytes;
            _currentRomMd5 = md5;
        }

        // Rows whose file has gone, matched by contents against files nobody has a row for; only files of a missing one's size are hashed, on a worker.
        private void ReclaimMovedGames()
        {
            if (_reclaiming) return;
            List<string> games = _allScan.Entries.Select(e => e.FullPath).ToList();
            ReclaimPlan? plan = _records.ThroughLibrary ? null : _records.Plan(games);
            if (!_records.ThroughLibrary && plan is null) return;

            _reclaiming = true;
            Task.Run(() =>
            {
                if (!_records.ThroughLibrary) return (Plan: GameRecords.Hash(plan), Moved: -1);
                try { return (Plan: (ReclaimPlan?)null, Moved: _records.Reclaim(games)); }
                catch (Exception e) when (e is ObjectDisposedException or EmuSen.Crystal.Client.Library.GameLibraryException or Microsoft.Data.Sqlite.SqliteException) { return (Plan: null, Moved: 0); }
            }).ContinueWith(reclaiming =>
            {
                Dispatcher.UIThread.Post(() =>
                {
                    _reclaiming = false;
                    if (_recordsClosed || !reclaiming.IsCompletedSuccessfully) return;
                    int moved = reclaiming.Result.Moved >= 0 ? reclaiming.Result.Moved : _records.Apply(reclaiming.Result.Plan);
                    if (moved == 0) return;
                    StatusText.Text = $"Found {moved} renamed or moved game{(moved == 1 ? "" : "s")}; {(moved == 1 ? "its" : "their")} favourites, play time and collections came along.";
                    if (LibraryView.IsVisible) ShowLibraryEntries();
                });
            });
        }

        private static string Provenance(StateRecord? record) =>
            record is null ? "" : $" It was saved by {record.Build}, {record.Core} state version {record.StateVersion}.";

        // A state from another copy of the game can crash it, so that one is asked about; an unknown hash on either side is not.
        private async Task<bool> SameGameOrConfirmedAsync(StateRecord? record, string romPath)
        {
            if (record?.RomMd5 is not string saved) return true;
            string? md5 = romPath == _currentRomPath ? _currentRomMd5 : null;
            if (md5 is null && Stat(romPath) is { } stat)
                md5 = _records.KnownHash(romPath, stat.Bytes, stat.Modified) ?? await Task.Run(() => TryMd5(romPath));
            if (md5 is null || md5 == saved) return true;
            return await Dialogs.ConfirmAsync(this, "A Different Copy of the Game",
                $"This state was saved from {record.RomFile}, whose contents are not those of {Path.GetFileName(romPath)}. " +
                "A state loaded into another version of a game can crash it. Load it anyway?", "Load Anyway", "Cancel");
        }

        private void CloseRecords()
        {
            _recordsClosed = true;
            _records.Dispose();
        }
    }
}
