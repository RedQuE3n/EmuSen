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
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class StageFPngFactAttribute : FactAttribute
    {
        public StageFPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes stage (f)'s pictures to ~/.cache/emusen/bigpicture/png/stage-f/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §16";
        }
    }

    // The PNGs of §16 at 1280 by 800: the settings sheet's two tabs and the About sheet over Art Book Next, and each grid variant; written outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class StageFPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(StageFPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "stage-f");

        private static readonly string MediaRoot = Path.Combine(Path.GetTempPath(), "EmuSenStageFMedia");

        private readonly ITestOutputHelper _out;

        public StageFPictureTool(ITestOutputHelper output) => _out = output;

        private static ThemedSession ArtBookNext(Action<EmuSen.Galaxia.Models.AppSettings>? more = null)
        {
            SyntheticLibrary.WriteMedia(MediaRoot);
            return new ThemedSession(1280, 800, a => { a.EsdeMediaDirectory = MediaRoot; more?.Invoke(a); }, themeDirectory: ArtBookNextFactAttribute.Folder);
        }

        private void Save(RenderedFrame frame, string name)
        {
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            frame.SavePng(path);
            _out.WriteLine(path);
        }

        // Art Book Next's three grid variants, at rest on the SNES gamelist and after two rows down, and the synthetic grid mid-step.
        [StageFPngFact]
        public Task Grid_variants() => Session.Dispatch(() =>
        {
            foreach (string variant in new[] { "gamelist-grid-cover", "gamelist-grid-cover-steamgriddb", "gamelist-grid-screenshot" })
            {
                using ThemedSession s = ArtBookNext(a => a.BigPicture[EmuSen.Mistress.Views.ThemeSettingsWindow.Key(ArtBookNextFactAttribute.Folder)] = new EmuSen.Galaxia.Models.BigPictureChoices { Variant = variant });
                foreach (EmuSen.Mistress.BigPicture.Scene.SceneGame g in SyntheticLibrary.Games(SyntheticTheme.Snes, ".sfc"))
                    if (!File.Exists(Path.Combine(s.RomDirectory, g.File))) File.WriteAllBytes(Path.Combine(s.RomDirectory, g.File), SyntheticRom.BuildBlank());
                typeof(EmuSen.Mistress.Views.MainWindow).GetMethod("RefreshLibrary", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(s.Window, null);
                s.Settle();
                ThemedLibraryPadTests.Enter(s, "snes");
                s.Run(500);
                Save(s.Capture(), $"{variant}-1280x800");
                s.Pad.Down(2);
                s.Run(600);
                Save(s.Capture(), $"{variant}-down2-1280x800");
            }
        }, default);

        [StageFPngFact]
        public Task Settings_and_about_sheets() => Session.Dispatch(() =>
        {
            Assert.True(File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml")), "Art Book Next is needed for these pictures");
            using ThemedSession s = ArtBookNext();
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Run(500);
            ThemedLibraryFlowTests.Choose(s, "Theme Settings");
            s.Settle();
            Save(s.Capture(), "settings-options-1280x800");
            s.Pad.R1();
            s.Settle();
            Save(s.Capture(), "settings-themes-1280x800");
            var sheets = s.Window.GetControl<SheetLayer>("Sheets");
            Control root = sheets.SheetOf(sheets.Current!)!;
            Button about = root.GetLogicalDescendants().OfType<Button>().First(b => b.Name?.StartsWith("ThemeAbout.") == true);
            about.RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            s.Settle();
            Save(s.Capture(), "about-artbooknext-1280x800");
        }, default);
    }
}
