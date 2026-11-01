using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.Tools.Library;

namespace EmuSen.Mistress.Views
{
    // Library, the library manager, opened from Mistress over its own games.db and media.db - see EmuSen_LibraryTools.md §10.
    public partial class MainWindow
    {
        public const string LibraryManagerMenu = "Library Manager";

        private LibraryWindow? _libraryWindow;

        // One open window is brought forward rather than a second opened.
        private void ShowLibraryManager()
        {
            if (_libraryWindow is { } open)
            {
                SheetLayer.Activate(open);
                return;
            }
            if (ScrapeNative && _appSettings.Scraping) OpenMediaStore();
            var window = new LibraryWindow(_appSettings.RomDirectory, new Records(this));
            _libraryWindow = window;
            window.Closed += (_, _) => { if (ReferenceEquals(_libraryWindow, window)) _libraryWindow = null; };
            _ = SheetLayer.Show(window, this);
        }

        // The hidden field of games.db's edits and the scraper's answers, lent to the tool; a change redraws the library at once.
        private sealed class Records(MainWindow window) : ILibraryRecords
        {
            public IReadOnlySet<string> Hidden() =>
                window._records.AllEdits().Where(e => e.Value.GetValueOrDefault(GameMetadata.Hidden) == GameMetadata.Yes).Select(e => e.Key).ToHashSet(StringComparer.Ordinal);

            public void SetHidden(string path, bool hidden)
            {
                window._records.SaveEdits(path, new Dictionary<string, string?> { [GameMetadata.Hidden] = hidden ? GameMetadata.Yes : null }, DateTime.Now);
                window.ShowLibraryEntries();
            }

            public IReadOnlyList<ScrapedFile> Scraped()
            {
                window.ReadScrapeSnapshot();
                var files = new List<ScrapedFile>();
                foreach ((string path, (ScrapeState state, bool _)) in window._scrapeOutcomes)
                {
                    if (state == ScrapeState.Found && window._scrapedText.TryGetValue(path, out ScrapedRecord? found))
                        files.Add(new ScrapedFile(path, true, found.GameId, found.Md5, found.Bytes, found.Name));
                    else if (state == ScrapeState.Unknown)
                        files.Add(new ScrapedFile(path, false, null, "", 0));
                }
                return files;
            }
        }
    }
}
