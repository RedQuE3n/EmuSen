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
        public IEnumerable<string> MediaAsked => Asked.Where(u => u.Contains("/mediaJeu.php"));
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
                if (url.Contains("/ssuserInfos.php")) return Json(new JsonObject { ["header"] = Header(), ["response"] = new JsonObject { ["ssuser"] = User?.DeepClone() } });
                return new HttpResponseMessage(HttpStatusCode.BadRequest) { Content = new StringContent("Erreur") };
            }
            finally
            {
                Interlocked.Decrement(ref _inFlight);
            }
        }

        private HttpResponseMessage JeuInfosAnswer(string url)
        {
            if (ForcedStatus != 0) return new HttpResponseMessage((HttpStatusCode)ForcedStatus) { Content = new StringContent("Erreur : forced by the test") };
            string md5 = Param(url, "md5");
            FakeGame? game;
            lock (Games) game = Games.FirstOrDefault(g => g.Md5s.Contains(md5, StringComparer.OrdinalIgnoreCase));
            if (game is null) return new HttpResponseMessage(HttpStatusCode.NotFound) { Content = new StringContent("Erreur : Rom/Iso/Dossier non trouvée !  ") };

            string credentials = $"devid={DevId}&devpassword={DevPassword}&softname={SoftName}&ssid=&sspassword=";
            var medias = new JsonArray(game.Media.Select(m => (JsonNode)new JsonObject
            {
                ["type"] = m.Type, ["parent"] = "jeu", ["region"] = m.Region, ["format"] = m.Format, ["crc"] = "00000000",
                ["url"] = $"https://neoclone.screenscraper.fr/api2/mediaJeu.php?{credentials}&systemeid={game.SystemId}&jeuid={game.Id}&media={m.Type}({m.Region})",
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
            var response = new JsonObject { ["serveurs"] = new JsonObject(), ["jeu"] = jeu };
            if (User is not null) response["ssuser"] = User.DeepClone();
            return Json(new JsonObject { ["header"] = Header(), ["response"] = response });
        }

        private static HttpResponseMessage MediaAnswer(string url)
        {
            if (url.Contains("NOMEDIA")) return new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent("NOMEDIA") };
            string media = Param(url, "media");
            byte[] png = new byte[400];
            byte[] label = Encoding.ASCII.GetBytes(media);
            Array.Copy(label, 0, png, 8, Math.Min(label.Length, 300));
            png[0] = 0x89;
            var content = new ByteArrayContent(png);
            content.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue("image/png");
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
