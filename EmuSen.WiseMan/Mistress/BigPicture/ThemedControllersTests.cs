using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Headless;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The themed view steered by two pads, its help bar following the pad last pressed, the Controller Type setting and the swap - see EmuSen_Settings_Reference.md §4.61.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedControllersTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedControllersTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public ThemedControllersTests(ITestOutputHelper output) => _out = output;

        private static AppSettings Settings(ThemedSession s) =>
            (AppSettings)typeof(MainWindow).GetField("_appSettings", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(s.Window)!;

        private static HintBar Bar(ThemedSession s) => s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>().Single();

        // Pixels that differ from a frame, inside the help bar and outside it.
        private static (int Inside, int Outside) Changed(ThemedSession s, RenderedFrame before, RenderedFrame after)
        {
            HintBar bar = Bar(s);
            var box = new Rect(bar.TranslatePoint(default, s.Window)!.Value, bar.Bounds.Size).Inflate(1);
            int inside = 0, outside = 0;
            for (int y = 0; y < after.Height; y++)
                for (int x = 0; x < after.Width; x++)
                {
                    int i = (y * after.Width + x) * 4;
                    if (after.Rgba[i] == before.Rgba[i] && after.Rgba[i + 1] == before.Rgba[i + 1] && after.Rgba[i + 2] == before.Rgba[i + 2]) continue;
                    if (box.Contains(new Point(x + 0.5, y + 0.5))) inside++; else outside++;
                }
            return (inside, outside);
        }

        [Fact]
        public Task Two_pads_both_steer_the_themed_view() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            PadDriver second = s.Pad.Plug("Second Pad");
            s.Pad.Tick();

            string? start = s.System;
            second.Right();
            string? moved = s.System;
            Assert.NotEqual(start, moved);
            s.Pad.Right();
            Assert.NotEqual(moved, s.System);

            second.A();
            Assert.Equal("gamelist", s.View);
            s.Pad.Down();
            second.Down();
            Assert.Equal(2, s.Themed.Stage!.Current.Index);
            s.Pad.B();
            Assert.Equal("system", s.View);
        }, default);

        // The help bar draws the family of the pad last pressed, whichever pad the player picks up.
        [Fact]
        public Task The_help_bar_follows_the_pad_last_pressed() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            s.Pad.Pad.Name = "DualSense Wireless Controller";
            s.Pad.Pad.Type = SDL.GamepadType.PS5;
            PadDriver second = s.Pad.Plug("Pro Controller", SDL.GamepadType.NintendoSwitchPro);
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.Equal(PadFamily.PlayStation, Bar(s).PadFamily);

            second.Down();
            Assert.Equal(PadFamily.Nintendo, Bar(s).PadFamily);
            s.Pad.Down();
            Assert.Equal(PadFamily.PlayStation, Bar(s).PadFamily);

            // The pad last pressed pulled out: the first pad's family again.
            second.Down();
            second.Unplug();
            s.Pad.Tick();
            Assert.Equal(PadFamily.PlayStation, Bar(s).PadFamily);
        }, default);

        // P43 for the setting: each type redraws the help bar's icons and no pixel outside it, whatever pad is in hand, and the pad still steers.
        [Fact]
        public Task The_controller_type_changes_the_help_bar_s_glyphs_and_nothing_else() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            s.Pad.Pad.Name = "DualSense Wireless Controller";
            s.Pad.Pad.Type = SDL.GamepadType.PS5;
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.Equal(PadFamily.PlayStation, Bar(s).PadFamily);
            RenderedFrame automatic = s.Capture();

            foreach ((string type, PadFamily family) in new[] { ("Xbox", PadFamily.Xbox), ("Nintendo", PadFamily.Nintendo), ("Generic", PadFamily.Generic) })
            {
                Settings(s).ControllerType = type;
                s.Pad.Tick();
                Assert.Equal(family, Bar(s).PadFamily);
                (int inside, int outside) = Changed(s, automatic, s.Capture());
                _out.WriteLine($"{type}: {inside} pixels changed in the help bar, {outside} outside it");
                Assert.True(inside > 50);
                Assert.Equal(0, outside);
            }

            Settings(s).ControllerType = AppSettings.ControllerTypeAutomatic;
            s.Pad.Tick();
            Assert.Equal(PadFamily.PlayStation, Bar(s).PadFamily);
            Assert.Equal((0, 0), Changed(s, automatic, s.Capture()));

            // Only the pictures: the buttons do what they did, and a view built after keeps the choice.
            Settings(s).ControllerType = "Nintendo";
            s.Pad.Down();
            Assert.Equal(ThemedSession.SnesGames[1], s.Game);
            s.Pad.B();
            Assert.Equal("system", s.View);
            Assert.Equal(PadFamily.Nintendo, Bar(s).PadFamily);
        }, default);

        // With the swap, East chooses and South goes back in the view, and the help bar names those buttons.
        [Fact]
        public Task The_swap_trades_the_view_s_buttons_and_the_help_bar_follows() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            RenderedFrame plain = s.Capture();
            Assert.Equal(PadGlyphButton.South, Bar(s).Entries.Single(e => e.Label == "Launch").Button);

            Settings(s).SwapPadButtons = true;
            s.Pad.Tick();
            IReadOnlyList<HintEntry> entries = Bar(s).Entries;
            Assert.Equal(PadGlyphButton.East, entries.Single(e => e.Label == "Launch").Button);
            Assert.Equal(PadGlyphButton.South, entries.Single(e => e.Label == "Back").Button);
            Assert.Equal(PadGlyphButton.West, entries.Single(e => e.Label == "Search").Button);
            (int inside, int outside) = Changed(s, plain, s.Capture());
            _out.WriteLine($"swap: {inside} pixels changed in the help bar, {outside} outside it");
            Assert.Equal(0, outside);

            s.Pad.A();
            Assert.Equal("system", s.View);
            s.Pad.B();
            Assert.Equal("gamelist", s.View);
            Assert.Equal(PadGlyphButton.East, Bar(s).Entries.Single(e => e.Label == "Launch").Button);
        }, default);
    }
}
