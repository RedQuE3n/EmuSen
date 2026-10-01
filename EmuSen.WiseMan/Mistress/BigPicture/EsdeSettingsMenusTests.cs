using System;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.LogicalTree;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Stage 2's other items: the settings sheets as ES-DE menus, ES-DE's list screen for an option row, the menu opening switch and the text popup - see EmuSen_BigPicture.md §34.13 to §34.16.
    [Collection(TestCollections.ProcessGlobals)]
    public class EsdeSettingsMenusTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(EsdeSettingsMenusTests).GetTypeInfo().Assembly);

        public static TheoryData<int, int> Sizes() => EsdeMenusTests.Sizes();

        private static SheetLayer Sheets(ThemedSession s) => s.Window.GetControl<SheetLayer>("Sheets");

        private static Control Sheet(ThemedSession s) => Sheets(s).SheetOf(Sheets(s).Current!)!;

        private static T Named<T>(ThemedSession s, string name) where T : Control =>
            Sheet(s).GetLogicalDescendants().OfType<T>().First(c => c.Name == name);

        private static void Reach(ThemedSession s, string name) => PadAudit.Reach(Sheet(s), s.Pad, e => e is Control { Name: { } n } && n == name);

        private static Control[] Rows(ThemedSession s) => Named<StackPanel>(s, "BigMenuRows").Children.Where(c => c.IsVisible).ToArray();

        private static string LabelOf(Control row) => row switch
        {
            MenuRow r => r.Label ?? "",
            MenuFieldRow f => f.Label ?? "",
            _ => MenuRows.GetLabel(row) ?? (row as ContentControl)?.Content as string ?? "",
        };

        private static MenuRow RowOf(Control c) => c as MenuRow ?? c.GetVisualDescendants().OfType<MenuRow>().First();

        // A sheet's menu is ES-DE's: centred, its first row's bar across the panel, a Back button under the rows, and nothing drawn outside the panel and the help bar.
        private static void AssertEsdeMenu(ThemedSession s, MenuPanel menu, string title)
        {
            Assert.True(SheetLayer.GetChromeless(Sheets(s).Current!));
            Assert.Equal(title, menu.Title);
            Rect panel = EsdeMenusTests.InWindow(menu, menu.PanelBounds, s.Window);
            EsdeMenusTests.AssertCentred(panel, s.Window.GetControl<Control>("ScreenContent").Bounds.Size);
            Assert.Empty(Sheet(s).GetVisualDescendants().OfType<FieldRow>().Where(r => r.IsEffectivelyVisible));
            Button back = Named<Button>(s, "BigMenuBack");
            Assert.True(EsdeMenusTests.InWindow(menu, menu.ButtonsBounds, s.Window).Contains(EsdeMenusTests.InWindow(back, new Rect(back.Bounds.Size), s.Window)));

            var focused = (Control)TopLevel.GetTopLevel(s.Window)!.FocusManager!.GetFocusedElement()!;
            RenderedFrame open = s.Capture();
            MenuRow bar = RowOf(focused);
            Assert.True(bar.IsHighlighted);
            EsdeMenusTests.AssertBarSpans(open, EsdeMenusTests.InWindow(bar, bar.Layout(bar.Bounds.Size).Bar, s.Window), panel, bar.BarColor);

            Rect help = EsdeMenusTests.InWindow(menu.HelpBar, new Rect(menu.HelpBar.Bounds.Size), s.Window);
            menu.IsVisible = false;
            RenderedFrame without = s.Capture();
            menu.IsVisible = true;
            s.Settle();
            ((InputElement)focused).Focus(NavigationMethod.Directional);
            (int inside, int outside) = EsdeMenusTests.Changed(open, without, panel.Inflate(1), help.Inflate(1));
            Assert.True(inside > 1000, $"only {inside} pixels of the panel were drawn");
            Assert.Equal(0, outside);
        }

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task Theme_settings_is_ES_DE_s_menu_its_options_first_and_themes_and_interface_as_submenus(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedLibraryFlowTests.Choose(s, "Theme Settings");
            s.Settle();
            var sheet = Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
            AssertEsdeMenu(s, sheet.Form!.Menu, "Theme Settings");
            string[] labels = Rows(s).Select(LabelOf).ToArray();
            Assert.Equal(["Themes", "Interface"], labels[^2..]);
            Assert.Contains("Transitions", labels);
            Assert.All(Rows(s)[^2..], r => Assert.Equal(MenuRowKind.Submenu, MenuRows.GetKind(r)));

            // A submenu is a screen of its own, titled with its name; B goes back a screen, then closes.
            Reach(s, "BigMenuPage2");
            s.Pad.A();
            Assert.Equal("Interface", sheet.Form.Menu.Title);
            Assert.Equal(1, sheet.Form.Depth);
            Assert.Equal(MenuRowKind.Switch, MenuRows.GetKind(Named<LunaSwitch>(s, "DisplayClock")));
            Assert.Equal("Display clock", MenuRows.GetLabel(Named<LunaSwitch>(s, "DisplayClock")));
            Assert.Equal(MenuRowKind.Option, MenuRows.GetKind(Named<Dropdown>(s, "LaunchScreenDuration")));
            Assert.Equal(MenuRowKind.Option, MenuRows.GetKind(Named<Slider>(s, "ScreensaverTimer")));
            Assert.Equal(s.Settings.BigPictureInterface.ScreensaverTimer == 0 ? "Never" : $"{s.Settings.BigPictureInterface.ScreensaverTimer / 60000} min", RowOf(Named<Slider>(s, "ScreensaverTimer")).Value);
            s.Pad.B();
            Assert.Equal("Theme Settings", sheet.Form.Menu.Title);
            Assert.True(Sheets(s).IsPresenting);
            s.Pad.B();
            Assert.False(Sheets(s).IsPresenting);
        }, default);

        // Game Collection Settings is one screen of rows: ES-DE's words on its switches, a submenu row to create a collection, option rows, and Back.
        [Theory]
        [MemberData(nameof(Sizes))]
        public Task Game_collection_settings_is_one_menu_of_switches_options_and_a_create_row(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedCollectionsTests.Choose(s, "Game Collection Settings");
            s.Settle();
            var sheet = Assert.IsType<CollectionSettingsWindow>(Sheets(s).Current);
            AssertEsdeMenu(s, sheet.Form!.Menu, "Game Collection Settings");
            string[] labels = Rows(s).Select(LabelOf).ToArray();
            Assert.Equal(["All Games", "Favorites", "Last Played"], labels[..3]);
            Assert.Contains("Sort favorite games above non-favorites", labels);
            Assert.Equal(MenuRowKind.Submenu, MenuRows.GetKind(Named<Button>(s, "CollectionCreate")));
            Assert.Equal("Create New Custom Collection", MenuRows.GetLabel(Named<Button>(s, "CollectionCreate")));
            Assert.Equal(MenuRowKind.Option, MenuRows.GetKind(Named<Dropdown>(s, "CollectionGrouping")));

            // A switch row is turned by A, and the setting follows.
            bool before = s.Settings.BigPictureCollections.FavoritesFirst;
            Reach(s, "FavoritesFirst");
            s.Pad.A();
            Assert.Equal(!before, s.Settings.BigPictureCollections.FavoritesFirst);
            Assert.Equal(!before, RowOf(Named<LunaSwitch>(s, "FavoritesFirst")).IsOn);
            s.Pad.B();
            Assert.False(Sheets(s).IsPresenting);
        }, default);

        // Preferences is a menu of its tabs as submenus; R1 still steps from one to the next, and the footer says what the focused row does.
        [Theory]
        [MemberData(nameof(Sizes))]
        public Task Preferences_is_a_menu_of_submenus_which_the_shoulders_still_step_through(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedLibraryFlowTests.Choose(s, "Preferences");
            s.Settle();
            var sheet = Assert.IsType<PreferencesWindow>(Sheets(s).Current);
            AssertEsdeMenu(s, sheet.Form!.Menu, "Preferences");
            Assert.Equal(["Library", "Scraping", "Gameplay", "Appearance", "Controllers", "System Files"], Rows(s).Select(LabelOf));

            Reach(s, "BigMenuPage2");
            s.Pad.A();
            Assert.Equal("Gameplay", sheet.Form.Menu.Title);
            Reach(s, "ResumeDropdown");
            Assert.StartsWith("Every game is saved when it is closed.", sheet.Form.Menu.Footer);
            s.Pad.R1();
            Assert.Equal("Appearance", sheet.Form.Menu.Title);
            Assert.True(Named<Slider>(s, "NavigationVolumeSlider").IsEffectivelyVisible);
            Reach(s, "NavigationVolumeSlider");
            s.Pad.Right();
            Assert.Equal(75, s.Settings.BigPictureInterface.NavigationVolume);
            Assert.Equal("75", RowOf(Named<Slider>(s, "NavigationVolumeSlider")).Value);
            ThemedSwitchesTests.PutAway(s);
            Assert.False(Sheets(s).IsPresenting);
        }, default);

        // Sync to display is a switch on Gameplay, on by default, saved to graphics.json as it turns, and its menu fits - see EmuSen_Settings_Reference.md §4.87.7.
        [Theory]
        [MemberData(nameof(Sizes))]
        public Task Sync_to_display_is_a_gameplay_switch_on_by_default_that_is_saved_and_fits(int width, int height) => Session.Dispatch(() =>
        {
            bool was = EmuSen.Graphics.GraphicsSettings.SyncToDisplay;
            using var s = new ThemedSession(width, height);
            try
            {
                Assert.True(new EmuSen.Galaxia.Models.GraphicsConfig().SyncToDisplay);
                EmuSen.Graphics.GraphicsSettings.SyncToDisplay = true;
                ThemedLibraryFlowTests.Choose(s, "Preferences");
                s.Settle();
                var sheet = Assert.IsType<PreferencesWindow>(Sheets(s).Current);
                Reach(s, "BigMenuPage2");
                s.Pad.A();
                Assert.Equal("Gameplay", sheet.Form!.Menu.Title);
                Reach(s, "SyncToDisplaySwitch");
                Assert.StartsWith("Runs one frame per refresh", sheet.Form.Menu.Footer);
                Assert.True(RowOf(Named<LunaSwitch>(s, "SyncToDisplaySwitch")).IsOn);
                Assert.Empty(FitAudit.Check(Sheet(s), WindowAllowances.For(sheet)));
                s.Pad.A();
                Assert.False(EmuSen.Graphics.GraphicsSettings.SyncToDisplay);
                Assert.False(EmuSen.Galaxia.Models.GraphicsConfig.Load().SyncToDisplay);
                Assert.False(RowOf(Named<LunaSwitch>(s, "SyncToDisplaySwitch")).IsOn);
                s.Pad.A();
                Assert.True(EmuSen.Galaxia.Models.GraphicsConfig.Load().SyncToDisplay);
                ThemedSwitchesTests.PutAway(s);
            }
            finally { EmuSen.Graphics.GraphicsSettings.SyncToDisplay = was; }
        }, default);

        // Q86: A on an option row opens ES-DE's list screen: the row's name as its title, a row per choice with the current one focused, Back; the menu beneath is not drawn.
        [Theory]
        [MemberData(nameof(Sizes))]
        public Task A_on_an_option_row_opens_the_list_screen_which_chooses_or_goes_back_unchanged(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedSwitchesTests.OpenInterface(s);
            Control interfaceSheet = Sheet(s);
            Reach(s, "LaunchScreenDuration");
            s.Pad.A();
            s.Settle();
            var list = Assert.IsType<OptionListWindow>(Sheets(s).Current);
            Assert.Equal("Launch Screen Duration", list.Menu.Title);
            Assert.Equal(InterfaceSettingsPane.LaunchScreenChoices.Select(c => c.Text), list.Choices);
            Assert.False(interfaceSheet.IsVisible);
            Button current = Named<Button>(s, "OptionListChoice4");
            Assert.True(current.IsFocused);
            Assert.True(RowOf(current).IsHighlighted);
            Rect panel = EsdeMenusTests.InWindow(list.Menu, list.Menu.PanelBounds, s.Window);
            EsdeMenusTests.AssertCentred(panel, s.Window.GetControl<Control>("ScreenContent").Bounds.Size);

            // B leaves it unchanged, back on the row.
            s.Pad.B();
            s.Settle();
            Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
            Assert.Equal(BigPictureInterface.LaunchDisabled, s.Settings.BigPictureInterface.LaunchScreenDuration);
            Assert.True(Named<Dropdown>(s, "LaunchScreenDuration").IsFocused);

            // A on a choice sets it and goes back.
            s.Pad.A();
            s.Settle();
            s.Pad.Up(3);
            s.Pad.A();
            s.Settle();
            Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
            Assert.Equal(BigPictureInterface.LaunchBrief, s.Settings.BigPictureInterface.LaunchScreenDuration);
            Assert.Equal("Brief", RowOf(Named<Dropdown>(s, "LaunchScreenDuration")).Value);
            ThemedSwitchesTests.PutAway(s);
        }, default);

        // The same list screen from stage 1's Gamelist Options.
        [Fact]
        public Task The_gamelist_options_sort_row_opens_the_list_screen_too() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            ThemedCollectionsTests.OpenMenu(s);
            PadAudit.Reach(Sheet(s), s.Pad, e => e is Control { Name: "GamelistSortBy" });
            s.Pad.A();
            s.Settle();
            var list = Assert.IsType<OptionListWindow>(Sheets(s).Current);
            Assert.Equal("Sort Games By", list.Menu.Title);
            s.Pad.Down();
            s.Pad.A();
            s.Settle();
            Assert.IsType<GameOptionsWindow>(Sheets(s).Current);
            Assert.Equal(1, Named<Dropdown>(s, "GamelistSortBy").SelectedIndex);
        }, default);

        // Q92: Scale-up grows a menu from half its size to its own over 117 ms, linearly; None draws it whole at once, and the launch screen's card follows the same switch.
        [Fact]
        public Task The_menu_opening_grows_a_menu_over_117_ms_or_not_at_all() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            Assert.Equal(BigPictureInterface.OpeningScaleUp, new BigPictureInterface().MenuOpeningEffect);
            s.Settings.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningScaleUp;
            ThemedLibraryPadTests.Enter(s, "snes");
            MenuPanel menu = s.Window.GetControl<MenuPanel>("PadMenuBig");
            s.Pad.Start();
            double first = menu.OpeningScale;
            Assert.InRange(first, 0.5, 0.95);
            s.Run(40, 8);
            Assert.Equal(Math.Min(1, first + 0.5 * 40 / 117.0), menu.OpeningScale, 0.02);
            s.Run(120);
            Assert.Equal(1, menu.OpeningScale);
            s.Pad.B();

            // A sheet's menu, and a submenu turned to, grow the same way.
            ThemedLibraryFlowTests.Choose(s, "Theme Settings");
            var sheet = (ThemeSettingsWindow)Sheets(s).Current!;
            Assert.True(sheet.Form!.Menu.OpeningScale < 1);
            s.Run(150);
            Assert.Equal(1, sheet.Form.Menu.OpeningScale);
            Reach(s, "BigMenuPage2");
            s.Pad.A();
            Assert.True(sheet.Form.Menu.OpeningScale < 1);
            s.Run(150);
            ThemedSwitchesTests.PutAway(s);

            s.Settings.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone;
            s.Pad.Start();
            Assert.Equal(1, menu.OpeningScale);
            s.Pad.B();
            Assert.Contains(InterfaceSettingsPane.OpeningChoices, c => c.Text == "Scale-up");
            Assert.Contains(InterfaceSettingsPane.OpeningChoices, c => c.Text == "None");
        }, default);

        // Q92 for the launch screen: with None its card is whole from the first frame.
        [Fact]
        public Task With_none_the_launch_screen_card_is_whole_at_once() => Session.Dispatch(() =>
        {
            using var s = LaunchScreenTests.Open(BigPictureInterface.LaunchNormal, more: a => a.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            LaunchScreen screen = LaunchScreenTests.Screen(s)!;
            Assert.Equal(1, screen.CardScale, 3);
            Assert.False(screen.ScalesUp);
            s.Run(3200);
        }, default);

        // Q100: a text row in a big-screen menu opens ES-DE's text popup, titled with the row's name, and it types as the keyboard did.
        [Fact]
        public Task A_text_row_opens_the_text_popup_titled_with_its_name() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            MetadataEditorWindow editor = ThemedGameOptionsTests.OpenEditor(s);
            ThemedGameOptionsTests.Reach(s, "Meta_" + EmuSen.Mistress.Library.GameMetadata.Developer);
            s.Pad.A();
            OnScreenKeyboard keyboard = OnScreenKeyboard.OpenOver(s.Window)!;
            Assert.True(keyboard.MenuLook);
            Assert.Equal("Enter Developer", keyboard.Title);
            PadCheatsTests.TypeByPad(s.Pad, keyboard, "abc");
            s.Pad.Start();
            Assert.Null(OnScreenKeyboard.OpenOver(s.Window));
            Assert.Equal("abc", ((TextBox)editor.EditorOf(EmuSen.Mistress.Library.GameMetadata.Developer)).Text);
        });
    }
}
