using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.VisualTree;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Renders Mistress in the state ES-DE was captured in, for side-by-side comparison outside the repository: EMUSEN_ESDE_COMPARE=HH:MM runs it - see EmuSen_BigPicture.md §13.8.
    public class EsdeCompareTool
    {
        private readonly ITestOutputHelper _output;

        public EsdeCompareTool(ITestOutputHelper output) => _output = output;

        // ES-DE's system order read from its capture (GB, GBC, SNES, N64, NES), and its default game order, favourites first.
        public static readonly string[] EsdeOrder = ["gb", "gbc", "snes", "n64", "nes"];

        public static IReadOnlyList<SceneSystem> Systems(int w, int h, string variant)
        {
            IReadOnlyList<SceneSystem> loaded = SyntheticLibrary.Load(ArtBookNextFactAttribute.Folder, new ThemeChoices { ScreenWidth = w, ScreenHeight = h, Variant = variant, ColorScheme = "dark-screenshots" });
            return EsdeOrder.Select(n => loaded.Single(s => s.System.Name == n))
                .Select(s => s with { Games = s.Games.OrderByDescending(g => g.Favorite).ThenBy(g => g.Name, StringComparer.OrdinalIgnoreCase).ToList() }).ToList();
        }

        public static SceneData Data(IReadOnlyList<SceneSystem> systems, int w, int h, DateTime now) =>
            new(systems, new Size(w, h))
            {
                SystemIndex = Array.IndexOf(EsdeOrder, "snes"), GameIndex = 0, Media = new EsdeMediaFolder(SceneRenderTool.MediaRoot), Now = now,
                Status = new DeviceStatus(Bluetooth: true),
            };

        [ArtBookNextFact]
        public System.Threading.Tasks.Task Render_as_ES_DE_was_captured() => UiTest.Run(() =>
        {
            string? time = Environment.GetEnvironmentVariable("EMUSEN_ESDE_COMPARE");
            if (string.IsNullOrEmpty(time)) return;
            string variant = Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_VARIANT") is { Length: > 0 } v ? v : "gamelist-list-metadata-cover";
            DateTime now = DateTime.Today.Add(TimeSpan.Parse(time));
            string folder = Path.Combine(SceneRenderTool.Cache, "compare");
            Directory.CreateDirectory(folder);
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                SceneData data = Data(Systems(w, h, variant), w, h, now);
                foreach (string view in new[] { "system", "gamelist" })
                {
                    SceneBuilder scene = SceneBuilder.Build(data.System.Theme.View(view), data);
                    SceneAssets.Render(scene).SavePng(Path.Combine(folder, $"mistress-{view}-{variant}-{w}x{h}.png"));
                    foreach (var mode in new[] { Avalonia.Media.Imaging.BitmapInterpolationMode.LowQuality, Avalonia.Media.Imaging.BitmapInterpolationMode.MediumQuality, Avalonia.Media.Imaging.BitmapInterpolationMode.None })
                    {
                        SceneBuilder probe = SceneBuilder.Build(data.System.Theme.View(view), data);
                        SceneAssets.Render(probe, window => { foreach (var i in window.GetVisualDescendants().OfType<FittedImage>()) i.Interpolation = mode; })
                            .SavePng(Path.Combine(folder, $"probe-{mode}-{view}-{w}x{h}.png"));
                    }
                    foreach (SceneEntry e in scene.Entries.Where(e => e.Control is not null))
                    {
                        scene.Canvas.Measure(new Size(w, h));
                        scene.Canvas.Arrange(new Rect(0, 0, w, h));
                        _output.WriteLine($"{w}x{h} {view} {e.Element} {e.Control!.Bounds}");
                    }
                }
            }
        });
    }
}
