using System.Collections.Generic;
using System.Linq;
using EmuSen.Galaxia.Models;

namespace EmuSen.Mistress.BigPicture
{
    // One entry of big picture's theme list: EmuSen's own library (no folder) or an ES-DE theme folder - see EmuSen_Settings_Reference.md §4.56.
    public sealed record BigPictureLook(string Name, InstalledTheme? Theme)
    {
        public bool BuiltIn => Theme is null;
    }

    // The theme list the sheet and Preferences both show, read from and written to LibraryStyle and BigPictureTheme alone.
    public static class BigPictureLooks
    {
        public const string BuiltInName = "EmuSen";

        public static BigPictureLook BuiltIn { get; } = new(BuiltInName, null);

        public static IReadOnlyList<BigPictureLook> All(AppSettings settings) =>
            [BuiltIn, .. ThemeDownloads.Installed(settings.BigPictureTheme).Select(t => new BigPictureLook(t.Name, t))];

        // EmuSen's own look is current when it was chosen, or when no theme folder is set.
        public static bool BuiltInCurrent(AppSettings settings) =>
            settings.LibraryStyle != AppSettings.LibraryStyleTheme || string.IsNullOrWhiteSpace(settings.BigPictureTheme);

        public static bool IsCurrent(AppSettings settings, BigPictureLook look) =>
            look.Theme is not { } theme
                ? BuiltInCurrent(settings)
                : !BuiltInCurrent(settings) && ThemeDownloads.SamePath(settings.BigPictureTheme!, theme.Directory);

        public static BigPictureLook Current(AppSettings settings) => All(settings).FirstOrDefault(l => IsCurrent(settings, l)) ?? BuiltIn;

        // EmuSen's own look keeps the theme folder, so choosing the theme again finds it and its options.
        public static void Choose(AppSettings settings, BigPictureLook look)
        {
            if (look.Theme is { } theme)
            {
                settings.BigPictureTheme = theme.Directory;
                settings.LibraryStyle = AppSettings.LibraryStyleTheme;
            }
            else settings.LibraryStyle = AppSettings.LibraryStyleMistress;
            settings.Save();
        }
    }
}
