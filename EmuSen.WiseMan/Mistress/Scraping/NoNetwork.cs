using System;
using System.Collections.Concurrent;
using System.Net.Http;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // Every window the suite opens starts with no network and no developer credentials; a test that wants either installs its own fake - see EmuSen_BigPicture.md §17.
    internal static class NoNetwork
    {
        private const BindingFlags Hidden = BindingFlags.NonPublic | BindingFlags.Static;

        public static readonly FieldInfo HttpFactory = typeof(MainWindow).GetField("HttpFactory", Hidden)!;
        public static readonly FieldInfo DeveloperSource = typeof(MainWindow).GetField("DeveloperSource", Hidden)!;
        public static readonly FieldInfo ScrapeClock = typeof(MainWindow).GetField("ScrapeClock", Hidden)!;
        public static readonly FieldInfo ScrapeTest = typeof(MainWindow).GetField("ScrapeTest", Hidden)!;

        private static ReplayServer? _replay;

        // A test's own fake server for both halves: the C#'s HttpClient over it, and the scraper in Rust sent to it over HTTPS at the real addresses, on the test's clock when it has one.
        public static void Serve(HttpMessageHandler handler, IScrapeClock? clock = null)
        {
            HttpFactory.SetValue(null, (Func<HttpClient>)(() => new HttpClient(handler)));
            _replay?.Dispose();
            var replay = _replay = new ReplayServer(handler);
            ScrapeTest.SetValue(null, (Func<string?>)(() =>
            {
                long? ticks = clock is null or SystemScrapeClock ? null : clock.Now.UtcTicks - DateTimeOffset.UnixEpoch.UtcTicks;
                return $"{{\"connect\":\"{replay.Address}\",\"roots\":{System.Text.Json.JsonSerializer.Serialize(ReplayServer.RootPem)}" + (ticks is long t ? $",\"clock\":{t}" : "") + "}";
            }));
        }

        // Every address a window tried while no test had installed a server of its own.
        public static readonly ConcurrentQueue<string> Refused = new();

        public static readonly Func<HttpClient> Factory = () => new HttpClient(new Refusing());
        public static readonly Func<DeveloperCredentials?> NoDeveloper = () => null;

        public static readonly Func<string?> Offline = () => "{\"offline\":true}";

        [ModuleInitializer]
        internal static void Refuse()
        {
            HttpFactory.SetValue(null, Factory);
            DeveloperSource.SetValue(null, NoDeveloper);
            ScrapeTest.SetValue(null, Offline);
            _replay?.Dispose();
            _replay = null;
        }

        private sealed class Refusing : HttpMessageHandler
        {
            protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
            {
                Refused.Enqueue(ScrapeRedactor.Redact(request.RequestUri?.AbsoluteUri));
                throw new HttpRequestException("The test harness allows no network.");
            }
        }
    }
}
