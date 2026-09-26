using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // EmuSen's own look as the first, built-in entry of the Themes list, and Preferences' Big Picture Theme row over the same setting - see EmuSen_BigPicture.md §19.
    [Collection(TestCollections.ProcessGlobals)]
    public class BigPictureThemeListTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(BigPictureThemeListTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public BigPictureThemeListTests(ITestOutputHelper output) => _out = output;

        private static SheetLayer Sheets(ThemedSession s) => s.Window.GetControl<SheetLayer>("Sheets");

        private static Control SheetRoot(ThemedSession s) => Sheets(s).SheetOf(Sheets(s).Current!)!;

        private static T Named<T>(ThemedSession s, string name) where T : Control =>
            SheetRoot(s).GetLogicalDescendants().OfType<T>().FirstOrDefault(c => c.Name == name) ?? throw new InvalidOperationException($"no {typeof(T).Name} {name}");

        private static bool Has(ThemedSession s, string name) => SheetRoot(s).GetLogicalDescendants().OfType<Control>().Any(c => c.Name == name);

        private static bool LibraryContentShown(ThemedSession s) => s.Window.GetControl<Control>("LibraryContent").IsVisible;

        private static TabControl TabsOf(ThemedSession s) => SheetRoot(s).GetVisualDescendants().OfType<TabControl>().First();

        private static void Click(Button button) => button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

        // The pad's route: the pad menu's Theme Settings, then the Themes tab, then down to the entry's Use and A.
        private static ThemeSettingsWindow OpenThemes(ThemedSession s)
        {
            ThemedLibraryFlowTests.Choose(s, "Theme Settings");
            s.Settle();
            var sheet = Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
            for (int i = 0; i < 3 && (TabsOf(s).SelectedItem as TabItem)?.Header as string != "Themes"; i++) s.Pad.R1();
            Assert.Equal("Themes", (TabsOf(s).SelectedItem as TabItem)?.Header);
            s.Window.UpdateLayout();
            return sheet;
        }

        private static void UseByPad(ThemedSession s, string button)
        {
            PadAudit.Reach(SheetRoot(s), s.Pad, e => e is Button b && b.Name == button);
            s.Pad.A();
            s.Settle();
        }

        // The Themes tab's rows: those holding a Use button.
        private static FieldRow[] ThemeRows(ThemedSession s) =>
            SheetRoot(s).GetLogicalDescendants().OfType<FieldRow>().Where(r => r.GetLogicalDescendants().OfType<Button>().Any(b => b.Name?.StartsWith("ThemeUse") == true)).ToArray();

        private static string OwnFolder(ThemedSession s) => Path.GetFileName(s.Theme.Root);

        // The list: EmuSen first and built in, the theme after it, and the one current marked In Use.
        [Fact]
        public Task EmuSen_is_the_first_entry_of_the_themes_list_and_the_current_one_is_marked() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            OpenThemes(s);
            FieldRow[] rows = ThemeRows(s);
            Assert.Equal("ThemeRowBuiltIn", rows[0].Name);
            Assert.Equal("EmuSen (built in)", rows[0].Label);
            Assert.Equal("Use", Named<Button>(s, "ThemeUseBuiltIn").Content);
            Assert.True(Named<Button>(s, "ThemeUseBuiltIn").IsEnabled);
            Button theme = Named<Button>(s, $"ThemeUse.{OwnFolder(s)}");
            Assert.Equal(("In Use", false), (theme.Content, theme.IsEnabled));
            _out.WriteLine(string.Join(" | ", rows.Select(r => r.Label)));
        }, default);

        // Choosing EmuSen swaps the view beneath the open sheet at once, empties the Options tab, and is remembered by a new window.
        [Fact]
        public Task Choosing_EmuSen_swaps_the_view_at_once_and_is_remembered() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            Assert.True(s.Shown);
            OpenThemes(s);
            UseByPad(s, "ThemeUseBuiltIn");

            Assert.True(Sheets(s).IsPresenting);
            Assert.False(s.Shown);
            Assert.True(LibraryContentShown(s));
            Assert.Equal(("In Use", false), (Named<Button>(s, "ThemeUseBuiltIn").Content, Named<Button>(s, "ThemeUseBuiltIn").IsEnabled));
            Assert.Equal("Use", Named<Button>(s, $"ThemeUse.{OwnFolder(s)}").Content);
            Assert.True(Has(s, "BuiltInOptionsNote"));
            AppSettings stored = AppSettings.Load();
            Assert.Equal(AppSettings.LibraryStyleMistress, stored.LibraryStyle);
            Assert.True(ThemeDownloads.SamePath(s.Theme.Root, stored.BigPictureTheme!));

            s.Pad.B();
            s.Settle();
            Assert.False(Sheets(s).IsPresenting);
            Assert.False(s.Shown);

            var again = new MainWindow { Width = 1280, Height = 800 };
            try
            {
                again.Show();
                Dispatcher.UIThread.RunJobs();
                again.UpdateLayout();
                Dispatcher.UIThread.RunJobs();
                Assert.True(again.GetControl<Control>("LibraryView").IsVisible);
                Assert.False((bool)typeof(MainWindow).GetProperty("ThemedLibraryShown", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(again)!);
                Assert.True(again.GetControl<Control>("LibraryContent").IsVisible);
            }
            finally { again.Close(); }
        }, default);

        // From EmuSen's own look the pad menu still offers the sheet, it opens on the Themes tab, and choosing the ES-DE theme swaps back at once.
        [Fact]
        public Task From_EmuSen_s_look_the_pad_reaches_the_sheet_and_an_ES_DE_theme_swaps_back() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a => a.LibraryStyle = AppSettings.LibraryStyleMistress);
            Assert.False(s.Shown);
            Assert.True(LibraryContentShown(s));
            ThemedLibraryFlowTests.Choose(s, "Theme Settings");
            s.Settle();
            Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
            Assert.Equal("Themes", (TabsOf(s).SelectedItem as TabItem)?.Header);
            Assert.Equal("In Use", Named<Button>(s, "ThemeUseBuiltIn").Content);
            s.Window.UpdateLayout();
            UseByPad(s, $"ThemeUse.{OwnFolder(s)}");

            Assert.True(Sheets(s).IsPresenting);
            Assert.True(s.Shown);
            Assert.False(LibraryContentShown(s));
            Assert.Equal(("Use", true), (Named<Button>(s, "ThemeUseBuiltIn").Content, Named<Button>(s, "ThemeUseBuiltIn").IsEnabled));
            Assert.False(Has(s, "BuiltInOptionsNote"));
            Assert.Equal(AppSettings.LibraryStyleTheme, AppSettings.Load().LibraryStyle);

            s.Pad.B();
            s.Settle();
            Assert.True(s.Shown);
            s.Pad.A();
            Assert.Equal("gamelist", s.View);
        }, default);

        // The pad menu offers Theme Settings over both looks, and the sheet's every control is reached by the pad over EmuSen's.
        [Theory]
        [InlineData(AppSettings.LibraryStyleTheme)]
        [InlineData(AppSettings.LibraryStyleMistress)]
        public Task The_sheet_is_reached_by_the_pad_from_both_looks(string style) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a => a.LibraryStyle = style);
            Assert.Equal(style == AppSettings.LibraryStyleTheme, s.Shown);
            ThemedLibraryFlowTests.Choose(s, "Theme Settings");
            s.Settle();
            Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
            Assert.Empty(s.Window.OwnedWindows);

            TabControl tabs = TabsOf(s);
            var missing = new System.Collections.Generic.List<string>();
            for (int page = 0; page < tabs.ItemCount; page++)
            {
                if (page > 0) s.Pad.R1();
                s.Window.UpdateLayout();
                var reached = PadAudit.Reachable(SheetRoot(s), s.Pad);
                missing.AddRange(PadAudit.Operable(SheetRoot(s)).Where(c => !reached.Contains(c)).Select(c => $"[{(tabs.SelectedItem as TabItem)?.Header}] {PadAudit.Describe(c)}"));
            }
            foreach (string m in missing) _out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);
            s.Pad.B();
            Assert.False(Sheets(s).IsPresenting);
        }, default);

        // The built-in row offers only Use; removing the theme in use falls back to it, and it stays first in the list.
        [Fact]
        public Task The_built_in_entry_cannot_be_removed_and_is_what_a_removed_theme_falls_back_to() => Session.Dispatch(() =>
        {
            using var own = new SyntheticTheme();
            ThemedSession.Write(own);
            string downloaded = "";
            using var s = new ThemedSession(themeDirectory: own.Root, settings: a =>
            {
                downloaded = Path.Combine(DataStore.Themes, "synthetic-book");
                Copy(own.Root, downloaded);
                File.WriteAllText(Path.Combine(downloaded, ThemeDownloads.StampFile),
                    JsonSerializer.Serialize(new ThemeStamp("someone", "synthetic-book", "main", ThemeDownloadsTests.Sha1, new DateTime(2026, 9, 26))));
                a.BigPictureTheme = downloaded;
            });
            Assert.True(s.Shown);
            OpenThemes(s);

            FieldRow builtIn = Named<FieldRow>(s, "ThemeRowBuiltIn");
            Assert.Equal(new[] { "ThemeUseBuiltIn" }, builtIn.GetLogicalDescendants().OfType<Button>().Select(b => b.Name));
            Assert.True(Has(s, "ThemeRemove.synthetic-book"));
            Assert.Single(BigPictureLooks.All(AppSettings.Load()), l => l.BuiltIn);

            Click(Named<Button>(s, "ThemeRemove.synthetic-book"));
            s.Settle();
            var confirm = Assert.IsAssignableFrom<Window>(Sheets(s).Current);
            Assert.IsNotType<ThemeSettingsWindow>(confirm);
            Click(SheetRoot(s).GetLogicalDescendants().OfType<Button>().First(b => b.Content as string == "Remove"));
            s.Settle();
            Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);

            Assert.False(Directory.Exists(downloaded));
            Assert.False(Has(s, "ThemeRemove.synthetic-book"));
            Assert.Equal(("In Use", false), (Named<Button>(s, "ThemeUseBuiltIn").Content, Named<Button>(s, "ThemeUseBuiltIn").IsEnabled));
            Assert.Equal("ThemeRowBuiltIn", ThemeRows(s).First().Name);
            Assert.False(s.Shown);
            Assert.True(LibraryContentShown(s));
            Assert.Equal(new[] { BigPictureLooks.BuiltIn }, BigPictureLooks.All(AppSettings.Load()));
        }, default);

        // Preferences' Big Picture Theme row lists the same entries, writes the same setting at once, and follows a choice made on the sheet over it.
        [Fact]
        public Task Preferences_and_the_themes_list_agree() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            string themeName = BigPictureLooks.All(AppSettings.Load())[1].Name;
            ThemedLibraryFlowTests.Choose(s, "Preferences");
            Assert.IsType<PreferencesWindow>(Sheets(s).Current);
            TabControl tabs = TabsOf(s);
            for (int i = 0; i < 6 && (tabs.SelectedItem as TabItem)?.Header as string != "Appearance"; i++) s.Pad.R1();
            s.Window.UpdateLayout();
            Dropdown row = Named<Dropdown>(s, "BigPictureThemeDropdown");
            Assert.Equal(new[] { "EmuSen (built in)", themeName }, ((System.Collections.IEnumerable)row.ItemsSource!).Cast<string>());
            Assert.Equal(themeName, row.SelectedItem);
            Assert.False(Has(s, "LibraryStyleDropdown"));

            PadAudit.Reach(SheetRoot(s), s.Pad, e => e is Dropdown { Name: "BigPictureThemeDropdown" });
            s.Pad.Left();
            s.Settle();
            Assert.Equal("EmuSen (built in)", row.SelectedItem);
            Assert.Equal(AppSettings.LibraryStyleMistress, AppSettings.Load().LibraryStyle);
            Assert.False(s.Shown);

            Click(Named<Button>(s, "ThemeSettingsButton"));
            s.Settle();
            Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
            Assert.Equal("Themes", (TabsOf(s).SelectedItem as TabItem)?.Header);
            Assert.Equal("In Use", Named<Button>(s, "ThemeUseBuiltIn").Content);
            Click(Named<Button>(s, $"ThemeUse.{OwnFolder(s)}"));
            s.Settle();
            Assert.True(s.Shown);

            s.Pad.B();
            s.Settle();
            Assert.IsType<PreferencesWindow>(Sheets(s).Current);
            Assert.Equal(themeName, Named<Dropdown>(s, "BigPictureThemeDropdown").SelectedItem);
            Assert.True(ThemeDownloads.SamePath(s.Theme.Root, Named<PathPickerRow>(s, "BigPictureThemeBox").Path));
            s.Pad.B();
            s.Settle();
            Assert.False(Sheets(s).IsPresenting);
            Assert.True(s.Shown);
        }, default);

        private static void Copy(string from, string to)
        {
            foreach (string dir in Directory.GetDirectories(from, "*", SearchOption.AllDirectories)) Directory.CreateDirectory(dir.Replace(from, to));
            Directory.CreateDirectory(to);
            foreach (string file in Directory.GetFiles(from, "*", SearchOption.AllDirectories)) File.Copy(file, file.Replace(from, to));
        }
    }
}
