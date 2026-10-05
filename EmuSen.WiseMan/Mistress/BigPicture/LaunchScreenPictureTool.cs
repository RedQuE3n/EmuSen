using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class Pass11PngFactAttribute : FactAttribute
    {
        public Pass11PngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes Pass 11's pictures to ~/.cache/emusen/bigpicture/png/pass11/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §33";
        }
    }

    // Pass 11's launch screen at 1280x800 and 1920x1200, written outside the repository beside the ES-DE captures - see EmuSen_BigPicture.md §33.
    [Collection(TestCollections.ProcessGlobals)]
    public class LaunchScreenPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(LaunchScreenPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "pass11");

        internal const string LongName = "Granite Choir and the Very Long Title That Keeps Going On (Synthetic)";

        private readonly ITestOutputHelper _out;

        public LaunchScreenPictureTool(ITestOutputHelper output) => _out = output;

        private void Save(RenderedFrame frame, string name)
        {
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            frame.SavePng(path);
            _out.WriteLine(path);
        }

        internal static ThemedSession Open(string duration, int w, int h, string? theme = null)
        {
            string? media = null;
            return new(w, h, themeDirectory: theme, settings: a =>
            {
                a.BigPictureInterface.LaunchScreenDuration = duration;
                a.EsdeMediaDirectory = media;
            }, roms: r =>
            {
                File.WriteAllBytes(Path.Combine(r, LongName + ".sfc"), SyntheticRom.BuildBlank());
                media = LaunchScreenTests.Media(Path.GetDirectoryName(r)!);
            });
        }

        // One launch of the game named, a picture at each moment asked for, then the game let go.
        private void Shoot(ThemedSession s, string game, string label, params int[] at)
        {
            for (int guard = 0; guard < 12 && s.Game != game; guard++) s.Pad.Down();
            Assert.Equal(game, s.Game);
            s.Pad.A();
            int done = 0;
            foreach (int ms in at)
            {
                s.Run(ms - done);
                done = ms;
                Save(s.Capture(), $"{label}-{ms:D4}ms");
            }
            s.Run(5000);
            LaunchScreenTests.Stop(s.Window);
            ThemedLibraryFlowTests.QuitGame(s);
            if (s.View != "gamelist") s.Pad.A();
        }

        [Pass11PngFact]
        public Task Pictures() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                string size = $"{w}x{h}";
                using (var s = Open(BigPictureInterface.LaunchNormal, w, h))
                {
                    ThemedLibraryPadTests.Enter(s, "snes");
                    Save(s.Capture(), $"before-{size}");
                    Shoot(s, LaunchScreenTests.Aurora, $"normal-marquee-{size}", 0, 17, 33, 50, 67, 83, 100, 117, 1500);
                    Shoot(s, LaunchScreenTests.Brass, $"normal-cover-{size}", 1500);
                    Shoot(s, LaunchScreenTests.Cobalt, $"normal-noart-{size}", 1500);
                    Shoot(s, LongName, $"normal-longname-{size}", 1500);
                }
                using (var s = Open(BigPictureInterface.LaunchPopup, w, h))
                {
                    ThemedLibraryPadTests.Enter(s, "snes");
                    Shoot(s, LaunchScreenTests.Aurora, $"popup-{size}", 0, 250, 500, 1500);
                }
                using (var s = Open(BigPictureInterface.LaunchBrief, w, h))
                {
                    ThemedLibraryPadTests.Enter(s, "nes");
                    s.Pad.A();
                    s.Run(1700);
                    s.Run(300);
                    Save(s.Capture(), $"failed-load-{size}");
                }
                if (ArtBookNextFactAttribute.Folder is { } abn && Directory.Exists(abn))
                {
                    using var s = Open(BigPictureInterface.LaunchNormal, w, h, abn);
                    ThemedLibraryPadTests.Enter(s, "snes");
                    Shoot(s, LaunchScreenTests.Aurora, $"abn-normal-{size}", 1500);
                }
            }
            using (var s = Open(BigPictureInterface.LaunchNormal, 1280, 800))
            {
                ThemeSettingsWindow sheet = ThemedSwitchesTests.OpenInterface(s);
                ThemedSwitchesTests.Named<EmuSen.LunaP.Controls.Dropdown>(sheet, "LaunchScreenDuration").Focus();
                s.Settle();
                Save(s.Capture(), "interface-tab-1280x800");
            }
        }, default);
    }
}
