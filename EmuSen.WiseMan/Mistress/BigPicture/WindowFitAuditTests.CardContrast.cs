using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class CardPngFactAttribute : FactAttribute
    {
        public CardPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_CARD_PNG") != "1")
                Skip = "Writes the Home card's and the Pause Card's pictures to ~/.cache/emusen/tmp/pausecard/built/; set EMUSEN_CARD_PNG=1 - see EmuSen_BigPicture.md §49.8";
        }
    }

    // Every word a card draws read against what is actually behind it, over the brightest and the darkest screen beneath - see EmuSen_BigPicture.md §49.8.
    public partial class WindowFitAuditTests
    {
        public static readonly string[] CardContrastViews = ["HomeCard", "HomeCardGame", "HomeCardTools", "HomeCardPower", "PauseCard", "PauseCardRewind", "PauseCardSave", "PauseCardLoad", "PauseCardGameOptions", "PauseCardLeave"];

        public static TheoryData<string, string> CardContrastCases()
        {
            var data = new TheoryData<string, string>();
            foreach (string v in CardContrastViews)
                foreach (string under in new[] { "white", "black" }) data.Add(v, under);
            return data;
        }

        [Theory]
        [MemberData(nameof(CardContrastCases))]
        public Task Every_word_on_a_card_reads_over_a_white_or_a_black_screen(string view, string under) => Session.Dispatch(() =>
        {
            (MainWindow window, _) = OpenCard(view, 1280, 800, under == "white" ? Colors.White : Colors.Black);
            try
            {
                Assert.False(window.GetControl<Control>("StatusBar").IsVisible, "the status bar under the card");
                Control surface = PadMenu.Card(window).GetVisualDescendants().OfType<Control>().Single(c => c.Name == "AdaptiveCardPane");
                Assert.Equal(PadMenu.Pane(window).Showing != "none", surface.IsEffectivelyVisible);
                List<string> faults = [];
                foreach ((string what, Color ink, Color behind, double ratio) in CardWords(window))
                {
                    _out.WriteLine($"{view} over {under}: {what} {ink} on {behind} {ratio:F2}:1");
                    if (ratio < 4.5) faults.Add($"{what} is {ratio:F2}:1, {ink} on {behind}");
                }
                Assert.True(faults.Count == 0, string.Join("\n", faults));
            }
            finally { Stop(window); window.Close(); }
        }, default);

        [CardPngFact]
        public Task The_cards_drawn_for_comparing_with_their_design() => Session.Dispatch(() =>
        {
            string folder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "tmp", "pausecard", "built");
            Directory.CreateDirectory(folder);
            foreach (string view in CardContrastViews)
                foreach ((int w, int h) in new[] { (1280, 800), (1920, 1080) })
                    foreach ((string name, Color? under) in new[] { ("plain", (Color?)null), ("white", Colors.White) })
                    {
                        (MainWindow window, _) = OpenCard(view, w, h, under);
                        try { Capture(window).Save(Path.Combine(folder, $"{view}-{name}-{w}x{h}.png")); }
                        finally { Stop(window); window.Close(); }
                    }
            foreach (string tool in new[] { "Cores", "PackagesPlan" })
                foreach ((int w, int h) in new[] { (1280, 800), (1920, 1080) })
                {
                    (MainWindow window, PadDriver pad) = InGame(w, h);
                    try
                    {
                        OpenInGame(window, pad, tool);
                        Settle(window);
                        Capture(window).Save(Path.Combine(folder, $"{tool}-plain-{w}x{h}.png"));
                    }
                    finally { Stop(window); window.Close(); }
                }
        }, default);

        // The Home card over the synthetic theme's gamelist, the tester's own case, on the game in hand and on Tools.
        [Theory]
        [InlineData(1280, 800)]
        [InlineData(1920, 1080)]
        public Task Every_word_on_the_home_card_reads_over_a_theme(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Start();
            s.Settle();
            List<string> faults = [];
            for (int page = 0; page < 2; page++)
            {
                if (page == 1) { s.Pad.Down(PadMenu.PathTo(s.Window, MainWindow.ToolsMenu, exact: true)![0]); s.Settle(); }
                foreach ((string what, Color ink, Color behind, double ratio) in CardWords(s.Window))
                {
                    _out.WriteLine($"theme {width}x{height} row {page}: {what} {ink} on {behind} {ratio:F2}:1");
                    if (ratio < 4.5) faults.Add($"{what} is {ratio:F2}:1, {ink} on {behind}");
                }
                if (Environment.GetEnvironmentVariable("EMUSEN_CARD_PNG") == "1")
                {
                    string folder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "tmp", "pausecard", "built");
                    Directory.CreateDirectory(folder);
                    Capture(s.Window).Save(Path.Combine(folder, $"HomeCardThemed{(page == 0 ? "Game" : "Tools")}-plain-{width}x{height}.png"));
                }
            }
            Assert.True(faults.Count == 0, string.Join("\n", faults));
        }, default);

        // A card at a size and state, with a flat colour standing in for whatever screen is beneath it when one is given.
        private (MainWindow, PadDriver) OpenCard(string view, int width, int height, Color? under)
        {
            bool home = view.StartsWith("HomeCard", StringComparison.Ordinal);
            if (home) ToolCounts.Answer();
            // The Home card's first page is a pad's on the desktop and a theme's; the big screen's own look opens it at Power from Home's place (EmuSen_BigPicture.md §51.2).
            bool desktop = home && view != "HomeCardPower";
            (MainWindow window, PadDriver pad) = InGame(width, height, bigScreen: !desktop);
            if (home)
            {
                Call(window, "ToggleLibrary");
                pad.Start();
                if (!desktop)
                {
                    window.HomeLayout!.FocusOn(EmuSen.LunaP.Controls.HubRow.Places, "power");
                    pad.A();
                }
            }
            else pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            Assert.True(PadMenu.IsOpen(window) && PadMenu.Card(window).IsVisible);
            Settle(window);
            switch (view)
            {
                case "HomeCardGame": pad.Down(1); break;
                case "HomeCardTools": PadMenu.Choose(window, pad, MainWindow.ToolsMenu, exact: true); break;

                case "PauseCardRewind": pad.Down(1); break;
                case "PauseCardSave": pad.Down(2); break;
                case "PauseCardLoad": pad.Down(3); break;
                case "PauseCardGameOptions": PadMenu.Choose(window, pad, MainWindow.GameOptionsMenu, exact: true); break;
                case "PauseCardLeave": PadMenu.Choose(window, pad, MainWindow.LeaveMenu, exact: true); break;
            }
            if (under is { } colour)
            {
                Control menu = window.GetControl<Control>("PadMenuPanel");
                var host = (Panel)menu.GetVisualParent()!;
                host.Children.Insert(host.Children.IndexOf(menu), new Border { Name = "ScreenBeneath", Background = new SolidColorBrush(colour) });
            }
            Settle(window);
            if (home) ToolCounts.Quiet();
            return (window, pad);
        }

        // Each run of words the card and its pane draw: its ink, the colour most often under its box, and their ratio.
        internal static IEnumerable<(string What, Color Ink, Color Behind, double Ratio)> CardWords(MainWindow window)
        {
            Picture frame = Capture(window);
            AdaptiveLayout card = PadMenu.Card(window);
            MenuPanel panel = card.CardPanel;
            if (panel.Title is { Length: > 0 } title)
                yield return Measure(frame, "title " + title, panel.TitleColor, 1, Box(panel, panel.TitleBounds, window));
            foreach (Visual v in card.GetVisualDescendants().Where(v => v.IsEffectivelyVisible))
            {
                Rect box = Box(v, new Rect(v.Bounds.Size), window);
                if (box.Width < 2 || box.Height < 2) continue;
                if (v is TextBlock { Text.Length: > 0 } words && words.Foreground is ISolidColorBrush ink)
                    yield return Measure(frame, $"{words.Name ?? "text"} \"{words.Text}\"", ink.Color, Opacity(v) * ink.Opacity, box);
                else if (v is MenuRow row && row.Label is { Length: > 0 })
                {
                    Color words2 = row.IsHighlighted ? row.HighlightTextColor : row.TextColor;
                    yield return Measure(frame, $"row \"{row.Label}\"", words2, Opacity(v), box);
                    if (row.Value is { Length: > 0 }) yield return Measure(frame, $"row \"{row.Label}\" value \"{row.Value}\"", row.ValueColor ?? words2, Opacity(v), box);
                }
            }
        }

        private static double Opacity(Visual v)
        {
            double o = 1;
            for (Visual? at = v; at is not null; at = at.GetVisualParent()) o *= at.Opacity;
            return o;
        }

        private static Rect Box(Visual v, Rect local, Visual window) =>
            v.TranslatePoint(local.TopLeft, window) is { } at ? new Rect(at, local.Size) : default;

        private static (string, Color, Color, double) Measure(Picture frame, string what, Color ink, double opacity, Rect box)
        {
            Color behind = frame.MostOften(box);
            double a = opacity * ink.A / 255.0;
            byte Mix(byte i, byte b) => (byte)Math.Round(i * a + b * (1 - a));
            Color seen = Color.FromRgb(Mix(ink.R, behind.R), Mix(ink.G, behind.G), Mix(ink.B, behind.B));
            return (what, ink, behind, Contrast(seen, behind));
        }

        internal static double Contrast(Color a, Color b)
        {
            static double L(byte v) { double c = v / 255.0; return c <= 0.03928 ? c / 12.92 : Math.Pow((c + 0.055) / 1.055, 2.4); }
            static double Lum(Color c) => 0.2126 * L(c.R) + 0.7152 * L(c.G) + 0.0722 * L(c.B);
            double la = Lum(a), lb = Lum(b);
            return (Math.Max(la, lb) + 0.05) / (Math.Min(la, lb) + 0.05);
        }

        private static Picture Capture(Window window)
        {
            Dispatcher.UIThread.RunJobs();
            using WriteableBitmap bitmap = window.CaptureRenderedFrame() ?? throw new InvalidOperationException("no frame");
            using ILockedFramebuffer fb = bitmap.Lock();
            int w = fb.Size.Width, h = fb.Size.Height;
            var rgba = new byte[w * h * 4];
            for (int y = 0; y < h; y++) System.Runtime.InteropServices.Marshal.Copy(fb.Address + y * fb.RowBytes, rgba, y * w * 4, w * 4);
            if (fb.Format == PixelFormat.Bgra8888)
                for (int i = 0; i < rgba.Length; i += 4) (rgba[i], rgba[i + 2]) = (rgba[i + 2], rgba[i]);
            return new Picture(rgba, w, h);
        }

        internal sealed record Picture(byte[] Rgba, int Width, int Height)
        {
            public Color MostOften(Rect box)
            {
                var counts = new Dictionary<int, int>();
                int x0 = Math.Max(0, (int)box.X), y0 = Math.Max(0, (int)box.Y), x1 = Math.Min(Width, (int)box.Right), y1 = Math.Min(Height, (int)box.Bottom);
                for (int y = y0; y < y1; y++)
                    for (int x = x0; x < x1; x++)
                    {
                        int i = (y * Width + x) * 4, key = Rgba[i] | Rgba[i + 1] << 8 | Rgba[i + 2] << 16;
                        counts[key] = counts.GetValueOrDefault(key) + 1;
                    }
                int best = counts.Count == 0 ? 0 : counts.MaxBy(p => p.Value).Key;
                return Color.FromRgb((byte)best, (byte)(best >> 8), (byte)(best >> 16));
            }

            public void Save(string path) => EmuSen.Serenity.Imaging.FrameImageWriter.SavePng(Rgba, Width, Height, path);
        }
    }
}
