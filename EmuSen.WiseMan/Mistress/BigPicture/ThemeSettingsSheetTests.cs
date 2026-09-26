using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The theme's settings sheet from capabilities.xml, its choices applied at once and kept per theme, the themes tab's downloads, and what a closed sheet leaves running - see EmuSen_BigPicture.md §16.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemeSettingsSheetTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemeSettingsSheetTests).GetTypeInfo().Assembly);

        private static readonly FieldInfo Factory = typeof(MainWindow).GetField("HttpFactory", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realFactory = Factory.GetValue(null)!;
        private readonly ITestOutputHelper _out;

        public ThemeSettingsSheetTests(ITestOutputHelper output) => _out = output;

        public void Dispose() => Factory.SetValue(null, _realFactory);

        private const string Capabilities =
            "<variant name=\"narrow\"><label>Narrow list</label></variant>" +
            "<variant name=\"wide\"><label>Wide list</label></variant>" +
            "<variant name=\"hidden\"><label>Hidden</label><selectable>false</selectable></variant>" +
            "<colorScheme name=\"dark\"><label>Dark</label></colorScheme>" +
            "<colorScheme name=\"light\"><label language=\"de_DE\">Hell</label><label language=\"en_US\">Light</label></colorScheme>" +
            "<fontSize>small</fontSize><fontSize>medium</fontSize>" +
            "<aspectRatio>4:3</aspectRatio><aspectRatio>16:10</aspectRatio>" +
            "<transitions name=\"glide\"><label>Glide</label><systemToGamelist>slide</systemToGamelist></transitions>" +
            "<suppressTransitionProfiles><entry>builtin-fade</entry></suppressTransitionProfiles>";

        // The session's theme, with two variants that move the list, two schemes that colour a backdrop, and two font sizes.
        private static void Write(SyntheticTheme theme, string capabilities = Capabilities, string extraBlocks = "")
        {
            ThemedSession.Write(theme);
            theme.Capabilities(capabilities);
            string xml = File.ReadAllText(theme.PathOf("theme.xml"));
            string blocks =
                "<variables><bg>404040</bg><listSize>0.030</listSize></variables>" +
                "<colorScheme name=\"dark\"><variables><bg>101030</bg></variables></colorScheme>" +
                "<colorScheme name=\"light\"><variables><bg>E0E0C0</bg></variables></colorScheme>" +
                "<fontSize name=\"small\"><variables><listSize>0.030</listSize></variables></fontSize>" +
                "<fontSize name=\"medium\"><variables><listSize>0.045</listSize></variables></fontSize>" +
                "<view name=\"gamelist\"><image name=\"backdrop\"><pos>0 0</pos><size>1 1</size><path>./art/snes.png</path><color>${bg}</color><saturation>0</saturation><zIndex>1</zIndex></image>" +
                "<textlist name=\"gamelist\"><fontSize>${listSize}</fontSize></textlist></view>" +
                "<variant name=\"wide\"><view name=\"gamelist\"><textlist name=\"gamelist\"><pos>0.3 0.08</pos></textlist></view></variant>" + extraBlocks;
            File.WriteAllText(theme.PathOf("theme.xml"), xml.Replace("</theme>", blocks + "</theme>"));
        }

        private static void Refresh(ThemedSession s)
        {
            typeof(MainWindow).GetMethod("RefreshLibrary", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(s.Window, null);
            s.Settle();
        }

        private static SheetLayer Sheets(ThemedSession s) => s.Window.GetControl<SheetLayer>("Sheets");

        private static ThemeSettingsWindow Open(ThemedSession s)
        {
            ThemedLibraryFlowTests.Choose(s, "Theme Settings");
            s.Settle();
            return Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
        }

        // A sheet's content is moved into the window's sheet layer, so it is searched there.
        private static Control RootOf(Window window) => SheetLayer.PresenterOf(window)?.SheetOf(window) ?? window;

        private static T Named<T>(Window window, string name) where T : Control =>
            RootOf(window).GetLogicalDescendants().OfType<T>().FirstOrDefault(c => c.Name == name) ?? throw new InvalidOperationException($"no {typeof(T).Name} {name}");

        private static string[] Items(Window window, string name) => ((System.Collections.IEnumerable)Named<Dropdown>(window, name).ItemsSource!).Cast<string>().ToArray();

        private static void Choose(ThemedSession s, Window window, string dropdown, string text)
        {
            Named<Dropdown>(window, dropdown).SelectedItem = text;
            s.Settle();
        }

        private static void Click(Button button) => button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

        // Runs the dispatcher, with real time passing, until a condition holds.
        private static bool Pump(Func<bool> until, int ms = 5000)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!until() && clock.ElapsedMilliseconds < ms)
            {
                Dispatcher.UIThread.RunJobs();
                Thread.Sleep(5);
            }
            Dispatcher.UIThread.RunJobs();
            return until();
        }

        // P48 on a synthetic theme: selectable variants in declared order under en_US labels, the schemes, sizes, Automatic and ratios, the profiles and unsuppressed built-ins, and no language row when none is declared.
        [Fact]
        public Task The_sheet_lists_what_the_theme_declares() => Session.Dispatch(() =>
        {
            using var theme = new SyntheticTheme();
            Write(theme);
            using var s = new ThemedSession(themeDirectory: theme.Root);
            ThemeSettingsWindow sheet = Open(s);
            Assert.Equal(new[] { "Narrow list", "Wide list" }, Items(sheet, "VariantDropdown"));
            Assert.Equal(new[] { "Dark", "Light" }, Items(sheet, "ColorSchemeDropdown"));
            Assert.Equal(new[] { "Medium", "Small" }, Items(sheet, "FontSizeDropdown"));
            Assert.Equal(new[] { "Automatic", "16:10", "4:3" }, Items(sheet, "AspectRatioDropdown"));
            Assert.Equal(new[] { "Automatic", "Glide", "Instant (built in)", "Slide (built in)" }, Items(sheet, "TransitionsDropdown"));
            Assert.Empty(RootOf(sheet).GetLogicalDescendants().OfType<Dropdown>().Where(d => d.Name == "LanguageDropdown"));
            Assert.Equal("Narrow list", Named<Dropdown>(sheet, "VariantDropdown").SelectedItem);
            Assert.Equal("Medium", Named<Dropdown>(sheet, "FontSizeDropdown").SelectedItem);
            Assert.Equal("Automatic", Named<Dropdown>(sheet, "AspectRatioDropdown").SelectedItem);
            s.Pad.B();
            Assert.False(Sheets(s).IsPresenting);

            File.WriteAllText(theme.PathOf("capabilities.xml"), $"<themeCapabilities>{Capabilities}<language>en_US</language><language>de_DE</language></themeCapabilities>");
            sheet = Open(s);
            Assert.Equal(new[] { "en_US", "de_DE" }, Items(sheet, "LanguageDropdown"));
        }, default);

        // P48 on Art Book Next, read in place: 20 variants in declared order, 31 schemes, 4 sizes, Automatic and 12 ratios, its two profiles, no language row.
        [ArtBookNextFact]
        public Task Art_Book_Next_s_sheet_lists_what_its_capabilities_declare() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(themeDirectory: ArtBookNextFactAttribute.Folder);
            ThemeSettingsWindow sheet = Open(s);
            var caps = EmuSen.Mistress.BigPicture.Theme.ThemeCapabilitiesReader.Read(ArtBookNextFactAttribute.Folder);
            string[] variants = Items(sheet, "VariantDropdown");
            Assert.Equal(20, variants.Length);
            Assert.Equal(caps.Variants.Select(v => v.Labels["en_US"]), variants);
            Assert.Equal("List: Metadata & Boxart", Named<Dropdown>(sheet, "VariantDropdown").SelectedItem);
            Assert.Equal(31, Items(sheet, "ColorSchemeDropdown").Length);
            Assert.Equal(new[] { "Medium", "Large", "Small", "Extra Large" }, Items(sheet, "FontSizeDropdown"));
            Assert.Equal(13, Items(sheet, "AspectRatioDropdown").Length);
            Assert.Equal(new[] { "Automatic", "Instant", "Slide" }, Items(sheet, "TransitionsDropdown"));
            Assert.Empty(RootOf(sheet).GetLogicalDescendants().OfType<Dropdown>().Where(d => d.Name == "LanguageDropdown"));
            _out.WriteLine(string.Join(" | ", variants));
        }, default);

        // P46: a choice redraws the view beneath the open sheet, and once the sheet is gone the frame equals a fresh static build under the new choice.
        [Fact]
        public Task A_choice_applies_at_once_beneath_the_sheet() => Session.Dispatch(() =>
        {
            using var theme = new SyntheticTheme();
            Write(theme);
            using var s = new ThemedSession(themeDirectory: theme.Root);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Down(2);
            SceneStage before = s.Themed.Stage!;
            ThemeSettingsWindow sheet = Open(s);

            Choose(s, sheet, "ColorSchemeDropdown", "Light");
            Assert.True(Sheets(s).IsPresenting);
            Assert.Equal("light", s.Themed.Stage!.Current.Data.System.Theme.Selection.ColorScheme);
            Choose(s, sheet, "VariantDropdown", "Wide list");
            Choose(s, sheet, "FontSizeDropdown", "Small");
            Choose(s, sheet, "AspectRatioDropdown", "4:3");
            var selection = s.Themed.Stage!.Current.Data.System.Theme.Selection;
            Assert.Equal(("wide", "light", "small", "4:3"), (selection.Variant, selection.ColorScheme, selection.FontSize, selection.AspectRatio));
            Assert.Equal(("snes", "gamelist", "Cobalt Harbor (Synthetic)"), (s.System, s.View, s.Game));
            Assert.NotSame(before, s.Themed.Stage);

            s.Pad.B();
            Assert.False(Sheets(s).IsPresenting);
            RenderedFrame shown = s.Capture();
            SceneData data = s.Themed.Stage!.Current.Data;
            RenderedFrame fresh = SceneAssets.Render(SceneBuilder.Build(data.System.Theme.View("gamelist"), data));
            Assert.Equal(0, SceneAssets.Differing(shown, fresh));

            Choose(s, Open(s), "AspectRatioDropdown", "Automatic");
            Assert.Equal("16:10", s.Themed.Stage!.Current.Data.System.Theme.Selection.AspectRatio);
        }, default);

        // P47: each theme folder keeps its own choices, in appsettings.json; a stored choice the theme no longer declares falls back without an error.
        [Fact]
        public Task Choices_are_kept_per_theme_and_a_stale_one_falls_back() => Session.Dispatch(() =>
        {
            using var first = new SyntheticTheme();
            using var second = new SyntheticTheme();
            Write(first);
            Write(second);
            using var s = new ThemedSession(themeDirectory: first.Root);
            Choose(s, Open(s), "ColorSchemeDropdown", "Light");
            s.Pad.B();

            AppSettings settings = AppSettings.Load();
            settings.BigPictureTheme = second.Root;
            settings.Save();
            s.Window.GetType().GetField("_appSettings", BindingFlags.Instance | BindingFlags.NonPublic)!.SetValue(s.Window, settings);
            ThemeSettingsWindow sheet = Open(s);
            Assert.Equal("Dark", Named<Dropdown>(sheet, "ColorSchemeDropdown").SelectedItem);
            Choose(s, sheet, "VariantDropdown", "Wide list");
            s.Pad.B();
            Assert.Equal("dark", s.Themed.Choices.ColorScheme ?? "dark");
            Assert.Equal("wide", s.Themed.Choices.Variant);

            AppSettings stored = AppSettings.Load();
            Assert.Equal("light", stored.BigPicture[ThemeSettingsWindow.Key(first.Root)].ColorScheme);
            Assert.Null(stored.BigPicture[ThemeSettingsWindow.Key(first.Root)].Variant);
            Assert.Equal("wide", stored.BigPicture[ThemeSettingsWindow.Key(second.Root)].Variant);
            Assert.Null(stored.BigPicture[ThemeSettingsWindow.Key(second.Root)].ColorScheme);

            stored.BigPicture[ThemeSettingsWindow.Key(second.Root)].Variant = "gone";
            stored.BigPictureTheme = first.Root;
            stored.Save();
            s.Window.GetType().GetField("_appSettings", BindingFlags.Instance | BindingFlags.NonPublic)!.SetValue(s.Window, stored);
            Refresh(s);
            Assert.Equal("light", s.Themed.Stage!.Current.Data.System.Theme.Selection.ColorScheme);

            stored.BigPictureTheme = second.Root;
            s.Window.GetType().GetField("_appSettings", BindingFlags.Instance | BindingFlags.NonPublic)!.SetValue(s.Window, stored);
            Refresh(s);
            Assert.True(s.Shown);
            Assert.Equal("narrow", s.Themed.Stage!.Current.Data.System.Theme.Selection.Variant);
            Assert.True(s.Themed.Stage.Current.Data.System.Theme.IsThemed);
        }, default);

        // The sheet is also reached from Preferences ▸ Appearance.
        [Fact]
        public Task Preferences_opens_the_theme_settings_sheet() => Session.Dispatch(() =>
        {
            using var theme = new SyntheticTheme();
            Write(theme);
            using var s = new ThemedSession(themeDirectory: theme.Root);
            ThemedLibraryFlowTests.Choose(s, "Preferences");
            Window preferences = Assert.IsType<PreferencesWindow>(Sheets(s).Current);
            Click(Named<Button>(preferences, "ThemeSettingsButton"));
            s.Settle();
            Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
            Assert.Empty(s.Window.OwnedWindows);
        }, default);

        // P49: every control of the settings sheet, its themes tab and the about sheet is reached by the pad at 1280 by 800.
        [Fact]
        public Task Every_control_of_the_theme_sheets_is_reached_by_the_pad() => Session.Dispatch(() =>
        {
            using var theme = new SyntheticTheme();
            Write(theme);
            using var s = new ThemedSession(themeDirectory: theme.Root);
            Open(s);
            var missing = new List<string>();
            Audit(s, missing);
            Click(Named<Button>(Sheets(s).Current!, $"ThemeAbout.{Path.GetFileName(theme.Root)}"));
            s.Settle();
            Assert.IsType<ThemeAboutWindow>(Sheets(s).Current);
            Audit(s, missing);
            foreach (string m in missing) _out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);
        }, default);

        private static void Audit(ThemedSession s, List<string> missing)
        {
            Control sheet = Sheets(s).SheetOf(Sheets(s).Current!)!;
            TabControl? tabs = sheet.GetVisualDescendants().OfType<TabControl>().FirstOrDefault();
            for (int page = 0; page < (tabs?.ItemCount ?? 1); page++)
            {
                if (page > 0) s.Pad.R1();
                s.Window.UpdateLayout();
                HashSet<Avalonia.Input.InputElement> reached = PadAudit.Reachable(sheet, s.Pad);
                missing.AddRange(PadAudit.Operable(sheet).Where(c => !reached.Contains(c)).Select(c => $"[{Sheets(s).Current!.Title}/{(tabs?.SelectedItem as TabItem)?.Header}] {PadAudit.Describe(c)}"));
            }
        }

        // The themes tab downloads Art Book Next from a fake GitHub, uses it when no theme was set, and opens its about sheet once, after the first download only.
        [Fact]
        public Task The_themes_tab_downloads_a_theme_uses_it_and_shows_its_about_sheet_once() => Session.Dispatch(() =>
        {
            string sha = ThemeDownloadsTests.Sha1;
            string themeXml = "<theme><view name=\"system\"><carousel name=\"c\"><pos>0 0.2</pos><size>1 0.5</size></carousel></view>" +
                              "<view name=\"gamelist\"><textlist name=\"l\"><pos>0.05 0.1</pos><size>0.5 0.8</size></textlist></view></theme>";
            var server = ThemeDownloadsTests.GitHub(() => ThemeDownloadsTests.Archive("one", themeXml: themeXml), () => sha);
            Factory.SetValue(null, (Func<HttpClient>)(() => new HttpClient(server)));
            using var s = new ThemedSession(settings: a => a.BigPictureTheme = null);
            Assert.False(s.Shown);
            ThemeSettingsWindow sheet = Open(s);
            Assert.NotNull(RootOf(sheet).GetLogicalDescendants().OfType<EmptyState>().FirstOrDefault());
            Click(Named<Button>(sheet, "DownloadArtBookNext"));
            Assert.True(Pump(() => sheet.Downloading is { IsCompleted: true } && Sheets(s).Current is ThemeAboutWindow), Named<TextBlock>(sheet, "ThemeStatus").Text);
            var about = (ThemeAboutWindow)Sheets(s).Current!;
            Assert.Equal("Synthetic Book", about.Attribution.Name);
            Assert.Equal(sha, about.Attribution.Commit);
            string installed = ThemeDownloads.DirectoryFor(ThemeSource.ArtBookNext);
            Assert.True(ThemeDownloads.SamePath(installed, AppSettings.Load().BigPictureTheme!));
            s.Pad.B();
            Assert.Same(sheet, Sheets(s).Current);
            s.Pad.B();
            s.Settle();
            Assert.True(s.Shown);

            sha = ThemeDownloadsTests.Sha2;
            sheet = Open(s);
            Click(Named<Button>(sheet, "ThemeUpdate.art-book-next-es-de"));
            Assert.True(Pump(() => Named<TextBlock>(sheet, "ThemeStatus").Text?.StartsWith("Update available") == true), Named<TextBlock>(sheet, "ThemeStatus").Text);
            Click(Named<Button>(sheet, "ThemeUpdate.art-book-next-es-de"));
            Assert.True(Pump(() => sheet.Downloading is { IsCompleted: true } && Named<TextBlock>(sheet, "ThemeStatus").Text?.StartsWith("Installed") == true));
            Assert.Same(sheet, Sheets(s).Current);
            Assert.Equal(ThemeDownloadsTests.Sha2, ThemeDownloads.Stamp(installed)!.Commit);
        }, default);

        // P55: closing the sheet during a download cancels it; without the sheet's cleanup the request would wait for ever.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public Task Closing_the_sheet_or_its_window_stops_the_download(bool closeWindow) => Session.Dispatch(() =>
        {
            var stall = new ThemeDownloadsTests.Stall();
            Factory.SetValue(null, (Func<HttpClient>)(() => new HttpClient(stall.Server)));
            using var s = new ThemedSession(settings: a => a.BigPictureTheme = null);
            ThemeSettingsWindow sheet = Open(s);
            Click(Named<Button>(sheet, "DownloadArtBookNext"));
            Assert.True(Pump(() => stall.Started.Task.IsCompleted), "the archive's body began");
            string installed = ThemeDownloads.DirectoryFor(ThemeSource.ArtBookNext);
            Assert.True(File.Exists(installed + ".zip.part"));

            if (closeWindow) s.Window.Close();
            else s.Pad.B();
            Assert.True(Pump(() => sheet.Downloading!.IsCompleted, 3000), "the download was still running after its sheet closed");
            Assert.True(sheet.Downloading!.IsCanceled || sheet.Downloading.Exception?.InnerException is OperationCanceledException);
            Assert.False(File.Exists(installed + ".zip.part"));
            Assert.False(Directory.Exists(installed + ".part"));
            Assert.False(Directory.Exists(installed));
        }, default);
    }
}
