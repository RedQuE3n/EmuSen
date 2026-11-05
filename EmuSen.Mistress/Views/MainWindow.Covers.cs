using System;
using System.IO;
using System.Net.Http;
using System.Threading.Tasks;
using Avalonia.Threading;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // OpenEmu's sources as the failover, which the scraper asks inside a run the player started; OpenVGDB's download and removal - see EmuSen_Settings_Reference.md §4.39 and §4.60.
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
        private bool _downloadingDatabase;

        private string OpenVgdbPath => OpenVgdb.DefaultPath;

        // The failover's own folder, apart from the player's art so that what the player placed can win: home/Media/openemu/<console>/.
        internal static string OpenEmuCoverDirectory => Path.Combine(DataStore.Media, "openemu");

        // Q42: an optional Remove to free the space, never automatic; a test answers the confirm instead of a person - see EmuSen_Settings_Reference.md §4.65.
        internal static Func<MainWindow, Task<bool>> ConfirmRemoveOpenVgdb = owner => Dialogs.ConfirmAsync(owner, "Remove OpenVGDB",
            "Delete OpenVGDB, the game database OpenEmu's fallback uses, from home/Library to free the space? The fallback is switched off until it is downloaded again. Covers it already found stay.",
            "Remove", "Keep");

        long? IScrapeHost.OpenVgdbBytes => File.Exists(OpenVgdbPath) ? new FileInfo(OpenVgdbPath).Length : null;

        // OpenVGDB fetched by the scraper in Rust; the tag, or the reason why not.
        private async Task<string> DownloadOpenVgdbNativeAsync(NativeScraping native)
        {
            _downloadingDatabase = true;
            ScrapeChanged?.Invoke();
            try
            {
                (string? tag, string? failed) = await Task.Run(native.DownloadOpenVgdb);
                if (_recordsClosed) return "";
                if (tag is null)
                {
                    ErrorLog.Error("covers", "Could not download OpenVGDB", null, failed);
                    return $"Could not download OpenVGDB: {ScrapeRedactor.Redact(failed)}";
                }
                _appSettings.OpenEmuFallback = true;
                _appSettings.Save();
                native.Settings(NativeSettings());
                StatusText.Text = $"OpenVGDB {tag} downloaded.";
                return $"OpenVGDB {tag} was downloaded, and OpenEmu's fallback is on.";
            }
            finally
            {
                _downloadingDatabase = false;
                ScrapeChanged?.Invoke();
            }
        }

        bool IScrapeHost.OpenVgdbBusy => _downloadingDatabase || ScrapeRunning;

        async Task<string?> IScrapeHost.RemoveOpenVgdbAsync()
        {
            if (!File.Exists(OpenVgdbPath)) return "OpenVGDB is not downloaded.";
            if (_downloadingDatabase || ScrapeRunning) return "OpenVGDB is in use by a run; remove it when the run has finished.";
            if (!await ConfirmRemoveOpenVgdb(this)) return null;
            if (_downloadingDatabase || ScrapeRunning) return "OpenVGDB is in use by a run; remove it when the run has finished.";
            try
            {
                long bytes = new FileInfo(OpenVgdbPath).Length;
                if (_native is { } n) n.RemoveOpenVgdb();
                File.Delete(OpenVgdbPath);
                if (File.Exists(OpenVgdbPath + ".part")) File.Delete(OpenVgdbPath + ".part");
                _appSettings.OpenEmuFallback = false;
                _appSettings.Save();
                _native?.Settings(NativeSettings());
                _sources?.Settings(NativeSettings());
                StatusText.Text = "OpenVGDB removed";
                ScrapeChanged?.Invoke();
                return $"OpenVGDB was removed ({ScrapeSizes.Text(bytes)} freed). OpenEmu's fallback is off until it is downloaded again.";
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
            if (!OpenNative()) return "The media store could not be opened, so OpenVGDB cannot be downloaded now.";
            return await DownloadOpenVgdbNativeAsync(_native!);
        }

        private void StopOnlineCovers() => _http?.Dispose();
    }
}
