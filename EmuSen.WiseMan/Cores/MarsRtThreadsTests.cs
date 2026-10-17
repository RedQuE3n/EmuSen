using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MarsRT's display processor on a thread and its picture deferred, against MarsRT on one thread - see Mars_Native.md §5.6, §7.5.
    [Collection("MarsStatics")]
    public class MarsRtThreadsTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly List<string> _temporary = new();

        public MarsRtThreadsTests(ITestOutputHelper output) => _output = output;

        public void Dispose()
        {
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
            new(expansionPak: true) { Options = TestCores.NoBattery, ThreadedRdp = threaded, RdpWorkers = workers, DeferredPresentation = deferred, SkipRendering = false, UseBlocks = blocks, BlockTier = tier };

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

        // The phases on a game with its workers busy: each frame's waits inside the frame's own time, and some frame waiting at all - see Mars_Native.md §6.6.2.
        [Theory]
        [InlineData("sm64.z64", "sm64.state")]
        [InlineData("oot.z64", "oot.state")]
        [InlineData("ge.z64", "ge-dam.state")]
        public void MarsRT_s_phases_on_a_game_are_each_frame_s_own(string romName, string stateName)
        {
            if (Game(romName, stateName) is not { } game) return;
            using MarsRtCore subject = Twin(threaded: true, deferred: true, workers: 4, blocks: true);
            Load(subject, game);
            int frames = Frames(300), waitedInMachine = 0, waitedInPresent = 0, joined = 0;
            double machine = 0, machineWaits = 0, present = 0, presentWaits = 0;
            var clock = new Stopwatch();
            for (int frame = 1; frame <= frames; frame++)
            {
                Drive(subject, subject, frame);
                clock.Restart();
                subject.RunFrame();
                double wall = clock.Elapsed.TotalMilliseconds;
                var p = subject.LastFramePhases.ToDictionary(x => x.Name, x => x.Milliseconds);
                if (p["machine/rdp-wait"] > p["machine"] || p["present/rdp-wait"] + p["present/walk-join"] > p["present"] || p["machine"] + p["present"] > wall)
                    throw new Xunit.Sdk.XunitException($"frame {frame}: {string.Join(", ", p.Select(x => $"{x.Key} {x.Value:F3}"))} in a frame of {wall:F3} ms");
                waitedInMachine += p["machine/rdp-wait"] > 0 ? 1 : 0;
                waitedInPresent += p["present/rdp-wait"] > 0 ? 1 : 0;
                joined += p["present/walk-join"] > 0 ? 1 : 0;
                (machine, machineWaits, present, presentWaits) = (machine + p["machine"], machineWaits + p["machine/rdp-wait"], present + p["present"], presentWaits + p["present/rdp-wait"] + p["present/walk-join"]);
            }
            _output.WriteLine($"{romName}: {frames} frames, the machine {machine / frames:F3} ms a frame of which drain waits {machineWaits / frames:F3} ({waitedInMachine} frames waited), present {present / frames:F3} of which waits {presentWaits / frames:F3} ({waitedInPresent} waited on the drain, {joined} on the walk)");
            Assert.True(waitedInMachine + waitedInPresent > 0, "no frame waited for the drain, so the waits were not tested");
        }

        // MarsRT's list shared by several processors against MarsRT on one thread - see Mars_Native.md §5.6.6.
        [Theory]
        [MemberData(nameof(Games))]
        public void MarsRT_with_its_list_shared_leaves_what_MarsRT_on_one_thread_leaves(string romName, string? stateName)
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

        private static void Load(MarsRtCore core, (string Rom, byte[]? State) game)
        {
            core.LoadRom(game.Rom);
            if (game.State is { } state) core.LoadState(state);
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
