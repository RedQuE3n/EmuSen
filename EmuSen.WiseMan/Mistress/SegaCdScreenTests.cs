using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Serenity;

namespace EmuSen.WiseMan.Mistress
{
    // The Sega CD on the Genesis's screen and pads, by its pack's AttachedTo and not by its name in more lists - see EmuSen_Serenity.md §3.12.
    [Collection(TestCollections.ProcessGlobals)]
    public class SegaCdScreenTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(SegaCdScreenTests).GetTypeInfo().Assembly);

        private const string SegaCd = "Sega CD", Genesis = "Genesis";

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSegaCdScreen", Guid.NewGuid().ToString("N"));
        private readonly string _disc;

        public SegaCdScreenTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
            _disc = Path.Combine(roms, "Game.iso");
            File.WriteAllBytes(_disc, SyntheticMdRom.Disc("U"));
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        [Fact]
        public void The_sega_cd_and_the_32x_take_the_genesiss_screen_and_pads_and_a_console_of_its_own_keeps_its()
        {
            Assert.Equal(new[] { Genesis, Genesis, Genesis, "SNES", "GB" }, new[] { Genesis, SegaCd, "32X", "SNES", "GB" }.Select(CoreCatalog.HostOf));
            Assert.True(GraphicsSettingsWindow.HasOverscan(SegaCd));
            Assert.Equal(ScreenFilters.NamesFor(Genesis), ShaderCatalog.BuiltIns(SegaCd).Select(e => e.Stored));
            Assert.True(ControllerDiagrams.IsGenesis(SegaCd) && ControllerDiagrams.IsGenesis("32X") && !ControllerDiagrams.IsGenesis("SNES"));
            Assert.Equal(LayoutOf(Genesis), LayoutOf(SegaCd));
            // Lottes holds no console's constants, so it suits every console that had a television.
            Assert.Equal(CrtFilter.TelevisionConsoles, CrtFilters.Lottes.Consoles);
        }

        private static string LayoutOf(string console) => ControllerDiagrams.LayoutFor(console, ControllerDiagrams.GenesisThreeButton).ToString();

        [Fact]
        public Task The_sega_cds_shader_menu_lists_the_crt_filter() => Session.Dispatch(() =>
        {
            Assert.Contains(SegaCd, CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console));
            var window = new ShaderSettingsWindow(new GraphicsConfig(), null, SegaCd);
            window.Show();
            ShaderPanel panel = window.PanelFor(SegaCd);
            var listed = panel.List.Models.Select(e => e.Stored).ToList();
            Assert.Contains(CrtFilter.Name, listed);
            Assert.Contains(CrtFilters.Lottes.Name, listed);
            Assert.Equal(window.PanelFor(Genesis).List.Models.Select(e => e.Stored), listed);
            window.Close();
        }, default);

        // A Sega CD game started with the CRT chosen for the Sega CD: the frame control draws it on the Genesis's line, which differs from the standard line a console without a row would get.
        [Fact]
        public Task A_sega_cd_frame_draws_through_the_crt_on_the_genesiss_line() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            config.SetValue(SegaCd, GraphicsSettingsWindow.ScreenFilterKey, CrtFilter.Name);
            config.Save();
            var window = new MainWindow().WithoutBattery();
            window.Show();
            typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!
                .Invoke(window, new object[] { _disc, "Game.iso" });
            GameFrameControl frame = window.GetControl<GameFrameControl>("GameFrame");
            Assert.Same(CrtFilter.Filter, frame.ActiveFilter);
            Assert.Equal(Genesis, frame.FilterConsole);
            window.Close();

            using var core = Assert.IsType<CoreEngine>(CoreFactory.Load(_disc, options: TestCores.NoBattery).Core);
            for (int f = 0; f < 120; f++) core.RunFrame();
            byte[] picture = core.GetFrameBufferRgba().ToArray();
            (int width, int height) = (core.ScreenWidth, core.ScreenHeight);
            Assert.Contains(picture, b => b != 0);

            byte[]? Draw(string console)
            {
                if (ScreenFilterDevice() is not { } device) return null;
                var control = new GameFrameControl { FilterConsole = console, ActiveFilter = CrtFilter.Filter };
                return ShaderBench.Picture(control, new[] { picture }, width, height, 1, 640, 480, device);
            }
            if (Draw(frame.FilterConsole!) is not { } drawn) return;
            Assert.Contains(drawn, b => b > 16);
            Assert.NotEqual(Convert.ToHexString(Draw(ScreenFilter.AnyConsole)!), Convert.ToHexString(drawn));
        }, default);

        // The GL device the CRT is drawn on, as ScreenFilterRenderTests finds it; none, and the drawing is not checked.
        private static string? ScreenFilterDevice()
        {
            string name = Environment.GetEnvironmentVariable(ShaderBenchTests.GlVariable) ?? "6800";
            try { using var gl = ShaderBench.GlContext.Create(name); return name; }
            catch (Exception) { return null; }
        }
    }
}
