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

        private static MenuPanel Menu(MetadataEditorWindow editor) => editor.Menu!;

        private static Func<string, string?> Env(params (string Name, string Value)[] vars) => name => vars.FirstOrDefault(v => v.Name == name).Value;

        // ---- the choice of keyboard, under simulated environments ----

        [Theory]
        [InlineData(AppSettings.OnScreenKeyboardAutomatic, true, true, KeyboardKind.Steam)]
        [InlineData(AppSettings.OnScreenKeyboardAutomatic, true, false, KeyboardKind.Steam)]
        [InlineData(AppSettings.OnScreenKeyboardAutomatic, false, true, KeyboardKind.EmuSen)]
        [InlineData(AppSettings.OnScreenKeyboardAutomatic, false, false, KeyboardKind.Field)]
        [InlineData(AppSettings.OnScreenKeyboardSteam, false, true, KeyboardKind.Steam)]
        [InlineData(AppSettings.OnScreenKeyboardSteam, false, false, KeyboardKind.Steam)]
        [InlineData(AppSettings.OnScreenKeyboardEmuSen, true, true, KeyboardKind.EmuSen)]
        [InlineData(AppSettings.OnScreenKeyboardEmuSen, true, false, KeyboardKind.EmuSen)]
        [InlineData(null, false, false, KeyboardKind.Field)]
        [InlineData("something unknown", true, false, KeyboardKind.Steam)]
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

        private static TextBox NameBox(MetadataEditorWindow editor) => (TextBox)editor.EditorOf(GameMetadata.Name);

        [Fact]
        public Task A_keyboard_s_Enter_on_a_text_row_opens_a_focused_field_whose_text_is_kept_on_Enter() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            s.Pad.Unplug();
            s.Pad.Tick();
            int launched = NoSteam.Launcher.Opened.Count;
            Assert.True(NameBox(editor).IsFocused);

            EsdeMenusTests.Press(s, Key.Enter);
            MenuTextPopup popup = MenuTextPopup.OpenOver(s.Window) ?? throw new InvalidOperationException("no text popup");
            Assert.Null(OnScreenKeyboard.OpenOver(s.Window));
            Assert.Equal("Enter Name", popup.Title);
            Assert.Same(popup.Field, TopLevel.GetTopLevel(s.Window)!.FocusManager!.GetFocusedElement());
            Assert.Equal(PadKeyboard.FieldHint, popup.Hint);

            // Typed keys reach the field, not the menu: Down and Backspace are the text's, not a move or a leave.
            s.Window.KeyTextInput(" II");
            EsdeMenusTests.Press(s, Key.Back);
            Assert.Equal(ThemedSession.SnesGames[0] + " I", popup.Field.Text);
            Assert.Equal(ThemedSession.SnesGames[0], NameBox(editor).Text);
            Assert.Same(editor, ThemedGameOptionsTests.Sheets(s).Current);

            EsdeMenusTests.Press(s, Key.Enter);
            Assert.False(popup.IsOpen);
            Assert.Equal(ThemedSession.SnesGames[0] + " I", NameBox(editor).Text);
            Assert.Equal(ThemedSession.SnesGames[0] + " I", editor.Draft.Value(GameMetadata.Name));
            Assert.True(NameBox(editor).IsFocused);
            // A physical keyboard off Steam never asks Steam.
            Assert.Equal(launched, NoSteam.Launcher.Opened.Count);

            // Enter's release left no key held: Down moves on at once.
            EsdeMenusTests.Press(s, Key.Down);
            Assert.False(NameBox(editor).IsFocused);
        });

        [Fact]
        public Task Escape_drops_what_was_typed_into_the_field() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            s.Pad.Unplug();
            s.Pad.Tick();
            EsdeMenusTests.Press(s, Key.Enter);
            MenuTextPopup popup = MenuTextPopup.OpenOver(s.Window)!;
            s.Window.KeyTextInput("zzz");
            EsdeMenusTests.Press(s, Key.Escape);
            Assert.False(popup.IsOpen);
            Assert.Equal(ThemedSession.SnesGames[0], NameBox(editor).Text);
            Assert.False(editor.Draft.IsDirty);
            // Escape was the popup's: the editor, and big picture, are still there.
            Assert.Same(editor, ThemedGameOptionsTests.Sheets(s).Current);
            Assert.True(s.Shown);

            // Enter's release while the popup had the focus was taken as the key let go: Enter opens the popup again at once.
            EsdeMenusTests.Press(s, Key.Enter);
            Assert.NotNull(MenuTextPopup.OpenOver(s.Window));
            EsdeMenusTests.Press(s, Key.Escape);
        });

        [Fact]
        public Task With_Steam_chosen_A_asks_Steam_for_exactly_its_keyboard_over_a_focused_field() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            s.Settings.OnScreenKeyboard = AppSettings.OnScreenKeyboardSteam;
            int before = NoSteam.Launcher.Opened.Count;

            s.Pad.A();
            MenuTextPopup popup = MenuTextPopup.OpenOver(s.Window) ?? throw new InvalidOperationException("no text popup");
            Assert.Null(OnScreenKeyboard.OpenOver(s.Window));
            Assert.Equal(new[] { "steam://open/keyboard" }, NoSteam.Launcher.Opened.Skip(before));
            Assert.Same(popup.Field, TopLevel.GetTopLevel(s.Window)!.FocusManager!.GetFocusedElement());
            Assert.Equal(PadKeyboard.SteamHint, popup.Hint);

            // What Steam's keyboard types arrives as the keyboard's text; A is not taken, Y asks again, Start keeps it.
            s.Window.KeyTextInput(" Deluxe");
            s.Pad.A();
            Assert.True(popup.IsOpen);
            s.Pad.Y();
            Assert.Equal(new[] { "steam://open/keyboard", "steam://open/keyboard" }, NoSteam.Launcher.Opened.Skip(before));
            s.Pad.Start();
            Assert.False(popup.IsOpen);
            Assert.Equal(ThemedSession.SnesGames[0] + " Deluxe", NameBox(editor).Text);

            // B drops what was typed.
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Developer);
            s.Pad.A();
            popup = MenuTextPopup.OpenOver(s.Window)!;
            s.Window.KeyTextInput("Nobody");
            s.Pad.B();
            Assert.False(popup.IsOpen);
            Assert.Equal("", ((TextBox)editor.EditorOf(GameMetadata.Developer)).Text);
            Assert.Same(editor, ThemedGameOptionsTests.Sheets(s).Current);
            Assert.Equal(3, NoSteam.Launcher.Opened.Count - before);
            Assert.All(NoSteam.Launcher.Opened.Skip(before), url => Assert.Equal(DeviceKeyboard.SteamKeyboardUrl, url));
        });

        [Fact]
        public Task EmuSen_s_keyboard_is_still_Mistress_s_own_from_a_pad_or_a_keyboard_and_asks_nothing() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            s.Settings.OnScreenKeyboard = AppSettings.OnScreenKeyboardEmuSen;
            int before = NoSteam.Launcher.Opened.Count;

            ThemedGameOptionsTests.Type(s, "Meta_" + GameMetadata.Genre, "puzzle");
            Assert.Equal("puzzle", ((TextBox)editor.EditorOf(GameMetadata.Genre)).Text);
            Assert.Null(MenuTextPopup.OpenOver(s.Window));

            s.Pad.Unplug();
            s.Pad.Tick();
            EsdeMenusTests.Press(s, Key.Enter);
            Assert.NotNull(OnScreenKeyboard.OpenOver(s.Window));
            Assert.Null(MenuTextPopup.OpenOver(s.Window));
            OnScreenKeyboard.OpenOver(s.Window)!.Cancel();
            Assert.Equal(before, NoSteam.Launcher.Opened.Count);
        }, a => a.OnScreenKeyboard = AppSettings.OnScreenKeyboardEmuSen);

        [Fact]
        public Task Automatic_with_a_pad_and_no_Steam_keeps_Mistress_s_keyboard() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            ThemedGameOptionsTests.OpenEditor(s);
            Assert.Equal(AppSettings.OnScreenKeyboardAutomatic, s.Settings.OnScreenKeyboard);
            s.Pad.A();
            Assert.NotNull(OnScreenKeyboard.OpenOver(s.Window));
            Assert.Null(MenuTextPopup.OpenOver(s.Window));
            s.Pad.Start();
        });

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

        [Fact]
        public void The_subtitle_is_the_file_and_its_system_in_capitals()
        {
            Assert.Equal("Aurora Drift (Synthetic).sfc [SNES]", MetadataEditorWindow.SubtitleOf("/roms/snes/Aurora Drift (Synthetic).sfc"));
            Assert.Equal("Kestrel Run.nes [NES]", MetadataEditorWindow.SubtitleOf("/roms/Kestrel Run.nes"));
            Assert.Equal("Hollow Comet.gb [GB]", MetadataEditorWindow.SubtitleOf("/roms/Hollow Comet.gb"));
            Assert.Equal("notes.txt", MetadataEditorWindow.SubtitleOf("/roms/notes.txt"));
        }

        private static string[] Labels(MenuPanel menu) => menu.Hints!.Select(h => h.Label).ToArray();

        private static PadGlyphButton[] Buttons(MenuPanel menu) => menu.Hints!.Select(h => h.Button!.Value).ToArray();

        [Fact]
        public Task The_editor_s_help_bar_names_what_A_does_on_the_focused_row_in_ES_DE_s_words() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            MenuPanel menu = Menu(editor);
            // A text row, in ES-DE's order: A, B, Y, then the pad.
            Assert.Equal(new[] { "Select", "Back", "Scrape", "Choose" }, Labels(menu));
            Assert.Equal(new[] { PadGlyphButton.South, PadGlyphButton.East, PadGlyphButton.North, PadGlyphButton.DPadUpDown }, Buttons(menu));

            (string Row, string[] Words)[] expected =
            [
                ("Meta_" + GameMetadata.Rating, ["Add Half Star", "Back", "Scrape", "Change", "Choose"]),
                ("Meta_" + GameMetadata.ReleaseDate, ["Edit Date", "Back", "Scrape", "Change", "Choose"]),
                ("Meta_" + GameMetadata.Developer, ["Select", "Back", "Scrape", "Choose"]),
                ("Meta_favourite", ["Toggle", "Back", "Scrape", "Choose"]),
                ("Meta_" + GameMetadata.Completed, ["Toggle", "Back", "Scrape", "Choose"]),
                ("Meta_" + GameMetadata.Controller, ["Select", "Back", "Scrape", "Change", "Choose"]),
                ("MetadataScrape", ["Scrape", "Back", "Scrape", "Choose"]),
                ("MetadataSave", ["Save Metadata", "Back", "Scrape", "Choose"]),
                ("MetadataCancel", ["Cancel Changes", "Back", "Scrape", "Choose"]),
                ("MetadataClear", ["Clear Metadata", "Back", "Scrape", "Choose"]),
                ("MetadataHide", ["Hide Game", "Back", "Scrape", "Choose"]),
            ];
            foreach ((string row, string[] words) in expected)
            {
                ThemedGameOptionsTests.Reach(s, row);
                Assert.True(words.SequenceEqual(Labels(menu)), $"{row}: {string.Join(", ", Labels(menu))}");
            }
            // On the buttons, Choose is Left and Right, as ES-DE's is.
            Assert.Equal(PadGlyphButton.DPadLeftRight, menu.Hints![^1].Button);

            // An edited switch adds Reset on West after Scrape.
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Completed);
            s.Pad.A();
            Assert.Equal(new[] { "Toggle", "Back", "Scrape", "Reset", "Choose" }, Labels(menu));
            Assert.Equal(PadGlyphButton.West, menu.Hints![3].Button);
        });

        [Fact]
        public Task A_on_the_stars_adds_half_a_star_and_past_five_starts_from_none() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            var stars = (RatingPicker)editor.EditorOf(GameMetadata.Rating);
            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Rating);
            s.Pad.A();
            Assert.Equal(0.1, stars.Value, 3);
            for (int i = 0; i < 9; i++) s.Pad.A();
            Assert.Equal(1.0, stars.Value, 3);
            Assert.Equal("1", editor.Draft.Value(GameMetadata.Rating));
            s.Pad.A();
            Assert.Equal(0.0, stars.Value, 3);
            Assert.True(stars.IsFocused);
            Assert.Same(editor, ThemedGameOptionsTests.Sheets(s).Current);
        });

        // ---- Q107 ----

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_editor_shows_the_scroll_indicator_at_its_title_s_right_while_its_rows_run_past_the_panel(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            MenuPanel menu = Menu(editor);
            double u = height / 800.0;
            Assert.Equal(MenuScrollIndicator.Down, menu.ScrollIndicator);
            (Rect up, Rect down) = menu.ScrollIndicatorBounds;
            Rect panel = menu.PanelBounds;
            Assert.Equal(panel.Right - 11 * u, down.Right, 0.5);
            Assert.Equal(25 * u, down.Width, 0.5);
            Assert.Equal(panel.Top + 49 * u, (up.Top + down.Bottom) / 2, 0.5);
            AssertDrawnOnlyIn(s, menu, down, up);

            ThemedGameOptionsTests.Reach(s, "Meta_" + GameMetadata.Completed);
            Assert.Equal(MenuScrollIndicator.Both, menu.ScrollIndicator);
            AssertDrawnOnlyIn(s, menu, up, down);

            // Scrolled to the last row, only the upper pair.
            var rows = (ScrollViewer)menu.Child!;
            rows.Offset = new Vector(0, rows.Extent.Height);
            s.Settle();
            Assert.Equal(MenuScrollIndicator.Up, menu.ScrollIndicator);
            AssertDrawnOnlyIn(s, menu, up, down);
        }, default);

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
            var pad = s.Window.GetControl<MenuPanel>("PadMenuBig");
            Assert.True(pad.IsVisible);
            // Its entries run past the panel at 800 lines, as the pictures of §32 show; Up from the first wraps to the last.
            Assert.Equal(MenuScrollIndicator.Down, pad.ScrollIndicator);
            Assert.NotNull(pad.Child!.GetVisualDescendants().OfType<ScrollViewer>().FirstOrDefault());
            s.Pad.Up();
            s.Settle();
            Assert.Equal(MenuScrollIndicator.Up, pad.ScrollIndicator);
        });

        // ---- the help glyphs ----

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_help_bars_draw_lettered_discs_in_menus_and_in_the_themed_view(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Run(300);
            HintBar themed = s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>().First(b => b.IsVisible);
            Assert.Equal(PadGlyphStyle.Filled, themed.GlyphStyle);
            AssertDiscs(s, themed);

            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            Assert.Equal(PadGlyphStyle.Filled, Menu(editor).HelpBar.GlyphStyle);
            AssertDiscs(s, Menu(editor).HelpBar);
        }, default);

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
