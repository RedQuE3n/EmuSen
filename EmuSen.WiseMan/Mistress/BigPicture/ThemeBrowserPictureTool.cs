using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.LogicalTree;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;
using static EmuSen.WiseMan.Mistress.BigPicture.ThemeBrowserSheetTests;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class ThemeBrowserPngFactAttribute : FactAttribute
    {
        public ThemeBrowserPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the theme browser's pictures to ~/.cache/emusen/bigpicture/png/theme-browser/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §25";
        }
    }

    // The browser at 1280 by 800 over the synthetic list and fake hosts: the list, a theme's detail, a download under way, and the Themes tab after it; outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemeBrowserPictureTool : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemeBrowserPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "theme-browser");

        private static readonly FieldInfo Factory = typeof(MainWindow).GetField("HttpFactory", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realFactory = Factory.GetValue(null)!;
        private readonly ITestOutputHelper _out;

        public ThemeBrowserPictureTool(ITestOutputHelper output) => _out = output;

        public void Dispose() => Factory.SetValue(null, _realFactory);

        private void Save(ThemedSession s, string name)
        {
            s.Run(100);
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            s.Capture().SavePng(path);
            _out.WriteLine(path);
        }

        [ThemeBrowserPngFact]
        public Task The_browser_s_four_pictures() => Session.Dispatch(() =>
        {
            var hosts = FakeThemeHosts.Standard();
            var random = new Random(7);
            hosts["Synthetic Book"].Extra["art/filler.txt"] = new string(Enumerable.Range(0, 3_000_000).Select(_ => (char)random.Next(33, 126)).ToArray());
            (hosts.StallArchiveOf, hosts.StallFraction) = ("Synthetic Book", 0.42);
            Serve(hosts);
            using var s = new ThemedSession(1280, 800);
            s.Run(300);
            ThemeSettingsWindow sheet = OpenSettings(s);
            ThemeBrowserWindow browser = OpenBrowser(s, sheet);
            Save(s, "browser-list-1280x800");

            ThemeDetailWindow detail = OpenDetail(s, browser, "Synthetic Book");
            Click(Named<Button>(detail, "ThemeDetailNext"));
            Pump(() => detail.ShotLoading is { IsCompleted: true });
            Save(s, "detail-screenshots-licence-1280x800");

            Click(Named<Button>(detail, "ThemeDetailDownload"));
            Pump(() => hosts.Stalled.Task.IsCompleted);
            Pump(() => false, 300);
            Named<ProgressBar>(detail, "ThemeDetailProgress").BringIntoView();
            Save(s, "detail-downloading-1280x800");
            Click(Named<Button>(detail, "ThemeDetailCancel"));
            Pump(() => detail.Downloading!.IsCompleted);

            hosts.StallArchiveOf = null;
            s.Pad.B();
            s.Settle();
            detail = OpenDetail(s, browser, "Plain Shelf");
            Click(Named<Button>(detail, "ThemeDetailDownload"));
            Pump(() => detail.Downloading is { IsCompleted: true } && Sheets(s).Current is ThemeAboutWindow);
            s.Pad.B();
            s.Settle();
            s.Pad.B();
            s.Settle();
            s.Pad.B();
            s.Settle();
            Assert.Same(sheet, Sheets(s).Current);
            if ((RootOf(sheet).GetLogicalDescendants().OfType<TabControl>().First().SelectedItem as TabItem)?.Header as string != "Themes") s.Pad.R1();
            s.Settle();
            Save(s, "themes-tab-after-install-1280x800");
        }, default);
    }
}
