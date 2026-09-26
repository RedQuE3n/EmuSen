using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using Avalonia.LogicalTree;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Windowing;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class ThemeListPngFactAttribute : FactAttribute
    {
        public ThemeListPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the theme list's pictures to ~/.cache/emusen/bigpicture/png/theme-list/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §19";
            else if (!File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml")))
                Skip = "Art Book Next is needed for these pictures - see EmuSen_BigPicture.md §12.5";
        }
    }

    // The Themes tab at 1280 by 800 with EmuSen's own look chosen and with Art Book Next chosen, and Preferences' row; outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemeListPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemeListPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "theme-list");

        private static readonly string MediaRoot = Path.Combine(Path.GetTempPath(), "EmuSenThemeListMedia");

        private readonly ITestOutputHelper _out;

        public ThemeListPictureTool(ITestOutputHelper output) => _out = output;

        private void Save(RenderedFrame frame, string name)
        {
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            frame.SavePng(path);
            _out.WriteLine(path);
        }

        private static Control SheetRoot(ThemedSession s)
        {
            var sheets = s.Window.GetControl<SheetLayer>("Sheets");
            return sheets.SheetOf(sheets.Current!)!;
        }

        private static void Click(ThemedSession s, string name)
        {
            SheetRoot(s).GetLogicalDescendants().OfType<Button>().First(b => b.Name == name).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            s.Settle();
        }

        [ThemeListPngFact]
        public Task Themes_tab_with_each_look_chosen() => Session.Dispatch(() =>
        {
            SyntheticLibrary.WriteMedia(MediaRoot);
            using var s = new ThemedSession(1280, 800, a => a.EsdeMediaDirectory = MediaRoot, themeDirectory: ArtBookNextFactAttribute.Folder);
            s.Run(500);
            ThemedLibraryFlowTests.Choose(s, "Theme Settings");
            s.Settle();
            s.Pad.R1();
            s.Settle();
            Save(s.Capture(), "themes-artbooknext-chosen-1280x800");

            Click(s, "ThemeUseBuiltIn");
            s.Run(200);
            Save(s.Capture(), "themes-emusen-chosen-1280x800");
            s.Pad.L1();
            s.Settle();
            Save(s.Capture(), "options-emusen-chosen-1280x800");
            s.Pad.B();
            s.Settle();

            ThemedLibraryFlowTests.Choose(s, "Preferences");
            s.Settle();
            TabControl tabs = SheetRoot(s).GetVisualDescendants().OfType<TabControl>().First();
            for (int i = 0; i < 6 && (tabs.SelectedItem as TabItem)?.Header as string != "Appearance"; i++) s.Pad.R1();
            s.Settle();
            SheetRoot(s).GetLogicalDescendants().OfType<Control>().First(c => c.Name == "BigPictureThemeDropdown").BringIntoView();
            s.Settle();
            Save(s.Capture(), "preferences-appearance-emusen-chosen-1280x800");
        }, default);
    }
}
