using System.Collections.Generic;
using EmuSen.Serenity.Slang;

namespace EmuSen.Serenity.Shaders
{
    // The CRT computed from its display chain, as the platform library's catalog gives it; its passes are the library's - see EmuSen_CRT.md §5 and §11 and EmuSen_Serenity.md §3.13.
    public static class CrtFilter
    {
        public const string Name = "CRT";

        public static ScreenFilter Filter => FilterCatalog.Named(Name);

        public static IReadOnlyList<SlangParameter> Parameters => Filter.Parameters!;

        // The parameters that decide which passes exist or what is written into them as constants.
        public static IReadOnlyList<string> Structural => Filter.Structural!;

        // Each console's line constants on both standards; "" is a standard line - see EmuSen_CRT.md §3.2 and §23.
        public static IReadOnlyDictionary<string, IReadOnlyDictionary<string, float>> ConsoleTiming => Filter.ConsoleDefaults!;

        // The consoles that drove a television, which every filter of a tube is offered for - see EmuSen_CRT.md §31.
        public static IReadOnlyList<string> TelevisionConsoles => ConsoleScreens.Televisions;
    }
}
