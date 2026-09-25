using System;
using System.IO;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.WiseMan.Fixtures
{
    // Writes a small ES-DE theme written for the tests into a temporary folder; nothing is copied from any real theme - see EmuSen_BigPicture.md §12.5.
    public sealed class SyntheticTheme : IDisposable
    {
        public SyntheticTheme()
        {
            Root = Path.Combine(Path.GetTempPath(), "EmuSenSyntheticTheme", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(Root);
        }

        public string Root { get; }

        public static ThemeSystem Snes { get; } = new("snes", "Super Nintendo", "snes");

        public SyntheticTheme Capabilities(string body) => File("capabilities.xml", $"<themeCapabilities>\n{body}\n</themeCapabilities>");

        public SyntheticTheme Theme(string body, string file = "theme.xml") => File(file, $"<theme>\n{body}\n</theme>");

        public SyntheticTheme File(string relative, string text)
        {
            string path = Path.Combine(Root, relative);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            System.IO.File.WriteAllText(path, text);
            return this;
        }

        // An asset of the given name, so path properties resolve to a file that exists.
        public SyntheticTheme Asset(string relative) => File(relative, "asset");

        public string PathOf(string relative) => Path.GetFullPath(Path.Combine(Root, relative));

        public ResolvedTheme Load(ThemeChoices? choices = null, ThemeSystem? system = null, MediaPresence? media = null) =>
            ThemeLoader.Load(Root, system ?? Snes, choices ?? new ThemeChoices(), media);

        public void Dispose()
        {
            try { if (Directory.Exists(Root)) Directory.Delete(Root, recursive: true); } catch { }
        }
    }
}
