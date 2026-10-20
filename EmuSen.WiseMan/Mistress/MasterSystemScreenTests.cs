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
using EmuSen.Mistress.Views;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Serenity;

namespace EmuSen.WiseMan.Mistress
{
    // The Master System's and the SG-1000's games drawn through the CRT on each console's own row - see EmuSen_CRT.md §31.
    [Collection(TestCollections.ProcessGlobals)]
    public class MasterSystemScreenTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MasterSystemScreenTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMasterSystemScreen", Guid.NewGuid().ToString("N"));
        private readonly string _roms;

        public MasterSystemScreenTests()
        {
            _roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            new AppSettings { RomDirectory = _roms, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // A game started with the CRT chosen for its console is drawn on that console's row, which differs from the standard line a console without one would get.
        [Theory]
        [InlineData("Master System", "Game.sms")]
        [InlineData("SG-1000", "Game.sg")]
        public Task A_game_draws_through_the_crt_on_its_console_s_own_row(string console, string name) => Session.Dispatch(() =>
        {
            string rom = Path.Combine(_roms, name);
            File.WriteAllBytes(rom, console == "SG-1000" ? SyntheticSmsRom.TmsPicture() : SyntheticSmsRom.Picture());
            var config = new GraphicsConfig();
            config.SetValue(console, GraphicsSettingsWindow.ScreenFilterKey, CrtFilter.Name);
            config.Save();
            var window = new MainWindow().WithoutBattery();
            window.Show();
            typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!
                .Invoke(window, new object[] { rom, name });
            GameFrameControl frame = window.GetControl<GameFrameControl>("GameFrame");
            Assert.Same(CrtFilter.Filter, frame.ActiveFilter);
            Assert.Equal(console, frame.FilterConsole);
            window.Close();

            using var core = Assert.IsType<CoreEngine>(CoreFactory.Load(rom, options: TestCores.NoBattery).Core);
            for (int f = 0; f < 60; f++) core.RunFrame();
            byte[] picture = core.GetFrameBufferRgba().ToArray();
            (int width, int height) = (core.ScreenWidth, core.ScreenHeight);
            Assert.Equal((256, 192), (width, height));
            Assert.Contains(picture, b => b != 0);

            if (Device() is not { } device) return;
            byte[] Draw(string on) => ShaderBench.Picture(new GameFrameControl { FilterConsole = on, ActiveFilter = CrtFilter.Filter }, new[] { picture, picture }, width, height, 1, 640, 480, device);
            byte[] drawn = Draw(console);
            Assert.Contains(drawn, b => b > 16);
            Assert.NotEqual(Convert.ToHexString(Draw(ScreenFilter.AnyConsole)), Convert.ToHexString(drawn));
        }, default);

        // The GL device the CRT is drawn on, as ScreenFilterRenderTests finds it; none, and the drawing is not checked.
        private static string? Device()
        {
            string name = Environment.GetEnvironmentVariable(ShaderBenchTests.GlVariable) ?? "6800";
            try { using var gl = ShaderBench.GlContext.Create(name); return name; }
            catch (Exception) { return null; }
        }
    }
}
