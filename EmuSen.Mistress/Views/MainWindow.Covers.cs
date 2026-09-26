using System;
using System.IO;
using System.Net.Http;
using System.Threading.Tasks;
using Avalonia.Threading;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // OpenEmu's sources as the failover for covers ScreenScraper has not got - see EmuSen_Settings_Reference.md §4.39 and §4.60.
    public partial class MainWindow
    {
        // Replaced by tests with a client over a fake handler; nothing here reaches a server under the harness.
        internal static Func<HttpClient> HttpFactory = () =>
        {
            var http = new HttpClient { Timeout = TimeSpan.FromSeconds(30) };
            http.DefaultRequestHeaders.UserAgent.ParseAdd($"EmuSen/{(BuildName ?? "unknown").Split('+')[0]}");
            return http;
        };

        private HttpClient? _http;
        private CoverFetcher? _fetcher;
        private bool _downloadingDatabase;
        private bool _coverRefreshPosted;

        private string OpenVgdbPath => OpenVgdb.DefaultPath;

        // The failover's own folder, apart from the player's art so that what the player placed can win: home/Media/openemu/<console>/.
        internal static string OpenEmuCoverDirectory => Path.Combine(DataStore.Media, "openemu");

        // Stops the failover when it is unticked; it starts only when a cover is first wanted from it.
        private void ApplyOnlineCovers()
        {
            if (_appSettings.OpenEmuFallback) return;
            _fetcher?.Dispose();
            _fetcher = null;
        }

        // Asked says the player wanted this, so its progress and failure go to the status line; an automatic start stays quiet.
        private void StartFailover(bool asked)
        {
            if (_fetcher is not null || _downloadingDatabase || !_appSettings.OpenEmuFallback || _recordsClosed) return;
            _http ??= HttpFactory();
            if (File.Exists(OpenVgdbPath))
            {
                StartFetcher();
                return;
            }
            _ = DownloadDatabaseAsync(asked);
        }

        private async Task DownloadDatabaseAsync(bool asked)
        {
            _downloadingDatabase = true;
            if (asked) StatusText.Text = "Downloading OpenVGDB, the game database OpenEmu uses...";
            try
            {
                string tag = await OpenVgdbDownload.FetchAsync(_http!, OpenVgdbPath);
                if (asked) StatusText.Text = $"OpenVGDB {tag} downloaded; missing covers will be looked up as they are shown.";
                if (_appSettings.OpenEmuFallback && !_recordsClosed) StartFetcher();
            }
            catch (Exception ex) when (ex is HttpRequestException or IOException or InvalidDataException or TaskCanceledException or System.Text.Json.JsonException)
            {
                if (asked && !_recordsClosed) StatusText.Text = $"Could not download OpenVGDB: {ex.Message}";
            }
            finally
            {
                _downloadingDatabase = false;
            }
        }

        private void StartFetcher()
        {
            _fetcher = new CoverFetcher(_http!, OpenVgdbPath, OpenEmuCoverDirectory, result => Dispatcher.UIThread.Post(() => CoverArrived(result)));
            if (LibraryView.IsVisible) LibraryGrid.Refresh(_shownEntries);
        }

        // The failover, for a tile with no art that ScreenScraper has not covered; a past answer is final until the player asks again.
        private void AskForCover(RomEntry entry, CoreDescriptor? core, bool asked = false)
        {
            if (!_appSettings.OpenEmuFallback || core is null || core.OpenVgdbSystemNames.Count == 0) return;
            if (_records.CoverLookup(entry.FullPath) is not null) return;
            StartFailover(asked);
            _fetcher?.Enqueue(entry, core);
        }

        private void CoverArrived(CoverResult result)
        {
            if (_recordsClosed) return;
            _records.RecordCoverLookup(result.RomPath, result.Outcome, result.RomName, DateTime.Now);
            if (result.Saved is null || _coverRefreshPosted) return;
            _coverRefreshPosted = true;
            Dispatcher.UIThread.Post(() => { _coverRefreshPosted = false; if (!_recordsClosed) RefreshCovers(); }, DispatcherPriority.Background);
        }

        private void LookUpCoverAgain(RomEntry entry)
        {
            _records.ForgetCoverLookup(entry.FullPath);
            _fetcher?.Forget(entry.FullPath);
            AskForCover(entry, EmuSen.Cores.CoreCatalog.ByDisplayName(entry.CoreDisplayName), asked: true);
            StatusText.Text = $"Looking up a cover for {entry.Title}...";
        }

        private void StopOnlineCovers()
        {
            _fetcher?.Dispose();
            _fetcher = null;
            _http?.Dispose();
        }
    }
}
