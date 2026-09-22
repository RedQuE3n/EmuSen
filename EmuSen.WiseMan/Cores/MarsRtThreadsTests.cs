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

        private static MarsRtCore Twin(bool threaded, bool deferred, int workers = 1) =>
            new(expansionPak: true, batteryRamDisabled: true) { ThreadedRdp = threaded, RdpWorkers = workers, DeferredPresentation = deferred, SkipRendering = false };

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
        public void MarsRT_threaded_leaves_what_the_csharp_core_threaded_leaves(string romName, string? stateName)
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
                using MarsRtCore twin = Twin(threaded: true, deferred);
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
                _output.WriteLine($"{romName} {stateName ?? "from power-on"}, deferred {deferred}: {frames} frames, state, picture and sound exact against the C# core threaded; {oracle.Bus!.Cycles} cycles");
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
            var runs = new (string Name, Func<EmuSen.Cores.ICore> Make)[]
            {
                ("MarsRT", () => Twin(threaded: false, deferred: false)),
                ("MarsRT threaded", () => Twin(threaded: true, deferred: false)),
                ("MarsRT threaded+deferred", () => Twin(threaded: true, deferred: true)),
                ("MarsRT split+deferred", () => Twin(threaded: true, deferred: true, workers: Math.Clamp(Environment.ProcessorCount / 3, 1, 4))),
                ("C# production", () => new MarsCore(expansionPak: true, batteryRamDisabled: true) { UseBlocks = true, ThreadedRdp = true, DeferredPresentation = true, RdpWorkers = Math.Clamp(Environment.ProcessorCount / 3, 1, 4) }),
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

        // C#'s wait for a range read narrows each page by its first eight bytes, so a capture whose first bytes no pending draw holds reads rows still to be drawn - see Mars_Native.md §5.6.2.
        [Fact]
        public void The_csharp_interface_lets_a_range_read_pass_the_draws_that_hold_all_but_its_first_bytes()
        {
            const uint framebuffer = 0x0020_0000, page = framebuffer + 0x1000, row = framebuffer + 320 * 2 * 8;
            ulong[] list =
            {
                (0x2FUL << 56) | (3UL << 52),
                (0x3FUL << 56) | (2UL << 51) | (319UL << 32) | framebuffer,
                (0x2DUL << 56) | ((320UL << 2) << 12) | (240UL << 2),
                (0x37UL << 56) | 0x1234_5678,
                (0x36UL << 56) | ((319UL << 2) << 44) | ((9UL << 2) << 32) | (8UL << 2),
                0x29UL << 56,
            };

            EmuSen.Cores.Nintendo.Mars.Memory.MemoryBus atOnce = new(), threaded = new();
            threaded.Dp.Threaded = true;
            ListTo(atOnce, list);
            threaded.Dp.Pause();
            ListTo(threaded, list);
            Assert.Equal(list.Length, threaded.Dp.Pending);

            var read = System.Threading.Tasks.Task.Run(() => threaded.Dp.WaitForReadRange(page, 0x1000, 8));
            bool passed = read.Wait(TimeSpan.FromSeconds(2));
            byte[] seen = threaded.Rdram.AsSpan((int)row, 0x80).ToArray();
            threaded.Dp.Resume();
            read.Wait();
            threaded.Dp.Join();

            _output.WriteLine($"C# returned from the range read {(passed ? "at once" : "only when the thread ran")}, reads narrowed {threaded.Dp.ReadsNarrowed}, freed {threaded.Dp.ReadsFreed}");
            Assert.True(passed, "C# waited for the draw, so the defect this test records is gone and the test should be turned around");
            Assert.False(seen.AsSpan().SequenceEqual(atOnce.Rdram.AsSpan((int)row, 0x80)), "the read found the drawn rows although it did not wait");
            Assert.Equal(atOnce.Rdram, threaded.Rdram);
        }

        // C#'s deferred path skips a repeated scan though a held line expired since the last walk, and so keeps the picture from before the expiry - see Mars_Native.md §5.6.5.
        [Fact]
        public void The_csharp_deferred_path_keeps_a_stale_picture_when_a_repeat_follows_an_expired_line()
        {
            string rom = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.BuildRunningFromRdram(new uint[] { 0x1000_FFFF, 0x0000_0000 }));
            _temporary.Add(rom);
            MarsCore now = new(batteryRamDisabled: true) { UseBlocks = false, SkipRendering = false };
            MarsCore later = new(batteryRamDisabled: true) { UseBlocks = false, SkipRendering = false, DeferredPresentation = true };
            now.LoadRom(rom);
            later.LoadRom(rom);
            foreach (MarsCore core in new[] { now, later })
            {
                uint state = 0x2468_ACE0;
                for (int i = 0; i < 0x30000; i++)
                {
                    state = state * 1103515245 + 12345;
                    core.Bus!.Rdram[0x0020_0000 - 0x8000 + i] = (byte)(state >> 16);
                }
            }

            uint[] tall = ViRegisters(rows: 120), shortened = ViRegisters(rows: 60);
            byte[]? previous = null;
            int stale = 0, changed = 0;
            foreach (uint[] registers in new[] { tall, tall, tall, shortened, shortened, shortened, shortened })
            {
                foreach (MarsCore core in new[] { now, later })
                    for (int i = 0; i < registers.Length; i++) core.Bus!.Write32(EmuSen.Cores.Nintendo.Mars.Memory.MemoryMap.ViBase + (uint)i * 4, registers[i]);
                now.RunFrame();
                later.RunFrame();
                if (previous != null && !later.GetFrameBufferRgba().AsSpan().SequenceEqual(previous)) stale++;
                if (previous != null && !now.GetFrameBufferRgba().AsSpan().SequenceEqual(previous)) changed++;
                previous = now.GetFrameBufferRgba().ToArray();
            }

            _output.WriteLine($"C# deferred: {later.RepeatedScans} scans skipped as repeats; {stale} frames differ from the immediate picture of the frame before; the immediate picture changed {changed} times");
            Assert.True(later.RepeatedScans >= 2 && changed >= 1, "the case was not reached");
            Assert.True(stale > 0, "the C# deferred picture was the immediate one's every frame, so the defect this test records is gone and the test should be turned around");
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

        // C#'s workers deadlock when a pause finds some at a barrier and the rest standing short of it; it leaves threads spinning, so it runs only when asked - see Mars_Native.md §5.6.6.
        [Fact]
        public void The_csharp_workers_deadlock_when_a_pause_finds_some_at_a_barrier_and_the_rest_short_of_it()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_MARS_DEADLOCK_PROBE") != "1")
            {
                _output.WriteLine("EMUSEN_MARS_DEADLOCK_PROBE unset, not run: a hang it finds leaves threads spinning");
                return;
            }

            var list = new List<ulong> { (0x2FUL << 56) | (3UL << 52), (0x2DUL << 56) | ((320UL << 2) << 12) | (240UL << 2), (0x37UL << 56) | 0x0F0F_0F0F };
            for (uint i = 0; i < 3000; i++)
            {
                list.Add((0x3FUL << 56) | (2UL << 51) | (319UL << 32) | (0x0020_0000 + (i % 4) * 640));
                uint row = i * 3 % 238;
                list.Add((0x36UL << 56) | ((319UL << 2) << 44) | ((ulong)((row + 1) << 2) << 32) | (row << 2));
            }
            list.Add(0x29UL << 56);

            int attempts = 0, hung = -1;
            for (int attempt = 0; attempt < 200 && hung < 0; attempt++, attempts++)
            {
                var bus = new EmuSen.Cores.Nintendo.Mars.Memory.MemoryBus();
                bus.Dp.Threaded = true;
                bus.Dp.Workers = 4;
                ListTo(bus, list.ToArray());
                System.Threading.Thread.SpinWait(attempt * 997 % 20_000);
                var pause = System.Threading.Tasks.Task.Run(() => bus.Dp.Pause());
                if (!pause.Wait(TimeSpan.FromSeconds(3))) { hung = attempt; break; }
                bus.Dp.Resume();
                bus.Dp.Join();
                bus.Dp.Threaded = false;
            }

            _output.WriteLine(hung >= 0 ? $"C#, four processors: a pause was never answered at attempt {hung} of {attempts}" : $"C#: every one of {attempts} pauses was answered");
            Assert.True(hung >= 0, "every pause was answered, so the defect this test records was not reached");
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
