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

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task Theme_settings_is_a_sheet_of_three_tabs_and_B_closes_it(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedLibraryFlowTests.Choose(s, "Themes");
            s.Settle();
            Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
            TabControl tabs = Named<TabControl>(s, "ThemeSettingsTabs");
            Assert.Equal(["Options", "Themes", "Interface"], tabs.Items.OfType<TabItem>().Select(t => t.Header as string));

            for (int i = 0; i < 3 && (tabs.SelectedItem as TabItem)?.Header as string != "Interface"; i++) s.Pad.R1();
            s.Settle();
            Assert.Equal("Display clock", Named<LunaSwitch>(s, "DisplayClock").Label);
            Assert.NotNull(Named<Dropdown>(s, "LaunchScreenDuration"));
            Assert.NotNull(Named<Slider>(s, "ScreensaverTimer"));
            ThemedSwitchesTests.PutAway(s);
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
            // Then Open replacement, the runtime's FirmwarePreference where the mockup draws it (decided 2026-11-03, EmuSen_Cores.md §3.13), the folder, its clearing and opening, and Check again.
            Assert.Equal(["fwreplacement", "fwfolder", "fwclear", "fwopen", "fwcheck"], layout.Rows.Skip(files.Count).Select(r => r.Key));
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

        // Theme Settings' choices on its tabbed sheet: A opens the dropdown in the window, B leaves it unchanged, and A on a choice sets it.
        [Theory]
        [MemberData(nameof(Sizes))]
        public Task A_on_a_theme_settings_choice_opens_its_dropdown_which_chooses_or_goes_back_unchanged(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedSwitchesTests.OpenInterface(s);
            Reach(s, "LaunchScreenDuration");
            var box = Named<Dropdown>(s, "LaunchScreenDuration");
            s.Pad.A();
            s.Settle();
            Assert.True(box.IsDropDownOpen);
            Assert.Equal(InterfaceSettingsPane.LaunchScreenChoices.Select(c => c.Text), box.Items.Cast<object>().Select(i => (i as ContentControl)?.Content as string ?? i as string));

            // B leaves it unchanged, back on the row.
            s.Pad.B();
            s.Settle();
            Assert.False(box.IsDropDownOpen);
            Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
            Assert.Equal(BigPictureInterface.LaunchDisabled, s.Settings.BigPictureInterface.LaunchScreenDuration);

            // A on a choice sets it.
            s.Pad.A();
            s.Settle();
            s.Pad.Up(2);
            s.Pad.A();
            s.Settle();
            Assert.False(box.IsDropDownOpen);
            Assert.Equal(BigPictureInterface.LaunchBrief, s.Settings.BigPictureInterface.LaunchScreenDuration);
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

            s.Settings.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone;
            s.Pad.Start();
            Assert.Equal(1, menu.OpeningScale);
            s.Pad.B();
            Assert.Contains(InterfaceSettingsPane.OpeningChoices, c => c.Text == "Scale-up");
            Assert.Contains(InterfaceSettingsPane.OpeningChoices, c => c.Text == "None");
        }, default);

    }
}
