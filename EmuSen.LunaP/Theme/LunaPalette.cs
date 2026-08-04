using Avalonia.Media;
using Avalonia.Media.Immutable;

namespace EmuSen.LunaP.Theme
{
    // How hard something is working, in the three bands every dashboard shares.
    public enum LoadLevel
    {
        Nominal,
        Busy,
        Hot,
    }

    // The C# half of Theme/Palette.axaml, for controls built in code; LunaPaletteTests pins the two together - see EmuSen_LunaP.md §2.1.
    public static class LunaPalette
    {
        public static readonly ISolidColorBrush Surface = Brush("#1E1E1E");
        public static readonly ISolidColorBrush InputSurface = Brush("#252526");
        public static readonly ISolidColorBrush Void = Brush("#000000");

        public static readonly ISolidColorBrush Text = Brush("#D4D4D4");
        public static readonly ISolidColorBrush MeterText = Brush("#DCDCDC");
        public static readonly ISolidColorBrush Muted = Brush("#808080");
        public static readonly ISolidColorBrush SectionHeader = Brush("#9CDCFE");
        public static readonly ISolidColorBrush Warning = Brush("#D08770");

        public static readonly ISolidColorBrush Nominal = Brush("#32CD32");
        public static readonly ISolidColorBrush Busy = Brush("#FFD700");
        public static readonly ISolidColorBrush Hot = Brush("#FF4500");

        public static readonly FontFamily MonoFont = new("Consolas,Menlo,monospace");

        public const double HintFontSize = 11;
        public const double HeaderFontSize = 14;

        public const double BusyPercent = 60;
        public const double HotPercent = 85;

        // The one place that decides what "getting busy" means, so no two dashboards disagree - see EmuSen_LunaP.md §2.2.
        public static LoadLevel LevelFor(double percent) =>
            percent >= HotPercent ? LoadLevel.Hot : percent >= BusyPercent ? LoadLevel.Busy : LoadLevel.Nominal;

        // The static answer, for code that cannot take a themed one; MeterRow uses pseudo-classes instead so a theme reaches it.
        public static ISolidColorBrush ForLoad(double percent) => LevelFor(percent) switch
        {
            LoadLevel.Hot => Hot,
            LoadLevel.Busy => Busy,
            _ => Nominal,
        };

        private static ImmutableSolidColorBrush Brush(string hex) => new(Color.Parse(hex));
    }
}
