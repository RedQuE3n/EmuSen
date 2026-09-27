using System.Globalization;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // Mistress's own words for what a theme's texts show but a game lacks, in one place for Pass 5's lookup to take over - see EmuSen_BigPicture.md §38.
    public static class SceneWords
    {
        public const string Unknown = "unknown";

        public const string Never = "never";

        public const string Yes = "yes";

        public const string No = "no";

        // "1 game", "12 games": a count and its noun, singular for one.
        public static string Count(int n, string one, string many) => n.ToString(CultureInfo.InvariantCulture) + " " + (n == 1 ? one : many);
    }
}
