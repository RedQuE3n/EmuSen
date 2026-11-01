using System;
using System.IO;
using System.Linq;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Serenity;
using EmuSen.Serenity.Native;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // The frontend API's 1.4: a hosted core lent a ring on the presenter's device, each frame's picture in it drawn through the console's filters as the same frame read back and uploaded, and one frame advance - see EmuSen_Backend.md §33.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class ApiRingTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly LibretroEntryFixture _entry = new(systemId: "playstation2");
        private readonly string _wanted = Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800";

        public ApiRingTests(ITestOutputHelper output)
        {
            _output = output;
            Assert.True(SerenityNative.Ready, SerenityNative.Report);
        }

        public void Dispose() => _entry.Dispose();

        private string Disc()
        {
            var disc = SyntheticPs2Disc.Dvd();
            "TCG1"u8.CopyTo(disc.Iso);
            return disc.WriteSet(_entry.Root, "Ringed");
        }

        private static CrystalFrameInfo Frame(CrystalSession s)
        {
            var info = new CrystalFrameInfo { size = (uint)System.Runtime.InteropServices.Marshal.SizeOf<CrystalFrameInfo>() };
            s.Frame(ref info);
            return info;
        }

        [Fact]
        public void A_hosted_cores_pictures_in_the_lent_ring_draw_as_the_frames_read_back_and_one_frame_advance_holds()
        {
            _output.WriteLine($"runtime: {CrystalRuntime.Implementation}");
            using SlangVulkan? device = SlangVulkan.TryCreate(out string report, _wanted, null, native: true);
            if (device is null) { _output.WriteLine($"not run: {report}"); return; }
            using var runtime = new CrystalRuntime(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory }, PlayerCores = new[] { _entry.Cores } });
            string disc = Disc();
            using (CrystalSession first = runtime.Open(new CrystalOpen { Path = disc, Engine = _entry.Found.EngineName, NoBattery = true }))
                first.SetSetting("tc_hw", "vulkan", game: false);
            var filters = new (string Name, SlangPreset Preset)[]
            {
                ("none", SlangPreset.Load(BuiltInPreset.Write(BuiltInShaders.AsFilter(ShaderEffect.None, false), null))),
                (CrtFilter.Filter.Name, SlangPreset.Load(BuiltInPreset.Write(CrtFilter.Filter, "PlayStation 2"))),
            };
            var chains = filters.Select(f => (f.Name, FromRing: new SlangChain(device, f.Preset), Uploaded: new SlangChain(device, f.Preset))).ToArray();
            try
            {
                using (CrystalSession s = runtime.Open(new CrystalOpen { Path = disc, Engine = _entry.Found.EngineName, NoBattery = true, Device = device.DeviceHandle, Ring = true }))
                {
                    nint handle = 0;
                    for (int n = 0; n < 6; n++)
                    {
                        CrystalFrameInfo f = Frame(s);
                        Assert.Equal((1u, 1u, 320, 240, 0u), (f.ran, f.picture_in_ring, f.width, f.height, f.fields));
                        Assert.Equal(0u, f.picture_on_device);
                        nint now = s.PictureRing();
                        Assert.NotEqual(0, now);
                        if (n > 0) Assert.Equal(handle, now);
                        handle = now;
                        SlangRing ring = SlangRing.Borrow(device, handle);
                        byte[] picture = new byte[s.PictureBytes];
                        s.CopyPicture(picture);
                        foreach (var (name, fromRing, uploaded) in chains)
                        {
                            fromRing.AdvanceRing(ring, (int)f.ring_slot, f.ring_value, f.width, f.height);
                            uploaded.Advance(picture, f.width, f.height, 1);
                            (int w, int h) = name == "none" ? (f.width, f.height) : (640, 480);
                            byte[] got = fromRing.Render(w, h), want = uploaded.Render(w, h);
                            Assert.True(got.AsSpan().SequenceEqual(want), $"frame {n}, {name}: the ring's picture drawn differs from the read-back one in {got.Zip(want).Count(p => p.First != p.Second)} bytes");
                        }
                    }
                    s.FrameAdvance();
                    CrystalFrameInfo one = Frame(s), held = Frame(s);
                    Assert.Equal((1u, 1u, 0u, 1u), (one.ran, one.picture_in_ring, held.ran, held.paused));
                    Assert.Equal(one.total_frames, held.total_frames);
                    TakeFrom(chains[0].FromRing, device, s, one);
                }
                using (CrystalSession plain = runtime.Open(new CrystalOpen { Path = disc, Engine = _entry.Found.EngineName, NoBattery = true, Device = device.DeviceHandle }))
                {
                    CrystalFrameInfo f = Frame(plain);
                    Assert.Equal((1u, 0u, 0), (f.ran, f.picture_in_ring, plain.PictureRing()));
                }
            }
            finally
            {
                foreach (var (_, fromRing, uploaded) in chains) { fromRing.Dispose(); uploaded.Dispose(); }
            }
            _output.WriteLine("6 frames from the API's ring through none and the CRT filter for PlayStation 2, each the read-back picture's drawing; one frame advanced and held");
        }

        // While the screen is not drawing on the device, each ring frame given back through SlangRingRelease keeps the core running past its three slots, and its picture is the one read back.
        [Fact]
        public void Ring_frames_given_back_while_the_screen_is_not_on_the_device_never_stall_the_core()
        {
            using SlangVulkan? device = SlangVulkan.TryCreate(out string report, _wanted, null, native: true);
            if (device is null) { _output.WriteLine($"not run: {report}"); return; }
            using var runtime = new CrystalRuntime(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory }, PlayerCores = new[] { _entry.Cores } });
            string disc = Disc();
            using (CrystalSession first = runtime.Open(new CrystalOpen { Path = disc, Engine = _entry.Found.EngineName, NoBattery = true }))
                first.SetSetting("tc_hw", "vulkan", game: false);
            using var release = new SlangRingRelease();
            using CrystalSession s = runtime.Open(new CrystalOpen { Path = disc, Engine = _entry.Found.EngineName, NoBattery = true, Device = device.DeviceHandle, Ring = true });
            for (int n = 0; n < 12; n++)
            {
                var frame = System.Threading.Tasks.Task.Run(() => Frame(s));
                Assert.True(frame.Wait(10_000), $"frame {n} did not return in 10 s: the core waits on a slot nobody gave back");
                CrystalFrameInfo f = frame.Result;
                Assert.Equal(1u, f.picture_in_ring);
                release.Release(new RingPicture(SlangRing.Borrow(device, s.PictureRing()), (int)f.ring_slot, f.ring_value, (int)f.fields, (int)f.field), f.width, f.height);
                Assert.Equal(f.width * f.height * 4, s.PictureBytes);
            }
            _output.WriteLine("12 ring frames given back with the screen off the device, none stalled");
        }

        // A core that draws past the maxima it declared raises MACHINE_INFO, and the session lends a larger ring there, whose pictures draw as the frames read back - see EmuSen_Backend.md §33.2.
        [Fact]
        public void A_ring_is_lent_again_at_machine_info_when_the_core_draws_past_its_maxima()
        {
            using SlangVulkan? device = SlangVulkan.TryCreate(out string report, _wanted, null, native: true);
            if (device is null) { _output.WriteLine($"not run: {report}"); return; }
            using var runtime = new CrystalRuntime(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory }, PlayerCores = new[] { _entry.Cores } });
            string disc = Disc();
            using (CrystalSession first = runtime.Open(new CrystalOpen { Path = disc, Engine = _entry.Found.EngineName, NoBattery = true }))
            {
                first.SetSetting("tc_hw", "vulkan", game: false);
                first.SetSetting("tc_geometry", "grow", game: false);
            }
            using var none = new SlangChain(device, SlangPreset.Load(BuiltInPreset.Write(BuiltInShaders.AsFilter(ShaderEffect.None, false), null)));
            using var uploaded = new SlangChain(device, SlangPreset.Load(BuiltInPreset.Write(BuiltInShaders.AsFilter(ShaderEffect.None, false), null)));
            using CrystalSession s = runtime.Open(new CrystalOpen { Path = disc, Engine = _entry.Found.EngineName, NoBattery = true, Device = device.DeviceHandle, Ring = true });
            var seen = new System.Collections.Generic.List<string>();
            nint smaller = 0, later = 0;
            for (int n = 0; n < 10; n++)
            {
                CrystalFrameInfo f = Frame(s);
                seen.Add($"{f.width}x{f.height} ring {f.picture_in_ring}");
                if (f.picture_in_ring == 0) continue;
                nint handle = s.PictureRing();
                if (f.width == 320) smaller = smaller == 0 ? handle : smaller;
                else later = handle;
                byte[] picture = new byte[s.PictureBytes];
                s.CopyPicture(picture);
                none.AdvanceRing(SlangRing.Borrow(device, handle), (int)f.ring_slot, f.ring_value, f.width, f.height);
                uploaded.Advance(picture, f.width, f.height, 1);
                byte[] got = none.Render(f.width, f.height), want = uploaded.Render(f.width, f.height);
                Assert.True(got.AsSpan().SequenceEqual(want), $"frame {n} at {f.width}x{f.height}, slot {f.ring_slot} value {f.ring_value}: the ring's picture differs from the read-back one");
            }
            _output.WriteLine(string.Join(", ", seen));
            Assert.Equal(new[] { "320x240 ring 1", "320x240 ring 1", "640x480 ring 0" }.Concat(Enumerable.Repeat("640x480 ring 1", 7)), seen);
            Assert.True(smaller != 0 && later != 0 && smaller != later, "a second, larger ring was lent at MACHINE_INFO");
        }

        // A core that dies with a ring lent ends the session in words within its deadline, and the next ringed session on the same device runs - see EmuSen_Backend.md §33.2.
        [Fact]
        public void A_core_that_crashes_with_a_ring_lent_ends_in_words_and_the_device_runs_the_next_ringed_session()
        {
            using SlangVulkan? device = SlangVulkan.TryCreate(out string report, _wanted, null, native: true);
            if (device is null) { _output.WriteLine($"not run: {report}"); return; }
            using var runtime = new CrystalRuntime(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory }, PlayerCores = new[] { _entry.Cores } });
            string disc = Disc();
            using (CrystalSession first = runtime.Open(new CrystalOpen { Path = disc, Engine = _entry.Found.EngineName, NoBattery = true }))
            {
                first.SetSetting("tc_hw", "vulkan", game: false);
                first.SetSetting("tc_fault", "segv", game: false);
            }
            using var release = new SlangRingRelease();
            CrystalException? ended = null;
            using (CrystalSession s = runtime.Open(new CrystalOpen { Path = disc, Engine = _entry.Found.EngineName, NoBattery = true, Device = device.DeviceHandle, Ring = true }))
            {
                for (int n = 0; n < 6 && ended is null; n++)
                {
                    var frame = System.Threading.Tasks.Task.Run(() => Frame(s));
                    try
                    {
                        Assert.True(frame.Wait(15_000), $"frame {n} did not return in 15 s");
                        CrystalFrameInfo f = frame.Result;
                        if (f.picture_in_ring == 1) release.Release(new RingPicture(SlangRing.Borrow(device, s.PictureRing()), (int)f.ring_slot, f.ring_value, (int)f.fields, (int)f.field), f.width, f.height);
                    }
                    catch (AggregateException e) when (e.InnerException is CrystalException c) { ended = c; }
                }
            }
            _output.WriteLine($"ended: {ended?.Message}");
            Assert.NotNull(ended);
            using (CrystalSession fault = runtime.Open(new CrystalOpen { Path = disc, Engine = _entry.Found.EngineName, NoBattery = true }))
                fault.SetSetting("tc_fault", "none", game: false);
            using CrystalSession next = runtime.Open(new CrystalOpen { Path = disc, Engine = _entry.Found.EngineName, NoBattery = true, Device = device.DeviceHandle, Ring = true });
            for (int n = 0; n < 4; n++)
            {
                CrystalFrameInfo f = Frame(next);
                Assert.Equal(1u, f.picture_in_ring);
                release.Release(new RingPicture(SlangRing.Borrow(device, next.PictureRing()), (int)f.ring_slot, f.ring_value, (int)f.fields, (int)f.field), f.width, f.height);
            }
        }

        // The frame advanced is taken too, so the core is never left waiting on a slot when the session closes.
        private static void TakeFrom(SlangChain chain, SlangVulkan device, CrystalSession s, CrystalFrameInfo f)
        {
            chain.AdvanceRing(SlangRing.Borrow(device, s.PictureRing()), (int)f.ring_slot, f.ring_value, f.width, f.height);
            chain.Render(f.width, f.height);
        }
    }
}
