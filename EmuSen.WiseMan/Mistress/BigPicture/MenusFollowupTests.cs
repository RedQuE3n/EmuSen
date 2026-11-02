using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.LogicalTree;
using Avalonia.Media;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The follow-ups to ES-DE's menus: lettered help glyphs, the editor's one-line subtitle and per-row help, the scroll indicator, and the device's keyboard - see EmuSen_BigPicture.md §40.
    [Collection(TestCollections.ProcessGlobals)]
    public class MenusFollowupTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MenusFollowupTests).GetTypeInfo().Assembly);

        public static TheoryData<int, int> Sizes() => EsdeMenusTests.Sizes();

        private static Func<string, string?> Env(params (string Name, string Value)[] vars) => name => vars.FirstOrDefault(v => v.Name == name).Value;

        // ---- the choice of keyboard, under simulated environments ----

        [Theory]
        [InlineData(AppSettings.OnScreenKeyboardAutomatic, true, true, KeyboardKind.Steam)]
        [InlineData(AppSettings.OnScreenKeyboardAutomatic, true, false, KeyboardKind.Field)]
        [InlineData(AppSettings.OnScreenKeyboardAutomatic, false, true, KeyboardKind.EmuSen)]
        [InlineData(AppSettings.OnScreenKeyboardAutomatic, false, false, KeyboardKind.Field)]
        [InlineData(AppSettings.OnScreenKeyboardSteam, false, true, KeyboardKind.Steam)]
        [InlineData(AppSettings.OnScreenKeyboardSteam, false, false, KeyboardKind.Steam)]
        [InlineData(AppSettings.OnScreenKeyboardEmuSen, true, true, KeyboardKind.EmuSen)]
        [InlineData(AppSettings.OnScreenKeyboardEmuSen, true, false, KeyboardKind.EmuSen)]
        [InlineData(null, false, false, KeyboardKind.Field)]
        [InlineData("something unknown", true, true, KeyboardKind.Steam)]
        [InlineData("something unknown", true, false, KeyboardKind.Field)]
        public void The_setting_Steam_and_the_device_in_use_choose_the_keyboard(string? setting, bool underSteam, bool padInUse, KeyboardKind expected) =>
            Assert.Equal(expected, DeviceKeyboard.Choose(setting, underSteam, padInUse));

        [Fact]
        public void Steam_is_detected_from_the_environment_and_a_running_client()
        {
            Func<bool> no = () => false, yes = () => true;
            // Game Mode's session, and a Deck's session with no desktop named.
            Assert.True(DeviceKeyboard.UnderSteam(Env(("XDG_CURRENT_DESKTOP", "gamescope")), no));
            Assert.True(DeviceKeyboard.UnderSteam(Env(("XDG_CURRENT_DESKTOP", "KDE:gamescope")), no));
            Assert.True(DeviceKeyboard.UnderSteam(Env(("SteamDeck", "1")), no));
            // Launched by Steam, including a non-Steam shortcut and a game under Proton.
            Assert.True(DeviceKeyboard.UnderSteam(Env(("XDG_CURRENT_DESKTOP", "KDE"), ("SteamGameId", "13579")), no));
            Assert.True(DeviceKeyboard.UnderSteam(Env(("XDG_CURRENT_DESKTOP", "KDE"), ("SteamAppId", "0")), no));
            Assert.True(DeviceKeyboard.UnderSteam(Env(("XDG_CURRENT_DESKTOP", "KDE"), ("STEAM_COMPAT_DATA_PATH", "/tmp/pfx")), no));
            Assert.True(DeviceKeyboard.UnderSteam(Env(("XDG_CURRENT_DESKTOP", "KDE"), ("STEAM_COMPAT_APP_ID", "0")), no));
            // Desktop Mode with the client running.
            Assert.True(DeviceKeyboard.UnderSteam(Env(("XDG_CURRENT_DESKTOP", "KDE")), yes));
            // A desktop with no Steam at all; an empty variable is not a set one.
            Assert.False(DeviceKeyboard.UnderSteam(Env(("XDG_CURRENT_DESKTOP", "KDE")), no));
            Assert.False(DeviceKeyboard.UnderSteam(Env(("XDG_CURRENT_DESKTOP", "GNOME"), ("SteamGameId", "")), no));
            Assert.False(DeviceKeyboard.UnderSteam(Env(), no));
            Assert.False(DeviceKeyboard.UnderSteam(Env(("SteamDeck", "0"), ("XDG_CURRENT_DESKTOP", "KDE")), no));
        }

        [Fact]
        public void The_harness_sees_no_Steam_and_launches_nothing()
        {
            Assert.Same(NoSteam.Launcher, DeviceKeyboard.Launcher);
            Assert.False(DeviceKeyboard.SteamRunning());
            Assert.Null(DeviceKeyboard.Environment("XDG_CURRENT_DESKTOP"));
            Assert.Equal(KeyboardKind.Field, DeviceKeyboard.ChooseNow(AppSettings.OnScreenKeyboardAutomatic, padInUse: false));
            Assert.Equal(KeyboardKind.EmuSen, DeviceKeyboard.ChooseNow(AppSettings.OnScreenKeyboardAutomatic, padInUse: true));
        }

        // ---- Q109 in the editor ----

        [Fact]
        public Task Preferences_offers_the_setting_in_its_Controllers_screen_and_stores_it() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryFlowTests.Choose(s, "Preferences");
            s.Settle();
            var sheet = Assert.IsType<PreferencesWindow>(ThemedGameOptionsTests.Sheets(s).Current);
            ThemedGameOptionsTests.Reach(s, "BigMenuPage4");
            s.Pad.A();
            Assert.Equal("Controllers", sheet.Form!.Menu.Title);
            var box = ThemedGameOptionsTests.Sheet(s).GetLogicalDescendants().OfType<Dropdown>().Single(d => d.Name == "OnScreenKeyboardDropdown");
            Assert.Equal(MenuRowKind.Option, MenuRows.GetKind(box));
            Assert.Equal("On-Screen Keyboard", MenuRows.GetLabel(box));
            Assert.Equal(new[] { "Automatic", "Steam", "EmuSen's" }, box.Items.Cast<object>().Select(i => (i as ContentControl)?.Content as string ?? i as string));
            ThemedGameOptionsTests.Reach(s, "OnScreenKeyboardDropdown");
            s.Pad.Right();
            Assert.Equal(AppSettings.OnScreenKeyboardSteam, s.Settings.OnScreenKeyboard);
            s.Pad.Right();
            Assert.Equal(AppSettings.OnScreenKeyboardEmuSen, s.Settings.OnScreenKeyboard);
            ThemedSwitchesTests.PutAway(s);
        });

        // ---- Q105 and Q106 ----

        private static string[] Labels(MenuPanel menu) => menu.Hints!.Select(h => h.Label).ToArray();

        private static PadGlyphButton[] Buttons(MenuPanel menu) => menu.Hints!.Select(h => h.Button!.Value).ToArray();

        // ---- Q107 ----

        // The indicator's pixels: every change it makes lies in its squares, the ones shown are inked, and a square not shown is not.
        private static void AssertDrawnOnlyIn(ThemedSession s, MenuPanel menu, Rect shown, Rect other)
        {
            Rect a = EsdeMenusTests.InWindow(menu, shown, s.Window), b = EsdeMenusTests.InWindow(menu, other, s.Window);
            RenderedFrame with = s.Capture();
            menu.ShowsScrollIndicator = false;
            RenderedFrame without = s.Capture();
            menu.ShowsScrollIndicator = true;
            (int inside, int outside) = EsdeMenusTests.Changed(with, without, a.Inflate(1), b.Inflate(1));
            Assert.Equal(0, outside);
            (int inShown, _) = EsdeMenusTests.Changed(with, without, a.Inflate(1));
            Assert.True(inShown > 40 * a.Width / 25, $"only {inShown} pixels in the shown square");
            bool both = menu.ScrollIndicator == MenuScrollIndicator.Both;
            Assert.True(both ? inside > inShown + 40 : inside == inShown, $"inside {inside}, shown {inShown}, {menu.ScrollIndicator}");
        }

        [Fact]
        public Task Every_big_screen_menu_that_overflows_gets_the_indicator_and_one_that_does_not_shows_none() => ThemedLibraryPadTests.Run(s =>
        {
            // Preferences' first screen fits; Theme Settings' Interface screen runs past the panel.
            ThemedLibraryFlowTests.Choose(s, "Preferences");
            s.Settle();
            var sheet = Assert.IsType<PreferencesWindow>(ThemedGameOptionsTests.Sheets(s).Current);
            Assert.Equal(MenuScrollIndicator.None, sheet.Form!.Menu.ScrollIndicator);
            ThemedSwitchesTests.PutAway(s);
            ThemeSettingsWindow theme = ThemedSwitchesTests.OpenInterface(s);
            Assert.Equal("Interface", theme.Form!.Menu.Title);
            Assert.Equal(MenuScrollIndicator.Down, theme.Form.Menu.ScrollIndicator);
            ThemedSwitchesTests.PutAway(s);

            // The pad menu, a list rather than a scroller, is watched through its own scroller.
            s.Pad.Chord(SDL3.SDL.GamepadButton.Back, SDL3.SDL.GamepadButton.Start);
            s.Settle();
            MenuPanel pad = PadMenu.Big(s.Window);
            Assert.True(pad.IsEffectivelyVisible);
            // The Home card's rows all show at 800 lines, so its scroller shows no indicator, even with Up wrapped to the last row.
            Assert.NotNull(pad.Child!.GetVisualDescendants().OfType<ScrollViewer>().FirstOrDefault());
            Assert.Equal(MenuScrollIndicator.None, pad.ScrollIndicator);
            s.Pad.Up();
            s.Settle();
            Assert.Equal(PadMenu.Lines(s.Window).Length - 1, PadMenu.Selected(s.Window));
            Assert.Equal(MenuScrollIndicator.None, pad.ScrollIndicator);
        });

        // ---- the help glyphs ----

        // The first entry's glyph, A, is a solid disc: with the family Generic its square is mostly ink, several times what the outlined set drew.
        private static void AssertDiscs(ThemedSession s, HintBar bar)
        {
            int index = bar.Entries!.ToList().FindIndex(e => e.Button is PadGlyphButton.South or PadGlyphButton.East);
            Rect icon = EsdeMenusTests.InWindow(bar, bar.Layout(bar.Bounds.Size)[index].Icon, s.Window);
            Color ink = bar.IconColor;
            int Inked(RenderedFrame f)
            {
                int n = 0;
                for (int y = (int)icon.Top; y < (int)Math.Ceiling(icon.Bottom); y++)
                    for (int x = (int)icon.Left; x < (int)Math.Ceiling(icon.Right); x++)
                    {
                        (byte r, byte g, byte b) = EsdeMenusTests.At(f, x, y);
                        if (Math.Abs(r - ink.R) <= 40 && Math.Abs(g - ink.G) <= 40 && Math.Abs(b - ink.B) <= 40) n++;
                    }
                return n;
            }
            int filled = Inked(s.Capture());
            bar.GlyphStyle = PadGlyphStyle.Outline;
            int outlined = Inked(s.Capture());
            bar.GlyphStyle = PadGlyphStyle.Filled;
            Assert.True(filled > icon.Width * icon.Height * 0.3, $"filled {filled} of {icon.Width * icon.Height}");
            Assert.True(filled > outlined * 2, $"filled {filled}, outlined {outlined}");
        }
    }
}
