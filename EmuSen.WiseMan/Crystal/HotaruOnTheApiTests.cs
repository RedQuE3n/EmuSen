using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Common;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Hotaru;
using EmuSen.Serenity;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // emusen-shell's game window on the frontend API: its commands, its events and its picture hand-off, each held to the defect it had - see EmuSen_Backend.md §14.11.
    [Collection(TestCollections.ProcessGlobals)]
    public class HotaruOnTheApiTests : IDisposable
    {
        private readonly CrystalFixture _f = new();

        public void Dispose() => _f.Dispose();

        private static JsonElement Json(string text) => JsonDocument.Parse(text).RootElement;

        private static void Frames(CrystalSession s, int count)
        {
            var info = new CrystalFrameInfo();
            for (int i = 0; i < count; i++) s.Frame(ref info);
        }

        private sealed record Host(CrystalRuntime Runtime, CrystalSession Session) : IHotaruHost;

        // C7, Hotaru's console case: its `state load` called the engine's LoadState, so the rewind history from before the load survived it; on the API the load clears it and raises the event the window resets its rate control on.
        [Fact]
        public void C7_hotaru_s_console_state_load_clears_rewind_and_resets_the_rate_control()
        {
            string game = _f.Game("Seven.tst");
            string kept = System.IO.Path.Combine(_f.Root, "Seven.state");
            using (var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery })
            {
                engine.LoadRom(game);
                var rewind = new RewindBuffer { Enabled = true };
                var before = DianaShInterpreter.CreateDefault(engine.CreateDebugTarget(), new IDianaShCommand[] { new StateCommand(engine.SaveState, engine.LoadState, () => kept) });
                Assert.Contains("[STATE] Saved", before.Execute("state save"));
                for (int i = 0; i < 20; i++)
                {
                    engine.RunFrame();
                    rewind.OnFrameCompleted(engine);
                }
                Assert.Contains("[STATE] Loaded", before.Execute("state load"));
                Assert.True(rewind.Depth > 0, "the old wiring's load left the rewind history it should have cleared");
            }

            using CrystalSession s = _f.Open(game);
            s.Rewind(true);
            var shell = DianaShInterpreter.CreateForService(s.ServiceTable(), HotaruCommands.For(new Host(_f.Runtime, s)));
            using var events = new HotaruEvents(_f.Runtime);
            Assert.Contains("[STATE] Saved", shell.Execute("state save"));
            Frames(s, 20);
            Assert.True(Json(s.RewindReel()).GetProperty("depth").GetInt32() > 0);
            int resets = 0;
            Assert.False(events.Drain(s.Id, () => resets++));
            Assert.Equal(0, resets);
            Assert.Contains("[STATE] Loaded", shell.Execute("state load"));
            Assert.False(events.Drain(s.Id, () => resets++));
            Assert.Equal((1, 0), (resets, Json(s.RewindReel()).GetProperty("depth").GetInt32()));
        }

        // The window's events keep its own session's: another session's load resets nothing, and its own session closed ends it.
        [Fact]
        public void The_window_s_events_are_its_session_s()
        {
            using CrystalSession mine = _f.Open(_f.Game("Mine.tst"));
            using CrystalSession other = _f.Open(_f.Game("Other.tst"));
            using var events = new HotaruEvents(_f.Runtime);
            other.SaveState();
            other.LoadState();
            int resets = 0;
            Assert.False(events.Drain(mine.Id, () => resets++));
            Assert.Equal(0, resets);
            mine.Close();
            Assert.True(events.Drain(mine.Id, () => resets++));
        }

        // C4's frontend half: the window's pictures go to the frame control in at most four buffers made once, where Hotaru's hand-over made a new one each frame.
        [Fact]
        public Task C4_the_window_s_pictures_reuse_their_buffers() => UiTest.Run(() =>
        {
            using CrystalSession s = _f.Open();
            var seen = new HashSet<byte[]>(ReferenceEqualityComparer.Instance);
            Action<byte[]>? releasePrevious = null;
            byte[]? previous = null;
            var slots = new PictureSlots((pixels, width, height, release) =>
            {
                if (previous is not null) releasePrevious!(previous);
                seen.Add(pixels);
                (previous, releasePrevious) = (pixels, release);
            });
            var info = new CrystalFrameInfo();
            int offered = 0;
            for (int i = 0; i < 100; i++)
            {
                s.Frame(ref info);
                if (slots.Rent(info.width * info.height * 4) is not { } pixels) continue;
                s.CopyPicture(pixels);
                slots.Offer(info.width, info.height);
                offered++;
                Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            }
            Assert.Equal(100, offered);
            Assert.InRange(seen.Count, 1, 4);
        });

        private static void Pump(int milliseconds)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (clock.ElapsedMilliseconds < milliseconds)
            {
                Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                Thread.Sleep(10);
            }
        }

        private uint[] SessionIds() => Json(_f.Runtime.Sessions()).EnumerateArray().Select(e => e.GetProperty("id").GetUInt32()).ToArray();

        // The window over a session of the test core runs it, and C6: a `core` swap opens the new session and closes the old on the UI thread, so nothing happens until that thread runs, after which the pads are rebound there.
        [Fact]
        public Task C6_the_window_runs_its_session_and_swaps_on_the_ui_thread() => UiTest.Run(() =>
        {
            TextWriter original = Console.Out;
            HotaruSession hotaru = HotaruSession.Build(_f.Game("Window.tst"), original, _f.Runtime);
            try
            {
                var window = hotaru.OpenWindow();
                window.Show();
                Pump(500);
                Assert.True(Json(hotaru.Session.Info()).GetProperty("total_frames").GetInt64() > 5);
                uint first = Assert.Single(SessionIds());

                string swapped = Path.Combine(EmuSen.DianaSh.DianaSh.Etc.DianaShSandbox.RootDirectory, "Swapped.tst");
                Directory.CreateDirectory(Path.GetDirectoryName(swapped)!);
                File.Copy(_f.Game("Swapped.tst"), swapped, overwrite: true);
                ((EmuSen.Hotaru.Views.ILiveShell)window).Submit("core test " + swapped);
                Thread.Sleep(300);
                Assert.Equal(new[] { first }, SessionIds());
                Pump(500);
                uint second = Assert.Single(SessionIds());
                Assert.NotEqual(first, second);
                Assert.True(_f.Runtime.SessionOf(second).Info().Contains("Swapped"));

                window.Close();
                Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                Assert.Empty(SessionIds());
            }
            finally
            {
                Console.SetOut(original);
                hotaru.LogWriter.Dispose();
            }
        });

        // A game's log says which implementation serves it, as Pharaoh's does - see EmuSen_Backend.md §29.28.
        [Fact]
        public void A_game_s_log_names_the_implementation_that_serves_it()
        {
            TextWriter original = Console.Out;
            var echoed = new StringWriter();
            HotaruSession hotaru = HotaruSession.Build(_f.Game("Named.tst"), echoed, _f.Runtime);
            try
            {
                hotaru.Session.Dispose();
            }
            finally
            {
                Console.SetOut(original);
                hotaru.LogWriter.Dispose();
            }
            Assert.Contains($"[ROM] Runtime: {CrystalRuntime.Implementation}", echoed.ToString().Split(Environment.NewLine));
        }

        // `core <name>` on the API knows every name the catalog's registry knows, each with the same display name, console and extensions, so its words are unchanged.
        [Fact]
        public void The_core_vocabulary_read_from_the_API_is_the_registry_s()
        {
            var cores = HotaruCores.Of(_f.Runtime).Registry;
            foreach (var (name, descriptor) in EmuSen.Cores.CoreCatalog.Registry)
            {
                Assert.True(cores.TryGetValue(name, out var choice), $"`core {name}` is not known on the API");
                Assert.Equal((descriptor.DisplayName, descriptor.Console, string.Join(",", descriptor.Extensions)), (choice!.DisplayName, choice.Console, string.Join(",", choice.Extensions)));
            }
        }

        // C3's other half: a copy waiting beside a frame loop that never pauses lands before the loop's next frame or so, never starved by it - see EmuSen_Backend.md §29.28.
        [Fact]
        public void C3_a_copy_beside_a_frame_loop_that_never_pauses_is_not_starved()
        {
            using CrystalSession s = _f.Open();
            bool stop = false;
            long ran = 0;
            var frames = new Thread(() => { var info = new CrystalFrameInfo(); while (!Volatile.Read(ref stop)) { s.Frame(ref info); Interlocked.Increment(ref ran); } });
            frames.Start();
            while (Interlocked.Read(ref ran) < 5) Thread.Yield();
            double longest = 0;
            int copies = 0;
            var copier = new Thread(() =>
            {
                var rgba = new byte[(int)s.PictureBytes];
                for (; copies < 20; copies++)
                {
                    long t0 = System.Diagnostics.Stopwatch.GetTimestamp();
                    s.CopyPicture(rgba);
                    longest = Math.Max(longest, System.Diagnostics.Stopwatch.GetElapsedTime(t0).TotalMilliseconds);
                }
            });
            copier.Start();
            bool done = copier.Join(TimeSpan.FromSeconds(30));
            int landed = Volatile.Read(ref copies);
            Volatile.Write(ref stop, true);
            frames.Join();
            copier.Join();
            Assert.True(done, $"{landed} of 20 copies in 30 s beside the frames");
            Assert.True(longest < 1000, $"a copy waited {longest:F0} ms beside the frames");
        }

        // C3 on the API: the feed window's copy, taken on another thread while frames run, is always one whole picture of the test core's, never a mix of two.
        [Fact]
        public async Task C3_a_picture_copied_beside_the_frames_is_always_whole()
        {
            using CrystalSession s = _f.Open();
            Frames(s, 2);
            int width = Json(s.Info()).GetProperty("picture").GetProperty("width").GetInt32();
            var info = new CrystalFrameInfo();
            using var stop = new CancellationTokenSource(TimeSpan.FromMilliseconds(500));
            Task frames = Task.Run(() => { while (!stop.IsCancellationRequested) s.Frame(ref info); });
            int copies = 0, broken = 0;
            await Task.Run(() =>
            {
                var rgba = new byte[(int)s.PictureBytes];
                while (!stop.IsCancellationRequested)
                {
                    s.CopyPicture(rgba);
                    copies++;
                    for (int p = 1; p < rgba.Length / 4; p++)
                        if (p % width != 0 && (byte)(rgba[p * 4] - rgba[(p - 1) * 4]) != 1) { broken++; break; }
                }
            });
            await frames;
            Assert.True(copies > 0);
            Assert.Equal(0, broken);
        }
    }
}
