using System;
using System.Collections.Generic;
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
    // The C# core threaded, split and deferred against itself on one thread at once, frame by frame from the gameplay states - see Mars_Rdp.md §2.8.
    [Collection("MarsStatics")]
    public class MarsThreadedGamesTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly List<string> _temporary = new();
        private readonly bool _rspBlocks = EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks;

        public MarsThreadedGamesTests(ITestOutputHelper output)
        {
            _output = output;
            EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = true;
        }

        public void Dispose()
        {
            EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = _rspBlocks;
            foreach (string path in _temporary)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        private static int Frames(int otherwise) => int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_MARS_GAME_FRAMES"), out int n) ? n : otherwise;

        // Each mode: its drain's shape, whether its picture is a frame late, and how often its state is compared; every sixtieth frame leaves the drain running across frames, and -1 compares a snapshot loaded into a scratch core.
        public static TheoryData<string, string, int, bool, int> Modes()
        {
            var data = new TheoryData<string, string, int, bool, int>();
            foreach (var (rom, state) in new[] { ("sm64.z64", "sm64.state"), ("oot.z64", "oot.state"), ("ge.z64", "ge-dam.state") })
                foreach (var (workers, deferred, every) in new[] { (1, false, 1), (4, false, 1), (0, true, 1), (1, true, 1), (4, true, 1), (4, true, 60), (4, true, -1) })
                    data.Add(rom, state, workers, deferred, every);
            return data;
        }

        [Theory]
        [MemberData(nameof(Modes))]
        public void The_csharp_core_threaded_split_and_deferred_leaves_what_it_leaves_on_one_thread_at_once(string romName, string stateName, int workers, bool deferred, int every)
        {
            string? folder = Environment.GetEnvironmentVariable(MarsRtTests.StatesVariable);
            if (folder is null || !File.Exists(Path.Combine(folder, romName)) || !File.Exists(Path.Combine(folder, stateName)))
            {
                _output.WriteLine($"{romName} {stateName}: absent, not run");
                return;
            }

            string rom = SyntheticN64Rom.WriteTemp(File.ReadAllBytes(Path.Combine(folder, romName)));
            _temporary.Add(rom);
            byte[] state = File.ReadAllBytes(Path.Combine(folder, stateName));
            int frames = Frames(300);

            MarsCore reference = new(expansionPak: true, batteryRamDisabled: true) { UseBlocks = true, ThreadedRdp = false, DeferredPresentation = false, SkipRendering = false };
            MarsCore subject = new(expansionPak: true, batteryRamDisabled: true) { UseBlocks = true, ThreadedRdp = workers > 0, RdpWorkers = Math.Max(workers, 1), DeferredPresentation = deferred, SkipRendering = false };
            foreach (MarsCore core in new[] { reference, subject })
            {
                core.LoadRom(rom);
                core.LoadState(new MemoryStream(state));
            }

            MarsCore scratch = new(expansionPak: true, batteryRamDisabled: true) { UseBlocks = false, ThreadedRdp = false, SkipRendering = true };
            scratch.LoadRom(rom);

            // Deferred and unthreaded, whose count of repeats a threaded capture must match, since a capture freed early compares bytes still being drawn - see Mars_Rdp.md §2.9.2.
            MarsCore? counter = deferred && workers > 0 ? new(expansionPak: true, batteryRamDisabled: true) { UseBlocks = true, ThreadedRdp = false, DeferredPresentation = true, SkipRendering = false } : null;
            if (counter is not null)
            {
                counter.LoadRom(rom);
                counter.LoadState(new MemoryStream(state));
            }

            byte[] previous = reference.GetFrameBufferRgba().ToArray();
            int pictures = 0;
            for (int frame = 1; frame <= frames; frame++)
            {
                Drive(reference, subject, frame);
                reference.RunFrame();
                subject.RunFrame();
                if (counter is not null)
                {
                    Drive(counter, counter, frame);
                    counter.RunFrame();
                }
                if (every < 0)
                {
                    using var snapshot = new MemoryStream();
                    var taking = System.Threading.Tasks.Task.Run(() => subject.SaveSnapshot(snapshot));
                    if (!taking.Wait(TimeSpan.FromSeconds(30))) throw new Xunit.Sdk.XunitException($"frame {frame}: the snapshot was never answered");
                    taking.GetAwaiter().GetResult();
                    snapshot.Position = 0;
                    scratch.LoadState(snapshot);
                    Same(frame, Save(reference), Save(scratch));
                }
                else if (frame % every == 0 || frame == frames) Same(frame, Save(reference), Save(subject));
                byte[] want = deferred ? previous : reference.GetFrameBufferRgba();
                byte[] got = subject.GetFrameBufferRgba();
                if (!want.AsSpan().SequenceEqual(got)) throw new Xunit.Sdk.XunitException($"frame {frame}: the pictures differ from byte {want.AsSpan().CommonPrefixLength(got)} of {want.Length} ({got.Length})");
                if (!previous.AsSpan().SequenceEqual(reference.GetFrameBufferRgba())) pictures++;
                previous = reference.GetFrameBufferRgba().ToArray();
            }

            long words = subject.Bus!.Dp.DrainWords;
            foreach (var (count, _) in subject.Bus.Dp.WorkerLoads) words += count;
            if (workers > 0) Assert.True(words > 0, "nothing reached the drain, so the threaded case was not met");
            if (counter is not null) Assert.True(counter.RepeatedScans == subject.RepeatedScans, $"{subject.RepeatedScans} repeats skipped threaded, {counter.RepeatedScans} unthreaded");
            _output.WriteLine($"{romName} {stateName}, workers {workers}, deferred {deferred}, state every {every}: {frames} frames exact; {pictures} distinct pictures, {words} words on the drain, {subject.RepeatedScans} repeats skipped");
        }

        private static byte[] Save(MarsCore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        private static void Same(int frame, byte[] want, byte[] got)
        {
            if (want.AsSpan().SequenceEqual(got)) return;
            throw new Xunit.Sdk.XunitException($"frame {frame}: the states differ from byte {want.AsSpan().CommonPrefixLength(got)} of {want.Length} ({got.Length}), in {Differing(want, got)}");
        }

        // The fields two states of one length differ in, named by MarsRT's layout of the format when its library is here.
        private static string Differing(byte[] a, byte[] b)
        {
            if (!MarsRtCore.Available || a.Length != b.Length) return "fields not named";
            using var machine = new MarsMachine(BitConverter.ToInt32(a, 8));
            machine.Load(a);
            var names = machine.Layout(snapshot: false).Split('\n', StringSplitOptions.RemoveEmptyEntries).Select(l => l.Split(' '))
                .Where(p => !a.AsSpan(int.Parse(p[0]), int.Parse(p[1])).SequenceEqual(b.AsSpan(int.Parse(p[0]), int.Parse(p[1])))).Select(p => p[3]);
            return string.Join(", ", names.Take(8));
        }

        // The same input to both cores, as MarsRtThreadsTests drives them.
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
