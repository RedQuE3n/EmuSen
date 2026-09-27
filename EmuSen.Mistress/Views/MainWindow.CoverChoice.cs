using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // Q45, Use Another Game's Cover...: a row of games.db names the game whose cover this one shows, first in the order; no file is copied or deleted - see EmuSen_Settings_Reference.md §4.65.
    public partial class MainWindow
    {
        private CoverPickerWindow? _coverPicker;
        private IReadOnlyDictionary<string, IReadOnlyDictionary<string, string>>? _choiceStampOf;
        private string _choiceStamp = "";

        public CoverPickerWindow? CoverPickerShown => _coverPicker;

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

        // Every listed game with a cover to give, other than this one and any whose cover already comes from it.
        private List<CoverCandidate> CoverCandidatesFor(string path)
        {
            var candidates = new List<CoverCandidate>();
            foreach (RomEntry entry in _allScan.Entries)
            {
                if (entry.FullPath == path || !Listed(entry) || LeadsTo(entry.FullPath, path)) continue;
                if (CoverPathFor(entry) is not string cover) continue;
                string console = EmuSen.Cores.CoreCatalog.ShelfByName(entry.Shelf)?.Label ?? entry.Shelf;
                candidates.Add(new CoverCandidate(entry.FullPath, CoverTitle(entry), console, cover));
            }
            return candidates;
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

        // A sheet in a big-screen session, a LunaP window on the desktop, as the options menu is.
        private void ShowCoverPicker(string path, string title)
        {
            if (_coverPicker is not null) { SheetLayer.Activate(_coverPicker); return; }
            string? from = CoverChoiceOf(path);
            var window = _coverPicker = new CoverPickerWindow(title, CoverCandidatesFor(path), from is null ? null : NameOf(from),
                chosen => ChooseCover(path, chosen.Path), from is null ? null : () => UseOwnCover(path));
            window.Closed += (_, _) => { if (ReferenceEquals(_coverPicker, window)) _coverPicker = null; };
            _ = SheetLayer.Show(window, this);
        }

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

        private void AddCoverEntries(List<GameOption> options, string path, string title)
        {
            options.Add(new GameOption("Use Another Game's Cover...", () => ShowCoverPicker(path, title)));
            if (CoverChoiceOf(path) is not null) options.Add(new GameOption("Use Its Own Cover", () => UseOwnCover(path)));
        }
    }
}
