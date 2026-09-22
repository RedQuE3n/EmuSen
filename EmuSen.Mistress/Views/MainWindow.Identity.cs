using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // A game known by its contents as well as its path, and a state that says who wrote it - see EmuSen_Settings_Reference.md §4.37.
    public partial class MainWindow
    {
        // The build a state record names; the SDK appends the commit to the version when it knows it.
        public static readonly string BuildName =
            typeof(EmulatorSession).Assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion ?? "unknown";

        // Read on the emulation thread when a state is written, so volatile; null until the hash is known.
        private volatile string? _currentRomMd5;
        private long _currentRomBytes;

        private bool _recordsClosed;
        private bool _reclaiming;

        private static (long Bytes, long Modified)? Stat(string path)
        {
            try
            {
                var file = new FileInfo(path);
                return file.Exists ? (file.Length, file.LastWriteTimeUtc.Ticks) : null;
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                return null;
            }
        }

        private static string? TryMd5(string path)
        {
            try { return RomHash.Md5(path); }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException) { return null; }
        }

        // From the cache when the file is unchanged, otherwise hashed on a worker and written back on the UI thread.
        private void IdentifyLater(string path)
        {
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
            if (path != _currentRomPath) return;
            _currentRomBytes = bytes;
            _currentRomMd5 = md5;
        }

        // Rows whose file has gone, matched by contents against files nobody has a row for; only files of a missing one's size are hashed.
        private void ReclaimMovedGames()
        {
            if (_reclaiming) return;
            var present = new HashSet<string>(_allScan.Entries.Select(e => e.FullPath), StringComparer.Ordinal);
            IReadOnlyList<(string Path, string Md5, long Bytes)> orphans = _records.Orphans(present);
            if (orphans.Count == 0) return;

            var sizes = orphans.Select(o => o.Bytes).ToHashSet();
            IReadOnlyDictionary<string, Library.GameRecord> rows = _records.All();
            var candidates = new List<(string Path, long Bytes, long Modified, string? Md5)>();
            foreach (Library.RomEntry entry in _allScan.Entries)
            {
                if (rows.ContainsKey(entry.FullPath) || Stat(entry.FullPath) is not { } stat || !sizes.Contains(stat.Bytes)) continue;
                candidates.Add((entry.FullPath, stat.Bytes, stat.Modified, _records.KnownHash(entry.FullPath, stat.Bytes, stat.Modified)));
            }
            if (candidates.Count == 0) return;

            _reclaiming = true;
            Task.Run(() => candidates.Select(c => c with { Md5 = c.Md5 ?? TryMd5(c.Path) }).ToList()).ContinueWith(hashing =>
            {
                Dispatcher.UIThread.Post(() =>
                {
                    _reclaiming = false;
                    if (_recordsClosed || !hashing.IsCompletedSuccessfully) return;
                    var claimed = new HashSet<string>(StringComparer.Ordinal);
                    int moved = 0;
                    foreach (var c in hashing.Result.Where(c => c.Md5 is not null)) _records.StoreHash(c.Path, c.Bytes, c.Modified, c.Md5!);
                    foreach ((string path, string md5, long _) in orphans)
                    {
                        var match = hashing.Result.FirstOrDefault(c => c.Md5 == md5 && !claimed.Contains(c.Path));
                        if (match.Path is null) continue;
                        _records.Move(path, match.Path);
                        claimed.Add(match.Path);
                        moved++;
                    }
                    if (moved == 0) return;
                    StatusText.Text = $"Found {moved} renamed or moved game{(moved == 1 ? "" : "s")}; {(moved == 1 ? "its" : "their")} favourites, play time and collections came along.";
                    if (LibraryView.IsVisible) ShowLibraryEntries();
                });
            });
        }

        // On the emulation thread, beside the state it describes; the ROM's hash is whatever is known by then.
        private void WriteStateRecord(EmulatorSession session, string statePath, string romPath)
        {
            if (session.Core is not IStateFormat format) return;
            new StateRecord
            {
                Console = CoreCatalog.ConsoleForRom(romPath) ?? session.CoreName,
                Core = session.CoreName,
                StateVersion = format.StateVersion,
                Build = BuildName,
                SavedAt = DateTime.Now,
                RomFile = Path.GetFileName(romPath),
                RomMd5 = _currentRomMd5,
                RomBytes = _currentRomBytes,
            }.Write(statePath);
        }

        // Why a state must not reach this game's core, or null; a version older than the core's is the core's to judge.
        private static string? Refusal(StateRecord? record, string romPath, EmulatorSession? session)
        {
            if (record is null) return null;
            string? console = CoreCatalog.ConsoleForRom(romPath);
            if (console is not null && record.Console != console)
                return $"That state was saved by the {record.Console} core, and {Path.GetFileName(romPath)} runs on the {console} core.";
            if (session?.Core is IStateFormat format && record.StateVersion > format.StateVersion)
                return $"That state was saved by a newer build ({record.Build}, {record.Core} state version {record.StateVersion}); this one reads up to {format.StateVersion}.";
            return null;
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
