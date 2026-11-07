using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;

namespace EmuSen.WiseMan.Fixtures
{
    // A filter the platform library's catalog offers, by the name a frontend stores.
    internal static class Catalog
    {
        public static ScreenFilter Named(string name) => ScreenFilters.All.First(f => string.Equals(f.Name, name, StringComparison.Ordinal)).Filter!;
    }

    // crt-lottes as the platform library's catalog gives it - see EmuSen_Serenity.md §3.4.
    public static class CrtFilters
    {
        public static ScreenFilter Lottes => Catalog.Named("CRT (Lottes)");

        public static IReadOnlyList<SlangParameter> LottesParameters => Lottes.Parameters!;
    }

    // The handheld LCD filters as the platform library's catalog gives them - see EmuSen_Serenity.md §3.5.
    public static class HandheldFilters
    {
        public static ScreenFilter DmgLcd => Catalog.Named("Game Boy LCD");
        public static ScreenFilter PocketLcd => Catalog.Named("Game Boy Pocket LCD");
        public static ScreenFilter LightLcd => Catalog.Named("Game Boy Light LCD");
        public static ScreenFilter GbcLcd => Catalog.Named("Game Boy Color LCD");
        public static ScreenFilter AgbLcd => Catalog.Named("Game Boy Advance LCD");
        public static ScreenFilter Ags101Lcd => Catalog.Named("Game Boy Advance SP (AGS-101) LCD");
    }

    // A television's 4:3, the shape the CRT filter's tube is drawn in - see EmuSen_Serenity.md §2.9.
    public static class DisplayShape
    {
        public const double Television = 4.0 / 3.0;
    }
}
