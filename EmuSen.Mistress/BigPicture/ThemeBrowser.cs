using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Galaxia.Library;

namespace EmuSen.Mistress.BigPicture
{
    [Flags]
    public enum ThemeListState { NotInstalled = 0, Installed = 1, UpdateAvailable = 2, LocalChanges = 4 }

    // A list entry with what Mistress knows of it here: its folder when installed, and the state the browser marks it with.
    public sealed record ThemeBrowserEntry(ThemeListEntry Theme, InstalledRecord? Installed, ThemeRemote? Remote, ThemeLocalChanges Changes)
    {
        public ThemeListState State =>
            Installed is null ? ThemeListState.NotInstalled
                : ThemeListState.Installed
                  | (Remote?.Head is { } head && Installed.Commit is { } c && head != c ? ThemeListState.UpdateAvailable : 0)
                  | (Changes.Any ? ThemeListState.LocalChanges : 0);
    }

    // ES-DE's theme list, its screenshots and what each host says of a theme, asked only when the player opens the browser or presses something in it - see EmuSen_BigPicture.md §25.
    public sealed class ThemeBrowser : IDisposable
    {
        public static TimeSpan ListExpiry { get; } = TimeSpan.FromHours(24);
        public static TimeSpan RemoteExpiry { get; } = TimeSpan.FromHours(24);
        public static TimeSpan ScreenshotExpiry { get; } = TimeSpan.FromDays(30);

        // The largest screenshot kept; the list's are JPEGs of a few hundred kilobytes.
        public const long MaxScreenshotBytes = 8 << 20;

        private readonly Func<HttpClient> _factory;
        private readonly Func<DateTimeOffset> _clock;
        private readonly CancellationTokenSource _closing = new();
        private HttpClient? _http;

        public ThemeBrowser(Func<HttpClient> http, Func<DateTimeOffset>? clock = null)
        {
            _factory = http;
            _clock = clock ?? (() => DateTimeOffset.UtcNow);
        }

        public static string ScreenshotFolder => Path.Combine(ThemeDownloads.Root, ".list", "screenshots");

        public bool Closed => _closing.IsCancellationRequested;

        // Cancelled when the browser closes: every request and download the browser started stops with it.
        public CancellationToken Closing => _closing.Token;

        private HttpClient Http
        {
            get
            {
                ObjectDisposedException.ThrowIf(Closed, this);
                return _http ??= _factory();
            }
        }

        // What themes.db holds, with no request: null when the list was never fetched.
        public static ThemeList? Cached()
        {
            if (!File.Exists(ThemeRecords.DefaultPath)) return null;
            using ThemeRecords db = ThemeRecords.Open();
            return db.List();
        }

        public bool IsFresh(ThemeList list) => _clock() - list.FetchedAt < ListExpiry;

        // The list from themes.db while it is younger than a day, else from GitLab; Refresh always asks.
        public async Task<ThemeList> ListAsync(bool refresh, CancellationToken cancel = default)
        {
            if (!refresh && Cached() is { } kept && IsFresh(kept)) return kept;
            using CancellationTokenSource both = CancellationTokenSource.CreateLinkedTokenSource(_closing.Token, cancel);
            using HttpResponseMessage response = await Http.GetAsync(ThemeList.Address, both.Token);
            response.EnsureSuccessStatusCode();
            ThemeList list = ThemeList.Parse(await response.Content.ReadAsStringAsync(both.Token), _clock());
            using (ThemeRecords db = ThemeRecords.Open()) db.SaveList(list);
            return list;
        }

        // Every list entry with its installed folder and the state known without asking anyone.
        public static IReadOnlyList<ThemeBrowserEntry> Entries(ThemeList list)
        {
            List<InstalledRecord> installed = ThemeDownloads.Installed().Select(t => ThemeDownloads.RecordOf(t.Directory)).OfType<InstalledRecord>().ToList();
            Dictionary<string, ThemeRemote> remotes = new(StringComparer.Ordinal);
            if (File.Exists(ThemeRecords.DefaultPath))
                using (ThemeRecords db = ThemeRecords.Open())
                    foreach (ThemeListEntry t in list.Themes)
                        if (db.Remote(t.Url) is { } r) remotes[t.Url] = r;
            return list.Themes.Select(t =>
            {
                InstalledRecord? mine = t.Source is { } s ? installed.FirstOrDefault(r => r.Source.SameRepository(s)) : null;
                return new ThemeBrowserEntry(t, mine, remotes.GetValueOrDefault(t.Url), mine is null ? ThemeLocalChanges.Unknown : ThemeDownloads.LocalChanges(mine.Directory));
            }).ToList();
        }

        // --- screenshots ---

        // The local file of a screenshot, fetched once and kept for a month; null when the host has none.
        public async Task<string?> ScreenshotAsync(ThemeScreenshot shot, CancellationToken cancel = default)
        {
            string folder = ScreenshotFolder;
            using (ThemeRecords db = ThemeRecords.Open())
                if (db.Screenshot(shot.Image) is { } kept && _clock() - kept.FetchedAt < ScreenshotExpiry && File.Exists(Path.Combine(folder, kept.File)))
                    return Path.Combine(folder, kept.File);
            if (ThemeList.ScreenshotAddress(shot.Image) is not { } address) return null;
            using CancellationTokenSource both = CancellationTokenSource.CreateLinkedTokenSource(_closing.Token, cancel);
            using HttpResponseMessage response = await Http.GetAsync(address, HttpCompletionOption.ResponseHeadersRead, both.Token);
            if (!response.IsSuccessStatusCode) return null;
            string ext = Path.GetExtension(shot.Image).ToLowerInvariant();
            if (ext is not (".png" or ".jpg" or ".jpeg" or ".webp" or ".gif")) ext = ".img";
            string name = Convert.ToHexStringLower(SHA1.HashData(Encoding.UTF8.GetBytes(shot.Image))) + ext;
            Directory.CreateDirectory(folder);
            string target = Path.Combine(folder, name), part = target + ".part";
            try
            {
                long bytes = 0;
                await using (Stream body = await response.Content.ReadAsStreamAsync(both.Token))
                await using (FileStream file = File.Create(part))
                {
                    var buffer = new byte[1 << 15];
                    for (int n; (n = await body.ReadAsync(buffer, both.Token)) > 0;)
                    {
                        bytes += n;
                        if (bytes > MaxScreenshotBytes) throw new InvalidDataException($"{shot.Image} is larger than {MaxScreenshotBytes >> 20} MB.");
                        await file.WriteAsync(buffer.AsMemory(0, n), both.Token);
                    }
                }
                File.Move(part, target, overwrite: true);
                using ThemeRecords db = ThemeRecords.Open();
                db.RememberScreenshot(new CachedScreenshot(shot.Image, name, bytes, _clock()));
                return target;
            }
            finally
            {
                if (File.Exists(part)) File.Delete(part);
            }
        }

        // --- what the host says ---

        public static ThemeRemote? KnownRemote(ThemeListEntry theme)
        {
            if (!File.Exists(ThemeRecords.DefaultPath)) return null;
            using ThemeRecords db = ThemeRecords.Open();
            return db.Remote(theme.Url);
        }

        // The default branch, its newest commit and date, and the licence line, asked of the host when the player opens a theme; kept for a day.
        public async Task<ThemeRemote> RemoteAsync(ThemeListEntry theme, bool refresh = false, CancellationToken cancel = default)
        {
            if (!refresh && KnownRemote(theme) is { Error: null } kept && _clock() - kept.FetchedAt < RemoteExpiry) return kept;
            if (theme.Source is not { } source) throw new InvalidDataException($"{theme.Url} is neither a GitHub nor a GitLab repository.");
            using CancellationTokenSource both = CancellationTokenSource.CreateLinkedTokenSource(_closing.Token, cancel);
            ThemeRemote remote;
            try
            {
                using JsonDocument? repo = await ThemeHostJson.GetAsync(Http, source.RepositoryAddress, both.Token);
                if (repo is null) throw new HttpRequestException($"{source.Url} did not answer for the repository.");
                string branch = repo.RootElement.TryGetProperty("default_branch", out JsonElement b) && b.ValueKind == JsonValueKind.String ? b.GetString()! : "main";
                string? hostLicence = repo.RootElement.TryGetProperty("license", out JsonElement l) && l.ValueKind == JsonValueKind.Object
                    && l.TryGetProperty("name", out JsonElement n) && n.ValueKind == JsonValueKind.String ? n.GetString() : null;
                ThemeSource on = source.OnBranch(branch);
                (string Sha, DateTimeOffset? Date)? head = await ThemeDownloads.HeadAsync(Http, on, both.Token);
                string? readme = null;
                using (HttpResponseMessage r = await Http.GetAsync(on.ReadmeAddress, both.Token))
                    if (r.IsSuccessStatusCode) readme = await r.Content.ReadAsStringAsync(both.Token);
                (IReadOnlyList<string> licence, string from) = LicenceLine(readme, hostLicence, source.Host);
                remote = new ThemeRemote(theme.Url, branch, head?.Sha, head?.Date, licence, from, _clock(), null);
            }
            catch (Exception ex) when (ex is HttpRequestException or JsonException or InvalidDataException)
            {
                ErrorLog.Error("themes", "A theme's licence could not be read", ex, theme.Url);
                remote = new ThemeRemote(theme.Url, null, null, null, [$"The licence could not be read: {ex.Message}"], "unknown", _clock(), ex.Message);
            }
            using (ThemeRecords db = ThemeRecords.Open()) db.RememberRemote(remote);
            return remote;
        }

        // An installed theme's newest commit only, for its Update mark when the browser opens: one request, on the branch it was downloaded from.
        public async Task<ThemeRemote?> HeadAsync(ThemeListEntry theme, InstalledRecord installed, bool refresh = false, CancellationToken cancel = default)
        {
            ThemeRemote? kept = KnownRemote(theme);
            if (!refresh && kept is { Error: null, Head: not null } && _clock() - kept.FetchedAt < RemoteExpiry) return kept;
            using CancellationTokenSource both = CancellationTokenSource.CreateLinkedTokenSource(_closing.Token, cancel);
            (string Sha, DateTimeOffset? Date)? head = await ThemeDownloads.HeadAsync(Http, installed.Source, both.Token);
            if (head is null) return kept;
            ThemeRemote remote = kept is null
                ? new ThemeRemote(theme.Url, installed.Source.Branch, head.Value.Sha, head.Value.Date, [], "unknown", _clock(), null)
                : kept with { Head = head.Value.Sha, HeadDate = head.Value.Date, FetchedAt = _clock(), Error = null };
            using (ThemeRecords db = ThemeRecords.Open()) db.RememberRemote(remote);
            return remote;
        }

        // The README's licence section, as the About sheet reads it after a download, else the licence the host detects, else none.
        public static (IReadOnlyList<string> Lines, string From) LicenceLine(string? readme, string? hostLicence, ThemeHost host)
        {
            if (readme is not null)
            {
                IReadOnlyList<string> section = ThemeAttribution.Section(readme.Replace("\r\n", "\n").Split('\n'), "licen");
                if (section.Count > 0) return (section.Take(6).ToList(), "readme");
            }
            if (!string.IsNullOrWhiteSpace(hostLicence) && hostLicence is not "Other" and not "NOASSERTION")
                return ([$"{hostLicence}, as {host} reads the repository's licence file."], "host");
            return (["The theme states no licence in its README, and " + host + " finds no licence file."], "none");
        }

        // --- download ---

        // The theme's archive on its default branch, through ThemeDownloads' checked swap; stopped by the browser's closing as by the cancel given.
        public async Task<ThemeStamp> DownloadAsync(ThemeListEntry theme, ThemeRemote remote, IProgress<(long Read, long? Total)>? progress, IProgress<(int Done, int Total)>? unpacked,
            bool replaceLocalChanges, CancellationToken cancel)
        {
            if (theme.Source is not { } source) throw new InvalidDataException($"{theme.Url} is neither a GitHub nor a GitLab repository.");
            using CancellationTokenSource both = CancellationTokenSource.CreateLinkedTokenSource(_closing.Token, cancel);
            ObjectDisposedException.ThrowIf(Closed, this);
            using HttpClient http = _factory();
            http.Timeout = TimeSpan.FromMinutes(30);
            ThemeStamp stamp = await ThemeDownloads.FetchAsync(http, source.OnBranch(remote.Branch ?? "main"), progress, both.Token, unpacked, replaceLocalChanges);
            using (ThemeRecords db = ThemeRecords.Open())
                db.RememberRemote(remote with { Head = stamp.Commit ?? remote.Head, HeadDate = stamp.Committed ?? remote.HeadDate });
            return stamp;
        }

        // Closing stops everything in flight and lets the client go; nothing the browser started outlives it.
        public void Dispose()
        {
            if (!_closing.IsCancellationRequested) _closing.Cancel();
            _http?.Dispose();
            _http = null;
        }
    }
}
