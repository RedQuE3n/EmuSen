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

        private static MarsRtCore Twin(bool threaded, bool deferred) =>
            new(expansionPak: true, batteryRamDisabled: true) { ThreadedRdp = threaded, DeferredPresentation = deferred, SkipRendering = false };

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
                ("C# shipped", () => new MarsCore(expansionPak: true, batteryRamDisabled: true) { UseBlocks = true, ThreadedRdp = true, DeferredPresentation = true, RdpWorkers = Math.Clamp(Environment.ProcessorCount / 3, 1, 4) }),
                ("C# shipped, one worker", () => new MarsCore(expansionPak: true, batteryRamDisabled: true) { UseBlocks = true, ThreadedRdp = true, DeferredPresentation = true, RdpWorkers = 1 }),
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
