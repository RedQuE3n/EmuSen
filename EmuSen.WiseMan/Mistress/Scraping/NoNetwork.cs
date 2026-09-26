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

        // Every address a window tried while no test had installed a server of its own.
        public static readonly ConcurrentQueue<string> Refused = new();

        public static readonly Func<HttpClient> Factory = () => new HttpClient(new Refusing());
        public static readonly Func<DeveloperCredentials?> NoDeveloper = () => null;

        [ModuleInitializer]
        internal static void Refuse()
        {
            HttpFactory.SetValue(null, Factory);
            DeveloperSource.SetValue(null, NoDeveloper);
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
