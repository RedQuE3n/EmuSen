using EmuSen.Cores.Native;
using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The GB tab's Model row, and a game loaded in Mistress on the console it names - see EmuSen_Settings_Reference.md §4.47.
    [Collection(TestCollections.ProcessGlobals)]
    public class MercuryModelSettingTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMercuryModelTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public MercuryModelSettingTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _rom = Path.Combine(roms, "Mono.gb");
            File.WriteAllBytes(_rom, SyntheticGbRom.Build(patches: (0, new byte[] { 0x18, 0xFE })));
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States") }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        [Fact]
        public Task The_gb_tab_offers_auto_game_boy_and_game_boy_color_and_stores_the_choice() => UiTest.Run(() =>
        {
            var config = new GraphicsConfig();
            string? told = null;
            var window = new GraphicsSettingsWindow(config, console => told = console, "GB");
            window.Show();

            Dropdown model = window.GetLogicalDescendants().OfType<Dropdown>().Where(d => d.Name == $"GB.{MercuryEngines.ModelKey}").Distinct().Single();
            Assert.Equal(new[] { MercuryEngines.ModelAuto, MercuryEngines.ModelGameBoy, MercuryEngines.ModelGameBoyColor }, model.Items.Cast<object>().Select(o => o.ToString()));
            Assert.Equal(MercuryEngines.ModelAuto, model.SelectedItem);
            UiTest.AssertLaidOut(window, "graphics-gb-model");

            model.SelectedItem = MercuryEngines.ModelGameBoyColor;
            Dispatcher.UIThread.RunJobs();
            Assert.Equal("GB", told);
            Assert.Equal(MercuryEngines.ModelGameBoyColor, ConsolesConfig.Load().Value("GB", MercuryEngines.ModelKey));
            UiTest.AssertLaidOut(window, "graphics-gb-model-color");
            window.Close();
        });

        // The stored choice reaches the core before the first frame, and a colour cartridge's GB tab is the same tab.
        [Theory]
        [InlineData((byte)0x00, MercuryEngines.ModelGameBoyColor, "GBC")]
        [InlineData((byte)0x80, MercuryEngines.ModelGameBoy, "GB")]
        public Task A_game_runs_on_the_console_the_gb_tab_names(byte cgb, string model, string runsAs) => UiTest.Run(() =>
        {
            ConsolesConfig config = ConsolesConfig.Load();
            config.SetValue("GB", MercuryEngines.ModelKey, model);
            config.Save();
            File.WriteAllBytes(_rom, SyntheticGbRom.Build(cgbFlag: cgb, patches: (0, new byte[] { 0x18, 0xFE })));

            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", _rom, "Mono.gb");
            WaitFor(() => Game(window).TotalFrames > 20);

            WindowGame game = Game(window);
            Assert.Equal(runsAs, MistressGame.Info(window).GetProperty("system").GetString()!.ToUpperInvariant());
            Assert.Equal(model, MistressGame.Runtime.Setting("GB", MercuryEngines.ModelKey));
            Assert.Equal("GB", (string)typeof(MainWindow).GetField("_activeConsole", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!);
            window.Close();
        });

        private static void WaitFor(Func<bool> condition)
        {
            var clock = Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("timed out waiting for the emulation thread");
                Thread.Sleep(2);
            }
        }

        private static WindowGame Game(MainWindow window) => new(window);

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}
