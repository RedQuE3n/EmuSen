using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.LogicalTree;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;
using Avalonia.VisualTree;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Settings' sections, the collection settings, the list screen and the menu opening, at 1280 by 800 and 1920 by 1200 on both themes; written outside the repository.
    [Trait("Category", "nightly")]
    [Collection(TestCollections.ProcessGlobals)]
    public class SettingsMenusPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(SettingsMenusPictureTool).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public SettingsMenusPictureTool(ITestOutputHelper output) => _out = output;

        private static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "esde-menus-2");

        private void Save(ThemedSession s, string name)
        {
            s.Settle();
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            s.Capture().SavePng(path);
            _out.WriteLine(path);
        }

        private static SheetLayer Sheets(ThemedSession s) => s.Window.GetControl<SheetLayer>("Sheets");

        private static void Reach(ThemedSession s, string name) =>
            PadAudit.Reach(Sheets(s).SheetOf(Sheets(s).Current!)!, s.Pad, e => e is Control { Name: { } n } && n == name);

        private void Walk(ThemedSession s, string p)
        {
            ThemedLibraryFlowTests.Choose(s, "Themes");
            Save(s, $"{p}-theme-settings");
            TabControl tabs = Sheets(s).SheetOf(Sheets(s).Current!)!.GetVisualDescendants().OfType<TabControl>().First(t => t.Name == "ThemeSettingsTabs");
            for (int i = 0; i < 4 && (tabs.SelectedItem as TabItem)?.Header as string != "Interface"; i++) s.Pad.R1();
            s.Settle();
            Save(s, $"{p}-theme-settings-interface");
            Reach(s, "LaunchScreenDuration");
            s.Pad.A();
            Save(s, $"{p}-list-screen");
            s.Pad.Down();
            s.Pad.A();
            Save(s, $"{p}-list-screen-chosen");
            s.Pad.Down(12);
            Save(s, $"{p}-theme-settings-interface-lower");
            s.Pad.L1();
            s.Settle();
            Save(s, $"{p}-theme-settings-themes");
            ThemedSwitchesTests.PutAway(s);
            s.Settle();

            ThemedCollectionsTests.Choose(s, "Collections");
            Save(s, $"{p}-collection-settings");
            s.Pad.Down(9);
            Save(s, $"{p}-collection-settings-lower");
            s.Pad.B();
            s.Settle();

            SettingsPad.Open(s.Window, s.Pad);
            foreach ((string section, string title) in EmuSen.Mistress.Pages.SettingsScreen.SectionList)
            {
                SettingsPad.Section(s.Window, s.Pad, section);
                s.Settle();
                Save(s, $"{p}-settings-{title.ToLowerInvariant()}");
            }
            for (int guard = 0; guard < 4 && s.Window.SettingsShown is not null; guard++) s.Pad.B();
            s.Settle();

            // Scale-up, caught part way: the pad menu at the start of its opening.
            s.Settings.BigPictureInterface.MenuOpeningEffect = EmuSen.Galaxia.Models.BigPictureInterface.OpeningScaleUp;
            s.Pad.Start();
            MenuPanel menu = EmuSen.WiseMan.Fixtures.PadMenu.Big(s.Window);
            menu.OpeningScale = 0.75;
            s.Capture().SavePng(Path.Combine(PngFolder, $"{p}-menu-opening-0.75.png"));
            s.Pad.B();
        }

        private static ThemedSession ArtBookNext(double width, double height)
        {
            string media = Path.Combine(Path.GetTempPath(), "EmuSenSettingsMenusMedia");
            SyntheticLibrary.WriteMedia(media);
            return new ThemedSession(width, height, a => a.EsdeMediaDirectory = media, themeDirectory: ArtBookNextFactAttribute.Folder);
        }

        [EsdeMenusPngFact]
        public Task Synthetic_theme() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
                using (var s = new ThemedSession(w, h)) Walk(s, $"synthetic-{w}x{h}");
        }, default);

        [EsdeMenusPngFact]
        public Task Art_book_next() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
                using (ThemedSession s = ArtBookNext(w, h)) Walk(s, $"artbooknext-{w}x{h}");
        }, default);
    }
}
