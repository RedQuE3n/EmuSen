using EmuSen.Crystal.Systems.N64;
using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MarsRT on the core ABI v1 alone: its sidecar, the shim a subclass of the v1 adapter, the generic adapter running it as the shim does, a held button across a state, and the kit - see Mars_Native.md §7.
    [Collection(TestCollections.ProcessGlobals)]
    public class MarsRtCoreAbiTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMarsRtV1_" + Guid.NewGuid().ToString("N"));
        private readonly ITestOutputHelper _output;

        public MarsRtCoreAbiTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDirectories(null);
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static DiscoveredCore Discovered => CoreDiscovery.Found.Single(c => c.Info.Id == "marsrt");

        private string Rom(byte[] image, string name = "game.z64")
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, image);
            return path;
        }

        [Fact]
        public void The_build_writes_marsrts_sidecar_and_the_shim_loads_it_through_the_v1_exports_alone()
        {
            Assert.Equal("MarsRT (Rust)", Discovered.EngineName);
            Assert.Equal(new[] { ".z64", ".n64", ".v64" }, Discovered.Info.Systems.Single().Extensions);
            Assert.True(MarsRt.Available, MarsRt.Report);
            CoreLibrary library = MarsRt.Library!;
            Assert.Equal(Path.GetFullPath(Discovered.Sidecar.SidecarPath), Path.GetFullPath(CoreSidecar.PathFor(library.Path)));
            Assert.Equal(0, MarsRt.Export("mars_machine_load_rom"));
            PreStableExports.AssertNone(library.Path);
            Assert.Equal(MarsRt.VideoSettings.Select(s => s.Key).Order(), library.Settings.Where(s => !s.Hidden).Select(s => s.Key).Order());
            ICore core = CoreFactory.Create("game.z64", engine: CoreCatalog.MarsRtEngine, options: TestCores.NoBattery);
            Assert.IsType<CoreEngine>(core);
            Assert.Null(CoreFactory.EngineNotice("game.z64", CoreCatalog.MarsRtEngine, core));
        }

        // The generic adapter, with no shim, is what a v1 host other than Mistress runs; it must see the game the shim sees.
        [Fact]
        public void The_v1_adapter_over_marsrt_runs_a_game_exactly_as_the_shim()
        {
            string rom = Rom(SyntheticN64Rom.BuildHomebrew(N64SaveType.Eeprom4k));
            var settings = new System.Collections.Generic.Dictionary<string, string> { ["ExpansionPak"] = "true" };
            using var shim = new MarsRtTestCore(expansionPak: true) { Options = TestCores.NoBattery };
            using var adapter = new CoreEngine(MarsRt.Library!, settings) { Options = TestCores.NoBattery };
            shim.LoadRom(rom);
            adapter.LoadRom(rom);
            Assert.Equal((shim.ScreenWidth, shim.ScreenHeight, shim.StateVersion), (adapter.ScreenWidth, adapter.ScreenHeight, adapter.StateVersion));
            Assert.Equal(shim.SupportedButtons, adapter.SupportedButtons);
            Assert.Equal(shim.SupportedAxes, adapter.SupportedAxes);
            for (int f = 0; f < 240; f++)
            {
                foreach (ICore core in new ICore[] { shim, adapter })
                {
                    core.SetButton(0, PadButton.Start, f % 60 < 4);
                    core.SetAxis(0, PadAxis.LeftX, Math.Sin(f / 10.0));
                    core.SetAxis(0, PadAxis.RightY, f % 40 < 10 ? 1 : 0);
                    core.RunFrame();
                }
                Assert.Equal(shim.GetFrameBufferRgba(), adapter.GetFrameBufferRgba());
                Assert.Equal(shim.DequeueAudioSamples(int.MaxValue), adapter.DequeueAudioSamples(int.MaxValue));
                if (f % 40 == 0) Assert.Equal(State(shim), State(adapter));
            }
            Assert.Equal(State(shim), State(adapter));
        }

        // The adapter's frame log reads a value in the N64 pack's byte order, the big end first, as the shim did - see EmuSen_Backend.md §18.7.
        [Fact]
        public void The_v1_adapter_logs_an_n64_value_big_end_first()
        {
            string rom = Rom(SyntheticN64Rom.BuildHomebrew(N64SaveType.Eeprom4k));
            using var adapter = new CoreEngine(MarsRt.Library!, new System.Collections.Generic.Dictionary<string, string> { ["ExpansionPak"] = "true" }) { Options = TestCores.NoBattery };
            adapter.LoadRom(rom);
            adapter.WriteSpace("RDRAM", 0x7F_0000, 0x12);
            adapter.WriteSpace("RDRAM", 0x7F_0001, 0x34);
            int id = adapter.FrameLog.AddEntry("RDRAM", 0x7F_0000, 2);
            adapter.RunFrame();
            Assert.Equal(0x1234, adapter.FrameLog.GetSamples(id).Single().Value);
        }

        // The generic target reads the processor's view, a space of 4 GiB, at the address asked, not wrapped at its clamped size - see EmuSen_Backend.md §18.10.
        [Fact]
        public void The_generic_target_reads_the_processor_s_view_at_kseg0()
        {
            string rom = Rom(SyntheticN64Rom.BuildHomebrew(N64SaveType.Eeprom4k));
            using var adapter = new CoreEngine(MarsRt.Library!, new System.Collections.Generic.Dictionary<string, string> { ["ExpansionPak"] = "true" }) { Options = TestCores.NoBattery };
            adapter.LoadRom(rom);
            adapter.WriteSpace("RDRAM", 0x40_0400, 0x5A);
            var cpu = adapter.CreateDebugTarget().GetMemorySpaces().Single(s => s.Name == "CPU");
            Assert.Equal(0x5A, cpu.Read(unchecked((int)0x8040_0400u)));
            Assert.Equal(0x5A, cpu.Read(unchecked((int)0xA040_0400u)));
        }

        private static byte[] State(ICore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        // EmuSen_NativeCores.md §8.2's R3: a state saved with a button held, loaded and run on pressing nothing more, is the run that went on from it.
        [Fact]
        public void A_state_saved_with_a_button_held_runs_on_as_the_run_that_went_on()
        {
            string rom = Rom(SyntheticN64Rom.BuildHomebrew(N64SaveType.Eeprom4k));
            using var through = new MarsRtTestCore(expansionPak: true) { Options = TestCores.NoBattery };
            through.LoadRom(rom);
            for (int f = 0; f < 120; f++)
            {
                through.SetButton(0, PadButton.Start, f % 40 < 3);
                through.SetButton(0, PadButton.A, f >= 100);
                through.SetAxis(0, PadAxis.LeftX, 0.75);
                through.RunFrame();
            }
            byte[] held = State(through);
            using var loaded = new MarsRtTestCore(expansionPak: true) { Options = TestCores.NoBattery };
            loaded.LoadRom(rom);
            loaded.LoadState(held);
            Assert.Equal(held, State(loaded));
            for (int f = 0; f < 60; f++)
            {
                through.RunFrame();
                loaded.RunFrame();
                Assert.True(State(through).AsSpan().SequenceEqual(State(loaded)), $"frame {f} differs after the load");
                Assert.Equal(through.GetFrameBufferRgba(), loaded.GetFrameBufferRgba());
            }
        }

        // The conformance kit on synthetic cartridges, one with each kind of save chip it builds: all 22 cases a core without a drive has.
        [Fact]
        public void The_kit_passes_on_synthetic_cartridges()
        {
            string runner = Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "emusen-core-conform.exe" : "emusen-core-conform");
            Assert.True(File.Exists(runner), $"no kit runner at {runner}");
            foreach (var (name, image) in new[] { ("plain.z64", SyntheticN64Rom.Build()), ("eeprom.z64", SyntheticN64Rom.BuildHomebrew(N64SaveType.Eeprom4k)), ("flash.v64", SyntheticN64Rom.ToByteSwapped(SyntheticN64Rom.BuildHomebrew(N64SaveType.FlashRam))) })
            {
                var start = new ProcessStartInfo(runner) { RedirectStandardOutput = true, RedirectStandardError = true };
                foreach (string a in new[] { "--core", MarsRt.Library!.Path, "--image", Rom(image, name), "--frames", "120" }) start.ArgumentList.Add(a);
                using var process = Process.Start(start)!;
                var stderr = process.StandardError.ReadToEndAsync();
                string output = process.StandardOutput.ReadToEnd();
                process.WaitForExit();
                _output.WriteLine($"{name}:\n{output}{stderr.Result}");
                Assert.True(process.ExitCode == 0, output + stderr.Result);
                Assert.Equal(22, output.Split('\n').Count(l => l.StartsWith("pass C")));
            }
        }

        // MarsRT's component exports are gone, its state handle's codes read in its own words, and the handle is a v1 machine of the size asked (moved from the deleted NativeHostTests).
        [Fact]
        public void MarsRTs_state_handle_is_a_v1_machine_with_its_own_words()
        {
            Assert.True(MarsRt.Available, MarsRt.Report);
            Assert.NotEqual(0, MarsRt.Export("emusen_core_create"));
            foreach (string gone in new[] { "mars_rsp_new", "mars_machine_new", "no_such_export" }) Assert.Equal(0, MarsRt.Export(gone));
            Assert.Equal("not a Mars save state", MarsMachine.Describe(-3));
            Assert.Equal("the buffer is too small", MarsMachine.Describe(-7));
            Assert.Equal("an RDRAM size that is neither 4 MB nor 8 MB", MarsMachine.Describe(-12));
            using var machine = new MarsMachine(4 * 1024 * 1024);
            Assert.Equal((0, 4 * 1024 * 1024), (machine.LoadedKind, machine.RdramBytes));
            Assert.NotEmpty(machine.Save(snapshot: false));
            Assert.NotEmpty(machine.Save(snapshot: true));
        }
    }
}
