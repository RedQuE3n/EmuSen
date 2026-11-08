using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class MoonrisePngFactAttribute : FactAttribute
    {
        public MoonrisePngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes Moonrise's finished frame with the systems' bodies to ~/.cache/emusen/tmp/moonrise/raw/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_Kit.md §7.16";
        }
    }

    // Mistress's own splash at 1280 by 720 with the canvas's eleven consoles, held on its finished frame by start-up work that never ends: the reference for the Qt program's splash - see EmuSen_Kit.md §7.16.
    [Trait("Category", "nightly")]
    [Collection(TestCollections.ProcessGlobals)]
    public class MoonrisePictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MoonrisePictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "tmp", "moonrise", "raw");

        // The courts canvas's consoles, drawn in the systems table's order whatever the build's engines (EmuSen_Interface.md §24.6).
        public static readonly string[] Consoles = ["nes", "snes", "n64", "gb", "gbc", "sms", "md", "mcd", "psx", "ps2", "dreamcast"];

        private readonly ITestOutputHelper _out;

        public MoonrisePictureTool(ITestOutputHelper output) => _out = output;

        [MoonrisePngFact]
        public Task Bodies() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(1280, 720, settings: a => a.StartupSplash = true, startupWork: new TaskCompletionSource().Task);
            s.Run(StartupSplashTests.LengthMs + 100);
            LunaSplash splash = Assert.IsType<LunaSplash>(StartupSplashTests.Splash(s));
            Assert.True(splash.AtEnd);
            Assert.Equal(1.0, splash.Opacity);
            var bodies = StartupSplash.Bodies(RuntimeCatalog.Systems.Where(x => Consoles.Contains(x.Id)).Select(x => x with { Engines = ["drawn"] }));
            Assert.Equal(Consoles.Length, bodies.Count);
            splash.Bodies = bodies;
            splash.Caption = StartupSplash.Caption(bodies.Count);
            s.Settle();
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, "LunaSplash-bodies-1280x720.png");
            s.Capture().SavePng(path);
            _out.WriteLine($"{path} {splash.Caption}: {string.Join(" ", splash.Bodies.Select(b => b.Name))}");
        }, default);
    }
}
