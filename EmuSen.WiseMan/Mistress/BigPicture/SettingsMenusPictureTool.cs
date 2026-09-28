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

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The settings sheets as ES-DE menus, the list screen and the menu opening, at 1280 by 800 and 1920 by 1200 on both themes; written outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class SettingsMenusPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(SettingsMenusPictureTool).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public SettingsMenusPictureTool(ITestOutputHelper output) => _out = output;

        private void Save(ThemedSession s, string name)
        {
            s.Settle();
            Directory.CreateDirectory(MetadataEditorPictureTool.PngFolder);
            string path = Path.Combine(MetadataEditorPictureTool.PngFolder, name + ".png");
            s.Capture().SavePng(path);
            _out.WriteLine(path);
        }

        private static SheetLayer Sheets(ThemedSession s) => s.Window.GetControl<SheetLayer>("Sheets");

        private static void Reach(ThemedSession s, string name) =>
            PadAudit.Reach(Sheets(s).SheetOf(Sheets(s).Current!)!, s.Pad, e => e is Control { Name: { } n } && n == name);

        private void Walk(ThemedSession s, string p)
        {
            ThemedLibraryFlowTests.Choose(s, "Theme Settings");
            Save(s, $"{p}-theme-settings");
            Reach(s, "BigMenuPage2");
            s.Pad.A();
            Save(s, $"{p}-theme-settings-interface");
            Reach(s, "LaunchScreenDuration");
            s.Pad.A();
            Save(s, $"{p}-list-screen");
            s.Pad.Down();
            s.Pad.A();
            Save(s, $"{p}-list-screen-chosen");
            s.Pad.Down(12);
            Save(s, $"{p}-theme-settings-interface-lower");
            s.Pad.B();
            Reach(s, "BigMenuPage1");
            s.Pad.A();
            Save(s, $"{p}-theme-settings-themes");
            s.Pad.B();
            s.Pad.B();
            s.Settle();

            ThemedCollectionsTests.Choose(s, "Game Collection Settings");
            Save(s, $"{p}-collection-settings");
            s.Pad.Down(9);
            Save(s, $"{p}-collection-settings-lower");
            s.Pad.B();
            s.Settle();

            ThemedLibraryFlowTests.Choose(s, "Preferences");
            Save(s, $"{p}-preferences");
            foreach (int page in new[] { 0, 1, 2, 3, 4, 5 })
            {
                Reach(s, $"BigMenuPage{page}");
                s.Pad.A();
                string title = ((PreferencesWindow)Sheets(s).Current!).Form!.Shown.ToLowerInvariant().Replace(' ', '-');
                Save(s, $"{p}-preferences-{title}");
                s.Pad.B();
            }
            s.Pad.B();
            s.Settle();

            // Scale-up, caught part way: the pad menu at the start of its opening.
            s.Settings.BigPictureInterface.MenuOpeningEffect = EmuSen.Galaxia.Models.BigPictureInterface.OpeningScaleUp;
            s.Pad.Start();
            MenuPanel menu = s.Window.GetControl<MenuPanel>("PadMenuBig");
            menu.OpeningScale = 0.75;
            s.Capture().SavePng(Path.Combine(MetadataEditorPictureTool.PngFolder, $"{p}-menu-opening-0.75.png"));
            s.Pad.B();
        }

        private static ThemedSession ArtBookNext(double width, double height)
        {
            string media = Path.Combine(Path.GetTempPath(), "EmuSenSettingsMenusMedia");
            SyntheticLibrary.WriteMedia(media);
            return new ThemedSession(width, height, a => a.EsdeMediaDirectory = media, themeDirectory: ArtBookNextFactAttribute.Folder);
        }

        [MetadataEditorPngFact]
        public Task Synthetic_theme() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
                using (var s = new ThemedSession(w, h)) Walk(s, $"synthetic-{w}x{h}");
        }, default);

        [MetadataEditorPngFact]
        public Task Art_book_next() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
                using (ThemedSession s = ArtBookNext(w, h)) Walk(s, $"artbooknext-{w}x{h}");
        }, default);
    }
}
