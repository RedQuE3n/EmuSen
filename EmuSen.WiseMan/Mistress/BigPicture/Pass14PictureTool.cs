using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class Pass14PngFactAttribute : FactAttribute
    {
        public Pass14PngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes pass 14's pictures to ~/.cache/emusen/bigpicture/png/pass14/ from the probe and downloaded themes there; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §36";
        }
    }

    // Mistress in the states ES-DE 3.4.1 was captured in for pass 14, from themes and media kept outside the repository - see EmuSen_BigPicture.md §36.
    public class Pass14PictureTool
    {
        private static readonly string Home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        public static readonly string PngFolder = Path.Combine(Home, ".cache", "emusen", "bigpicture", "png", "pass14");
        private static readonly string Probe = Path.Combine(Home, ".cache", "emusen", "probe", "pass14", "probe-themes");
        private static readonly string Real = Path.Combine(Home, ".cache", "emusen", "bigpicture", "pass14-themes");
        private static readonly string Media = Path.Combine(Home, ".cache", "emusen", "bigpicture", "esde", "home-pass14", "ES-DE", "downloaded_media");

        private readonly ITestOutputHelper _out;

        public Pass14PictureTool(ITestOutputHelper output) => _out = output;

        // ES-DE's system order on the probe library, by full name, with the names it gives them.
        private static readonly (string Name, string FullName)[] Order =
        [
            ("atari2600", "Atari 2600"), ("pcengine", "NEC PC Engine"), ("n64", "Nintendo 64"), ("nes", "Nintendo Entertainment System"), ("gb", "Nintendo Game Boy"),
            ("gba", "Nintendo Game Boy Advance"), ("gbc", "Nintendo Game Boy Color"), ("snes", "Nintendo SNES (Super Nintendo)"), ("gamegear", "Sega Game Gear"),
            ("genesis", "Sega Genesis"), ("mastersystem", "Sega Master System"), ("ngp", "SNK Neo Geo Pocket"),
        ];

        private static readonly DateTime Played = new(2026, 9, 20, 10, 0, 0);

        // The probe library's games, as the ES-DE runs' gamelists write them; the hidden genesis game is left out, as a list without hidden games leaves it.
        private static IReadOnlyList<SceneGame> Games(string system)
        {
            (string Extension, IReadOnlyList<SceneGame>? Own) g = system switch
            {
                "nes" => (".nes", null), "snes" => (".sfc", null), "n64" => (".z64", null), "gb" => (".gb", null), "gbc" => (".gbc", null),
                "gba" => (".zip", [new("Aurora Drift", "Aurora Drift (Synthetic).zip"), new("Brass Lantern", "Brass Lantern (Synthetic).zip") { PlayCount = 3, LastPlayed = Played }, new("Cobalt Harbor", "Cobalt Harbor (Synthetic).zip")]),
                "genesis" => (".zip", [new("Aurora Drift", "Aurora Drift (Synthetic).zip") { PlayCount = 9, LastPlayed = Played.AddDays(3), NotCounted = true },
                    new("Brass Lantern", "Brass Lantern (Synthetic).zip") { PlayCount = 2, LastPlayed = Played.AddDays(1) }, new("Dune Relay", "Dune Relay (Synthetic).zip") { PlayCount = 1, LastPlayed = Played.AddDays(-1) }]),
                _ => (".zip", [new("Aurora Drift", "Aurora Drift (Synthetic).zip")]),
            };
            if (g.Own is { } own) return own;
            var theme = SyntheticLibrary.Systems.Single(s => s.System.Name == system).System;
            return SyntheticLibrary.Games(theme, g.Extension).OrderByDescending(x => x.Favorite).ThenBy(x => x.Name, StringComparer.OrdinalIgnoreCase).ToList();
        }

        private void Render(string label, string themeDir, ThemeChoices choices, string system, string view = "system", int w = 1280, int h = 800, int shuffle = 0)
        {
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(themeDir);
            choices = choices with { ScreenWidth = w, ScreenHeight = h };
            var systems = Order.Select(s => new ThemeSystem(s.Name, s.FullName, s.Name)).Select(t => new SceneSystem(t, ThemeLoader.Load(caps, t, choices, new MediaPresence(new HashSet<string> { "cover", "screenshot", "marquee", "fanart", "titlescreen", "miximage" })), Games(t.Name))).ToList();
            foreach (ThemeDiagnostic d in systems.SelectMany(s => s.Theme.Errors).Distinct()) _out.WriteLine($"{label}: {d.Message}");
            var data = new SceneData(systems, new Size(w, h))
            {
                SystemIndex = systems.FindIndex(s => s.System.Name == system), GameIndex = 0, Media = new EsdeMediaFolder(Media), Shuffle = shuffle,
                Now = new DateTime(2026, 9, 27, 5, 0, 0), ShowClock = false, Status = new EmuSen.LunaP.Controls.DeviceStatus(Bluetooth: true),
            };
            SceneBuilder scene = SceneBuilder.Build(data.System.Theme.View(view), data);
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, $"mistress-{label}-{w}x{h}.png");
            SceneAssets.Render(scene).SavePng(path);
            _out.WriteLine(path);
        }

        [Pass14PngFact]
        public System.Threading.Tasks.Task Render_the_probe_themes_as_ES_DE_was_captured() => UiTest.Run(() =>
        {
            string wheel = Path.Combine(Probe, "p14-wheel-es-de"), sel = Path.Combine(Probe, "p14-sel-es-de");
            foreach (string variant in ThemeCapabilitiesReader.Read(wheel).Variants.Select(v => v.Name))
                Render("w-" + variant, wheel, new ThemeChoices { Variant = variant }, "snes");
            foreach ((string label, string variant, string system) in new[] { ("s-multi", "multi", "snes"), ("s-multi-gba", "multi", "gba"), ("s-multi-genesis", "multi", "genesis"), ("s-none", "none", "snes"), ("s-single", "single", "snes"), ("s-order1", "order1", "snes") })
                Render(label, sel, new ThemeChoices { Variant = variant }, system);
        });

        [Pass14PngFact]
        public System.Threading.Tasks.Task Render_the_downloaded_themes_as_ES_DE_was_captured() => UiTest.Run(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                Render("r-codywheel", Path.Combine(Real, "codywheel-es-de"), new ThemeChoices(), "mastersystem", w: w, h: h);
                Render("r-aura-sys", Path.Combine(Real, "aura-es-de"), new ThemeChoices { Variant = "fullscreen-carousel-boxart" }, "mastersystem", w: w, h: h);
                Render("r-aura-gl", Path.Combine(Real, "aura-es-de"), new ThemeChoices { Variant = "fullscreen-carousel-boxart" }, "snes", "gamelist", w, h);
                Render("r-mania", Path.Combine(Real, "mania-menu-es-de"), new ThemeChoices { Variant = "backgroundArtScreenshot" }, "mastersystem", w: w, h: h);
            }

            string wheel = Path.Combine(Probe, "p14-wheel-es-de");
            foreach (string variant in new[] { "vw", "hw", "hRefl", "vwAxis" }) Render("w-" + variant, wheel, new ThemeChoices { Variant = variant }, "snes", w: 1920, h: 1200);
            Render("s-multi", Path.Combine(Probe, "p14-sel-es-de"), new ThemeChoices { Variant = "multi" }, "snes", w: 1920, h: 1200);
        });
    }
}
