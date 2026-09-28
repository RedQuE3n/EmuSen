using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The carousel's wheels, offsets, clipping, growth and reflections against ES-DE 3.4.1's captures of the same probe theme, as numbers read from them - see EmuSen_BigPicture.md §36.3.
    public class WheelCarouselTests
    {
        private const int W = 1280, H = 800;

        // ES-DE's order of the probe library's twelve systems, by full name, and each logo's flat colour.
        private static readonly (string Name, byte R, byte G, byte B)[] Systems =
        [
            ("atari2600", 120, 120, 120), ("pcengine", 220, 100, 150), ("n64", 40, 80, 230), ("nes", 230, 40, 40), ("gb", 230, 200, 30), ("gba", 30, 200, 200),
            ("gbc", 200, 40, 200), ("snes", 40, 200, 60), ("gamegear", 120, 200, 20), ("genesis", 240, 130, 20), ("mastersystem", 130, 60, 220), ("ngp", 150, 90, 40),
        ];

        // Each probe variant's carousel, as the probe theme wrote it for ES-DE.
        private static readonly Dictionary<string, string> Variants = new()
        {
            ["vw"] = "<type>verticalWheel</type><pos>0.3 0</pos><size>0.4 1</size><itemSize>0.2 0.1</itemSize>",
            ["vwRot"] = "<type>verticalWheel</type><pos>0.3 0</pos><size>0.4 1</size><itemSize>0.2 0.1</itemSize><itemRotation>-10</itemRotation><itemRotationOrigin>3 0.5</itemRotationOrigin>",
            ["vwAxis"] = "<type>verticalWheel</type><pos>0.3 0</pos><size>0.4 1</size><itemSize>0.2 0.1</itemSize><itemAxisHorizontal>true</itemAxisHorizontal>",
            ["vwLeft"] = "<type>verticalWheel</type><pos>0.3 0</pos><size>0.4 1</size><itemSize>0.2 0.1</itemSize><wheelHorizontalAlignment>left</wheelHorizontalAlignment>",
            ["vwItemRight"] = "<type>verticalWheel</type><pos>0.3 0</pos><size>0.4 1</size><itemSize>0.25 0.1</itemSize><itemHorizontalAlignment>right</itemHorizontalAlignment><itemRotation>15</itemRotation>",
            ["vwScaleLeft"] = "<type>verticalWheel</type><pos>0.3 0</pos><size>0.4 1</size><itemSize>0.25 0.1</itemSize><itemScale>1.5</itemScale><itemHorizontalAlignment>left</itemHorizontalAlignment><itemRotation>15</itemRotation><color>202020FF</color>",
            ["vwOriginY"] = "<type>verticalWheel</type><pos>0.3 0</pos><size>0.4 1</size><itemSize>0.2 0.1</itemSize><itemRotationOrigin>-3 2</itemRotationOrigin>",
            ["vwClip"] = "<type>verticalWheel</type><pos>0.3 0.25</pos><size>0.4 0.5</size><itemSize>0.2 0.1</itemSize><color>202020FF</color>",
            ["vwOffset"] = "<type>verticalWheel</type><pos>0.3 0</pos><size>0.4 1</size><itemSize>0.2 0.1</itemSize><horizontalOffset>0.2</horizontalOffset><verticalOffset>0.1</verticalOffset>",
            ["hw"] = "<type>horizontalWheel</type><pos>0 0.3</pos><size>1 0.4</size><itemSize>0.1 0.2</itemSize>",
            ["hwOne"] = "<type>horizontalWheel</type><pos>0 0</pos><size>1 1</size><itemSize>0.1 0.2</itemSize><itemsBeforeCenter>0</itemsBeforeCenter><itemsAfterCenter>1</itemsAfterCenter><itemRotation>30</itemRotation>",
            ["hwOneY"] = "<type>horizontalWheel</type><pos>0 0</pos><size>1 1</size><itemSize>0.1 0.2</itemSize><itemsBeforeCenter>0</itemsBeforeCenter><itemsAfterCenter>1</itemsAfterCenter><itemRotation>30</itemRotation><itemRotationOrigin>-1 1.5</itemRotationOrigin>",
            ["hwOneAxis"] = "<type>horizontalWheel</type><pos>0 0</pos><size>1 1</size><itemSize>0.1 0.2</itemSize><itemsBeforeCenter>0</itemsBeforeCenter><itemsAfterCenter>1</itemsAfterCenter><itemRotation>30</itemRotation><itemAxisHorizontal>true</itemAxisHorizontal>",
            ["vwOneAxisR"] = "<type>verticalWheel</type><pos>0 0</pos><size>1 1</size><itemSize>0.2 0.1</itemSize><itemsBeforeCenter>0</itemsBeforeCenter><itemsAfterCenter>1</itemsAfterCenter><itemRotation>-30</itemRotation><itemRotationOrigin>2 0.5</itemRotationOrigin><itemAxisHorizontal>true</itemAxisHorizontal>",
            ["hOff"] = "<type>horizontal</type><pos>0 0.3</pos><size>1 0.3</size><itemSize>0.15 0.2</itemSize><maxItemCount>5</maxItemCount><horizontalOffset>0.25</horizontalOffset><verticalOffset>0.25</verticalOffset><color>202020FF</color>",
            ["hClip"] = "<type>horizontal</type><pos>0 0.45</pos><size>1 0.1</size><itemSize>0.15 0.2</itemSize><maxItemCount>5</maxItemCount><color>202020FF</color>",
            ["hReflB"] = "<type>horizontal</type><pos>0 0.2</pos><size>1 0.5</size><itemSize>0.15 0.2</itemSize><maxItemCount>5</maxItemCount><reflections>true</reflections><reflectionsOpacity>1</reflectionsOpacity><reflectionsFalloff>2</reflectionsFalloff><verticalOffset>-0.2</verticalOffset><color>202020FF</color>",
            ["hScaleTop"] = "<type>horizontal</type><pos>0 0.3</pos><size>1 0.4</size><itemSize>0.15 0.2</itemSize><maxItemCount>5</maxItemCount><itemScale>1.5</itemScale><itemVerticalAlignment>top</itemVerticalAlignment><color>202020FF</color>",
            ["mH"] = "<type>horizontal</type><pos>0 0.3</pos><size>1 0.4</size><itemSize>0.12 0.2</itemSize><maxItemCount>5</maxItemCount><selectedItemMargins>0.1 0.05</selectedItemMargins><color>202020FF</color>",
            ["mHneg"] = "<type>horizontal</type><pos>0 0.3</pos><size>1 0.4</size><itemSize>0.12 0.2</itemSize><maxItemCount>5</maxItemCount><selectedItemMargins>-0.05 -0.02</selectedItemMargins><color>202020FF</color>",
            ["mHscale"] = "<type>horizontal</type><pos>0 0.3</pos><size>1 0.4</size><itemSize>0.12 0.2</itemSize><maxItemCount>5</maxItemCount><selectedItemMargins>0.04 0.04</selectedItemMargins><itemScale>1.3</itemScale><color>202020FF</color>",
            ["mHhalf"] = "<type>horizontal</type><pos>0.25 0.3</pos><size>0.5 0.4</size><itemSize>0.08 0.2</itemSize><maxItemCount>5</maxItemCount><selectedItemMargins>0.1 0.1</selectedItemMargins><color>202020FF</color>",
            ["mV"] = "<type>vertical</type><pos>0.3 0</pos><size>0.4 1</size><itemSize>0.2 0.08</itemSize><maxItemCount>7</maxItemCount><selectedItemMargins>0.1 0.05</selectedItemMargins><color>202020FF</color>",
            ["mVhalf"] = "<type>vertical</type><pos>0.3 0.25</pos><size>0.4 0.5</size><itemSize>0.2 0.06</itemSize><maxItemCount>5</maxItemCount><selectedItemMargins>0.05 0.05</selectedItemMargins><color>202020FF</color>",
            ["hRefl"] = "<type>horizontal</type><pos>0 0.2</pos><size>1 0.5</size><itemSize>0.15 0.2</itemSize><maxItemCount>5</maxItemCount><reflections>true</reflections><color>202020FF</color>",
            ["hReflScale"] = "<type>horizontal</type><pos>0 0.2</pos><size>1 0.5</size><itemSize>0.15 0.2</itemSize><maxItemCount>5</maxItemCount><reflections>true</reflections><itemScale>1.5</itemScale><unfocusedItemOpacity>0.5</unfocusedItemOpacity><color>202020FF</color>",
        };

        // Each logo's centre of colour in ES-DE 3.4.1's capture at 1280 by 800, measured 2026-09-27 (§36.3); logos ES-DE's own help bar covers are left out, since Mistress draws no help bar a theme does not ask for.
        private static readonly Dictionary<string, (string System, double X, double Y)[]> Esde = new()
        {
            ["vw"] = [("gb", 573.5, 57.4), ("gba", 611.7, 168.0), ("gbc", 634.5, 283.3), ("snes", 642.0, 400.6), ("gamegear", 634.2, 517.9), ("genesis", 611.2, 633.1), ("mastersystem", 572.7, 743.6)],
            ["vwRot"] = [("nes", 815.5, 23.4), ("gb", 726.9, 81.7), ("gba", 680.1, 182.5), ("gbc", 651.5, 289.9), ("snes", 642.0, 400.6), ("gamegear", 651.9, 511.3), ("genesis", 680.8, 618.5), ("mastersystem", 728.0, 719.2), ("ngp", 815.5, 775.6)],
            ["vwAxis"] = [("nes", 536.5, 27.5), ("gb", 584.0, 106.6), ("gba", 616.1, 201.6), ("gbc", 635.1, 300.6), ("snes", 642.0, 400.6), ("gamegear", 635.2, 500.6), ("genesis", 616.0, 599.6), ("mastersystem", 584.0, 694.6)],
            ["vwLeft"] = [("gb", 450.1, 56.0), ("gba", 483.7, 168.0), ("gbc", 506.5, 283.3), ("snes", 514.0, 400.6), ("gamegear", 506.2, 517.9), ("genesis", 483.2, 633.1)],
            ["vwItemRight"] = [("gbc", 681.4, 89.3), ("snes", 722.0, 400.6), ("gamegear", 680.8, 711.8)],
            ["vwScaleLeft"] = [("gbc", 452.5, 129.1), ("snes", 523.3, 401.1), ("gamegear", 454.0, 671.0)],
            ["vwOriginY"] = [("nes", 426.6, 13.7), ("gb", 528.1, 65.8), ("gba", 580.7, 172.1), ("gbc", 618.8, 284.3), ("snes", 642.0, 400.6), ("gamegear", 649.9, 518.9), ("genesis", 642.2, 637.2), ("mastersystem", 617.2, 751.1)],
            ["vwClip"] = [("gbc", 634.5, 283.3), ("snes", 642.0, 400.6), ("gamegear", 634.2, 517.9)],
            ["vwOffset"] = [("nes", 618.4, 40.6), ("gb", 676.5, 136.6), ("gba", 714.1, 248.0), ("gbc", 736.9, 363.3), ("snes", 744.1, 480.6), ("gamegear", 736.6, 597.9), ("genesis", 713.5, 713.1), ("mastersystem", 649.5, 787.6)],
            ["hw"] = [("gb", 440.2, 446.6), ("gba", 494.3, 424.0), ("gbc", 550.8, 408.7), ("snes", 641.5, 400.4), ("gamegear", 731.3, 407.5), ("genesis", 787.9, 423.3), ("mastersystem", 842.0, 446.3)],
            ["hwOne"] = [("snes", 641.5, 400.4), ("gamegear", 864.8, 461.3)],
            ["hwOneY"] = [("snes", 641.5, 400.4), ("gamegear", 758.3, 347.0)],
            ["hwOneAxis"] = [("snes", 641.5, 400.4), ("gamegear", 833.7, 451.4)],
            ["vwOneAxisR"] = [("snes", 642.0, 400.6), ("gamegear", 711.0, 656.6)],
            ["hOff"] = [("gb", 194.4, 420.7), ("gba", 450.4, 420.7), ("gbc", 706.4, 420.7), ("snes", 962.4, 420.7), ("gamegear", 1202.4, 421.0)],
            ["hClip"] = [("gba", 129.9, 400.4), ("gbc", 385.9, 400.4), ("snes", 641.9, 400.4), ("gamegear", 897.9, 400.4), ("genesis", 1153.9, 400.4)],
            ["hReflB"] = [("gba", 129.6, 205.9), ("gbc", 385.6, 205.9), ("snes", 641.6, 205.9), ("gamegear", 897.6, 205.9), ("genesis", 1153.6, 205.9)],
            ["mH"] = [("gbc", 258.1, 400.1), ("snes", 642.1, 400.1), ("gamegear", 962.1, 400.1)],
            ["mHneg"] = [("gba", 194.1, 400.1), ("gbc", 450.1, 400.1), ("snes", 642.1, 400.1), ("gamegear", 873.0, 400.0), ("genesis", 1129.0, 400.0)],
            ["mHscale"] = [("gba", 79.0, 400.5), ("gbc", 335.0, 400.5), ("snes", 642.6, 400.8), ("gamegear", 949.0, 400.5), ("genesis", 1205.0, 400.5)],
            ["mHhalf"] = [("gbc", 385.2, 399.7), ("snes", 641.2, 399.7), ("gamegear", 897.2, 399.7)],
            ["mV"] = [("gba", 641.7, 91.4), ("gbc", 641.5, 206.4), ("snes", 641.5, 400.4), ("gamegear", 641.5, 554.4), ("genesis", 641.5, 669.4)],
            ["mVhalf"] = [("gbc", 641.1, 280.2), ("snes", 641.1, 400.2), ("gamegear", 641.1, 520.2)],
            ["hScaleTop"] = [("gba", 130.4, 288.7), ("gbc", 386.4, 288.7), ("snes", 644.0, 313.4), ("gamegear", 898.4, 288.7), ("genesis", 1154.4, 288.7)],
        };

        private static void Logo(string path, (string Name, byte R, byte G, byte B) s)
        {
            using var target = new RenderTargetBitmap(new PixelSize(200, 100));
            using (DrawingContext dc = target.CreateDrawingContext())
            {
                dc.FillRectangle(new SolidColorBrush(Color.FromRgb(s.R, s.G, s.B)), new Rect(0, 0, 200, 100));
                dc.FillRectangle(Brushes.White, new Rect(0, 0, 26, 26));
            }

            System.IO.Directory.CreateDirectory(System.IO.Path.GetDirectoryName(path)!);
            target.Save(path);
        }

        private static RenderedFrame Render(string variant)
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme("<view name=\"system\"><carousel name=\"systems\"><staticImage>./logos/${system.theme}.png</staticImage><color>00000000</color>"
                + $"<unfocusedItemOpacity>1</unfocusedItemOpacity><itemScale>1</itemScale>{Variants[variant]}</carousel></view>");
            foreach (var s in Systems) Logo(theme.PathOf($"logos/{s.Name}.png"), s);
            var systems = Systems.Select(s => new ThemeSystem(s.Name, s.Name, s.Name))
                .Select(t => new SceneSystem(t, theme.Load(new ThemeChoices { ScreenWidth = W, ScreenHeight = H }, t), [new SceneGame("Aurora Drift", "a.zip")])).ToList();
            var data = new SceneData(systems, new Size(W, H)) { SystemIndex = Array.FindIndex(Systems, s => s.Name == "snes") };
            return SceneAssets.Render(SceneBuilder.Build(data.System.Theme.SystemView, data));
        }

        // The centre of every pixel within 12 levels of a logo's colour, as the capture was read.
        private static (double X, double Y, int N) Centre(RenderedFrame f, (string Name, byte R, byte G, byte B) s)
        {
            double x = 0, y = 0;
            int n = 0;
            for (int py = 0; py < f.Height; py++)
            for (int px = 0; px < f.Width; px++)
            {
                int i = (py * f.Width + px) * 4;
                if (Math.Abs(f.Rgba[i] - s.R) < 12 && Math.Abs(f.Rgba[i + 1] - s.G) < 12 && Math.Abs(f.Rgba[i + 2] - s.B) < 12) { x += px; y += py; n++; }
            }

            return n == 0 ? (double.NaN, double.NaN, 0) : (x / n, y / n, n);
        }

        public static IEnumerable<object[]> Measured() => Esde.Keys.Select(k => new object[] { k });

        [Theory]
        [MemberData(nameof(Measured))]
        public Task Each_logo_sits_where_ES_DE_drew_it(string variant) => UiTest.Run(() =>
        {
            RenderedFrame frame = Render(variant);
            var misses = new List<string>();
            foreach ((string system, double x, double y) in Esde[variant])
            {
                (double mx, double my, int n) = Centre(frame, Systems.Single(s => s.Name == system));
                double d = Math.Sqrt((mx - x) * (mx - x) + (my - y) * (my - y));
                if (!(d <= 1.5)) misses.Add(string.Create(CultureInfo.InvariantCulture, $"{system}: ES-DE ({x}, {y}), Mistress ({mx:0.0}, {my:0.0}), {d:0.0} px apart"));
            }

            Assert.True(misses.Count == 0, string.Join("\n", misses));
        });

        // A column through a logo and its reflection: ES-DE's green levels over the 0x20 background, measured 2026-09-27; 5 levels is about a pixel of the steepest fade.
        [Theory]
        [InlineData("hRefl", 160, new[] { 336, 372, 408, 420 }, new[] { 108, 77, 46, 35 })]
        [InlineData("hReflScale", 120, new[] { 332, 368, 404, 416 }, new[] { 72, 56, 40, 35 })]
        [InlineData("hReflB", 168, new[] { 252, 272, 292, 300 }, new[] { 184, 114, 44, 32 })]
        public Task A_reflection_fades_as_ES_DE_measured(string variant, int x, int[] rows, int[] green) => UiTest.Run(() =>
        {
            RenderedFrame frame = Render(variant);
            for (int k = 0; k < rows.Length; k++)
                Assert.InRange(frame.Rgba[(rows[k] * W + x) * 4 + 1], green[k] - 5, green[k] + 5);
        });
    }
}
