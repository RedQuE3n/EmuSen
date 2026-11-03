using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // The Scraper window opened from Mistress, and Scrape Games' choice made by the emusen.scrape module, its one owner - see EmuSen_LibraryTools.md §9.
    public partial class MainWindow
    {
        private ScraperWindow? _scraperWindow;

        // The Scraper, over Mistress's own scraper lent to it, so the quota is counted once; one open window is brought forward.
        private void ShowScraper()
        {
            if (_scrapeClosed) return;
            if (_scraperWindow is { } open)
            {
                SheetLayer.Activate(open);
                return;
            }
            if (ScrapeNative && _appSettings.Scraping) OpenMediaStore();
            var window = new ScraperWindow(() => Wrap(() => Runtime.Systems()));
            window.Pane.RunEnded += PostCoverRefresh;
            window.Pane.RunEnded += () => ToolDataChanged(Pages.HomeScreen.Scraper);
            window.Pane.SummaryChanged += s => ToolSummaryHeard(Pages.HomeScreen.Scraper, s);
            _scraperWindow = window;
            window.Closed += (_, _) => { if (ReferenceEquals(_scraperWindow, window)) _scraperWindow = null; };
            _ = SheetLayer.Show(window, this);
        }

        void IScrapeHost.ShowScraper() => ShowScraper();

        // ES-DE's "Scrape these games" as the command's words.
        internal static IReadOnlyList<string> CriteriaWords(ScrapeCriteria criteria) => criteria switch
        {
            ScrapeCriteria.Favourites => ["--favorites"],
            ScrapeCriteria.NoMetadata => ["--missing", "metadata"],
            ScrapeCriteria.NoGameImage => ["--missing", "miximage,screenshot,titlescreen,cover"],
            ScrapeCriteria.NoGameVideo => ["--missing", "video"],
            ScrapeCriteria.NoCover => ["--missing", "cover"],
            _ => [],
        };

        // A wide scope's games as `scrape ls` chooses them; one game is named by hand and taken as it is.
        private List<(string Path, string System)> ChosenGames(ScrapeScope scope)
        {
            if (scope.Game is not null) return ScopeGames(scope).Select(g => (g.FullPath, SystemOf(g)!)).ToList();
            var words = new List<string> { "ls" };
            if (scope.Shelf is not null)
            {
                if (Library.RuntimeCatalog.ShelfByName(scope.Shelf)?.EsdeSystem is not { Length: > 0 } system) return [];
                words.AddRange(["--system", system]);
            }
            words.AddRange(CriteriaWords(scope.Effective));
            try
            {
                using JsonDocument doc = JsonDocument.Parse(ScrapeCommand.Ask(ScrapeCommand.Request(words, sources: false)));
                return doc.RootElement.GetProperty("games").EnumerateArray().Select(g => (g.GetProperty("path").GetString()!, g.GetProperty("system").GetString()!)).ToList();
            }
            catch (ScrapeCommandException e)
            {
                ErrorLog.Warning("scraping", $"The games to scrape could not be chosen: {ScrapeRedactor.Redact(e.Message)}");
                StatusText.Text = $"The games to scrape could not be chosen: {ScrapeRedactor.Redact(e.Message)}";
                return [];
            }
        }
    }
}
