using System;
using System.IO;
using System.Net.Http;
using System.Threading.Tasks;
using Avalonia.Threading;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // Missing covers looked up online, only when the player has turned it on - see EmuSen_Settings_Reference.md §4.39.
    public partial class MainWindow
    {
        // Replaced by tests with a client over a fake handler; nothing here reaches a server under the harness.
        private static Func<HttpClient> HttpFactory = () =>
        {
            var http = new HttpClient { Timeout = TimeSpan.FromSeconds(30) };
            http.DefaultRequestHeaders.UserAgent.ParseAdd($"EmuSen/{(BuildName ?? "unknown").Split('+')[0]}");
            return http;
        };

        private HttpClient? _http;
        private CoverFetcher? _fetcher;
        private bool _downloadingDatabase;
        private bool _coverRefreshPosted;
        private readonly System.Collections.Generic.List<(string Console, string Stem, string File)> _fetchedCovers = new();

        private string OpenVgdbPath => OpenVgdb.DefaultPath;

        // Starts or stops the fetcher to match the setting; the database is fetched first when it is not there.
        private void ApplyOnlineCovers()
        {
            if (!_appSettings.OnlineCovers)
            {
                _fetcher?.Dispose();
                _fetcher = null;
                return;
            }
            if (_fetcher is not null || _downloadingDatabase) return;
            _http ??= HttpFactory();
            if (File.Exists(OpenVgdbPath))
            {
                StartFetcher();
                return;
            }
            _ = DownloadDatabaseAsync();
        }

        private async Task DownloadDatabaseAsync()
        {
            _downloadingDatabase = true;
            StatusText.Text = "Downloading OpenVGDB, the game database OpenEmu uses...";
            try
            {
                string tag = await OpenVgdbDownload.FetchAsync(_http!, OpenVgdbPath);
                StatusText.Text = $"OpenVGDB {tag} downloaded; missing covers will be looked up as they are shown.";
                if (_appSettings.OnlineCovers) StartFetcher();
            }
            catch (Exception ex) when (ex is HttpRequestException or IOException or InvalidDataException or TaskCanceledException or System.Text.Json.JsonException)
            {
                StatusText.Text = $"Could not download OpenVGDB: {ex.Message}";
            }
            finally
            {
                _downloadingDatabase = false;
            }
        }

        private void StartFetcher()
        {
            _fetcher = new CoverFetcher(_http!, OpenVgdbPath, ArtworkDirectory, result => Dispatcher.UIThread.Post(() => CoverArrived(result)));
            if (LibraryView.IsVisible) LibraryGrid.Refresh(_shownEntries);
        }

        // From BindCover, for a tile with no art; a past answer is final until the player asks again.
        private void AskForCover(RomEntry entry, CoreDescriptor? core)
        {
            if (_fetcher is null || core is null || core.OpenVgdbSystemNames.Count == 0) return;
            if (_records.CoverLookup(entry.FullPath) is not null) return;
            _fetcher.Enqueue(entry, core);
        }

        private void CoverArrived(CoverResult result)
        {
            if (_recordsClosed) return;
            _records.RecordCoverLookup(result.RomPath, result.Outcome, result.RomName, DateTime.Now);
            if (result.Saved is not string saved) return;
            string stem = Path.GetFileNameWithoutExtension(result.RomPath);
            _artwork.Add(result.Console, stem, saved);
            _fetchedCovers.Add((result.Console, stem, saved));
            if (_coverRefreshPosted) return;
            _coverRefreshPosted = true;
            Dispatcher.UIThread.Post(() => { _coverRefreshPosted = false; LibraryGrid.Refresh(_shownEntries); }, DispatcherPriority.Background);
        }

        private void LookUpCoverAgain(RomEntry entry)
        {
            _records.ForgetCoverLookup(entry.FullPath);
            _fetcher?.Forget(entry.FullPath);
            AskForCover(entry, EmuSen.Cores.CoreCatalog.ByDisplayName(entry.CoreDisplayName));
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
