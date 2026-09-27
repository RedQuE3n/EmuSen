using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class Pass10PngFactAttribute : FactAttribute
    {
        public Pass10PngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes Pass 10's pictures to ~/.cache/emusen/bigpicture/png/pass10/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §37";
        }
    }

    // Pass 10's screensaver at 1280x800 and 1920x1200, written outside the repository beside the ES-DE captures - see EmuSen_BigPicture.md §37.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScreensaverPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ScreensaverPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "pass10");

        private readonly ITestOutputHelper _out;

        public ScreensaverPictureTool(ITestOutputHelper output) => _out = output;

        private void Save(ThemedSession s, string name)
        {
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            s.Capture().SavePng(path);
            _out.WriteLine(path);
        }

        // From the idle clock's end, a picture at each moment asked for.
        private void Shoot(ThemedSession s, string label, params int[] at)
        {
            s.Run(59000, step: 1000);
            while (ScreensaverTests.Saver(s) is null) s.Run(16);
            int done = 0;
            foreach (int ms in at)
            {
                s.Run(ms - done);
                done = ms;
                Save(s, $"{label}-{ms:D4}ms");
            }
            s.Pad.B();
        }

        [Pass10PngFact]
        public Task Pictures() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                string size = $"{w}x{h}";
                using (var s = ScreensaverTests.Open(i => i.ScreensaverTimer = 60000, w, h))
                {
                    ThemedLibraryPadTests.Enter(s, "snes");
                    Save(s, $"{size}-view");
                    Shoot(s, $"{size}-dim", 0, 50, 100, 167, 5000);
                    s.Settings.BigPictureInterface.ScreensaverType = BigPictureInterface.SaverBlack;
                    Shoot(s, $"{size}-black", 0, 70, 140);
                }

                using (var s = ScreensaverTests.Open(i => { i.ScreensaverTimer = 60000; i.ScreensaverType = BigPictureInterface.SaverSlideshow; }, w, h))
                {
                    ThemedCollectionsTests.Records(s).ToggleFavourite(ThemedCollectionsTests.Rom(s, ScreensaverTests.Aurora));
                    ThemedCollectionsTests.Refresh(s);
                    Shoot(s, $"{size}-slideshow", 0, 60, 117, 200, 217, 300, 450, 5000, 10000, 10500);
                    s.Settings.BigPictureInterface.ScreensaverSlideshowGameInfo = false;
                    Shoot(s, $"{size}-slideshow-nooverlay", 1000);
                    s.Settings.BigPictureInterface.ScreensaverSlideshowGameInfo = true;
                    s.Settings.BigPictureInterface.ScreensaverStretchImages = true;
                    Shoot(s, $"{size}-slideshow-stretch", 1000);
                }
            }

            using (var s = ScreensaverTests.Open())
            {
                ThemeSettingsWindow sheet = ThemedSwitchesTests.OpenInterface(s);
                Control row = ThemedSwitchesTests.Named<Control>(sheet, "ScreensaverTimer");
                row.BringIntoView();
                s.Settle();
                Save(s, "1280x800-interface-tab");
                ThemedSwitchesTests.Named<Control>(sheet, "ScreensaverSlideshowCustomDir").BringIntoView();
                s.Settle();
                Save(s, "1280x800-interface-tab-2");
                s.Pad.B();
            }
        }, default);
    }
}
