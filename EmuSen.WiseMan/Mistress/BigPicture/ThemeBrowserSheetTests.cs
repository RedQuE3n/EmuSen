using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
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
    // The theme browser as the player meets it: no request before it opens, the list and a theme's detail, the licence before Download, install, cancel, close, and the pad - see EmuSen_BigPicture.md §25.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemeBrowserSheetTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemeBrowserSheetTests).GetTypeInfo().Assembly);

        private static readonly FieldInfo Factory = typeof(MainWindow).GetField("HttpFactory", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realFactory = Factory.GetValue(null)!;
        private readonly ITestOutputHelper _out;

        public ThemeBrowserSheetTests(ITestOutputHelper output) => _out = output;

        public void Dispose() => Factory.SetValue(null, _realFactory);

        internal static void Serve(FakeThemeHosts hosts) => Factory.SetValue(null, (Func<HttpClient>)hosts.Client);

        internal static SheetLayer Sheets(ThemedSession s) => s.Window.GetControl<SheetLayer>("Sheets");

        internal static Control RootOf(Window window) => SheetLayer.PresenterOf(window)?.SheetOf(window) ?? window;

        internal static T Named<T>(Window window, string name) where T : Control =>
            RootOf(window).GetLogicalDescendants().OfType<T>().FirstOrDefault(c => c.Name == name) ?? throw new InvalidOperationException($"no {typeof(T).Name} {name}");

        internal static void Click(Button button) => button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

        internal static bool Pump(Func<bool> until, int ms = 5000)
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

        internal static ThemeSettingsWindow OpenSettings(ThemedSession s)
        {
            ThemedLibraryFlowTests.Choose(s, "Theme Settings");
            s.Settle();
            return Assert.IsType<ThemeSettingsWindow>(Sheets(s).Current);
        }

        // Opens the browser by its button and waits for the list and the first preview.
        internal static ThemeBrowserWindow OpenBrowser(ThemedSession s, ThemeSettingsWindow sheet)
        {
            Click(Named<Button>(sheet, "BrowseThemes"));
            s.Settle();
            ThemeBrowserWindow browser = Assert.IsType<ThemeBrowserWindow>(Sheets(s).Current);
            Assert.True(Pump(() => browser.Loading is { IsCompleted: true } && browser.PreviewLoading is { IsCompleted: true }), Named<TextBlock>(browser, "ThemeBrowserStatus").Text);
            return browser;
        }

        internal static ThemeDetailWindow OpenDetail(ThemedSession s, ThemeBrowserWindow browser, string theme)
        {
            ThemeBrowserEntry entry = browser.Entries.First(e => e.Theme.Name == theme);
            var list = Named<LunaList<ThemeBrowserEntry>>(browser, "ThemeBrowserList");
            list.Select(entry);
            Click(Named<Button>(browser, "ThemeBrowserDetails"));
            s.Settle();
            ThemeDetailWindow detail = Assert.IsType<ThemeDetailWindow>(Sheets(s).Current);
            Assert.True(Pump(() => detail.DetailsLoading.IsCompleted && detail.ShotLoading is { IsCompleted: true }));
            return detail;
        }

        // Opening the theme sheet and its Themes tab asks nothing; opening the browser asks for the list and one screenshot; opening it again within a day asks nothing more; Refresh asks for the list again.
        [Fact]
        public Task Nothing_is_asked_until_the_browser_opens_or_refreshes() => Session.Dispatch(() =>
        {
            var hosts = FakeThemeHosts.Standard();
            Serve(hosts);
            using var s = new ThemedSession();
            s.Run(300);
            ThemeSettingsWindow sheet = OpenSettings(s);
            s.Pad.R1();
            s.Settle();
            Assert.Equal(0, hosts.Requests);

            ThemeBrowserWindow browser = OpenBrowser(s, sheet);
            Assert.Equal(new[] { ThemeList.Address, ThemeList.ScreenshotAddress(hosts.Themes[0].Shots.First()) }, hosts.Asked);
            Assert.StartsWith("ES-DE's theme list: 3 themes, fetched ", Named<TextBlock>(browser, "ThemeBrowserFetched").Text);
            s.Pad.B();
            s.Settle();
            Assert.Same(sheet, Sheets(s).Current);

            browser = OpenBrowser(s, sheet);
            Assert.Equal(2, hosts.Requests);
            Click(Named<Button>(browser, "ThemeBrowserRefresh"));
            Assert.True(Pump(() => hosts.Requests == 3 && Named<Button>(browser, "ThemeBrowserRefresh").IsEnabled));
            Assert.Equal(2, hosts.AskedFor(ThemeList.Address));
        }, default);

        // The list names every theme with its state, and the preview shows the selected theme's name, author, counts, state and first screenshot, cached in Mistress's folder.
        [Fact]
        public Task The_list_and_the_preview_show_what_the_list_states() => Session.Dispatch(() =>
        {
            var hosts = FakeThemeHosts.Standard();
            Serve(hosts);
            using var s = new ThemedSession();
            ThemeBrowserWindow browser = OpenBrowser(s, OpenSettings(s));
            var list = Named<LunaList<ThemeBrowserEntry>>(browser, "ThemeBrowserList");
            Assert.Equal(new[] { "Synthetic Book", "Plain Shelf", "Lab Wheel   ·   New" }, list.Models.Select(list.Label));
            Assert.Equal("Synthetic Book", Named<TextBlock>(browser, "ThemeBrowserName").Text);
            Assert.Equal("by Ada Example   ·   github.com/ada/synthetic-book-es-de", Named<TextBlock>(browser, "ThemeBrowserAuthor").Text);
            Assert.Equal("2 variants · 3 colour schemes · 3 aspect ratios", Named<TextBlock>(browser, "ThemeBrowserSupports").Text);
            Assert.Equal("Not installed", Named<TextBlock>(browser, "ThemeBrowserState").Text);
            Assert.Equal("Synthetic Book, screenshot 1", Named<TextBlock>(browser, "ThemeBrowserCaption").Text);
            string? shot = Named<FittedImage>(browser, "ThemeBrowserScreenshot").Source;
            Assert.StartsWith(ThemeBrowser.ScreenshotFolder, shot);

            Named<LunaList<ThemeBrowserEntry>>(browser, "ThemeBrowserList").SelectedIndex = 1;
            Assert.True(Pump(() => Named<TextBlock>(browser, "ThemeBrowserName").Text == "Plain Shelf" && browser.PreviewLoading is { IsCompleted: true }
                                   && Named<FittedImage>(browser, "ThemeBrowserScreenshot").Source is not null));
            Assert.Equal("2 variants · no colour schemes stated · 3 aspect ratios", Named<TextBlock>(browser, "ThemeBrowserSupports").Text);
        }, default);

        // A theme's detail: every screenshot fetched as it is shown, what the list states, the last update, and the licence line above a Download that waits for it.
        [Fact]
        public Task A_theme_s_detail_shows_its_screenshots_supports_update_and_licence_before_download() => Session.Dispatch(() =>
        {
            var hosts = FakeThemeHosts.Standard();
            Serve(hosts);
            using var s = new ThemedSession();
            ThemeBrowserWindow browser = OpenBrowser(s, OpenSettings(s));
            int before = hosts.Requests;
            ThemeDetailWindow detail = OpenDetail(s, browser, "Synthetic Book");
            FakeTheme book = hosts["Synthetic Book"];
            Assert.Equal(new[] { book.Source.RepositoryAddress, book.Source.CommitAddress, book.Source.ReadmeAddress }, hosts.Asked.Skip(before));

            Assert.Equal("List, Grid", Named<TextBlock>(detail, "ThemeDetailVariants").Text);
            Assert.Equal("Dark, Light, Night", Named<TextBlock>(detail, "ThemeDetailColorSchemes").Text);
            Assert.Equal("16:9, 16:10, 4:3", Named<TextBlock>(detail, "ThemeDetailAspectRatios").Text);
            Assert.Equal("Medium, Large", Named<TextBlock>(detail, "ThemeDetailFontSizes").Text);
            Assert.Equal("Instant, Slide", Named<TextBlock>(detail, "ThemeDetailTransitions").Text);
            Assert.Equal("en_US, de_DE", Named<TextBlock>(detail, "ThemeDetailLanguages").Text);
            Assert.StartsWith($"{book.Date.ToLocalTime():yyyy-MM-dd}, commit 1111111, on branch main", Named<TextBlock>(detail, "ThemeDetailUpdated").Text);
            Assert.Equal("Creative Commons BY-NC-SA 4.0, as stated for this synthetic theme.", Named<TextBlock>(detail, "ThemeDetailLicence0").Text);
            Assert.Equal("1 of 3", Named<TextBlock>(detail, "ThemeDetailPosition").Text);

            Button download = Named<Button>(detail, "ThemeDetailDownload");
            Assert.True(download.IsEnabled && download.IsVisible);
            Control licence = Named<FieldRow>(detail, "ThemeDetailLicenceRow");
            Control top = RootOf(detail);
            Assert.True(licence.TranslatePoint(default, top)!.Value.Y + licence.Bounds.Height <= download.TranslatePoint(default, top)!.Value.Y, "the licence line sits above the Download button");

            int now = hosts.Requests;
            Click(Named<Button>(detail, "ThemeDetailNext"));
            Assert.True(Pump(() => detail.ShotLoading is { IsCompleted: true }));
            Assert.Equal("2 of 3", Named<TextBlock>(detail, "ThemeDetailPosition").Text);
            Assert.Equal("Synthetic Book, screenshot 2", Named<TextBlock>(detail, "ThemeDetailCaption").Text);
            Assert.Equal(ThemeList.ScreenshotAddress(book.Shots.ElementAt(1)), hosts.Asked.Last());
            Click(Named<Button>(detail, "ThemeDetailPrevious"));
            Assert.True(Pump(() => detail.ShotLoading is { IsCompleted: true }));
            Assert.Equal(now + 1, hosts.Requests);

            s.Pad.B();
            s.Settle();
            detail = OpenDetail(s, browser, "Lab Wheel");
            Assert.Contains("states no licence", Named<TextBlock>(detail, "ThemeDetailLicence0").Text);
        }, default);

        // Q27: until the host has answered, the licence line says it is being read and Download waits; then the line is shown and Download offered.
        [Fact]
        public Task Download_waits_for_the_licence_line() => Session.Dispatch(() =>
        {
            var hosts = FakeThemeHosts.Standard();
            Serve(hosts);
            using var s = new ThemedSession();
            ThemeBrowserWindow browser = OpenBrowser(s, OpenSettings(s));
            var hold = hosts.HoldRepository = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            Named<LunaList<ThemeBrowserEntry>>(browser, "ThemeBrowserList").Select(browser.Entries[0]);
            Click(Named<Button>(browser, "ThemeBrowserDetails"));
            s.Settle();
            ThemeDetailWindow detail = Assert.IsType<ThemeDetailWindow>(Sheets(s).Current);
            Pump(() => false, 200);
            Assert.False(detail.DetailsLoading.IsCompleted);
            Assert.StartsWith("Reading the licence line", Named<TextBlock>(detail, "ThemeDetailLicence0").Text);
            Assert.False(Named<Button>(detail, "ThemeDetailDownload").IsEnabled);
            Click(Named<Button>(detail, "ThemeDetailDownload"));
            Assert.Null(detail.Downloading);

            hold.SetResult();
            Assert.True(Pump(() => detail.DetailsLoading.IsCompleted));
            Assert.Equal("Creative Commons BY-NC-SA 4.0, as stated for this synthetic theme.", Named<TextBlock>(detail, "ThemeDetailLicence0").Text);
            Assert.True(Named<Button>(detail, "ThemeDetailDownload").IsEnabled);
        }, default);

        // Download from the detail: progress, the theme installed and used when none was, its About sheet once, the Themes tab listing it with Use beside EmuSen's own.
        [Fact]
        public Task A_theme_downloads_from_its_detail_and_appears_in_the_themes_tab() => Session.Dispatch(() =>
        {
            var hosts = FakeThemeHosts.Standard();
            Serve(hosts);
            using var s = new ThemedSession(settings: a => a.BigPictureTheme = null);
            ThemeSettingsWindow sheet = OpenSettings(s);
            ThemeBrowserWindow browser = OpenBrowser(s, sheet);
            ThemeDetailWindow detail = OpenDetail(s, browser, "Plain Shelf");
            Assert.Equal("MIT License, as GitHub reads the repository's licence file.", Named<TextBlock>(detail, "ThemeDetailLicence0").Text);
            Click(Named<Button>(detail, "ThemeDetailDownload"));
            Assert.True(Pump(() => detail.Downloading is { IsCompleted: true } && Sheets(s).Current is ThemeAboutWindow), Named<TextBlock>(detail, "ThemeDetailStatus").Text);
            Assert.True(detail.Downloading!.IsCompletedSuccessfully);
            var about = (ThemeAboutWindow)Sheets(s).Current!;
            Assert.Equal("Plain Shelf", about.Attribution.Name);
            string installed = ThemeDownloads.DirectoryFor(hosts["Plain Shelf"].Source);
            Assert.True(ThemeDownloads.SamePath(installed, AppSettings.Load().BigPictureTheme!));
            s.Pad.B();
            s.Settle();
            Assert.Same(detail, Sheets(s).Current);
            Assert.StartsWith("Installed Plain Shelf at commit 1111111", Named<TextBlock>(detail, "ThemeDetailStatus").Text);
            Assert.False(Named<Button>(detail, "ThemeDetailDownload").IsVisible);
            Assert.Equal("In Use", Named<Button>(detail, "ThemeDetailUse").Content);
            Assert.False(Named<Button>(detail, "ThemeDetailUpdate").IsEnabled);
            s.Pad.B();
            s.Settle();
            Assert.Equal("Plain Shelf   ·   Installed", Named<LunaList<ThemeBrowserEntry>>(browser, "ThemeBrowserList").Label(browser.Entries[1]));
            s.Pad.B();
            s.Settle();
            Assert.Same(sheet, Sheets(s).Current);
            Assert.Equal("Use", Named<Button>(sheet, "ThemeUseBuiltIn").Content);
            Assert.Equal("In Use", Named<Button>(sheet, "ThemeUse.plain-shelf-es-de").Content);
            Assert.True(s.Shown);
        }, default);

        // Cancel Download, closing the detail, and closing the main window each stop a download, leave no partial file, and hold nothing open under Mistress's folder.
        [Theory]
        [InlineData("cancel")]
        [InlineData("detail")]
        [InlineData("browser")]
        [InlineData("window")]
        public Task A_stopped_download_leaves_nothing_half_written(string how) => Session.Dispatch(() =>
        {
            var hosts = FakeThemeHosts.Standard();
            hosts.StallArchiveOf = "Synthetic Book";
            Serve(hosts);
            using var s = new ThemedSession(settings: a => a.BigPictureTheme = null);
            ThemeBrowserWindow browser = OpenBrowser(s, OpenSettings(s));
            ThemeDetailWindow detail = OpenDetail(s, browser, "Synthetic Book");
            Click(Named<Button>(detail, "ThemeDetailDownload"));
            Assert.True(Pump(() => hosts.Stalled.Task.IsCompleted), "the archive's body began");
            Assert.True(Named<ProgressBar>(detail, "ThemeDetailProgress").IsVisible);
            Assert.True(Named<Button>(detail, "ThemeDetailCancel").IsVisible);
            string dir = ThemeDownloads.DirectoryFor(hosts["Synthetic Book"].Source);
            Assert.True(File.Exists(dir + ".zip.part"));

            if (how == "cancel") Click(Named<Button>(detail, "ThemeDetailCancel"));
            else if (how == "detail") s.Pad.B();
            else if (how == "browser") browser.Close();
            else s.Window.Close();
            Assert.True(Pump(() => detail.Downloading!.IsCompleted, 3000), "the download was still running after it was stopped");
            Assert.False(detail.Downloading!.IsCompletedSuccessfully);
            Assert.False(File.Exists(dir + ".zip.part"));
            Assert.False(Directory.Exists(dir + ".part"));
            Assert.False(Directory.Exists(dir));
            Assert.Empty(ThemeBrowserModelTests.OpenUnder(DataStore.Themes));
            using (ThemeRecords db = ThemeRecords.Open()) Assert.Empty(db.Installed());
            if (how == "cancel")
            {
                Pump(() => Named<TextBlock>(detail, "ThemeDetailStatus").Text?.StartsWith("The download was stopped") == true);
                Assert.StartsWith("The download was stopped", Named<TextBlock>(detail, "ThemeDetailStatus").Text);
                Assert.True(Named<Button>(detail, "ThemeDetailDownload").IsEnabled);
            }
        }, default);

        // An update over local changes asks first; Cancel downloads nothing and leaves the edit.
        [Fact]
        public Task An_update_over_local_changes_asks_first() => Session.Dispatch(() =>
        {
            var hosts = FakeThemeHosts.Standard();
            Serve(hosts);
            FakeTheme book = hosts["Synthetic Book"];
            using var s = new ThemedSession();
            hosts.Install(book.Source);
            string dir = ThemeDownloads.DirectoryFor(book.Source);
            File.WriteAllText(Path.Combine(dir, "colors.xml"), "edited");
            book.Sha = "2222222bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
            ThemeBrowserWindow browser = OpenBrowser(s, OpenSettings(s));
            Assert.Equal("Synthetic Book   ·   Installed, update available, local changes", Named<LunaList<ThemeBrowserEntry>>(browser, "ThemeBrowserList").Label(browser.Entries[0]));
            ThemeDetailWindow detail = OpenDetail(s, browser, "Synthetic Book");
            Assert.Contains("Local changes to 1 files (colors.xml)", Named<TextBlock>(detail, "ThemeDetailState").Text);
            int asked = hosts.Requests;
            Click(Named<Button>(detail, "ThemeDetailUpdate"));
            s.Settle();
            Assert.IsNotType<ThemeDetailWindow>(Sheets(s).Current);
            s.Pad.B();
            s.Settle();
            Assert.Same(detail, Sheets(s).Current);
            Assert.Null(detail.Downloading);
            Assert.Equal(asked, hosts.Requests);
            Assert.Equal("edited", File.ReadAllText(Path.Combine(dir, "colors.xml")));
        }, default);

        // Every control of the browser and of a theme's detail is reached by the pad at 1280 by 800, and A on a row opens its detail.
        [Fact]
        public Task Every_control_of_the_browser_and_the_detail_is_reached_by_the_pad() => Session.Dispatch(() =>
        {
            var hosts = FakeThemeHosts.Standard();
            Serve(hosts);
            using var s = new ThemedSession();
            hosts.Install(hosts["Plain Shelf"].Source);
            ThemeSettingsWindow sheet = OpenSettings(s);
            s.Pad.R1();
            s.Settle();
            PadAudit.Reach(RootOf(sheet), s.Pad, e => e is Button { Name: "BrowseThemes" });
            s.Pad.A();
            s.Settle();
            ThemeBrowserWindow browser = Assert.IsType<ThemeBrowserWindow>(Sheets(s).Current);
            Assert.True(Pump(() => browser.Loading is { IsCompleted: true } && browser.PreviewLoading is { IsCompleted: true }));
            var missing = new List<string>();
            Audit(s, browser, missing);

            PadAudit.Reach(RootOf(browser), s.Pad, e => e is ListBoxItem row && row.FindAncestorOfType<ListBox>()?.IndexFromContainer(row) == 1);
            s.Pad.A();
            s.Settle();
            ThemeDetailWindow detail = Assert.IsType<ThemeDetailWindow>(Sheets(s).Current);
            Assert.Equal("Plain Shelf", detail.Entry.Theme.Name);
            Assert.True(Pump(() => detail.DetailsLoading.IsCompleted && detail.ShotLoading is { IsCompleted: true }));
            Audit(s, detail, missing);
            foreach (string m in missing) _out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);
        }, default);

        private static void Audit(ThemedSession s, Window window, List<string> missing)
        {
            Control root = RootOf(window);
            s.Window.UpdateLayout();
            if (TopLevel.GetTopLevel(root)?.FocusManager?.GetFocusedElement() is not Visual at || !at.GetVisualAncestors().Contains(root)) s.Pad.Up();
            HashSet<Avalonia.Input.InputElement> reached = PadAudit.Reachable(root, s.Pad);
            missing.AddRange(PadAudit.Operable(root).Where(c => !reached.Contains(c)).Select(c => $"[{window.Title}] {PadAudit.Describe(c)}"));
        }
    }
}
