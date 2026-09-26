using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;

namespace EmuSen.Mistress.Scraping
{
    // ScreenScraper's answers, by what the client must do about them - see EmuSen_BigPicture.md §5.1's table.
    public enum ScrapeStatus
    {
        Found,
        NotFound,          // 404: record as unknown
        BadRequest,        // 400: this request is wrong, and will be again
        ServerBusy,        // 401: closed to non-members while the server is loaded; wait five minutes
        BadCredentials,    // 403: stop
        ApiClosed,         // 423: stop
        Blacklisted,       // 426: stop; the software needs a new build
        TooManyRequests,   // 429: slow down
        DailyQuota,        // 430: stop until tomorrow
        DailyKoQuota,      // 431: stop until tomorrow
        Malformed,         // an answer that is not the documented shape
        Failed,            // no answer: the network, a timeout, another status
    }

    public sealed record JeuInfosAnswer(ScrapeStatus Status, ScrapedGame? Game, ScrapeQuota? Quota, string Detail, TimeSpan Took, string? Body = null);

    public enum MediaOutcome { Saved, AlreadyThere, NoMedia, NotAnImage, TooSmall, Failed }

    public sealed record MediaAnswer(MediaOutcome Outcome, string? Path, long Bytes, string? Sha1, string Detail, TimeSpan Took);

    // ScreenScraper's API v2 over one HttpClient: jeuInfos by hashes, size and name; the media it names; the member's quota; the systems - see EmuSen_BigPicture.md §17.
    public sealed class ScreenScraperClient
    {
        public const string Api = "https://api.screenscraper.fr/api2/";

        // ES-DE's threshold, and §4.39's: anything smaller is an error page, not a picture.
        public const int SmallestImage = 80;

        private readonly HttpClient _http;
        private readonly DeveloperCredentials _developer;
        private volatile MemberAccount? _member;

        public ScreenScraperClient(HttpClient http, DeveloperCredentials developer, MemberAccount? member)
        {
            _http = http;
            _developer = developer;
            Member = member;
        }

        // Read at every request, so signing out mid-run stops ssid and sspassword from the next one - see EmuSen_Settings_Reference.md §4.60.
        public MemberAccount? Member
        {
            get => _member;
            set => _member = value is { IsSet: true } ? value : null;
        }

        public bool HasMember => _member is not null;

        public string SoftName => _developer.SoftName;

        public static ScrapeStatus StatusOf(HttpStatusCode code) => (int)code switch
        {
            200 => ScrapeStatus.Found,
            400 => ScrapeStatus.BadRequest,
            401 => ScrapeStatus.ServerBusy,
            403 => ScrapeStatus.BadCredentials,
            404 => ScrapeStatus.NotFound,
            423 => ScrapeStatus.ApiClosed,
            426 => ScrapeStatus.Blacklisted,
            429 => ScrapeStatus.TooManyRequests,
            430 => ScrapeStatus.DailyQuota,
            431 => ScrapeStatus.DailyKoQuota,
            _ => ScrapeStatus.Failed,
        };

        // The credentials first, as every call carries them; output=json; romtype rom; the three hashes, the size, the file's own name and the system.
        public string JeuInfosUrl(int systemId, RomHashes hashes, string fileName) =>
            Url("jeuInfos.php",
                ("romtype", "rom"), ("systemeid", systemId.ToString(CultureInfo.InvariantCulture)),
                ("crc", hashes.Crc32), ("md5", hashes.Md5), ("sha1", hashes.Sha1),
                ("romtaille", hashes.Size.ToString(CultureInfo.InvariantCulture)), ("romnom", Path.GetFileName(fileName)));

        public string Url(string endpoint, params (string Name, string Value)[] query)
        {
            var url = new StringBuilder(Api).Append(endpoint).Append('?');
            Append(url, "devid", _developer.DevId);
            Append(url, "devpassword", _developer.DevPassword);
            Append(url, "softname", _developer.SoftName);
            Append(url, "output", "json");
            if (_member is { } member)
            {
                Append(url, "ssid", member.User);
                Append(url, "sspassword", member.Password);
            }
            foreach ((string name, string value) in query) Append(url, name, value);
            return url.ToString(0, url.Length - 1);
        }

        private static void Append(StringBuilder url, string name, string value) =>
            url.Append(name).Append('=').Append(Uri.EscapeDataString(value)).Append('&');

        public async Task<JeuInfosAnswer> JeuInfosAsync(int systemId, RomHashes hashes, string fileName, CancellationToken stop)
        {
            (ScrapeStatus status, string body, TimeSpan took) = await GetTextAsync(JeuInfosUrl(systemId, hashes, fileName), stop);
            if (status != ScrapeStatus.Found) return new JeuInfosAnswer(status, null, null, Snippet(body), took, body);
            try
            {
                (ScrapedGame? game, ScrapeQuota? quota) = ScreenScraperJson.JeuInfos(body);
                return game is null
                    ? new JeuInfosAnswer(ScrapeStatus.NotFound, null, quota, "no game in the answer", took, body)
                    : new JeuInfosAnswer(ScrapeStatus.Found, game, quota, "", took, body);
            }
            catch (Exception ex) when (ex is JsonException or KeyNotFoundException or FormatException or InvalidOperationException)
            {
                return new JeuInfosAnswer(ScrapeStatus.Malformed, null, null, ScrapeRedactor.Redact(ex.Message), took, body);
            }
        }

        // A media address carries the account its lookup was made with; with no member now, ssid and sspassword are taken out of it.
        public string MediaUrl(string url)
        {
            if (_member is not null) return url;
            int query = url.IndexOf('?');
            if (query < 0) return url;
            string[] kept = url[(query + 1)..].Split('&').Where(p => !(p.StartsWith("ssid=", StringComparison.OrdinalIgnoreCase) || p.StartsWith("sspassword=", StringComparison.OrdinalIgnoreCase))).ToArray();
            return url[..(query + 1)] + string.Join('&', kept);
        }

        // One ssuserInfos request with the member and the developer credentials, only when the player presses Log In or Check - see EmuSen_Settings_Reference.md §4.60.
        public async Task<SignInAnswer> SignInAsync(CancellationToken stop)
        {
            if (_member is null) return new SignInAnswer(SignInResult.Incomplete, null, "Type your ScreenScraper name and password first.");
            (ScrapeStatus status, string body, _) = await GetTextAsync(Url("ssuserInfos.php"), stop);
            return SignInAnswer.From(status, body);
        }

        public async Task<(ScrapeStatus Status, ScrapeQuota? Quota, string Detail)> UserInfosAsync(CancellationToken stop)
        {
            (ScrapeStatus status, string body, _) = await GetTextAsync(Url("ssuserInfos.php"), stop);
            if (status != ScrapeStatus.Found) return (status, null, Snippet(body));
            try { return (status, ScreenScraperJson.User(body), ""); }
            catch (Exception ex) when (ex is JsonException or KeyNotFoundException or InvalidOperationException) { return (ScrapeStatus.Malformed, null, ScrapeRedactor.Redact(ex.Message)); }
        }

        public async Task<(ScrapeStatus Status, IReadOnlyDictionary<int, string> Systems, string Detail)> SystemsAsync(CancellationToken stop)
        {
            (ScrapeStatus status, string body, _) = await GetTextAsync(Url("systemesListe.php"), stop);
            if (status != ScrapeStatus.Found) return (status, new Dictionary<int, string>(), Snippet(body));
            try { return (status, ScreenScraperJson.Systems(body), ""); }
            catch (Exception ex) when (ex is JsonException or KeyNotFoundException or InvalidOperationException) { return (ScrapeStatus.Malformed, new Dictionary<int, string>(), ScrapeRedactor.Redact(ex.Message)); }
        }

        // A media file written beside its final name and moved into place, never over one already there; only an image of at least 80 bytes is kept.
        public async Task<MediaAnswer> DownloadAsync(string url, string target, CancellationToken stop)
        {
            var clock = Stopwatch.StartNew();
            if (File.Exists(target)) return new MediaAnswer(MediaOutcome.AlreadyThere, target, new FileInfo(target).Length, null, "", clock.Elapsed);
            url = MediaUrl(url);
            try
            {
                using HttpResponseMessage response = await _http.GetAsync(url, HttpCompletionOption.ResponseContentRead, stop);
                byte[] bytes = await response.Content.ReadAsByteArrayAsync(stop);
                if (response.StatusCode != HttpStatusCode.OK)
                    return new MediaAnswer(MediaOutcome.Failed, null, 0, null, $"HTTP {(int)response.StatusCode}", clock.Elapsed);
                string text = bytes.Length < 64 ? Encoding.ASCII.GetString(bytes).Trim() : "";
                if (text is "NOMEDIA") return new MediaAnswer(MediaOutcome.NoMedia, null, 0, null, "NOMEDIA", clock.Elapsed);
                string? type = response.Content.Headers.ContentType?.MediaType;
                if (type is null || !type.StartsWith("image/", StringComparison.Ordinal))
                    return new MediaAnswer(MediaOutcome.NotAnImage, null, bytes.Length, null, ScrapeRedactor.Redact(text.Length > 0 ? text : type ?? "no type"), clock.Elapsed);
                if (bytes.Length < SmallestImage) return new MediaAnswer(MediaOutcome.TooSmall, null, bytes.Length, null, $"{bytes.Length} bytes", clock.Elapsed);

                Directory.CreateDirectory(Path.GetDirectoryName(target)!);
                string temp = target + ".part";
                await File.WriteAllBytesAsync(temp, bytes, stop);
                try { File.Move(temp, target, overwrite: false); }
                catch (IOException) when (File.Exists(target))
                {
                    File.Delete(temp);
                    return new MediaAnswer(MediaOutcome.AlreadyThere, target, new FileInfo(target).Length, null, "", clock.Elapsed);
                }
                return new MediaAnswer(MediaOutcome.Saved, target, bytes.Length, Convert.ToHexStringLower(SHA1.HashData(bytes)), "", clock.Elapsed);
            }
            catch (Exception ex) when (ex is HttpRequestException or IOException or UnauthorizedAccessException || (ex is TaskCanceledException && !stop.IsCancellationRequested))
            {
                return new MediaAnswer(MediaOutcome.Failed, null, 0, null, ScrapeRedactor.Redact(ex.Message), clock.Elapsed);
            }
        }

        private async Task<(ScrapeStatus Status, string Body, TimeSpan Took)> GetTextAsync(string url, CancellationToken stop)
        {
            var clock = Stopwatch.StartNew();
            try
            {
                using HttpResponseMessage response = await _http.GetAsync(url, HttpCompletionOption.ResponseContentRead, stop);
                string body = await response.Content.ReadAsStringAsync(stop);
                return (StatusOf(response.StatusCode), body, clock.Elapsed);
            }
            catch (Exception ex) when (ex is HttpRequestException or IOException || (ex is TaskCanceledException && !stop.IsCancellationRequested))
            {
                return (ScrapeStatus.Failed, ex.Message, clock.Elapsed);
            }
        }

        // The start of a body, for the status line: never more than a line, and never a credential.
        private static string Snippet(string body)
        {
            string line = body.Split('\n', 2)[0].Trim();
            return ScrapeRedactor.Redact(line.Length > 160 ? line[..160] : line);
        }
    }
}
