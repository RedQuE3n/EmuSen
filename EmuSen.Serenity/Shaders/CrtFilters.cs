using System.Collections.Generic;
using EmuSen.Serenity.Slang;

namespace EmuSen.Serenity.Shaders
{
    // crt-lottes, Timothy Lottes' public-domain shader at its default settings, as the library's catalog gives it - see EmuSen_Serenity.md §3.4.
    public static class CrtFilters
    {
        public static ScreenFilter Lottes => FilterCatalog.Named("CRT (Lottes)");

        public static IReadOnlyList<SlangParameter> LottesParameters => Lottes.Parameters!;
    }
}
