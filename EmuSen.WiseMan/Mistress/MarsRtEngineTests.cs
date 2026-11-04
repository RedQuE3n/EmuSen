using EmuSen.Crystal.Systems.N64;
using EmuSen.Cores.Native;
using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Crystal.Cheats;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Mistress.Pages;

namespace EmuSen.WiseMan.Mistress
{
    // MarsRT, the N64's one engine, played in Mistress: frames, states, the battery save, cheats, rewind, and a build without its library - see EmuSen_Settings_Reference.md §4.44.
    [Collection(TestCollections.ProcessGlobals)]
    public class MarsRtEngineTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MarsRtEngineTests).GetTypeInfo().Assembly);


        // Block 5 of the 4 Kbit EEPROM, which the synthetic system never writes; it writes block 2.
        private const int Marked = 5 * 8, Written = 2 * 8;

        // The largest EEPROM a cartridge carries, 16 kilobits - see Mars_Save.md §2.
        private const int EepromSize = 0x800;
        private static readonly byte[] Marker = "MARSRT!!"u8.ToArray();

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMarsRtEngineTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public MarsRtEngineTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _rom = Path.Combine(roms, "System.z64");
            File.WriteAllBytes(_rom, SyntheticN64System.Build(rsp: true));
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States"), LogDirectory = Path.Combine(_root, "Logs") }.Save();

            // This collection runs alone, so the process-wide switch can be held off here and put back.
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static void Choose(string engine)
        {
            ConsolesConfig config = ConsolesConfig.Load();
            config.SetValue("N64", CoreCatalog.EngineKey, engine);
            config.Save();
        }

        // The N64 has one engine since Mars (C#) was retired, so its page in Settings offers it alone - see Mars_Native.md §6.7.
        [Fact]
        public Task The_N64_page_offers_one_engine() => Session.Dispatch(() =>
        {
            MainWindow window = SettingsDesk.Open(SettingsScreen.Consoles, "console:N64");
            Assert.True(!SettingsDesk.Keys(window).Contains("engine") || SettingsDesk.Row(window, "engine").Choices?.Count == 1, "the page offers a choice of engine");
            window.Close();
        }, default);

        // The mark is in a block the game never writes, so it is in the file afterwards only if the start read it - see Mars_Save.md §7.
        [Fact]
        public Task A_battery_save_is_read_when_MarsRT_starts_a_game_and_written_when_it_stops() => Session.Dispatch(() =>
        {
            Choose(CoreCatalog.MarsRtEngine);
            string save = SaveLibrary.SramPathFor(_rom, EmuSen.Cores.BatterySave.N64);
            byte[] original = Enumerable.Repeat((byte)0xFF, EepromSize).ToArray();
            Marker.CopyTo(original, Marked);
            Directory.CreateDirectory(Path.GetDirectoryName(save)!);
            File.WriteAllBytes(save, original);

            MainWindow window = Start();
            WaitFor(() => Game(window).TotalFrames > 40);
            Invoke(window, "ShowLibrary");

            byte[] file = File.ReadAllBytes(save);
            Assert.Equal(Marker, file[Marked..(Marked + 8)]);
            Assert.NotEqual(original[Written..(Written + 8)], file[Written..(Written + 8)]);
            window.Close();
        }, default);

        // A code typed into the window's list is decoded by the session's codecs and applied by the session at the frame's end.
        [Fact]
        public Task A_cheat_added_to_the_list_reaches_MarsRT_at_the_frame_s_end() => Session.Dispatch(() =>
        {
            Choose(CoreCatalog.MarsRtEngine);
            MainWindow window = Start();

            var (added, words) = MistressGame.Cheats(window).AddCode("80000300 0042", "Poke");
            Assert.True(added, words);
            using var listed = System.Text.Json.JsonDocument.Parse(MistressGame.Of(window).Crystal.CheatOp(EmuSen.Crystal.Client.Contract.CrystalSessionAbi.CheatList, "{}"));
            Assert.Single(listed.RootElement.GetProperty("cheats").EnumerateArray());

            WaitFor(() => MistressGame.Peek(window, "RDRAM", 0x300) == 0x42);
            window.Close();
        }, default);

        // With no MarsRT beside the program, an N64 game says which engine is missing, as every console with one discovered engine does - see VenusRT_Native.md §68.5.
        [Fact]
        public void With_no_marsrt_beside_the_program_an_n64_game_says_the_engine_is_missing()
        {
            CoreDiscovery.UseDirectories(new[] { Path.Combine(_root, "no-cores") });
            try
            {
                var refused = Assert.Throws<NotSupportedException>(() => CoreFactory.Create(_rom, engine: CoreCatalog.MarsRtEngine));
                Assert.StartsWith("This build has no N64 core: MarsRT (Rust) is missing", refused.Message);
            }
            finally
            {
                CoreDiscovery.UseDirectories(null);
            }
        }

        private MainWindow Start()
        {
            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", _rom, "System.z64");
            WaitFor(() => Game(window).TotalFrames > 20);
            return window;
        }

        private static long WaitUntilParked(MainWindow window)
        {
            long frames;
            do
            {
                frames = Game(window).TotalFrames;
                Thread.Sleep(250);
            }
            while (Game(window).TotalFrames != frames);

            return frames;
        }

        // Pumps the UI thread's queue, since the emulation thread reports back by posting to it.
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


        private static string StatePath(MainWindow window) =>
            (string)typeof(MainWindow).GetProperty("CurrentStatePath", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static TextBlock Status(MainWindow window) => window.GetControl<TextBlock>("StatusText");

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}
