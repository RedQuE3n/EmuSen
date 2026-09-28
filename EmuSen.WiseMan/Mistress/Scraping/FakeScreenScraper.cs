using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Text.Json.Nodes;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Mistress.Scraping;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // One game as the fake server knows it: which MD5s find it, and what it answers with.
    public sealed record FakeGame(long Id, string Name, params string[] Md5s)
    {
        public List<(string Type, string? Region, string Format)> Media { get; init; } =
            [("box-2D", "us", "png"), ("box-2D", "eu", "png"), ("box-2D", "jp", "png"), ("ss", "wor", "png"), ("wheel-hd", "wor", "png"), ("wheel", "wor", "png"), ("mixrbv2", "wor", "png"), ("sstitle", "wor", "png")];
        public string Synopsis { get; init; } = "A synthetic game written for the tests.";
        public int SystemId { get; init; } = 4;

        // Every kind Pass 8 fetches as well as the first five, as §21.1's answers offered them (§38).
        public static List<(string Type, string? Region, string Format)> AllMedia =>
            [("box-2D", "us", "png"), ("ss", "wor", "png"), ("wheel-hd", "wor", "png"), ("mixrbv2", "wor", "png"), ("sstitle", "wor", "png"),
             ("box-2D-back", "us", "png"), ("box-3D", "us", "png"), ("support-2D", "us", "png"), ("fanart", null, "jpg"), ("manuel", "us", "pdf"), ("video-normalized", null, "mp4")];
    }

    // ScreenScraper as its API page documents it, answered from responses the tests write; never the network - see EmuSen_BigPicture.md §17.
    public sealed class FakeScreenScraper : HttpMessageHandler
    {
        public const string DevId = "FAKEDEVID";
        public const string DevPassword = "FAKEDEVPASSWORD";
        public const string SoftName = "EmuSen-Mistress-Test";

        public static DeveloperCredentials Developer => new(DevId, DevPassword, SoftName);

        public readonly ConcurrentQueue<string> Asked = new();
        public readonly List<FakeGame> Games = new();

        // The status every jeuInfos answers with instead of looking, when set; 0 looks.
        public int ForcedStatus;

        // A status for one game's MD5 alone, with the body it answers.
        public readonly ConcurrentDictionary<string, (int Code, string Body)> StatusByMd5 = new(StringComparer.OrdinalIgnoreCase);

        // The one member account ssuserInfos accepts; any other, or none, is 403 as the live server answered (plan §17.9). Null accepts any.
        public (string User, string Password)? Member;

        public string MemberLevel = "3";

        // ssuserInfos answers this status and body instead, when set.
        public int ForcedUserStatus;
        public string ForcedUserBody = "Erreur : forced by the test";

        // What the ssuser block says; null leaves the block out.
        public JsonObject? User = Quota(maxThreads: 1, perMinute: 60, perDay: 20000, koPerDay: 2000, today: 10, koToday: 1);

        // Answers for anything that is not ScreenScraper, such as the OpenEmu failover's servers.
        public Func<string, HttpResponseMessage> Other = _ => new HttpResponseMessage(HttpStatusCode.NotFound);

        // Held open until released, to catch a request in flight; null answers at once.
        public SemaphoreSlim? Gate;

        private int _inFlight;
        public int MostAtOnce;

        public static JsonObject Quota(int maxThreads, int perMinute, int perDay, int koPerDay, int today, int koToday) => new()
        {
            ["id"] = "", ["maxthreads"] = maxThreads.ToString(), ["maxdownloadspeed"] = "128", ["requeststoday"] = today.ToString(),
            ["requestskotoday"] = koToday.ToString(), ["maxrequestspermin"] = perMinute.ToString(), ["maxrequestsperday"] = perDay.ToString(),
            ["maxrequestskoperday"] = koPerDay.ToString(),
        };

        public IEnumerable<string> JeuInfos => Asked.Where(u => u.Contains("/jeuInfos.php"));
        public IEnumerable<string> Searches => Asked.Where(u => u.Contains("/jeuRecherche.php"));
        public IEnumerable<string> MediaAsked => Asked.Where(u => u.Contains("/mediaJeu.php"));

        // A media type's content revision: raising it changes the bytes served and the checksum the answer states, as an improved scan upstream would.
        public readonly ConcurrentDictionary<string, int> MediaRevision = new(StringComparer.Ordinal);

        // Whether a jeuInfos answer states each media's sha1; ScreenScraper's did for every file in §17.9's answers.
        public bool StateChecksums = true;

        // Media bytes sent, for the tests' byte counts.
        private long _bytesSent;
        public long BytesSent => Interlocked.Read(ref _bytesSent);
        public IEnumerable<string> OthersAsked => Asked.Where(u => !u.Contains("screenscraper.fr"));

        public static string Param(string url, string name) =>
            new Uri(url).Query.TrimStart('?').Split('&').Select(p => p.Split('=', 2)).Where(p => p[0] == name).Select(p => Uri.UnescapeDataString(p[1])).FirstOrDefault() ?? "";

        protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
        {
            string url = request.RequestUri!.AbsoluteUri;
            Asked.Enqueue(url);
            int now = Interlocked.Increment(ref _inFlight);
            lock (Games) MostAtOnce = Math.Max(MostAtOnce, now);
            try
            {
                if (Gate is { } gate) await gate.WaitAsync(cancellationToken);
                if (!url.Contains("screenscraper.fr")) return Other(url);
                if (url.Contains("/mediaJeu.php")) return MediaAnswer(url);
                if (url.Contains("/jeuInfos.php")) return JeuInfosAnswer(url);
                if (url.Contains("/jeuRecherche.php")) return SearchAnswer(url);
                if (url.Contains("/ssuserInfos.php")) return UserInfosAnswer(url);
                return new HttpResponseMessage(HttpStatusCode.BadRequest) { Content = new StringContent("Erreur") };
            }
            finally
            {
                Interlocked.Decrement(ref _inFlight);
            }
        }

        private HttpResponseMessage UserInfosAnswer(string url)
        {
            if (ForcedUserStatus != 0) return new HttpResponseMessage((HttpStatusCode)ForcedUserStatus) { Content = new StringContent(ForcedUserBody) };
            string ssid = Param(url, "ssid"), sspassword = Param(url, "sspassword");
            if (Member is { } member && (ssid != member.User || sspassword != member.Password))
                return new HttpResponseMessage(HttpStatusCode.Forbidden) { Content = new StringContent("Erreur de login : Vérifier les identifiants utilisateurs !") };
            var user = (JsonObject?)User?.DeepClone();
            if (user is not null)
            {
                user["id"] = ssid;
                user["niveau"] = MemberLevel;
            }
            return Json(new JsonObject { ["header"] = Header(), ["response"] = new JsonObject { ["ssuser"] = user } });
        }

        private HttpResponseMessage JeuInfosAnswer(string url)
        {
            if (ForcedStatus != 0) return new HttpResponseMessage((HttpStatusCode)ForcedStatus) { Content = new StringContent("Erreur : forced by the test") };
            string md5 = Param(url, "md5");
            if (StatusByMd5.TryGetValue(md5, out var forced)) return new HttpResponseMessage((HttpStatusCode)forced.Code) { Content = new StringContent(forced.Body) };
            FakeGame? game;
            lock (Games) game = Games.FirstOrDefault(g => g.Md5s.Contains(md5, StringComparer.OrdinalIgnoreCase));
            // A game asked by its id, as a pick from a name search is, when its hashes find nothing.
            if (game is null && long.TryParse(Param(url, "gameid"), out long gameId)) lock (Games) game = Games.FirstOrDefault(g => g.Id == gameId);
            if (game is null) return new HttpResponseMessage(HttpStatusCode.NotFound) { Content = new StringContent("Erreur : Rom/Iso/Dossier non trouvée !  ") };

            var response = new JsonObject { ["serveurs"] = new JsonObject(), ["jeu"] = Jeu(game, url) };
            if (User is not null) response["ssuser"] = User.DeepClone();
            return Json(new JsonObject { ["header"] = Header(), ["response"] = response });
        }

        // jeuRecherche: every game of the system whose name holds the text, as jeuInfos writes each; none is [{}], an entry with no id.
        private HttpResponseMessage SearchAnswer(string url)
        {
            if (ForcedStatus != 0) return new HttpResponseMessage((HttpStatusCode)ForcedStatus) { Content = new StringContent("Erreur : forced by the test") };
            string text = Param(url, "recherche");
            int.TryParse(Param(url, "systemeid"), out int system);
            List<FakeGame> hits;
            lock (Games) hits = Games.Where(g => g.SystemId == system && g.Name.Contains(text, StringComparison.OrdinalIgnoreCase)).Take(30).ToList();
            var jeux = hits.Count == 0 ? new JsonArray(new JsonObject()) : new JsonArray(hits.Select(g => (JsonNode)Jeu(g, url)).ToArray());
            var response = new JsonObject { ["serveurs"] = new JsonObject(), ["jeux"] = jeux };
            if (User is not null) response["ssuser"] = User.DeepClone();
            return Json(new JsonObject { ["header"] = Header(), ["response"] = response });
        }

        private JsonObject Jeu(FakeGame game, string url)
        {
            string credentials = $"devid={DevId}&devpassword={DevPassword}&softname={SoftName}&ssid={Uri.EscapeDataString(Param(url, "ssid"))}&sspassword={Uri.EscapeDataString(Param(url, "sspassword"))}";
            var medias = new JsonArray(game.Media.Select(m => (JsonNode)new JsonObject
            {
                ["type"] = m.Type, ["parent"] = "jeu", ["region"] = m.Region, ["format"] = m.Format, ["crc"] = "00000000",
                ["url"] = $"https://neoclone.screenscraper.fr/api2/mediaJeu.php?{credentials}&systemeid={game.SystemId}&jeuid={game.Id}&media={m.Type}({m.Region})",
                ["sha1"] = StateChecksums ? Convert.ToHexStringLower(System.Security.Cryptography.SHA1.HashData(Bytes(m.Type + "(" + m.Region + ")", game.Id.ToString()))) : null,
                ["size"] = Bytes(m.Type + "(" + m.Region + ")", game.Id.ToString()).Length.ToString(),
            }).ToArray());
            var jeu = new JsonObject
            {
                ["id"] = game.Id.ToString(), ["romid"] = (game.Id * 10).ToString(), ["notgame"] = "false",
                ["noms"] = new JsonArray(new JsonObject { ["region"] = "ss", ["text"] = game.Name }, new JsonObject { ["region"] = "us", ["text"] = game.Name + " (US title)" }),
                ["systeme"] = new JsonObject { ["id"] = game.SystemId.ToString(), ["text"] = "Synthetic" },
                ["editeur"] = new JsonObject { ["id"] = "1", ["text"] = "Synthetic Publisher" },
                ["developpeur"] = new JsonObject { ["id"] = "2", ["text"] = "Synthetic Developer" },
                ["joueurs"] = new JsonObject { ["text"] = "1-2" },
                ["note"] = new JsonObject { ["text"] = "15" },
                ["synopsis"] = new JsonArray(new JsonObject { ["langue"] = "fr", ["text"] = "Un jeu synthétique." }, new JsonObject { ["langue"] = "en", ["text"] = game.Synopsis }),
                ["dates"] = new JsonArray(new JsonObject { ["region"] = "jp", ["text"] = "1990-11-21" }, new JsonObject { ["region"] = "us", ["text"] = "1991-08-23" }),
                ["genres"] = new JsonArray(
                    new JsonObject { ["id"] = "10", ["principale"] = "0", ["noms"] = new JsonArray(new JsonObject { ["langue"] = "en", ["text"] = "Other" }) },
                    new JsonObject { ["id"] = "11", ["principale"] = "1", ["noms"] = new JsonArray(new JsonObject { ["langue"] = "fr", ["text"] = "Course" }, new JsonObject { ["langue"] = "en", ["text"] = "Racing" }) }),
                ["medias"] = medias,
            };
            return jeu;
        }

        // A media file's bytes: a picture for the picture kinds, a PDF for manuel and an MP4 for the videos, each marked with its type, game and revision.
        private byte[] Bytes(string media, string game)
        {
            string type = media.Split('(')[0];
            byte[] label = Encoding.ASCII.GetBytes($"{media}|{game}|r{MediaRevision.GetValueOrDefault(type)}");
            byte[] bytes = new byte[400];
            Array.Copy(label, 0, bytes, 16, Math.Min(label.Length, 300));
            byte[] head = type switch
            {
                "manuel" => Encoding.ASCII.GetBytes("%PDF-1.4\n"),
                "video-normalized" or "video" => [0, 0, 0, 0x18, (byte)'f', (byte)'t', (byte)'y', (byte)'p', (byte)'m', (byte)'p', (byte)'4', (byte)'2'],
                _ => [0x89, (byte)'P', (byte)'N', (byte)'G'],
            };
            Array.Copy(head, bytes, head.Length);
            return bytes;
        }

        private HttpResponseMessage MediaAnswer(string url)
        {
            if (url.Contains("NOMEDIA")) return new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent("NOMEDIA") };
            string media = Param(url, "media");
            byte[] bytes = Bytes(media, Param(url, "jeuid"));
            if (Param(url, "sha1") is { Length: > 0 } theirs && theirs == Convert.ToHexStringLower(System.Security.Cryptography.SHA1.HashData(bytes)))
                return new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent("SHA1OK") };
            Interlocked.Add(ref _bytesSent, bytes.Length);
            var content = new ByteArrayContent(bytes);
            string type = media.Split('(')[0];
            content.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue(type == "manuel" ? "application/pdf" : type.StartsWith("video") ? "video/mp4" : "image/png");
            return new HttpResponseMessage(HttpStatusCode.OK) { Content = content };
        }

        private static JsonObject Header() => new() { ["APIversion"] = "2.0", ["success"] = "true", ["error"] = "" };

        private static HttpResponseMessage Json(JsonObject body) =>
            new(HttpStatusCode.OK) { Content = new StringContent(body.ToJsonString(), Encoding.UTF8, "application/json") };
    }

    // A clock the test moves: every delay advances it at once and is recorded.
    public sealed class FakeScrapeClock : IScrapeClock
    {
        private readonly object _gate = new();
        private DateTimeOffset _now;

        public FakeScrapeClock(DateTimeOffset start) => _now = start;

        public List<TimeSpan> Delays { get; } = new();

        public DateTimeOffset Now { get { lock (_gate) return _now; } set { lock (_gate) _now = value; } }

        public Task Delay(TimeSpan wait, CancellationToken stop)
        {
            stop.ThrowIfCancellationRequested();
            lock (_gate)
            {
                Delays.Add(wait);
                if (wait > TimeSpan.Zero) _now += wait;
            }
            return Task.CompletedTask;
        }
    }
}
