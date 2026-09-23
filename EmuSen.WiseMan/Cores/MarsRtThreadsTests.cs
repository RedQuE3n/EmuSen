using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MarsRT's display processor on a thread and its picture deferred, against MarsRT on one thread and against the C# Mars threaded - see Mars_Native.md §5.6.
    [Collection("MarsStatics")]
    public class MarsRtThreadsTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly List<string> _temporary = new();
        private readonly bool _nativeWas = EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseNative;

        public MarsRtThreadsTests(ITestOutputHelper output)
        {
            _output = output;
            EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseNative = false;
        }

        public void Dispose()
        {
            EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseNative = _nativeWas;
            foreach (string path in _temporary)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        public static TheoryData<string, string?> Games() => new()
        {
            { "sm64.z64", null }, { "oot.z64", null }, { "ge.z64", null },
            { "sm64.z64", "sm64.state" }, { "oot.z64", "oot.state" }, { "ge.z64", "ge-dam.state" },
        };

        private static int Frames(int otherwise) => int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_MARSRT_FRAMES"), out int n) ? n : otherwise;

        // The game's ROM copied to a scratch file, and its state; null when the folder does not hold them.
        private (string Rom, byte[]? State)? Game(string romName, string? stateName)
        {
            string? folder = Environment.GetEnvironmentVariable(MarsRtTests.StatesVariable);
            if (folder is null || !File.Exists(Path.Combine(folder, romName)) || (stateName != null && !File.Exists(Path.Combine(folder, stateName))))
            {
                _output.WriteLine($"{romName} {stateName}: absent, not run");
                return null;
            }

            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = SyntheticN64Rom.WriteTemp(File.ReadAllBytes(Path.Combine(folder, romName)));
            _temporary.Add(rom);
            return (rom, stateName is null ? null : File.ReadAllBytes(Path.Combine(folder, stateName)));
        }

        private static MarsRtCore Twin(bool threaded, bool deferred, int workers = 1, int tier = 0, bool blocks = false) =>
            new(expansionPak: true, batteryRamDisabled: true) { ThreadedRdp = threaded, RdpWorkers = workers, DeferredPresentation = deferred, SkipRendering = false, UseBlocks = blocks, BlockTier = tier };

        // MarsRT on a thread, its state compared every frame (which joins) or every sixtieth (which leaves the thread running across frames), picture and sound every frame.
        [Theory]
        [MemberData(nameof(Games))]
        public void MarsRT_threaded_leaves_what_MarsRT_on_one_thread_leaves(string romName, string? stateName)
        {
            if (Game(romName, stateName) is not { } game) return;
            int frames = Frames(300);
            foreach (var (deferred, every) in new[] { (false, 1), (false, 60), (true, 60) })
            {
                using MarsRtCore reference = Twin(threaded: false, deferred: false), subject = Twin(threaded: true, deferred);
                Load(reference, game);
                Load(subject, game);
                byte[] previous = reference.GetFrameBufferRgba().ToArray();
                (int, int, int) previousShape = (reference.ScreenWidth, reference.ScreenHeight, reference.RowRepeat);

                for (int frame = 1; frame <= frames; frame++)
                {
                    Drive(reference, subject, frame);
                    reference.RunFrame();
                    subject.RunFrame();
                    if (frame % every == 0 || frame == frames) SameState(frame, reference.Save(false), subject.Save(false));

                    byte[] want = deferred ? previous : reference.GetFrameBufferRgba();
                    var shape = deferred ? previousShape : (reference.ScreenWidth, reference.ScreenHeight, reference.RowRepeat);
                    SamePicture(frame, shape, want, subject);
                    previous = reference.GetFrameBufferRgba().ToArray();
                    previousShape = (reference.ScreenWidth, reference.ScreenHeight, reference.RowRepeat);
                    SameSound(frame, reference, subject);
                }

                long[] counters = subject.ThreadCounterValues();
                _output.WriteLine($"{romName} {stateName ?? "from power-on"}, deferred {deferred}, state every {every}: {frames} frames exact; the drain ran {counters[1]} words; waits by site {string.Join(",", counters.Skip(14).Take(12))}; scans repeated {counters[13]}");
                Assert.True(counters[0] == 1 && (counters[1] > 0 || stateName is null), "the drain never ran, so nothing was compared");
            }
        }

        // MarsRT's list shared by several processors against MarsRT on one thread, then against the C# core's list shared as widely - see Mars_Native.md §5.6.6.
        [Theory]
        [MemberData(nameof(Games))]
        public void MarsRT_with_its_list_shared_leaves_what_MarsRT_on_one_thread_leaves_and_is_compared_with_the_csharp_split(string romName, string? stateName)
        {
            if (Game(romName, stateName) is not { } game) return;
            int frames = Frames(300);
            foreach (int workers in new[] { 2, 4 })
            {
                using MarsRtCore reference = Twin(threaded: false, deferred: false), subject = Twin(threaded: true, deferred: false, workers);
                Load(reference, game);
                Load(subject, game);
                for (int frame = 1; frame <= frames; frame++)
                {
                    Drive(reference, subject, frame);
                    reference.RunFrame();
                    subject.RunFrame();
                    SameState(frame, reference.Save(false), subject.Save(false));
                    SamePicture(frame, (reference.ScreenWidth, reference.ScreenHeight, reference.RowRepeat), reference.GetFrameBufferRgba(), subject);
                    SameSound(frame, reference, subject);
                }
                _output.WriteLine($"{romName} {stateName ?? "from power-on"}, {workers} workers: {frames} frames exact against MarsRT on one thread");
            }

            // The C# split is an oracle only where it is exact; where it parts from MarsRT, the fields it parts in are named.
            MarsCore oracle = MarsRtTests.Oracle(expansionPak: true);
            oracle.ThreadedRdp = true;
            oracle.RdpWorkers = 4;
            oracle.SkipRendering = false;
            using MarsRtCore twin = Twin(threaded: true, deferred: false, workers: 4);
            oracle.LoadRom(game.Rom);
            twin.LoadRom(game.Rom);
            if (game.State is { } loaded)
            {
                oracle.LoadState(new MemoryStream(loaded));
                twin.LoadState(loaded);
            }
            for (int frame = 1; frame <= frames; frame++)
            {
                Drive(oracle, twin, frame);
                oracle.RunFrame();
                twin.RunFrame();
                byte[] want = Save(oracle), got = twin.Save(false);
                if (!want.AsSpan().SequenceEqual(got))
                {
                    _output.WriteLine($"{romName} {stateName ?? "from power-on"}, four workers: the C# split parts from MarsRT's at frame {frame}, in {Differing(want, got)}");
                    return;
                }
                SamePicture(frame, (oracle.ScreenWidth, oracle.ScreenHeight, oracle.RowRepeat), oracle.GetFrameBufferRgba(), twin);
            }
            _output.WriteLine($"{romName} {stateName ?? "from power-on"}, four workers: {frames} frames exact against the C# split too");
        }

        // The fields two states differ in, by the layout MarsRT writes.
        private static string Differing(byte[] a, byte[] b)
        {
            using var machine = new MarsMachine(BitConverter.ToInt32(a, 8));
            machine.Load(a);
            var names = machine.Layout(snapshot: false).Split('\n', StringSplitOptions.RemoveEmptyEntries).Select(l => l.Split(' '))
                .Where(p => !a.AsSpan(int.Parse(p[0]), int.Parse(p[1])).SequenceEqual(b.AsSpan(int.Parse(p[0]), int.Parse(p[1])))).Select(p => p[3]);
            return string.Join(", ", names.Take(6));
        }

        // MarsRT threaded against the C# Mars threaded, the interpreter in both, presented at once and then both deferred.
        [Theory]
        [MemberData(nameof(Games))]
        public void MarsRT_threaded_leaves_what_the_csharp_core_threaded_leaves(string romName, string? stateName) => AgainstTheCsharpCore(romName, stateName, blocks: false);

        public static TheoryData<string, string, int, string, bool> GamesAtMultiples()
        {
            var data = new TheoryData<string, string, int, string, bool>();
            foreach (var (rom, state) in new[] { ("sm64.z64", "sm64.state"), ("oot.z64", "oot.state"), ("ge.z64", "ge-dam.state") })
                foreach (var (scale, level, gpu) in new[] { (2, "Off", false), (4, "Off", false), (2, "2x", false), (4, "4x", false), (1, "2x", false), (2, "Off", true), (4, "Off", true), (2, "2x", true), (1, "4x", true) })
                    data.Add(rom, state, scale, level, gpu);
            return data;
        }

        // MarsRT at a multiple against the C# core at the same multiple, averaging and device, picture for picture, the state that of the machine at one - see Mars_Native.md §6.4.
        [Theory]
        [MemberData(nameof(GamesAtMultiples))]
        public void MarsRT_at_a_multiple_leaves_what_the_csharp_core_at_that_multiple_leaves(string romName, string stateName, int scale, string level, bool gpu)
        {
            if (gpu && EmuSen.Cores.Nintendo.Mars.Rdp.Gpu.GpuDevice.DeviceNames().Count == 0) { _output.WriteLine("no Vulkan device: not run"); return; }
            if (Game(romName, stateName) is not { } game) return;
            int frames = Frames(300);
            foreach (bool deferred in new[] { false, true })
            {
                // Unthreaded, since C#'s threaded scan decides whether the multiple has drawn before its drain is joined - see Mars_Native.md §6.4.4.
                MarsCore oracle = MarsRtTests.Oracle(expansionPak: true);
                oracle.ThreadedRdp = false;
                oracle.DeferredPresentation = deferred;
                oracle.SkipRendering = false;
                using MarsRtCore twin = Twin(threaded: true, deferred, workers: 4, tier: 2, blocks: true);
                MarsCore atOne = MarsRtTests.Oracle(expansionPak: true);
                atOne.SkipRendering = true;
                ((EmuSen.Cores.ICoreSettings)oracle).Set("RenderScale", scale.ToString());
                ((EmuSen.Cores.ICoreSettings)oracle).Set("Antialiasing", level);
                ((EmuSen.Cores.ICoreSettings)twin).Set("RenderScale", scale.ToString());
                ((EmuSen.Cores.ICoreSettings)twin).Set("Antialiasing", level);
                ((EmuSen.Cores.ICoreSettings)oracle).Set("Gpu", gpu ? "true" : "false");
                ((EmuSen.Cores.ICoreSettings)twin).Set("Gpu", gpu ? "true" : "false");
                Assert.Equal(oracle.EffectiveAntialiasing, twin.EffectiveAntialiasing);
                foreach (EmuSen.Cores.ICore core in new EmuSen.Cores.ICore[] { oracle, twin, atOne })
                {
                    core.LoadRom(game.Rom);
                    core.LoadState(new MemoryStream(game.State!));
                }
                if (gpu) Assert.Equal(oracle.GpuReport, twin.GpuReport);
                Assert.DoesNotContain("not available", twin.GpuReport);

                int atTheMultiple = 0;
                for (int frame = 1; frame <= frames; frame++)
                {
                    Drive(oracle, twin, frame);
                    Drive(atOne, atOne, frame);
                    oracle.RunFrame();
                    twin.RunFrame();
                    atOne.RunFrame();
                    byte[] want = Save(oracle);
                    SameState(frame, want, twin.Save(false));
                    Assert.True(want.AsSpan().SequenceEqual(Save(atOne)), $"the multiple changed the C# state at frame {frame}");
                    SamePicture(frame, (oracle.ScreenWidth, oracle.ScreenHeight, oracle.RowRepeat), oracle.GetFrameBufferRgba(), twin);
                    SameSound(frame, oracle, twin);
                    if (oracle.ScreenWidth == MarsCore.ScreenWidthPixels * scale) atTheMultiple++;
                }
                Assert.True(atTheMultiple > frames / 2, $"the picture was at the multiple in {atTheMultiple} frames of {frames}");
                _output.WriteLine($"{romName} {stateName}, {scale}x, antialiasing {level} (drawn at {scale * oracle.EffectiveAntialiasing}), deferred {deferred}, device {twin.GpuReport}: {frames} frames, state, picture and sound exact against the C# core at the multiple; {atTheMultiple} frames at the multiple, {oracle.ScreenWidth}x{oracle.ScreenHeight}");
            }
        }

        // The same, with MarsRT's recompiler on: the C# interpreter threaded is still the oracle - see Mars_Native.md §5.8.
        [Theory]
        [MemberData(nameof(Games))]
        public void MarsRT_recompiled_and_threaded_leaves_what_the_csharp_core_threaded_leaves(string romName, string? stateName) => AgainstTheCsharpCore(romName, stateName, blocks: true);

        private void AgainstTheCsharpCore(string romName, string? stateName, bool blocks)
        {
            if (Game(romName, stateName) is not { } game) return;
            int frames = Frames(600);
            foreach (bool deferred in new[] { false, true })
            {
                MarsCore oracle = MarsRtTests.Oracle(expansionPak: true);
                oracle.ThreadedRdp = true;
                oracle.RdpWorkers = 1;
                oracle.DeferredPresentation = deferred;
                oracle.SkipRendering = false;
                using MarsRtCore twin = Twin(threaded: true, deferred, blocks: blocks);
                oracle.LoadRom(game.Rom);
                twin.LoadRom(game.Rom);
                if (game.State is { } state)
                {
                    oracle.LoadState(new MemoryStream(state));
                    twin.LoadState(state);
                }

                SameState(0, Save(oracle), twin.Save(false));
                for (int frame = 1; frame <= frames; frame++)
                {
                    Drive(oracle, twin, frame);
                    oracle.RunFrame();
                    twin.RunFrame();
                    SameState(frame, Save(oracle), twin.Save(false));
                    SamePicture(frame, (oracle.ScreenWidth, oracle.ScreenHeight, oracle.RowRepeat), oracle.GetFrameBufferRgba(), twin);
                    SameSound(frame, oracle, twin);
                }
                _output.WriteLine($"{romName} {stateName ?? "from power-on"}, deferred {deferred}{(blocks ? ", recompiled" : "")}: {frames} frames, state, picture and sound exact against the C# core threaded; {oracle.Bus!.Cycles} cycles{(blocks ? $"; blocks {string.Join(' ', twin.BlockCounterValues())}" : "")}");
            }
        }

        // Milliseconds a frame, flat out from each state with the picture on: MarsRT three ways and the C# core as it ships, interleaved, three rounds.
        [Theory]
        [InlineData("sm64.z64", "sm64.state")]
        [InlineData("oot.z64", "oot.state")]
        [InlineData("ge.z64", "ge-dam.state")]
        public void Bench(string romName, string stateName)
        {
            if (Environment.GetEnvironmentVariable(MarsRtTests.BenchVariable) != "1") return;
            if (Game(romName, stateName) is not { } game) return;
            int frames = Frames(300);
            int workers = Math.Clamp(Environment.ProcessorCount / 3, 1, 4);
            var runs = new (string Name, Func<EmuSen.Cores.ICore> Make)[]
            {
                ("MarsRT split+deferred, interpreter", () => Twin(threaded: true, deferred: true, workers)),
                ("MarsRT split+deferred, decoded blocks", () => Twin(threaded: true, deferred: true, workers, tier: 1, blocks: true)),
                ("MarsRT split+deferred, compiled", () => Twin(threaded: true, deferred: true, workers, tier: 2, blocks: true)),
                ("MarsRT split+deferred, compiled with registers held", () => Twin(threaded: true, deferred: true, workers, tier: 3, blocks: true)),
                ("C# production", () => new MarsCore(expansionPak: true, batteryRamDisabled: true) { UseBlocks = true, ThreadedRdp = true, DeferredPresentation = true, RdpWorkers = workers }),
            };

            bool rspBlocks = EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks;
            try
            {
                EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = true;
                for (int round = 1; round <= 3; round++)
                {
                    var times = new List<string>();
                    foreach (var (name, make) in round % 2 == 0 ? runs.Reverse() : runs)
                    {
                        EmuSen.Cores.ICore core = make();
                        core.LoadRom(game.Rom);
                        core.LoadState(new MemoryStream(game.State!));
                        var clock = Stopwatch.StartNew();
                        for (int i = 0; i < frames; i++) core.RunFrame();
                        times.Add($"{name} {clock.Elapsed.TotalMilliseconds / frames:F2}");
                        if (core is MarsRtCore rt && rt.UseBlocks) times[^1] += $" (blocks {string.Join(' ', rt.BlockCounterValues())})";
                        (core as IDisposable)?.Dispose();
                    }
                    _output.WriteLine($"{romName} round {round}, {frames} frames, ms a frame: {string.Join("; ", times)}");
                }
            }
            finally
            {
                EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = rspBlocks;
            }
        }

        // Milliseconds a frame at a multiple from each state: MarsRT and the C# core, each on the processor and on the device, interleaved, three rounds; SCALE, AA - see Mars_Native.md §6.4.
        [Theory]
        [InlineData("sm64.z64", "sm64.state")]
        [InlineData("oot.z64", "oot.state")]
        [InlineData("ge.z64", "ge-dam.state")]
        public void Bench_at_a_multiple(string romName, string stateName)
        {
            if (Environment.GetEnvironmentVariable(MarsRtTests.BenchVariable) != "1") return;
            if (Game(romName, stateName) is not { } game) return;
            int frames = Frames(600);
            string scale = Environment.GetEnvironmentVariable("SCALE") ?? "4", level = Environment.GetEnvironmentVariable("AA") ?? "Off";
            int workers = Math.Clamp(Environment.ProcessorCount / 3, 1, 4);
            EmuSen.Cores.ICore Set(EmuSen.Cores.ICore core, bool gpu)
            {
                var settings = (EmuSen.Cores.ICoreSettings)core;
                settings.Set("RenderScale", scale);
                settings.Set("Antialiasing", level);
                settings.Set("Gpu", gpu ? "true" : "false");
                return core;
            }
            var runs = new (string Name, Func<EmuSen.Cores.ICore> Make)[]
            {
                ("MarsRT processor", () => Set(Twin(threaded: true, deferred: true, workers, tier: 2, blocks: true), false)),
                ("MarsRT device", () => Set(Twin(threaded: true, deferred: true, workers, tier: 2, blocks: true), true)),
                ("C# processor", () => Set(new MarsCore(expansionPak: true, batteryRamDisabled: true) { UseBlocks = true, ThreadedRdp = true, DeferredPresentation = true, RdpWorkers = workers }, false)),
                ("C# device", () => Set(new MarsCore(expansionPak: true, batteryRamDisabled: true) { UseBlocks = true, ThreadedRdp = true, DeferredPresentation = true, RdpWorkers = workers }, true)),
            };

            bool rspBlocks = EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks;
            try
            {
                EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = true;
                for (int round = 1; round <= 3; round++)
                {
                    var times = new List<string>();
                    foreach (var (name, make) in round % 2 == 0 ? runs.Reverse() : runs)
                    {
                        EmuSen.Cores.ICore core = make();
                        core.LoadRom(game.Rom);
                        core.LoadState(new MemoryStream(game.State!));
                        var clock = Stopwatch.StartNew();
                        for (int i = 0; i < frames; i++) core.RunFrame();
                        string report = core is MarsRtCore rt ? rt.GpuReport : ((MarsCore)core).GpuReport;
                        times.Add($"{name} {clock.Elapsed.TotalMilliseconds / frames:F2} ({report}, {core.ScreenWidth}x{core.ScreenHeight})");
                        (core as IDisposable)?.Dispose();
                        GC.Collect();
                        GC.WaitForPendingFinalizers();
                    }
                    _output.WriteLine($"{romName} {scale}x antialiasing {level}, round {round}, {frames} frames, ms a frame: {string.Join("; ", times)}");
                }
            }
            finally
            {
                EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = rspBlocks;
            }
        }

        // C#'s split carries each processor's own last level-of-detail fraction into rows that compute none, and a primitive drawn alone then assembles a stale one - see Mars_Native.md §5.6.6.
        [Fact]
        public void The_csharp_split_assembles_a_stale_level_of_detail_fraction_from_rows_that_computed_none()
        {
            const System.Reflection.BindingFlags Hidden = System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static;
            object Call(string name, params object[] args) => typeof(MarsThreadedRdpTests).GetMethod(name, Hidden)!.Invoke(null, args)!;
            var fraction = typeof(EmuSen.Cores.Nintendo.Mars.Rdp.Rdp).GetField("_lodFraction", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance)!;

            int stale = 0, seeds = 16;
            for (uint seed = 0; seed < seeds; seed++)
            {
                // A scene drawn alone that computes the fraction, a split one that computes none, then one drawn alone, which assembles first.
                ulong[] list =
                [
                    .. (ulong[])Call("Scene", 0x1234_5679u + seed, false, 32, true),
                    .. (ulong[])Call("Shaded", 0x2468_1357u + seed, false, false, false, false),
                    .. (ulong[])Call("Shaded", 0x1357_2468u + seed, true, true, true, false),
                ];
                EmuSen.Cores.Nintendo.Mars.Memory.MemoryBus atOnce = new(), split = new();
                split.Dp.Threaded = true;
                split.Dp.Workers = 2;
                Call("HandOver", atOnce, list, 0x0010_0000u);
                Call("HandOver", split, list, 0x0010_0000u);
                split.Dp.Join();

                Assert.Equal(atOnce.Rdram, split.Rdram);
                int want = (int)fraction.GetValue(atOnce.Dp.Processor)!, got = (int)fraction.GetValue(split.Dp.Processor)!;
                if (want != got) stale++;
                bool same = ((byte[])Call("State", atOnce, false)).AsSpan().SequenceEqual((byte[])Call("State", split, false));
                Assert.Equal(want == got, same);
            }

            _output.WriteLine($"C#, two processors: the state differs in _lodFraction alone for {stale} of {seeds} seeds, the memory for none");
            Assert.True(stale > 0, "C#'s split assembled the raster order's fraction for every seed, so the defect this test records is gone and the test should be turned around");
        }

        private static void ListTo(EmuSen.Cores.Nintendo.Mars.Memory.MemoryBus bus, ulong[] list)
        {
            const uint at = 0x0010_0000;
            for (int i = 0; i < list.Length; i++) bus.Write64(at + (uint)i * 8, list[i]);
            bus.Write32(EmuSen.Cores.Nintendo.Mars.Memory.MemoryMap.DpCommandBase, at);
            bus.Write32(EmuSen.Cores.Nintendo.Mars.Memory.MemoryMap.DpCommandBase + 4, at + (uint)list.Length * 8);
        }

        // A 256-column sixteen-bit picture of the given rows at 0x200000, anti-alias mode 3, as MarsDeferredPresentationTests programs one.
        private static uint[] ViRegisters(uint rows)
        {
            var registers = new uint[14];
            registers[0] = 2u | (3u << 8);
            registers[1] = 0x0020_0000;
            registers[2] = 64;
            registers[6] = 525;
            registers[9] = (108u << 16) | (108u + 256);
            registers[10] = (34u << 16) | (34u + rows * 2);
            registers[12] = 0x400;
            registers[13] = 0x400;
            return registers;
        }

        private static void Load(MarsRtCore core, (string Rom, byte[]? State) game)
        {
            core.LoadRom(game.Rom);
            if (game.State is { } state) core.LoadState(state);
        }

        private static byte[] Save(MarsCore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        private static void SameState(int frame, byte[] want, byte[] got)
        {
            if (want.AsSpan().SequenceEqual(got)) return;
            throw new Xunit.Sdk.XunitException($"frame {frame}: the states differ from byte {want.AsSpan().CommonPrefixLength(got)} of {want.Length} ({got.Length})");
        }

        private static void SamePicture(int frame, (int Width, int Height, int Repeat) shape, byte[] want, MarsRtCore twin)
        {
            if (shape != (twin.ScreenWidth, twin.ScreenHeight, twin.RowRepeat))
                throw new Xunit.Sdk.XunitException($"frame {frame}: {shape} expected, {(twin.ScreenWidth, twin.ScreenHeight, twin.RowRepeat)} shown");
            byte[] got = twin.GetFrameBufferRgba();
            if (!want.AsSpan().SequenceEqual(got)) throw new Xunit.Sdk.XunitException($"frame {frame}: the pictures differ from byte {want.AsSpan().CommonPrefixLength(got)} of {want.Length}");
        }

        private static void SameSound(int frame, EmuSen.Cores.ICore a, EmuSen.Cores.ICore b)
        {
            short[] want = a.DequeueAudioSamples(1 << 20), got = b.DequeueAudioSamples(1 << 20);
            if (a.AudioSampleRate != b.AudioSampleRate || !want.AsSpan().SequenceEqual(got))
                throw new Xunit.Sdk.XunitException($"frame {frame}: {want.Length} samples at {a.AudioSampleRate} Hz against {got.Length} at {b.AudioSampleRate} Hz");
        }

        // The same input to both cores, as MarsRtTests drives them.
        private static void Drive(EmuSen.Cores.ICore a, EmuSen.Cores.ICore b, int frame)
        {
            PadButton[] buttons = { PadButton.A, PadButton.B, PadButton.Start, PadButton.Up, PadButton.L2, PadButton.R };
            PadButton button = buttons[frame % buttons.Length];
            bool pressed = (frame / buttons.Length) % 2 == 0;
            double x = Math.Sin(frame * 0.37), y = Math.Cos(frame * 0.23), c = (frame % 7 - 3) / 3.0;
            foreach (EmuSen.Cores.ICore core in new[] { a, b })
            {
                core.SetButton(0, button, pressed);
                core.SetAxis(0, PadAxis.LeftX, x);
                core.SetAxis(0, PadAxis.LeftY, y);
                core.SetAxis(0, PadAxis.RightX, c);
                core.SetAxis(0, PadAxis.RightY, -c);
            }
        }
    }
}
