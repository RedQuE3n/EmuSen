using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Serenity;
using EmuSen.Serenity.Native;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // A hosted core's picture on the slang device's ring, taken by a chain as its input and drawn through the console's screen filters, the accurate CRT filter among them - see EmuSen_LibretroHost.md §10f.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed unsafe class RingChainTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly LibretroEntryFixture _entry;
        private readonly string _wanted = Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800";

        public RingChainTests(ITestOutputHelper output)
        {
            _output = output;
            Assert.True(SerenityNative.Ready, SerenityNative.Report);
            _entry = new LibretroEntryFixture(systemId: "playstation2");
        }

        public void Dispose() => _entry.Dispose();

        private static void Lend(CoreMachine machine, SlangRing ring)
        {
            var images = ring.Images.Select(i => new CoreInterface.RingImage { Size = (uint)sizeof(CoreInterface.RingImage), MemoryType = i.MemoryType, Handle = i.Handle, AllocationSize = i.AllocationSize, Width = i.Width, Height = i.Height, Format = i.Format, Usage = i.Usage }).ToArray();
            fixed (CoreInterface.RingImage* first = images)
            {
                var lend = new CoreInterface.Ring { Size = (uint)sizeof(CoreInterface.Ring), Count = (uint)images.Length, Images = first, ImageSize = (nuint)sizeof(CoreInterface.RingImage), Ready = (ulong)ring.Ready, Done = (ulong)ring.Done };
                for (int i = 0; i < 16; i++) (lend.DeviceUuid[i], lend.DriverUuid[i]) = (ring.DeviceUuid[i], ring.DriverUuid[i]);
                Assert.True(machine.RingLend(&lend), "the core refused the ring");
            }
        }

        // Keyed on the console the machine reports, each frame from the ring drawn through a filter is the same frame's software picture, from a machine lent no ring, uploaded and drawn through the same filter, byte for byte; through no filter it is the picture itself.
        [Fact]
        public void A_ring_slot_drawn_through_the_consoles_filters_is_the_uploaded_picture_drawn_through_them()
        {
            SlangVulkan? device = SlangVulkan.TryCreate(out string report, _wanted, null, native: true);
            if (device is null) { _output.WriteLine($"not run: {report}"); return; }
            using (device)
            {
                using SlangRing? ring = SlangRing.TryCreate(device, 640, 480, 3, out string? why);
                if (ring is null) { _output.WriteLine($"not run: {why}"); return; }
                string game = Path.Combine(_entry.Root, "game.tcg");
                File.WriteAllBytes(game, "TCG1 a test game"u8.ToArray());
                using var engine = new CoreEngine(_entry.Found.Open() ?? throw new Xunit.Sdk.XunitException(_entry.Found.Report)) { Options = TestCores.NoBattery };
                engine.Set("tc_hw", "vulkan");
                engine.LoadRom(game);
                using var plain = new CoreEngine(_entry.Found.Open()!) { Options = TestCores.NoBattery };
                plain.LoadRom(game);
                string system = engine.Machine.Info.System;
                string? console = EmuSen.Crystal.Systems.SystemPacks.For(system)?.Entry.Console;
                Assert.Equal(("ps2", "PlayStation 2"), (system, console));
                Lend(engine.Machine, ring);
                var filters = new (string Name, SlangPreset Preset)[]
                {
                    ("none", SlangPreset.Load(BuiltInPreset.Write(BuiltInShaders.AsFilter(ShaderEffect.None, false), null))),
                    (CrtFilter.Filter.Name, SlangPreset.Load(BuiltInPreset.Write(CrtFilter.Filter, console))),
                    (CrtFilters.Lottes.Name, SlangPreset.Load(BuiltInPreset.Write(CrtFilters.Lottes, console))),
                };
                var chains = filters.Select(f => (f.Name, FromRing: new SlangChain(device, f.Preset), Uploaded: new SlangChain(device, f.Preset))).ToArray();
                try
                {
                    for (int n = 0; n < 6; n++)
                    {
                        engine.RunFrame();
                        plain.RunFrame();
                        CoreInterface.RingFrame f = engine.Machine.FrameRing() ?? throw new Xunit.Sdk.XunitException($"frame {n} is not in the ring");
                        Assert.Equal((320, 240, (ulong)n + 1, 0u), (f.Width, f.Height, f.Value, f.Fields));
                        byte[] picture = plain.GetFrameBufferRgba();
                        Assert.Equal(f.Width * f.Height * 4, picture.Length);
                        foreach (var (name, fromRing, uploaded) in chains)
                        {
                            fromRing.AdvanceRing(ring, (int)f.Slot, f.Value, f.Width, f.Height);
                            uploaded.Advance(picture, f.Width, f.Height, 1);
                            (int w, int h) = name == "none" ? (f.Width, f.Height) : (640, 480);
                            byte[] got = fromRing.Render(w, h), want = uploaded.Render(w, h);
                            Assert.True(got.AsSpan().SequenceEqual(want), $"frame {n}, {name}: the ring's picture drawn differs from the uploaded one in {got.Zip(want).Count(p => p.First != p.Second)} bytes");
                            if (name == "none") Assert.True(got.AsSpan().SequenceEqual(picture), $"frame {n}: the ring's picture is not the frame");
                        }
                        // §10h: a screenshot of a ringed frame reads its picture back on demand, never blank.
                        if (n % 2 == 1) Assert.True(engine.GetFrameBufferRgba().AsSpan().SequenceEqual(picture), $"frame {n}: the screenshot of the ringed frame is not the frame");
                    }
                }
                finally
                {
                    foreach (var (_, fromRing, uploaded) in chains) { fromRing.Dispose(); uploaded.Dispose(); }
                    engine.Machine.RingLend(null);
                }
                _output.WriteLine($"6 frames from the ring through {string.Join(", ", filters.Select(f => f.Name))} for {console}, each the uploaded picture's drawing");
            }
        }
    }
}
