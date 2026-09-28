using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Automation.Peers;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The swap trading X and Y as well as A and B (Q160), the themed search following the On-Screen Keyboard setting (Q164), Enter never asking Steam (Q165), and the popup above Steam's keyboard - see EmuSen_BigPicture.md §40.13-§40.17.
    [Collection(TestCollections.ProcessGlobals)]
    public class SwapAndKeyboardTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(SwapAndKeyboardTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public SwapAndKeyboardTests(ITestOutputHelper output) => _out = output;

        private static void Swapped(AppSettings a) => a.SwapPadButtons = true;

        private static HintBar Bar(ThemedSession s) => s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>().First(b => b.IsVisible);

        private static HintEntry Entry(IEnumerable<HintEntry>? entries, string label) => (entries ?? []).Single(e => e.Label == label);

        // Steam simulated for one test: a running client, or a variable Steam gives what it launches; the harness's refusal is put back after.
        private static void UnderSteam(Action test, bool byEnvironment = false)
        {
            try
            {
                if (byEnvironment) DeviceKeyboard.Environment = name => name == "SteamGameId" ? "13579" : null;
                else DeviceKeyboard.SteamRunning = () => true;
                Assert.True(DeviceKeyboard.UnderSteam(DeviceKeyboard.Environment, DeviceKeyboard.SteamRunning));
                test();
            }
            finally
            {
                NoSteam.Refuse();
            }
            Assert.Same(NoSteam.Launcher, DeviceKeyboard.Launcher);
            Assert.False(DeviceKeyboard.SteamRunning());
        }

        // ---- Q160: the swap trades both pairs ----

        [Fact]
        public void The_swap_trades_South_with_East_and_North_with_West_and_the_hints_letters_follow()
        {
            PadGlyphButton[] faces = [PadGlyphButton.South, PadGlyphButton.East, PadGlyphButton.West, PadGlyphButton.North];
            Assert.Equal(faces, faces.Select(b => PadHints.Of(b, false)));
            Assert.Equal(new[] { PadGlyphButton.East, PadGlyphButton.South, PadGlyphButton.North, PadGlyphButton.West }, faces.Select(b => PadHints.Of(b, true)));
            Assert.Equal(PadGlyphButton.Start, PadHints.Of(PadGlyphButton.Start, true));
            Assert.Equal(PadGlyphButton.DPadUpDown, PadHints.Of(PadGlyphButton.DPadUpDown, true));

            bool was = PadHints.Swapped;
            try
            {
                PadHints.Swapped = false;
                Assert.Equal(PadKeyboard.Hint, PadHints.Face(PadKeyboard.Hint));
                PadHints.Swapped = true;
                Assert.Equal("B  Type      A  Erase      X  Space      Select  Shift      L1 R1  Layout      Start  Done", PadHints.Face(PadKeyboard.Hint));
                Assert.Equal("Enter or Start  Done      A  Cancel      X  Keyboard", PadHints.Face(PadKeyboard.SteamHint));
                Assert.Equal("Y", PadHints.Letter("X"));
                Assert.Equal("X", PadHints.Letter("Y"));
                Assert.Equal(PadGlyphButton.West, PadHints.Glyph(PadGlyphButton.North));
            }
            finally
            {
                PadHints.Swapped = was;
            }
        }

        [Fact]
        public Task With_the_swap_X_is_the_favourite_Y_does_nothing_and_the_help_bar_draws_X_for_it() => ThemedLibraryPadTests.Run(s =>
        {
            s.Settings.ControllerType = "Xbox";
            EnterSwapped(s, "snes");
            s.Pad.Down(3);
            string chosen = ThemedSession.SnesGames[3];
            Assert.Equal(chosen, s.Game);

            s.Sounds.Clear();
            s.Pad.Y();
            Assert.False(s.Themed.SelectedGame!.Favorite);
            Assert.Null(OnScreenKeyboard.OpenOver(s.Window));
            Assert.Empty(s.Sounds);
            s.Pad.X();
            Assert.Equal(new[] { "favorite" }, s.Sounds);
            Assert.Equal(chosen, s.Game);
            Assert.True(s.Themed.SelectedGame!.Favorite);

            s.Run(300);
            HintBar bar = Bar(s);
            Assert.Equal(PadGlyphButton.West, Entry(bar.Entries, "Favorite").Button);
            Assert.Equal(PadGlyphButton.East, Entry(bar.Entries, "Launch").Button);
            Assert.Equal(PadGlyphButton.South, Entry(bar.Entries, "Back").Button);
            Assert.Equal("X", PadGlyph.Describe(bar.PadFamily, PadGlyphButton.West, bar.GlyphStyle));
            string spoken = ControlAutomationPeer.CreatePeerForElement(bar).GetName();
            Assert.Contains("X Favorite", spoken);
            Assert.Contains("B Launch", spoken);

            // The glyph drawn for Favorite is the X disc: drawing Y there instead changes pixels in its square and nowhere else.
            int at = bar.Entries!.ToList().FindIndex(e => e.Label == "Favorite");
            Rect icon = EsdeMenusTests.InWindow(bar, bar.Layout(bar.Bounds.Size)[at].Icon, s.Window);
            RenderedFrame drawn = s.Capture();
            IReadOnlyList<HintEntry> shown = bar.Entries!;
            bar.Entries = shown.Select((e, i) => i == at ? new HintEntry(e.Label, e.IconPath) { Button = PadGlyphButton.North } : e).ToList();
            RenderedFrame withY = s.Capture();
            bar.Entries = shown;
            (int inside, int outside) = EsdeMenusTests.Changed(drawn, withY, icon.Inflate(1));
            _out.WriteLine($"X against Y: {inside} pixels in the glyph's square, {outside} outside it");
            Assert.True(inside > icon.Width * icon.Height * 0.05, $"only {inside} pixels differ");
            Assert.Equal(0, outside);
        }, Swapped);

        [Fact]
        public Task With_the_swap_Y_starts_the_screensaver_in_the_system_view_and_its_help_names_Y() => Session.Dispatch(() =>
        {
            using var s = ScreensaverTests.Open(i => i.ScreensaverTimer = 0, more: Swapped);
            Assert.Equal("system", s.View);
            s.Run(300);
            Assert.Equal(PadGlyphButton.North, Entry(Bar(s).Entries, "Screensaver").Button);
            s.Pad.X();
            Assert.Null(ScreensaverTests.Saver(s));
            s.Pad.Y();
            Assert.NotNull(ScreensaverTests.Saver(s));
            s.Run(5000);
            s.Pad.Y();
            Assert.Null(ScreensaverTests.Saver(s));
            Assert.Equal("system", s.View);
        }, default);

        // Right until the named system, then the swapped Accept, which is B.
        private static void EnterSwapped(ThemedSession s, string system)
        {
            for (int guard = 0; guard < 6 && s.System != system; guard++) s.Pad.Right();
            Assert.Equal(system, s.System);
            s.Pad.A();
            Assert.Equal("system", s.View);
            s.Pad.B();
            Assert.Equal("gamelist", s.View);
        }

        // Select, the pad walked to the entry, and the swapped Accept, which is B.
        private static void ChooseSwapped(ThemedSession s, string label)
        {
            ThemedGameOptionsTests.OpenOptions(s);
            PadAudit.Reach(ThemedGameOptionsTests.Sheet(s), s.Pad, e => e is Button { Content: string text } && text == label);
            s.Pad.B();
            s.Settle();
        }

        [Fact]
        public Task With_the_swap_the_editor_names_X_for_Scrape_and_Y_for_Reset_and_Y_resets() => ThemedLibraryPadTests.Run(s =>
        {
            EnterSwapped(s, "snes");
            ChooseSwapped(s, "Edit This Game's Metadata");
            var editor = Assert.IsType<MetadataEditorWindow>(ThemedGameOptionsTests.Sheets(s).Current);
            MenuPanel menu = editor.Menu!;
            Assert.Equal(new[] { "Select", "Back", "Scrape", "Choose" }, menu.Hints!.Select(h => h.Label));
            Assert.Equal(new[] { PadGlyphButton.East, PadGlyphButton.South, PadGlyphButton.West, PadGlyphButton.DPadUpDown }, menu.Hints!.Select(h => h.Button!.Value));

            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Completed);
            s.Pad.B();
            Assert.True(editor.CanReset(GameMetadata.Completed));
            Assert.Equal(new[] { "Toggle", "Back", "Scrape", "Reset", "Choose" }, menu.Hints!.Select(h => h.Label));
            Assert.Equal(PadGlyphButton.West, menu.Hints![2].Button);
            Assert.Equal(PadGlyphButton.North, menu.Hints![3].Button);

            // X is the scraper's now, so it resets nothing; Y resets the field.
            s.Pad.Y();
            Assert.False(editor.CanReset(GameMetadata.Completed));
            Assert.Same(editor, ThemedGameOptionsTests.Sheets(s).Current);
        }, Swapped);

        // ---- Q164: the themed search follows the setting ----

        // Select's Search..., chosen with the pad's A or, the pad unplugged, the keyboard's Enter.
        private static void OpenSearch(ThemedSession s, bool byKeyboard)
        {
            ThemedGameOptionsTests.OpenOptions(s);
            PadAudit.Reach(ThemedGameOptionsTests.Sheet(s), s.Pad, e => e is Button { Content: "Search..." });
            if (!byKeyboard)
            {
                s.Pad.A();
                s.Settle();
                return;
            }
            s.Pad.Unplug();
            s.Pad.Tick();
            EsdeMenusTests.Press(s, Key.Enter);
        }

        [Theory]
        [InlineData(AppSettings.OnScreenKeyboardAutomatic, false, true, KeyboardKind.Field)]
        [InlineData(AppSettings.OnScreenKeyboardAutomatic, false, false, KeyboardKind.EmuSen)]
        [InlineData(AppSettings.OnScreenKeyboardAutomatic, true, false, KeyboardKind.Steam)]
        [InlineData(AppSettings.OnScreenKeyboardAutomatic, true, true, KeyboardKind.Field)]
        [InlineData(AppSettings.OnScreenKeyboardSteam, false, true, KeyboardKind.Steam)]
        [InlineData(AppSettings.OnScreenKeyboardSteam, false, false, KeyboardKind.Steam)]
        [InlineData(AppSettings.OnScreenKeyboardEmuSen, true, true, KeyboardKind.EmuSen)]
        [InlineData(AppSettings.OnScreenKeyboardEmuSen, true, false, KeyboardKind.EmuSen)]
        public Task The_themed_search_follows_the_On_Screen_Keyboard_setting(string setting, bool underSteam, bool byKeyboard, KeyboardKind expected) => ThemedLibraryPadTests.Run(s =>
        {
            void Test()
            {
                ThemedLibraryPadTests.Enter(s, "snes");
                int before = NoSteam.Launcher.Opened.Count;
                OpenSearch(s, byKeyboard);
                Assert.False(ThemedGameOptionsTests.Sheets(s).IsPresenting);
                Assert.True(ThemedLibraryPadTests.SearchBox(s).IsEffectivelyVisible);
                MenuTextPopup? popup = MenuTextPopup.OpenOver(s.Window);
                OnScreenKeyboard? keyboard = OnScreenKeyboard.OpenOver(s.Window);
                Assert.Equal(expected == KeyboardKind.EmuSen, keyboard is not null);
                Assert.Equal(expected != KeyboardKind.EmuSen, popup is not null);
                Assert.Equal(expected == KeyboardKind.Steam ? new[] { DeviceKeyboard.SteamKeyboardUrl } : [], NoSteam.Launcher.Opened.Skip(before));
                if (keyboard is not null)
                {
                    Assert.Same(ThemedLibraryPadTests.SearchBox(s), keyboard.Target);
                    keyboard.Cancel();
                    return;
                }
                Assert.Equal("Search", popup!.Title);
                Assert.Same(ThemedLibraryPadTests.SearchBox(s), popup.Target);
                Assert.Same(popup.Field, TopLevel.GetTopLevel(s.Window)!.FocusManager!.GetFocusedElement());
                Assert.Equal(expected == KeyboardKind.Steam ? PadKeyboard.SteamHint : PadKeyboard.FieldHint, popup.Hint);
                popup.Cancel();
            }
            if (underSteam) UnderSteam(Test);
            else Test();
        }, a => a.OnScreenKeyboard = setting);

        [Fact]
        public Task A_keyboard_s_Enter_on_Search_types_into_the_field_and_Enter_filters_the_list() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            int before = NoSteam.Launcher.Opened.Count;
            OpenSearch(s, byKeyboard: true);
            MenuTextPopup popup = MenuTextPopup.OpenOver(s.Window) ?? throw new InvalidOperationException("no text popup");
            s.Window.KeyTextInput("cobx");
            EsdeMenusTests.Press(s, Key.Back);
            Assert.Equal("cob", popup.Field.Text);
            Assert.Equal("", s.Themed.Filter);
            Assert.True(ThemedLibraryPadTests.SearchBox(s).IsEffectivelyVisible);

            EsdeMenusTests.Press(s, Key.Enter);
            Assert.False(popup.IsOpen);
            Assert.Equal("cob", s.Themed.Filter);
            Assert.Equal(new[] { ThemedSession.SnesGames[2] }, s.Themed.Stage!.Current.Data.System.Games.Select(g => g.Name));
            Assert.Equal(ThemedSession.SnesGames[2], s.Game);
            Assert.Equal(before, NoSteam.Launcher.Opened.Count);

            // Escape on a second search leaves the first one's filter as it was.
            s.Pad.Replug();
            s.Pad.Tick();
            OpenSearch(s, byKeyboard: true);
            popup = MenuTextPopup.OpenOver(s.Window)!;
            Assert.Equal("cob", popup.Field.Text);
            s.Window.KeyTextInput("zz");
            EsdeMenusTests.Press(s, Key.Escape);
            Assert.False(popup.IsOpen);
            Assert.Equal("cob", s.Themed.Filter);
            Assert.True(s.Shown);
            Assert.Equal("gamelist", s.View);
        });

        [Fact]
        public Task With_Steam_chosen_the_pad_s_search_asks_Steam_and_Start_keeps_what_it_typed() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            int before = NoSteam.Launcher.Opened.Count;
            OpenSearch(s, byKeyboard: false);
            MenuTextPopup popup = MenuTextPopup.OpenOver(s.Window) ?? throw new InvalidOperationException("no text popup");
            s.Window.KeyTextInput("dune");
            s.Pad.A();
            Assert.True(popup.IsOpen);
            s.Pad.Y();
            s.Pad.Start();
            Assert.False(popup.IsOpen);
            Assert.Equal("dune", s.Themed.Filter);
            Assert.Equal(ThemedSession.SnesGames[3], s.Game);
            Assert.Equal(new[] { DeviceKeyboard.SteamKeyboardUrl, DeviceKeyboard.SteamKeyboardUrl }, NoSteam.Launcher.Opened.Skip(before));
        }, a => a.OnScreenKeyboard = AppSettings.OnScreenKeyboardSteam);

        // ---- the popup above Steam's keyboard ----

        // The popup in the window's coordinates, and every piece of it: none outside the popup or the window, none over another.
        private static Rect AssertWhole(ThemedSession s, MenuTextPopup popup)
        {
            Rect box = new(popup.TranslatePoint(default, s.Window)!.Value, popup.Bounds.Size);
            Assert.True(new Rect(s.Window.ClientSize).Contains(box), $"popup {box} outside the window {s.Window.ClientSize}");
            Control[] parts = ((StackPanel)popup.Child!).Children.Where(c => c.IsVisible).ToArray();
            Assert.Equal(3, parts.Length);
            Rect[] placed = parts.Select(c => new Rect(c.TranslatePoint(default, s.Window)!.Value, c.Bounds.Size)).ToArray();
            for (int i = 0; i < placed.Length; i++)
            {
                Assert.True(box.Contains(placed[i]), $"{parts[i].GetType().Name} {placed[i]} outside the popup {box}");
                Assert.True(placed[i].Height > 0 && placed[i].Width > 0, $"{parts[i].GetType().Name} has no size");
                for (int j = i + 1; j < placed.Length; j++)
                    Assert.False(placed[i].Intersects(placed[j]), $"{parts[i].GetType().Name} {placed[i]} overlaps {parts[j].GetType().Name} {placed[j]}");
            }
            foreach (Control c in parts) Assert.True(c.DesiredSize.Height <= c.Bounds.Height + c.Margin.Top + c.Margin.Bottom + 0.5 && c.DesiredSize.Width <= box.Width + 0.5, $"{c.GetType().Name} wants {c.DesiredSize}, has {c.Bounds.Size}");
            return box;
        }

        [Theory]
        [MemberData(nameof(EsdeMenusTests.Sizes), MemberType = typeof(EsdeMenusTests))]
        public Task Asking_Steam_puts_the_popup_in_the_upper_part_of_the_screen_and_other_modes_keep_it_centred(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);

            // With Steam, from the pad: the popup's top at a twentieth of the height, its bottom above 45%, and its field four lines tall.
            s.Settings.OnScreenKeyboard = AppSettings.OnScreenKeyboardSteam;
            s.Pad.A();
            MenuTextPopup popup = MenuTextPopup.OpenOver(s.Window) ?? throw new InvalidOperationException("no text popup");
            s.Settle();
            Rect steam = AssertWhole(s, popup);
            _out.WriteLine($"{width}x{height} Steam: {steam}, top {steam.Top / height:P1}, bottom {steam.Bottom / height:P1}");
            Assert.InRange(steam.Top / height, 0.03, 0.07);
            Assert.True(popup.Field.Bounds.Height >= 4 * popup.Field.FontSize, $"field {popup.Field.Bounds.Height} for text {popup.Field.FontSize}");
            Assert.True(steam.Bottom < 0.45 * height, $"bottom {steam.Bottom} of {height}");
            Assert.True(Math.Abs(width / 2.0 - steam.Center.X) <= 1, $"centre {steam.Center.X} of {width}");
            s.Pad.B();

            // From a keyboard (Automatic, and under Steam too) and with Steam chosen but the field reached by Enter: centred as before.
            s.Settings.OnScreenKeyboard = AppSettings.OnScreenKeyboardAutomatic;
            s.Pad.Unplug();
            s.Pad.Tick();
            foreach (bool underSteam in new[] { false, true })
            {
                void Centred()
                {
                    EsdeMenusTests.Press(s, Key.Enter);
                    MenuTextPopup field = MenuTextPopup.OpenOver(s.Window) ?? throw new InvalidOperationException("no text popup");
                    s.Settle();
                    Rect centred = AssertWhole(s, field);
                    _out.WriteLine($"{width}x{height} field{(underSteam ? " under Steam" : "")}: {centred}");
                    Assert.True(Math.Abs(height / 2.0 - centred.Center.Y) <= 1, $"centre {centred.Center.Y} of {height}");
                    Assert.True(Math.Abs(width / 2.0 - centred.Center.X) <= 1, $"centre {centred.Center.X} of {width}");
                    Assert.Equal(steam.Size, centred.Size);
                    EsdeMenusTests.Press(s, Key.Escape);
                }
                if (underSteam) UnderSteam(Centred);
                else Centred();
            }
            Assert.Same(editor, ThemedGameOptionsTests.Sheets(s).Current);
        }, default);

        // ---- Q165: Enter under Steam gets the field alone; a pad's A still asks Steam ----

        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public Task Under_Steam_Enter_on_a_text_row_gets_the_field_alone_and_a_pad_s_A_asks_Steam(bool byEnvironment) => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            Assert.Equal(AppSettings.OnScreenKeyboardAutomatic, s.Settings.OnScreenKeyboard);
            UnderSteam(() =>
            {
                int before = NoSteam.Launcher.Opened.Count;
                s.Pad.Unplug();
                s.Pad.Tick();
                Assert.True(editor.EditorOf(GameMetadata.Name).IsFocused);
                EsdeMenusTests.Press(s, Key.Enter);
                MenuTextPopup popup = MenuTextPopup.OpenOver(s.Window) ?? throw new InvalidOperationException("no text popup");
                Assert.Equal(PadKeyboard.FieldHint, popup.Hint);
                Assert.Equal(before, NoSteam.Launcher.Opened.Count);
                s.Window.KeyTextInput(" II");
                EsdeMenusTests.Press(s, Key.Enter);
                Assert.Equal(ThemedSession.SnesGames[0] + " II", ((TextBox)editor.EditorOf(GameMetadata.Name)).Text);
                Assert.Equal(before, NoSteam.Launcher.Opened.Count);

                s.Pad.Replug();
                s.Pad.Tick();
                s.Pad.A();
                popup = MenuTextPopup.OpenOver(s.Window) ?? throw new InvalidOperationException("no text popup");
                Assert.Equal(PadKeyboard.SteamHint, popup.Hint);
                Assert.Equal(new[] { DeviceKeyboard.SteamKeyboardUrl }, NoSteam.Launcher.Opened.Skip(before));
                s.Pad.B();
                Assert.False(popup.IsOpen);
            }, byEnvironment);
        });
    }
}
