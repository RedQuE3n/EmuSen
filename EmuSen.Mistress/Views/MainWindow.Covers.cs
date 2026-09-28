using System;
using System.IO;
using System.Net.Http;
using System.Threading.Tasks;
using Avalonia.Threading;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // OpenEmu's sources as the failover, asked only inside a scrape the player started, for covers ScreenScraper has not got - see EmuSen_Settings_Reference.md §4.39 and §4.60.
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
        private readonly System.Collections.Generic.List<(RomEntry Entry, CoreDescriptor Core)> _waitingForDatabase = new();

        private string OpenVgdbPath => OpenVgdb.DefaultPath;

        // The failover's own folder, apart from the player's art so that what the player placed can win: home/Media/openemu/<console>/.
        internal static string OpenEmuCoverDirectory => Path.Combine(DataStore.Media, "openemu");

        // Stops the failover when it is unticked; it never starts here.
        private void ApplyOnlineCovers()
        {
            if (_appSettings.OpenEmuFallback) return;
            _fetcher?.Dispose();
            _fetcher = null;
        }

        private async Task DownloadDatabaseAsync()
        {
            _downloadingDatabase = true;
            StatusText.Text = "Downloading OpenVGDB, the game database OpenEmu uses...";
            try
            {
                string tag = await OpenVgdbDownload.FetchAsync(_http!, OpenVgdbPath);
                if (_recordsClosed) return;
                StatusText.Text = $"OpenVGDB {tag} downloaded.";
                ScrapeChanged?.Invoke();
                if (_appSettings.OpenEmuFallback && ScrapeRunning) StartFetcher();
            }
            catch (Exception ex) when (ex is HttpRequestException or IOException or InvalidDataException or TaskCanceledException or System.Text.Json.JsonException)
            {
                if (_recordsClosed) return;
                ErrorLog.Error("covers", "Could not download OpenVGDB", ex);
                StatusText.Text = $"Could not download OpenVGDB: {ex.Message}";
                foreach ((RomEntry entry, _) in _waitingForDatabase) FailoverArrived(new CoverResult(entry.FullPath, "", CoverOutcome.Failed, null, null, ex.Message));
                _waitingForDatabase.Clear();
            }
            finally
            {
                _downloadingDatabase = false;
            }
        }

        private void StartFetcher()
        {
            _fetcher = new CoverFetcher(_http!, OpenVgdbPath, OpenEmuCoverDirectory, result => Dispatcher.UIThread.Post(() => CoverArrived(result)));
            foreach ((RomEntry entry, CoreDescriptor core) in _waitingForDatabase) _fetcher.Enqueue(entry, core);
            _waitingForDatabase.Clear();
        }

        // Called only by a run the player started; true when the game was handed to the failover. A past answer is final until "Scrape This Game".
        private bool AskForCover(RomEntry entry, CoreDescriptor? core)
        {
            if (!_appSettings.OpenEmuFallback || core is null || core.OpenVgdbSystemNames.Count == 0 || _recordsClosed) return false;
            if (_records.CoverLookup(entry.FullPath) is not null) return false;
            _http ??= HttpFactory();
            if (_fetcher is not null) return _fetcher.Enqueue(entry, core);
            _waitingForDatabase.Add((entry, core));
            if (!_downloadingDatabase)
            {
                if (File.Exists(OpenVgdbPath)) StartFetcher();
                else _ = DownloadDatabaseAsync();
            }
            return true;
        }

        private void CoverArrived(CoverResult result)
        {
            if (_recordsClosed) return;
            if (result.Outcome != CoverOutcome.Failed) _records.RecordCoverLookup(result.RomPath, result.Outcome, result.RomName, DateTime.Now);
            FailoverArrived(result);
            if (result.Saved is null || _coverRefreshPosted) return;
            _coverRefreshPosted = true;
            Dispatcher.UIThread.Post(() => { _coverRefreshPosted = false; if (!_recordsClosed) RefreshCovers(); }, DispatcherPriority.Background);
        }

        // Q42: an optional Remove to free the space, never automatic; a test answers the confirm instead of a person - see EmuSen_Settings_Reference.md §4.65.
        internal static Func<MainWindow, Task<bool>> ConfirmRemoveOpenVgdb = owner => Dialogs.ConfirmAsync(owner, "Remove OpenVGDB",
            "Delete OpenVGDB, the game database OpenEmu's fallback uses, from home/Library to free the space? The fallback is switched off until it is downloaded again. Covers it already found stay.",
            "Remove", "Keep");

        long? IScrapeHost.OpenVgdbBytes => File.Exists(OpenVgdbPath) ? new FileInfo(OpenVgdbPath).Length : null;

        bool IScrapeHost.OpenVgdbBusy => _downloadingDatabase || ScrapeRunning;

        async Task<string?> IScrapeHost.RemoveOpenVgdbAsync()
        {
            if (!File.Exists(OpenVgdbPath)) return "OpenVGDB is not downloaded.";
            if (_downloadingDatabase || ScrapeRunning) return "OpenVGDB is in use by a run; remove it when the run has finished.";
            if (!await ConfirmRemoveOpenVgdb(this)) return null;
            if (_downloadingDatabase || ScrapeRunning) return "OpenVGDB is in use by a run; remove it when the run has finished.";
            // The fallback's worker holds the database open; it is let go of first, and the fallback stays off until the database is back.
            _fetcher?.Dispose();
            _fetcher = null;
            try
            {
                long bytes = new FileInfo(OpenVgdbPath).Length;
                File.Delete(OpenVgdbPath);
                if (File.Exists(OpenVgdbPath + ".part")) File.Delete(OpenVgdbPath + ".part");
                _appSettings.OpenEmuFallback = false;
                _appSettings.Save();
                StatusText.Text = "OpenVGDB removed";
                ScrapeChanged?.Invoke();
                return $"OpenVGDB was removed ({ScrapePreferencesPane.Size(bytes)} freed). OpenEmu's fallback is off until it is downloaded again.";
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                ErrorLog.Error("covers", "Could not remove OpenVGDB", ex);
                return $"Could not remove OpenVGDB: {ex.Message}";
            }
        }

        // The player's Download on the same row: one request to GitHub, then the fallback on again.
        async Task<string> IScrapeHost.DownloadOpenVgdbAsync()
        {
            if (_downloadingDatabase || ScrapeRunning) return "A run is using OpenEmu's fallback; download when it has finished.";
            _http ??= HttpFactory();
            _downloadingDatabase = true;
            ScrapeChanged?.Invoke();
            try
            {
                string tag = await OpenVgdbDownload.FetchAsync(_http, OpenVgdbPath);
                if (_recordsClosed) return "";
                _appSettings.OpenEmuFallback = true;
                _appSettings.Save();
                StatusText.Text = $"OpenVGDB {tag} downloaded.";
                return $"OpenVGDB {tag} was downloaded, and OpenEmu's fallback is on.";
            }
            catch (Exception ex) when (ex is HttpRequestException or IOException or InvalidDataException or TaskCanceledException or System.Text.Json.JsonException)
            {
                ErrorLog.Error("covers", "Could not download OpenVGDB", ex);
                return $"Could not download OpenVGDB: {ScrapeRedactor.Redact(ex.Message)}";
            }
            finally
            {
                _downloadingDatabase = false;
                ScrapeChanged?.Invoke();
            }
        }

        private void StopOnlineCovers()
        {
            _fetcher?.Dispose();
            _fetcher = null;
            _http?.Dispose();
        }
    }
}
