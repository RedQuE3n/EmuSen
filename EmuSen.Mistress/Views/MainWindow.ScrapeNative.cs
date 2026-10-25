using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Threading;
using EmuSen.Crystal.Client.Scrape;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // The scraper in Rust behind the binding (EMUSEN_SCRAPE_NATIVE, state 2): the window's store, runs and pictures through emusen_scrape.h - see EmuSen_Backend.md §30.14.
    public partial class MainWindow
    {
        // Replaced by tests: the scraper's test seams as JSON (a server's address and root, a clock, offline); null in a player's run.
        internal static Func<string?> ScrapeTest = () => null;

        private NativeScraping? _native;
        private Thread? _nativePump;
        private volatile bool _nativePumping;
        private long _nativeApplied;
        private bool _nativeWorkers;
        private ScrapeQuota? _nativeLimits;
        private int _nativeRun;

        // State 2 needs the library through Rust too, since the scraper keeps cover lookups and follows covers chosen there.
        private bool ScrapeNative => MediaService.Active && _records.ThroughLibrary;

        public NativeScraping? ScrapeNativeService => _native;

        private bool DeveloperFromFiles => DeveloperSource == (Func<DeveloperCredentials?>)DeveloperCredentials.Load;

        private byte[]? NativeEmbedded() => DeveloperFromFiles ? DeveloperCredentials.EmbeddedBlob() : DeveloperSource() is { } d ? DeveloperCredentials.Encode(d) : null;

        private string NativeSettings()
        {
            string json = JsonSerializer.Serialize(new Dictionary<string, object?>
            {
                ["roms"] = _appSettings.RomDirectory, ["art"] = ArtworkDirectory, ["esde_media"] = _appSettings.EsdeMediaDirectory, ["openemu"] = _appSettings.OpenEmuFallback,
                ["user_agent"] = $"EmuSen/{(BuildName ?? "unknown").Split('+')[0]}", ["developer_files"] = DeveloperFromFiles,
                ["cores"] = RuntimeCatalog.Consoles.Select(c => c.Systems).ToArray(), ["packages"] = PackagesByGame,
            });
            return ScrapeTest() is string test ? json[..^1] + ",\"test\":" + test + "}" : json;
        }

        private bool OpenNative()
        {
            if (_native is not null) return true;
            try
            {
                _native = NativeScraping.Open(MediaStore.DefaultRoot, NativeSettings(), _records.Library, NativeEmbedded());
                return true;
            }
            catch (MediaServiceException ex)
            {
                ErrorLog.Error("scraping", "The media store could not be opened", ex);
                StatusText.Text = $"The media store could not be opened: {ScrapeRedactor.Redact(ex.Message)}";
                return false;
            }
        }

        // A run started through the scraper; its progress drained on a thread of the window's and posted to the window.
        private bool StartNative(ScrapeScope scope, bool resume, ScrapedGame? chosen)
        {
            NativeScraping native = _native!;
            var games = resume ? [] : ScopeGames(scope).Select(g => (g.FullPath, SystemOf(g)!)).ToList();
            if (!resume && scope.Game is not null) _fetcher?.Forget(scope.Game);
            var priority = scope.Game is not null ? ScrapePriority.Shown : scope.Shelf is not null ? ScrapePriority.Console : ScrapePriority.Library;
            long queued;
            try { queued = native.Start(games, priority, scope.Game is not null, resume, scope.Refresh && !resume, chosen, _appSettings.Scraping, _scrapeChoices); }
            catch (MediaServiceException ex) when (ex.Status == MediaServiceException.Busy) { return false; }
            if (queued == 0) return false;
            _scrapeRun = new ScrapeProgress((int)queued, native.Now);
            _nativeApplied = 0;
            _nativeWorkers = true;
            int id = ++_nativeRun;
            ReadScrapeSnapshot();
            ApplyNative(native.Progress(0), id);
            StartPump();
            ScrapeLine(_scrapeRun.Describe());
            ScrapeChanged?.Invoke();
            ShowScrapeStatus();
            return true;
        }

        private void StartPump()
        {
            StopPump();
            _nativePumping = true;
            NativeScraping native = _native!;
            int id = _nativeRun;
            _nativePump = new Thread(() =>
            {
                long serial = -1;
                while (_nativePumping)
                {
                    long now = native.Wait(serial, 200);
                    if (now == serial || !_nativePumping) continue;
                    serial = now;
                    string json = native.Progress(0);
                    Dispatcher.UIThread.Post(() => ApplyNative(json, id));
                    using JsonDocument doc = JsonDocument.Parse(json);
                    if (doc.RootElement.GetProperty("state").GetString() != "running") break;
                }
            }) { IsBackground = true, Name = "scrape progress" };
            _nativePump.Start();
        }

        private void StopPump()
        {
            _nativePumping = false;
            Thread? pump = _nativePump;
            _nativePump = null;
            if (pump is not null && pump != Thread.CurrentThread) pump.Join(TimeSpan.FromSeconds(2));
        }

        private static ScrapeOutcome Outcome(string? word) => word switch
        {
            "found" => ScrapeOutcome.Found, "unknown" => ScrapeOutcome.Unknown, "error" => ScrapeOutcome.Error, "retry" => ScrapeOutcome.Retry, "stopped" => ScrapeOutcome.Stopped, _ => ScrapeOutcome.Missing,
        };

        // The drained progress laid onto the window's ScrapeProgress, each new game's side effects taken as ScrapeArrived takes them.
        private void ApplyNative(string json, int id)
        {
            if (_scrapeClosed || id != _nativeRun || _scrapeRun is not { } run) return;
            using JsonDocument doc = JsonDocument.Parse(json);
            JsonElement p = doc.RootElement;
            string? Text(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;
            var rows = p.GetProperty("results").EnumerateArray().ToList();
            var recent = rows.Select(r => new ScrapeRecent(Text(r, "path")!, Text(r, "system")!, Outcome(Text(r, "outcome")), r.GetProperty("kinds").EnumerateArray().Select(k => k.GetString()!).ToList(),
                Text(r, "detail") ?? "", r.GetProperty("skipped").GetBoolean()) { FromFailover = r.GetProperty("from_failover").GetBoolean() }).ToList();
            ScrapeActivity? current = p.GetProperty("current").ValueKind == JsonValueKind.Object
                ? new ScrapeActivity(Text(p.GetProperty("current"), "path")!, Text(p.GetProperty("current"), "system")!, Text(p.GetProperty("current"), "step") switch { "downloading" => ScrapeStep.Downloading, "arrived" => ScrapeStep.Arrived, _ => ScrapeStep.LookingUp },
                    Text(p.GetProperty("current"), "kind"), Text(p.GetProperty("current"), "saved"))
                : null;
            JsonElement failover = p.GetProperty("failover");
            run.Mirror(p.GetProperty("done").GetInt32(), p.GetProperty("found").GetInt32(), p.GetProperty("unknown").GetInt32(), p.GetProperty("failed").GetInt32(), p.GetProperty("skipped").GetInt32(),
                p.GetProperty("retrying").GetInt32(), p.GetProperty("unchanged").GetInt32(), p.GetProperty("requests").GetInt32(), Text(p, "last_failure"),
                failover.GetProperty("asked").GetInt32(), failover.GetProperty("found").GetInt32(), current, Text(p, "last_picture"), recent);
            _nativeWorkers = p.GetProperty("workers").GetBoolean();
            _nativeLimits = NativeScraping.Limits(p.GetProperty("quota"));
            run.SetPaused(p.GetProperty("paused").GetBoolean(), ScrapeNow);
            if (Text(p, "why") is string why) run.Why = why;
            foreach (JsonElement r in rows.Where(r => r.GetProperty("serial").GetInt64() > _nativeApplied))
                if (Outcome(Text(r, "outcome")) == ScrapeOutcome.Error) ErrorLog.Warning("scraping", $"ScreenScraper could not identify the game: {ScrapeRedactor.Redact(Text(r, "detail"))}", Text(r, "path"));
            if (rows.Count > 0) _nativeApplied = Math.Max(_nativeApplied, rows.Max(r => r.GetProperty("serial").GetInt64()));
            string state = Text(p, "state")!;
            if (state == "running")
            {
                ScrapeLine(run.Describe());
                PostCoverRefresh();
                ScrapeChanged?.Invoke();
                return;
            }
            if (run.State != ScrapeRunState.Running) return;
            if (state == "stopped" && run.Why is { } reason) ErrorLog.Warning("scraping", $"A scrape run stopped: {ScrapeRedactor.Redact(reason)}");
            EndRun(state switch { "done" => ScrapeRunEnd.Done, "stopped" => ScrapeRunEnd.Stopped, _ => ScrapeRunEnd.Cancelled });
        }

        // What the store holds, read for the tests whichever half the switch chose.
        public IReadOnlyList<(string Path, string System)> ScrapeQueue() => _native is { } n ? n.Queue() : _mediaStore?.Queue().Select(q => (q.Path, q.System)).ToList() ?? [];

        public IReadOnlyList<StoredMedia> ScrapeMedia(string md5, long bytes) => _native is { } n ? n.Media(md5, bytes) : _mediaStore?.Media(md5, bytes) ?? [];

        // Whether a run's ScreenScraper workers are live, whichever half the switch chose.
        public bool ScrapeWorkersLive => _native is not null ? ScrapeRunning && _nativeWorkers : _scraper is not null;

        // Whether the media store is open, whichever half the switch chose.
        public bool ScrapeStoreOpen => _native is not null || _mediaStore is { IsOpen: true };

        private void StopNative()
        {
            StopPump();
            _native?.Dispose();
            _native = null;
        }
    }
}
