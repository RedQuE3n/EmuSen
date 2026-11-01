using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Client.Library;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Client.Scrape
{
    // The host's scraper lent to every shell through emusen_scraping_lend, so that a process holds one scraper and one quota - see EmuSen_LibraryTools.md §8.
    public static unsafe class ScrapingLend
    {
        private struct LendStruct
        {
            public uint Size;
            public nint Scraper;
            public nint Library;
            public byte* Systems;
            public nuint SystemsLen;
            public delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long> ThemeKinds;
            public nint Ctx;
        }

        private static readonly object Gate = new();
        private static MediaService? _service;
        private static GameLibrary? _library;
        private static volatile Func<string, IReadOnlyList<string>?>? _themeKinds;

        private static delegate* unmanaged[Cdecl]<LendStruct*, int> Export()
        {
            nint f = PlatformLibrary.Export("emusen_scraping_lend");
            return (delegate* unmanaged[Cdecl]<LendStruct*, int>)f;
        }

        // Whether this platform library takes a lend at all.
        public static bool Available => PlatformLibrary.Export("emusen_scraping_lend") != 0;

        // Lends the scraper, its games.db library, the systems the build opens and the active theme's kinds per ES-DE system; a later lend replaces it.
        public static void Lend(MediaService service, GameLibrary library, IEnumerable<string> systems, Func<string, IReadOnlyList<string>?>? themeKinds)
        {
            var lend = Export();
            if (lend == null) return;
            byte[] s = Encoding.UTF8.GetBytes(JsonSerializer.Serialize(systems.Distinct().ToArray()));
            lock (Gate)
            {
                _themeKinds = themeKinds;
                fixed (byte* sp = s)
                {
                    var l = new LendStruct
                    {
                        Size = (uint)sizeof(LendStruct), Scraper = service.Handle, Library = library.Handle, Systems = sp, SystemsLen = (nuint)s.Length,
                        ThemeKinds = themeKinds is null ? null : &ThemeKinds,
                    };
                    int status = lend(&l);
                    if (status != 0) throw new MediaServiceException(status, "The scraper could not be lent to the shell.");
                }
                (_service, _library) = (service, library);
            }
        }

        // Says the host scrapes with no scraper open now: the shells refuse rather than open one of their own.
        public static void Withdraw()
        {
            var lend = Export();
            if (lend == null) return;
            lock (Gate)
            {
                lend(null);
                (_service, _library, _themeKinds) = (null, null, null);
            }
        }

        // Withdraws the lend when it holds this handle, before the handle closes.
        internal static void Release(object handle)
        {
            lock (Gate)
                if (ReferenceEquals(_service, handle) || ReferenceEquals(_library, handle)) Withdraw();
        }

        [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
        private static long ThemeKinds(nint ctx, byte* system, nuint len, byte* output, nuint outLen)
        {
            try
            {
                IReadOnlyList<string>? kinds = _themeKinds?.Invoke(Encoding.UTF8.GetString(system, (int)len));
                if (kinds is null) return -1;
                byte[] answer = Encoding.UTF8.GetBytes(JsonSerializer.Serialize(kinds));
                if (output != null && (nuint)answer.Length <= outLen) answer.CopyTo(new Span<byte>(output, answer.Length));
                return answer.Length;
            }
            catch (Exception) { return -1; }
        }
    }
}
