using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // Q45, another game's cover, chosen on the game's page under Details ▸ Cover Art: a row of games.db names the game whose cover this one shows, first in the order; no file is copied or deleted - see EmuSen_Settings_Reference.md §4.65.
    public partial class MainWindow
    {
        private IReadOnlyDictionary<string, IReadOnlyDictionary<string, string>>? _choiceStampOf;
        private string _choiceStamp = "";

        private string? CoverChoiceOf(string path) =>
            _editSnapshot.TryGetValue(path, out IReadOnlyDictionary<string, string>? edits) && edits.TryGetValue(GameMetadata.CoverFrom, out string? from) && from.Length > 0 ? from : null;

        // Changes whenever any game's choice does, so the themed view's pictures are looked up again; worked out once per snapshot.
        private string CoverChoiceStamp
        {
            get
            {
                if (ReferenceEquals(_choiceStampOf, _editSnapshot)) return _choiceStamp;
                _choiceStampOf = _editSnapshot;
                var choices = _editSnapshot.Where(e => e.Value.ContainsKey(GameMetadata.CoverFrom)).Select(e => e.Key + ">" + e.Value[GameMetadata.CoverFrom]).Order(StringComparer.Ordinal);
                return _choiceStamp = Convert.ToHexString(System.Security.Cryptography.SHA1.HashData(System.Text.Encoding.UTF8.GetBytes(string.Join("\n", choices))))[..12];
            }
        }

        private string CoverTitle(RomEntry entry) => DisplayTitle(entry);

        // Every listed game with a cover to give, other than this one and any whose cover already comes from it, those sharing its first words first.
        private IReadOnlyList<CoverCandidate> CoverCandidatesFor(string path)
        {
            var candidates = new List<CoverCandidate>();
            foreach (RomEntry entry in _allScan.Entries)
            {
                if (entry.FullPath == path || !Listed(entry) || LeadsTo(entry.FullPath, path)) continue;
                if (CoverPathFor(entry) is not string cover) continue;
                string console = EmuSen.Mistress.Library.RuntimeCatalog.ShelfByName(entry.Shelf)?.Label ?? entry.Shelf;
                candidates.Add(new CoverCandidate(entry.FullPath, CoverTitle(entry), console, cover));
            }
            return CoverCandidates.Rank(NameOf(path), candidates, null);
        }

        private bool LeadsTo(string start, string target)
        {
            var seen = new HashSet<string>(StringComparer.Ordinal);
            for (string? at = CoverChoiceOf(start); at is not null && seen.Add(at); at = CoverChoiceOf(at))
                if (at == target) return true;
            return false;
        }

        private string NameOf(string path) =>
            _allScan.Entries.FirstOrDefault(e => e.FullPath == path) is RomEntry e ? DisplayTitle(e) : Path.GetFileNameWithoutExtension(path);

        private void ChooseCover(string path, string from)
        {
            if (_recordsClosed) return;
            _records.SetCoverChoice(path, from, DateTime.Now);
            StatusText.Text = $"{Path.GetFileNameWithoutExtension(path)} shows the cover of {NameOf(from)}";
            ShowLibraryEntries();
        }

        private void UseOwnCover(string path)
        {
            if (_recordsClosed) return;
            _records.SetCoverChoice(path, null, DateTime.Now);
            StatusText.Text = $"{Path.GetFileNameWithoutExtension(path)} shows its own cover again";
            ShowLibraryEntries();
        }
    }

    public sealed record CoverCandidate(string Path, string Title, string Console, string Cover);

    // The order the Cover Art page lists games in: those whose name starts with the most of this game's words first, then by name - see EmuSen_Settings_Reference.md §4.65.
    public static class CoverCandidates
    {
        public const int Shown = 60;

        private static string[] Words(string title) =>
            ArtworkIndex.Untagged(title).ToLowerInvariant().Split([' ', '-', ':', ',', '.', '\'', '!', '&'], StringSplitOptions.RemoveEmptyEntries);

        public static int SharedWords(string a, string b)
        {
            string[] x = Words(a), y = Words(b);
            int n = 0;
            while (n < x.Length && n < y.Length && x[n] == y[n]) n++;
            return n;
        }

        public static IReadOnlyList<CoverCandidate> Rank(string title, IEnumerable<CoverCandidate> all, string? search) =>
            all.Where(c => string.IsNullOrWhiteSpace(search) || FilterBar.Matches(search, c.Title))
               .OrderByDescending(c => SharedWords(title, c.Title))
               .ThenBy(c => c.Title, StringComparer.OrdinalIgnoreCase)
               .ThenBy(c => c.Path, StringComparer.Ordinal)
               .Take(Shown)
               .ToList();
    }
}
