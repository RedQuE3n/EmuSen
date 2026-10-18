using System;
using System.Diagnostics;
using System.IO;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Serenity;
using EmuSen.Serenity.Native;
using EmuSen.Serenity.Slang;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // The device lent through the frontend API's 1.1: the runtime makes each machine's lend from the presenter's device and takes inside frame, the presenter borrows it, and no path between frames stalls the core - see EmuSen_Backend.md §14.12.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class ApiDeviceLendTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenApiDeviceLend_" + Guid.NewGuid().ToString("N"));
        private readonly string _wanted = Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800";

        public ApiDeviceLendTests(ITestOutputHelper output)
        {
            _output = output;
            Assert.True(SerenityNative.Ready, SerenityNative.Report);
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = _root;
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // Every frame call returns within the limit, so a path that left a take owed would show as a frame that never comes back.
        private static CrystalFrameInfo Frame(CrystalSession s)
        {
            var info = new CrystalFrameInfo();
            var clock = Stopwatch.StartNew();
            s.Frame(ref info);
            Assert.True(clock.ElapsedMilliseconds < 5_000, $"a frame took {clock.ElapsedMilliseconds} ms: the core stalled");
            return info;
        }

        private static void Taken(SlangChain chain, SlangVulkan device, CrystalSession s, CrystalFrameInfo info, string when)
        {
            Assert.Equal(1u, info.picture_on_device);
            nint lend = s.PictureLend();
            Assert.NotEqual(0, lend);
            Assert.True(chain.AdvanceLent(SlangLend.Borrow(device, lend)), $"{when}: no picture was taken");
        }

        // Lent on open, a reset, held rewind's steps, undrawn frames and a session opened again (a new machine with a new lend) each leave the next drawn picture taken, and none stalls a frame.
        [Fact]
        public void Each_machine_is_lent_and_no_path_between_frames_stalls_the_core()
        {
            using SlangVulkan? device = SlangVulkan.TryCreate(out string report, _wanted, null, native: true);
            if (device is null) { _output.WriteLine($"not run: {report}"); return; }
            using (SlangLend? probe = SlangLend.TryCreate(device, out string? why))
                if (probe is null) { _output.WriteLine($"not run: {why}"); return; }
            string path = Path.Combine(_root, "triangle.psexe");
            File.WriteAllBytes(path, DiamondEnhancementsTests.Triangle());
            using var chain = new SlangChain(device, SlangPreset.Load(EmuSen.Serenity.Shaders.BuiltInPreset.Write(EmuSen.Serenity.Shaders.BuiltInShaders.AsFilter(ShaderEffect.None, false), null)));
            using var runtime = new CrystalRuntime();
            var open = new CrystalOpen { Path = path, NoBattery = true, Device = device.DeviceHandle };

            nint first;
            using (CrystalSession s = runtime.Open(open))
            {
                string console = System.Text.Json.JsonDocument.Parse(s.Info()).RootElement.GetProperty("console").GetString()!;
                runtime.SetSetting(console, "resolution", "2");
                for (int f = 0; f < 6; f++) Frame(s);
                CrystalFrameInfo info = Frame(s);
                Taken(chain, device, s, info, "after open");
                first = s.PictureLend();

                s.Reset();
                Frame(s);
                Taken(chain, device, s, Frame(s), "after a reset");
                Assert.Equal(first, s.PictureLend());

                s.Rewind(true, intervalFrames: 1);
                for (int f = 0; f < 6; f++) Frame(s);
                var step = new CrystalFrameInfo();
                for (int f = 0; f < 3; f++) s.RewindStep(1, ref step);
                Taken(chain, device, s, Frame(s), "after held rewind's steps");

                s.Speed(100, drawNone: true);
                for (int f = 0; f < 4; f++) Assert.Equal(0u, Frame(s).rendered);
                s.Speed(100);
                Frame(s);
                Taken(chain, device, s, Frame(s), "after undrawn frames");
            }

            using CrystalSession again = runtime.Open(open);
            for (int f = 0; f < 6; f++) Frame(again);
            Taken(chain, device, again, Frame(again), "in a session opened again");
            Assert.NotEqual(0, again.PictureLend());
            _output.WriteLine($"lends: {first:X} then {again.PictureLend():X}");
        }
    }
}
