using System;
using System.Diagnostics;
using System.IO;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Serenity;
using EmuSen.Serenity.Native;
using EmuSen.Serenity.Slang;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // ApiDeviceLendTests' cases on the Rust runtime, through its exports: each machine lent the presenter's device, its pictures taken inside the frame, no path between frames stalling the core - see EmuSen_Backend.md §29.20.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class ApiDeviceLendRustTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenApiDeviceLendRust_" + Guid.NewGuid().ToString("N"));
        private readonly string _wanted = Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800";

        public ApiDeviceLendRustTests(ITestOutputHelper output)
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

        // Set when a frame never came back, so the runtime it holds is left rather than freed under it.
        private bool _stalled;

        // Every frame call returns within the limit, watched from another thread, so a stalled core fails the case rather than hanging it.
        private CrystalFrameInfo Frame(CrystalImpl i, nint rt, uint s)
        {
            var clock = Stopwatch.StartNew();
            var frame = System.Threading.Tasks.Task.Run(() => i.FrameInfo(rt, s));
            _stalled = !frame.Wait(10_000);
            Assert.False(_stalled, "a frame did not return in 10 s: the core stalled");
            Assert.True(clock.ElapsedMilliseconds < 5_000, $"a frame took {clock.ElapsedMilliseconds} ms: the core stalled");
            return frame.Result;
        }

        private void Taken(SlangChain chain, SlangVulkan device, CrystalImpl i, nint rt, uint s, CrystalFrameInfo info, string when)
        {
            Assert.Equal(1u, info.picture_on_device);
            nint lend = i.PictureLend(rt, s);
            Assert.NotEqual(0, lend);
            Assert.True(chain.AdvanceLent(SlangLend.Borrow(device, lend)), $"{when}: no picture was taken");
        }

        [Fact]
        public void Each_machine_of_the_rust_runtime_is_lent_and_no_path_between_frames_stalls_the_core()
        {
            using SlangVulkan? device = SlangVulkan.TryCreate(out string report, _wanted, null, native: true);
            if (device is null) { _output.WriteLine($"not run: {report}"); return; }
            using (SlangLend? probe = SlangLend.TryCreate(device, out string? why))
                if (probe is null) { _output.WriteLine($"not run: {why}"); return; }
            string path = Path.Combine(_root, "triangle.psexe");
            File.WriteAllBytes(path, DiamondEnhancementsTests.Triangle());
            using var chain = new SlangChain(device, SlangPreset.Load(EmuSen.Serenity.Shaders.BuiltInPreset.Write(EmuSen.Serenity.Shaders.BuiltInShaders.AsFilter(ShaderEffect.None, false), null)));
            CrystalImpl i = CrystalImpl.Native;
            nint rt = i.New(new[] { AppContext.BaseDirectory, Path.Combine(AppContext.BaseDirectory, "cores") });
            try
            {
                uint s = uint.Parse(i.Open(rt, path, flags: CrystalAbi.OpenNoBattery, device: device.DeviceHandle));
                string console = System.Text.Json.JsonDocument.Parse(i.Read("session_info", rt, s)).RootElement.GetProperty("console").GetString()!;
                Assert.Equal("0", i.SettingsSet(rt, 0, 0, console, "resolution", "2"));
                for (int f = 0; f < 6; f++) Frame(i, rt, s);
                Taken(chain, device, i, rt, s, Frame(i, rt, s), "after open");
                nint first = i.PictureLend(rt, s);

                Assert.Equal("0", i.Call("reset", rt, s));
                Frame(i, rt, s);
                Taken(chain, device, i, rt, s, Frame(i, rt, s), "after a reset");
                Assert.Equal(first, i.PictureLend(rt, s));

                Assert.Equal("0", i.RewindSet(rt, s, true, 1, 0, 0));
                for (int f = 0; f < 6; f++) Frame(i, rt, s);
                for (int f = 0; f < 3; f++) Assert.StartsWith("1 ", i.RewindStep(rt, s, 1));
                Taken(chain, device, i, rt, s, Frame(i, rt, s), "after held rewind's steps");

                Assert.Equal("0", i.Speed(rt, s, 100, 300, 25, CrystalAbi.SpeedDrawNone));
                for (int f = 0; f < 4; f++) Assert.Equal(0u, Frame(i, rt, s).rendered);
                Assert.Equal("0", i.Speed(rt, s, 100, 300, 25, 0));
                Frame(i, rt, s);
                Taken(chain, device, i, rt, s, Frame(i, rt, s), "after undrawn frames");
                Assert.Equal("0", i.Call("close", rt, s));

                uint again = uint.Parse(i.Open(rt, path, flags: CrystalAbi.OpenNoBattery, device: device.DeviceHandle));
                for (int f = 0; f < 6; f++) Frame(i, rt, again);
                Taken(chain, device, i, rt, again, Frame(i, rt, again), "in a session opened again");
                _output.WriteLine($"lends: {first:X} then {i.PictureLend(rt, again):X}");

                uint none = uint.Parse(i.Open(rt, path, flags: CrystalAbi.OpenNoBattery));
                for (int f = 0; f < 3; f++) Assert.Equal(0u, Frame(i, rt, none).picture_on_device);
                Assert.Equal(0, i.PictureLend(rt, none));
            }
            finally
            {
                // A machine whose pictures were never taken can hold its free on the device, so the free is watched too.
                if (!_stalled && !System.Threading.Tasks.Task.Run(() => i.Free(rt)).Wait(10_000)) _output.WriteLine("freeing the runtime did not return in 10 s");
            }
        }
    }
}
