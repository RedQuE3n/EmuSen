using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Rewind on MarsRT with its display processor on four workers and its picture deferred, as Mistress runs it - see Mars_Native.md §6.6.3.
    [Collection("MarsStatics")]
    public class MarsRtRewindTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly List<string> _temporary = new();

        public MarsRtRewindTests(ITestOutputHelper output) => _output = output;

        public void Dispose()
        {
            foreach (string path in _temporary)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        public static TheoryData<string, string> Games() => new()
        {
            { "sm64.z64", "sm64.state" }, { "oot.z64", "oot.state" }, { "ge.z64", "ge-dam.state" },
        };

        private static int Frames(int otherwise) => int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_MARSRT_FRAMES"), out int n) ? n : otherwise;

        private (string Rom, byte[] State)? Game(string romName, string stateName)
        {
            string? folder = Environment.GetEnvironmentVariable(MarsRtTests.StatesVariable);
            if (folder is null || !File.Exists(Path.Combine(folder, romName)) || !File.Exists(Path.Combine(folder, stateName)))
            {
                _output.WriteLine($"{romName} {stateName}: absent, not run");
                return null;
            }

            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = SyntheticN64Rom.WriteTemp(File.ReadAllBytes(Path.Combine(folder, romName)));
            _temporary.Add(rom);
            return (rom, File.ReadAllBytes(Path.Combine(folder, stateName)));
        }

        // Mistress's settings for MarsRT: four workers, the picture deferred, the recompiler on.
        private static MarsRtCore Production() => new(expansionPak: true, batteryRamDisabled: true) { ThreadedRdp = true, RdpWorkers = 4, DeferredPresentation = true, SkipRendering = false };

        private static MarsRtCore OneThread() => new(expansionPak: true, batteryRamDisabled: true) { ThreadedRdp = false, DeferredPresentation = false, SkipRendering = false, UseBlocks = false };

        // The same input for a frame whichever run reaches it, so a frame run again after a rewind is the frame the reference ran.
        private static void Drive(MarsRtCore core)
        {
            long frame = core.TotalFrames + 1;
            PadButton[] buttons = { PadButton.A, PadButton.B, PadButton.Start, PadButton.Up, PadButton.L2, PadButton.R };
            core.SetButton(0, buttons[frame % buttons.Length], (frame / buttons.Length) % 2 == 0);
            core.SetAxis(0, PadAxis.LeftX, Math.Sin(frame * 0.37));
            core.SetAxis(0, PadAxis.LeftY, Math.Cos(frame * 0.23));
            core.SetAxis(0, PadAxis.RightX, (frame % 7 - 3) / 3.0);
        }

        private static readonly System.Reflection.FieldInfo Newest = typeof(RewindBuffer).GetField("_newest", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!;

        // The words unrun in the snapshot the buffer holds newest: after a capture the one just taken, after a step the one landed on.
        private static int Unrun(RewindBuffer rewind) => Newest.GetValue(rewind) is byte[] state ? BitConverter.ToInt32(state, state.Length - 4 - 8 * DpInterface.SnapshotWords) : 0;

        // The display processor's command numbers among the words the snapshot the buffer holds newest left unrun.
        private static void Tally(RewindBuffer rewind, SortedDictionary<int, int> commands)
        {
            if (Newest.GetValue(rewind) is not byte[] state) return;
            int tail = state.Length - 4 - 8 * DpInterface.SnapshotWords, count = BitConverter.ToInt32(state, tail);
            for (int i = 0; i < count; i++)
            {
                int command = (int)(BitConverter.ToUInt64(state, tail + 4 + 8 * i) >> 56) & 0x3F;
                commands[command] = commands.GetValueOrDefault(command) + 1;
            }
        }

        private static string Hash(byte[] bytes) => Convert.ToHexString(SHA256.HashData(bytes));

        private static string Picture(MarsRtCore core) => $"{core.ScreenWidth}x{core.ScreenHeight}/{core.RowRepeat} {Hash(core.GetFrameBufferRgba())}";

        // Frame by frame on one thread: the state, the picture the frame presented, and the picture a load of that state presents.
        private static Dictionary<long, (string State, string Shown, string Loaded)> Reference((string Rom, byte[] State) game, int frames)
        {
            using MarsRtCore reference = OneThread(), loader = OneThread();
            reference.LoadRom(game.Rom);
            loader.LoadRom(game.Rom);
            reference.LoadState(game.State);
            var history = new Dictionary<long, (string, string, string)>();
            for (int frame = 0; frame <= frames; frame++)
            {
                if (frame > 0)
                {
                    Drive(reference);
                    reference.RunFrame();
                }
                byte[] state = reference.Save(false);
                loader.LoadState(state);
                history[reference.TotalFrames] = (Hash(state), Picture(reference), Picture(loader));
            }
            return history;
        }

        // A capture every frame and a step back each fourth: neither may allocate an array of the state's size on the emulation thread, or rewind brings back §6.13's collections - see Mars_Native.md §6.6.3.
        [Fact]
        public void A_capture_and_a_step_back_allocate_no_array_of_the_state_s_size()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = SyntheticN64Rom.WriteTemp(SyntheticN64System.Build(rsp: true));
            _temporary.Add(rom);
            using MarsRtCore core = Production();
            core.LoadRom(rom);
            var rewind = new RewindBuffer { Enabled = true, IntervalFrames = 1 };
            for (int frame = 0; frame < 20; frame++)
            {
                core.RunFrame();
                rewind.OnFrameCompleted(core);
            }
            Thread.Sleep(50);
            Assert.True(rewind.Rewind(core), "the warming step back found no history");

            long captured = 0, stepped = 0, captures = 0, steps = 0;
            var clock = new System.Diagnostics.Stopwatch();
            double captureMs = 0, stepMs = 0;
            for (int frame = 0; frame < 80; frame++)
            {
                core.RunFrame();
                long before = GC.GetAllocatedBytesForCurrentThread();
                clock.Restart();
                rewind.OnFrameCompleted(core);
                captureMs += clock.Elapsed.TotalMilliseconds;
                captured += GC.GetAllocatedBytesForCurrentThread() - before;
                captures++;
                if (frame % 4 != 3) continue;
                // The capture's delta is encoded on a pool thread; a step taken before it finishes runs it here, which is the buffer's cost, not the core's.
                Thread.Sleep(50);
                before = GC.GetAllocatedBytesForCurrentThread();
                clock.Restart();
                Assert.True(rewind.Rewind(core));
                stepMs += clock.Elapsed.TotalMilliseconds;
                stepped += GC.GetAllocatedBytesForCurrentThread() - before;
                steps++;
            }

            int size = rewind.SnapshotBytes;
            _output.WriteLine($"a {size}-byte snapshot: {captured / captures} bytes allocated a capture ({captureMs / captures:F2} ms) and {stepped / steps} a step back ({stepMs / steps:F2} ms)");
            Assert.True(captured / captures < size / 16, $"{captured / captures} bytes a capture against a snapshot of {size}");
            Assert.True(stepped / steps < size / 16, $"{stepped / steps} bytes a step back against a snapshot of {size}");
        }

        // Every frame of the run and every step back compared with the reference: the state always, the picture wherever a step lands - see Mars_Native.md §6.6.3.
        [Theory]
        [MemberData(nameof(Games))]
        public void Rewinding_with_the_workers_deferred_lands_on_the_state_and_picture_the_machine_had(string romName, string stateName)
        {
            if (Game(romName, stateName) is not { } game) return;
            int frames = Frames(600);
            var history = Reference(game, frames + 1);
            long first = history.Keys.Min(), last = first + frames;
            int[] distances = { 1, 2, 3, 5, 8, 13, 21, 4, 7, 1 };

            foreach (int interval in new[] { 1, RewindBuffer.DefaultIntervalFrames })
            {
                using MarsRtCore subject = Production();
                subject.LoadRom(game.Rom);
                subject.LoadState(game.State);
                var rewind = new RewindBuffer { Enabled = true, IntervalFrames = interval };
                rewind.CaptureNow(subject);

                int ran = 0, rewound = 0, steps = 0, stateChecks = 0, shownAsReference = 0, shownAsLoaded = 0, pending = 0, captures = 0, replayed = 0, burst = 0;
                var reached = new HashSet<long>();
                var commands = new SortedDictionary<int, int>();
                while (subject.TotalFrames < last)
                {
                    Drive(subject);
                    subject.RunFrame();
                    rewind.OnFrameCompleted(subject);
                    if (subject.TotalFrames % interval == 0)
                    {
                        pending += Unrun(rewind) > 0 ? 1 : 0;
                        captures++;
                    }
                    ran++;
                    long at = subject.TotalFrames;
                    if (interval == 1 || at % 60 == 0)
                    {
                        Assert.True(history[at].State == Hash(subject.Save(false)), $"{romName}, interval {interval}: frame {at} run again after {rewound} steps back is not the reference's");
                        stateChecks++;
                    }
                    if (!reached.Add(at) || ran % 30 != 0) continue;

                    // A burst of steps back, each checked where it lands.
                    int distance = distances[burst++ % distances.Length];
                    for (int step = 0; step < distance; step++)
                    {
                        long before = subject.TotalFrames;
                        if (!rewind.Rewind(subject)) break;
                        steps++;
                        replayed += Unrun(rewind) > 0 ? 1 : 0;
                        Tally(rewind, commands);
                        long landed = subject.TotalFrames;
                        Assert.True(step == 0 ? landed <= before - interval && landed > before - 2 * interval : landed == before - interval, $"{romName}, interval {interval}: a step back from frame {before} landed on {landed}");
                        Assert.True(history[landed].State == Hash(subject.Save(false)), $"{romName}, interval {interval}: the step back from {before} to {landed} is not the state the machine had");
                        stateChecks++;
                        string shown = Picture(subject);
                        shownAsReference += shown == history[landed].Shown ? 1 : 0;
                        shownAsLoaded += shown == history[landed].Loaded ? 1 : 0;
                        Assert.True(shown == history[landed].Loaded, $"{romName}, interval {interval}: frame {landed} shows {shown}, a load of it shows {history[landed].Loaded}");
                    }
                    rewound += distance;
                }

                _output.WriteLine($"{romName} {stateName}, interval {interval}: {ran} frames run to reach {frames}, {steps} steps back in {burst} bursts, {stateChecks} states compared, all identical; {pending} of {captures} captures held words unrun and {replayed} landings replayed some; the picture where a step landed was a load's in {shownAsLoaded} of {steps} and the one the frame first presented in {shownAsReference}; rewind held {rewind.Depth} steps, {rewind.BufferedBytes >> 20} MB; the landings' unrun commands {string.Join(" ", commands.Select(c => $"{c.Key:X2}x{c.Value}"))}");
                long[] counters = subject.ThreadCounterValues();
                Assert.True(counters[0] == 1 && counters[1] > 0, "the drain never ran, so the workers were never busy at a snapshot");
            }
        }

        // Rewind and advance alternated at random while the workers are busy, each operation watched: none may take longer than the watchdog allows.
        [Theory]
        [MemberData(nameof(Games))]
        public void Alternating_rewind_and_advance_with_the_workers_busy_never_freezes(string romName, string stateName)
        {
            if (Game(romName, stateName) is not { } game) return;
            int operations = Frames(600);
            var watchdog = TimeSpan.FromSeconds(30);
            int tail = -1, snapshots = 0, withWords = 0, steps = 0, frames = 0;
            long words = 0;
            string last = "";
            var log = new Queue<string>();

            using MarsRtCore subject = Production();
            subject.LoadRom(game.Rom);
            subject.LoadState(game.State);
            var rewind = new RewindBuffer { Enabled = true, IntervalFrames = 1 };
            var random = new Random(romName.Length * 7919 ^ 0x5EED);
            void Count(byte[] snapshot)
            {
                if (tail < 0) tail = snapshot.Length - 4 - 8 * DpInterface.SnapshotWords;
                int pending = BitConverter.ToInt32(snapshot, tail);
                snapshots++;
                withWords += pending > 0 ? 1 : 0;
                words += pending;
            }

            for (int op = 0; op < operations; op++)
            {
                int kind = random.Next(10);
                int count = random.Next(1, 6);
                int spin = random.Next(3) == 0 ? random.Next(0, 40_000) : 0;
                last = $"op {op}: {(kind < 6 ? "advance" : kind < 9 ? "rewind" : "snapshot")} {count} at frame {subject.TotalFrames}";
                log.Enqueue(last);
                if (log.Count > 8) log.Dequeue();
                var work = Task.Factory.StartNew(() =>
                {
                    if (kind < 6)
                    {
                        for (int i = 0; i < count; i++)
                        {
                            Drive(subject);
                            subject.RunFrame();
                            Thread.SpinWait(spin);
                            if (random.Next(2) == 0) Count(subject.Save(snapshot: true));
                            rewind.OnFrameCompleted(subject);
                            frames++;
                        }
                    }
                    else if (kind < 9)
                    {
                        for (int i = 0; i < count && rewind.Rewind(subject); i++) steps++;
                        _ = subject.GetFrameBufferRgba();
                    }
                    else
                    {
                        Drive(subject);
                        subject.RunFrame();
                        Thread.SpinWait(spin);
                        Count(subject.Save(snapshot: true));
                        frames++;
                    }
                }, TaskCreationOptions.LongRunning);
                if (!work.Wait(watchdog)) Assert.Fail($"{romName}: frozen for {watchdog.TotalSeconds} s at {last}; before it: {string.Join("; ", log)}");
            }

            _output.WriteLine($"{romName} {stateName}: {operations} operations, {frames} frames run and {steps} steps back without a freeze; {withWords} of {snapshots} snapshots taken where a capture was, or alone, caught words unrun ({words} in all)");
            Assert.True(steps > operations / 2 && frames > operations, "too few rewinds or frames to stress anything");
        }
    }
}
