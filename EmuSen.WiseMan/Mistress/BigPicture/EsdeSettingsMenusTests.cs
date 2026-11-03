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
            EsdeMenusTests.AssertBarSpans(open, EsdeMenusTests.InWindow(bar, bar.Layout(bar.Bounds.Size).Bar, s.Window), panel, bar.BarColor, bar.AccentColor);

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

        // The collection settings are Settings ▸ Library ▸ Collections: the create row, the automatic collections, option rows, and B back to the library.
        [Theory]
        [MemberData(nameof(Sizes))]
        public Task Game_collection_settings_are_a_page_of_settings_with_a_create_row(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedCollectionsTests.Choose(s, "Collections");
            s.Settle();
            EmuSen.Mistress.Pages.SettingsScreen settings = s.Window.SettingsShown!;
            Assert.Equal(["Library", "Collections"], settings.Trail);
            string[] keys = settings.Rows.Select(r => r.Key).ToArray();
            Assert.Equal(["newcollection", "autoall", "autofavorites", "autorecent"], keys[..4]);
            Assert.Equal(PageRowKind.Asks, settings.Rows.Single(r => r.Key == "newcollection").Kind);
            Assert.Equal(PageRowKind.Steps, settings.Rows.Single(r => r.Key == "grouping").Kind);

            // A switch row is turned by A, and the setting follows.
            bool before = s.Settings.BigPictureCollections.FavoritesFirst;
            s.Pad.Down(keys.ToList().IndexOf("favfirst"));
            Assert.Equal("favfirst", s.Window.SettingsLayoutShown!.Focused);
            s.Pad.A();
            Assert.Equal(!before, s.Settings.BigPictureCollections.FavoritesFirst);
            s.Pad.B();
            s.Pad.B();
            s.Settle();
            Assert.Null(s.Window.SettingsShown);
        }, default);

        // Settings under a theme: the pad menu's Settings opens it, the shoulders step its sections, and the pane says what the focused row does (EmuSen_Interface.md §2.7).
        private static SettingsLayout OpenSettings(ThemedSession s, int sections)
        {
            ThemedLibraryFlowTests.Choose(s, MainWindow.SettingsMenu);
            s.Settle();
            for (int i = 0; i < sections; i++) s.Pad.R1();
            s.Settle();
            return s.Window.SettingsLayoutShown!;
        }

        private static void ToRow(ThemedSession s, string key)
        {
            SettingsLayout layout = s.Window.SettingsLayoutShown!;
            List<string> keys = layout.Rows.Select(r => r.Key).ToList();
            Assert.Contains(key, keys);
            int by = keys.IndexOf(key) - keys.IndexOf(layout.Focused!);
            if (by > 0) s.Pad.Down(by); else if (by < 0) s.Pad.Up(-by);
            Assert.Equal(key, layout.Focused);
        }

        private static string PaneWords(ThemedSession s) =>
            string.Join(" ", s.Window.SettingsLayoutShown!.GetVisualDescendants().OfType<TextBlock>().Where(t => t.Name == "SettingsLine").Select(t => t.Text ?? string.Concat(t.Inlines!.OfType<Avalonia.Controls.Documents.Run>().Select(r => r.Text))));

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task Settings_steps_its_sections_with_the_shoulders_and_its_pane_says_what_a_row_does(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            SettingsLayout layout = OpenSettings(s, 0);
            Assert.Equal(["Controllers", "Display", "Sound", "Consoles", "Library", "EmuSen"], layout.Sections.Select(x => x.Label));
            Assert.Equal(EmuSen.Mistress.Pages.SettingsScreen.Controllers, layout.SelectedSection);
            s.Pad.R1();
            Assert.Equal(EmuSen.Mistress.Pages.SettingsScreen.Display, layout.SelectedSection);
            s.Pad.R1();
            ToRow(s, "soundvolume");
            Assert.StartsWith("How loud the navigation sounds are", PaneWords(s));
            int before = s.Settings.BigPictureInterface.NavigationVolume;
            s.Pad.Right();
            Assert.Equal(Math.Min(100, before + 5), s.Settings.BigPictureInterface.NavigationVolume);
            Assert.Equal($"{Math.Min(100, before + 5)}%", layout.Rows.Single(r => r.Key == "soundvolume").Status);
            Assert.Empty(FitAudit.Check(layout, []));
            s.Pad.B();
            s.Settle();
            Assert.Null(s.Window.SettingsShown);
        }, default);

        // Firmware is Consoles ▸ the console ▸ Firmware: a row per need the runtime's firmware_status names, its status as the value, the folder and Check again; A on a file's row changes nothing (EmuSen_Interface.md §18.10).
        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_firmware_page_under_a_theme_is_rows_of_the_runtime_s_firmware_status(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            SettingsLayout layout = OpenSettings(s, 3);
            ToRow(s, layout.Rows.Single(r => r.Label == "PlayStation").Key);
            s.Pad.A();
            ToRow(s, "firmware");
            s.Pad.A();
            s.Settle();
            Assert.Equal(["Consoles", "PlayStation", "Firmware"], layout.Trail);
            int opened = EmuSen.WiseMan.Mistress.NoSteam.Launcher.Opened.Count;
            var files = layout.Rows.Where(r => r.Key.StartsWith("fw:", StringComparison.Ordinal)).ToList();
            Assert.NotEmpty(files);
            // The PlayStation's regional BIOS files are one need, found with the copy recognised or not found, never Not needed.
            PageRow bios = Assert.Single(files);
            Assert.Equal("PlayStation BIOS", bios.Label);
            Assert.True(bios.Status is "Not found" or "⚠ Not found" || bios.Status!.StartsWith("✓ Found · ", StringComparison.Ordinal), bios.Status);
            // Then Open replacement, the runtime's FirmwarePreference where the mockup draws it (decided 2026-11-03, EmuSen_Cores.md §3.13), the folder and Check again.
            Assert.Equal(["fwreplacement", "fwfolder", "fwcheck"], layout.Rows.Skip(files.Count).Select(r => r.Key));
            Assert.Equal("Open replacement", layout.Rows.Single(r => r.Key == "fwreplacement").Label);
            foreach (PageRow file in files)
            {
                ToRow(s, file.Key);
                Assert.False(string.IsNullOrEmpty(PaneWords(s)));
                s.Pad.A();
                Assert.Equal(["Consoles", "PlayStation", "Firmware"], layout.Trail);
                Assert.Empty(FitAudit.Check(layout, []));
            }
            Assert.Equal(opened, EmuSen.WiseMan.Mistress.NoSteam.Launcher.Opened.Count);
            Assert.Null(EmuSen.Galaxia.Models.AppSettings.Load().FirmwareDirectory);
        }, default);

        // Smooth motion is Display ▸ Smooth motion, on by default, saved to graphics.json as it turns without losing what else the file holds - see EmuSen_Settings_Reference.md §4.87.7.
        [Theory]
        [MemberData(nameof(Sizes))]
        public Task Smooth_motion_is_a_display_switch_on_by_default_that_is_saved_and_fits(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            var frame = s.Window.GetControl<EmuSen.Serenity.GameFrameControl>("GameFrame");
            Assert.True(new EmuSen.Galaxia.Models.GraphicsConfig().SyncToDisplay);
            Assert.True(frame.SyncToDisplay);
            SettingsLayout layout = OpenSettings(s, 1);
            ToRow(s, "smooth");
            Assert.StartsWith("One frame per refresh", PaneWords(s));
            Assert.Equal("On", layout.Rows.Single(r => r.Key == "smooth").Status);
            Assert.Empty(FitAudit.Check(layout, []));
            var held = (EmuSen.Galaxia.Models.GraphicsConfig)typeof(MainWindow).GetField("_graphics", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!.GetValue(s.Window)!;
            held.SetValue("NES", "Engine", "MoonRT (Rust)");
            s.Pad.Right();
            Assert.Equal("MoonRT (Rust)", EmuSen.Galaxia.Models.GraphicsConfig.Load().Value("NES", "Engine"));
            Assert.False(frame.SyncToDisplay);
            Assert.False(EmuSen.Galaxia.Models.GraphicsConfig.Load().SyncToDisplay);
            Assert.Equal("Off", layout.Rows.Single(r => r.Key == "smooth").Status);
            s.Pad.Right();
            Assert.True(EmuSen.Galaxia.Models.GraphicsConfig.Load().SyncToDisplay);
            Assert.True(frame.SyncToDisplay);
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
            Assert.IsType<GamelistOptionsWindow>(Sheets(s).Current);
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
            MenuPanel menu = PadMenu.Big(s.Window);
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

    }
}
