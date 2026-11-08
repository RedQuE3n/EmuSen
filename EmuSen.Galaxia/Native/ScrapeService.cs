using System;
using System.Text;

namespace EmuSen.Galaxia.Native
{
    // emusen_scrape_service.h's table, version 2.0, as emusen_scraping_service gives it: scraping's one surface - see EmuSen_LibraryTools.md §8.
    public unsafe struct ScrapeService
    {
        public uint Size;
        public delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> Get, Change;
        public delegate* unmanaged[Cdecl]<byte*, nuint, long> Start;
        public delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> Progress;
        public delegate* unmanaged[Cdecl]<ulong, uint, long> Wait;
        public delegate* unmanaged[Cdecl]<int, int> Pause;
        public delegate* unmanaged[Cdecl]<int> Cancel;
        public delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> ThemeKinds;
        public delegate* unmanaged[Cdecl]<byte*, nuint, long> LastError;

        private static readonly Lazy<nint> Found = new(() =>
        {
            nint export = PlatformLibrary.Export("emusen_scraping_service");
            if (export == 0) return 0;
            var table = ((delegate* unmanaged[Cdecl]<ScrapeService*>)export)();
            return table != null && table->Size >= sizeof(ScrapeService) ? (nint)table : 0;
        });

        // The platform library's table, or null where it has none or one shorter than 2.0's.
        public static ScrapeService* Table => (ScrapeService*)Found.Value;

        // The words of this thread's last failing call through the table, redacted by the scraper.
        public static string Words(ScrapeService* table)
        {
            long n = table->LastError(null, 0);
            var words = new byte[Math.Max(0, n)];
            fixed (byte* w = words) table->LastError(w, (nuint)words.Length);
            return Encoding.UTF8.GetString(words);
        }
    }
}
