using System;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The frontend API through its binding on the C# implementation, over the v1 test core: the session's life, the frame and its hand-over, input, speed, rewind, states, settings, firmware, save media, events and the twins - see EmuSen_Backend.md §14.
    [Collection(TestCollections.ProcessGlobals)]
    public class ManagedCrystalTests : IDisposable
    {
        private readonly CrystalFixture _f = new();

        public void Dispose() => _f.Dispose();

        private static JsonElement Json(string text) => JsonDocument.Parse(text).RootElement;

        private static CrystalFrameInfo Frame(CrystalSession s)
        {
            var info = new CrystalFrameInfo();
            s.Frame(ref info);
            return info;
        }

        private static byte[] Picture(CrystalSession s)
        {
            var bytes = new byte[s.PictureBytes];
            s.CopyPicture(bytes);
            return bytes;
        }

        [Fact]
        public void The_C_sharp_implementation_serves_the_API_while_the_switch_is_unset()
        {
            Assert.False(CrystalRuntime.Native);
            Assert.Contains("C# implementation", CrystalRuntime.Implementation);
            Assert.Equal(CrystalAbi.Version, EmuSen.Crystal.Client.Contract.Crystal.AbiVersion());
            Assert.StartsWith("EmuSen's C# implementation of the frontend API 1.0", CrystalRuntime.VersionText);
        }

        [Fact]
        public void A_game_opens_lists_describes_itself_and_closes()
        {
            string path = _f.Game();
            using CrystalSession s = _f.Open(path);
            Assert.Equal(1u, s.Id);
            JsonElement listed = Json(_f.Runtime.Sessions()).EnumerateArray().Single();
            Assert.Equal((1u, true), (listed.GetProperty("id").GetUInt32(), listed.GetProperty("current").GetBoolean()));
            JsonElement info = Json(s.Info());
            Assert.Equal("V1TestCore (test)", info.GetProperty("engine").GetString());
            Assert.Equal("test", info.GetProperty("system").GetString());
            Assert.Equal(path, info.GetProperty("game").GetProperty("path").GetString());
            Assert.Equal("running", info.GetProperty("run").GetString());
            Assert.True(info.GetProperty("ports").GetArrayLength() >= 1);
            Assert.Contains((int)PadButton.A, info.GetProperty("buttons").EnumerateArray().Select(b => b.GetInt32()));
            s.Close();
            Assert.Empty(Json(_f.Runtime.Sessions()).EnumerateArray());
            Assert.Equal(CrystalSessionAbi.NoSession, Assert.Throws<CrystalException>(() => Frame(s)).Status);
        }

        [Fact]
        public void What_open_refuses_it_refuses_with_its_status_and_words()
        {
            Assert.Equal(CrystalSessionAbi.BadArgument, Assert.Throws<CrystalException>(() => _f.Runtime.Open(new CrystalOpen())).Status);
            Assert.Equal(CrystalSessionAbi.BadArgument, Assert.Throws<CrystalException>(() => _f.Runtime.Open(new CrystalOpen { Path = "a.tst", Console = "test" })).Status);
            Assert.Equal(CrystalAbi.Io, Assert.Throws<CrystalException>(() => _f.Runtime.Open(new CrystalOpen { Path = Path.Combine(_f.Root, "none.tst") })).Status);
            string unknown = Path.Combine(_f.Root, "game.xyz");
            File.WriteAllBytes(unknown, new byte[] { 1 });
            Assert.Equal(CrystalAbi.NoEngine, Assert.Throws<CrystalException>(() => _f.Runtime.Open(new CrystalOpen { Path = unknown })).Status);
            string garbage = Path.Combine(_f.Root, "garbage.tst");
            File.WriteAllBytes(garbage, "garbage!"u8.ToArray());
            var refused = Assert.Throws<CrystalException>(() => _f.Runtime.Open(new CrystalOpen { Path = garbage }));
            Assert.Equal(CrystalSessionAbi.Fault, refused.Status);
            Assert.Contains("not a test core image", refused.Message);
        }

        // The frame call against the engine run directly beside it: the same pictures and the same sound - see EmuSen_Backend.md §14.
        [Fact]
        public void The_frame_call_hands_over_what_the_engine_makes()
        {
            string path = _f.Game(payload: new byte[] { 9, 8, 7, 6, 5 });
            using CrystalSession s = _f.Open(path);
            using var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            engine.LoadRom(path);
            var drained = new short[8192];
            for (int i = 0; i < 30; i++)
            {
                CrystalFrameInfo info = Frame(s);
                engine.RunFrame();
                Assert.Equal((1u, 0u, 1u), (info.ran, info.halted, info.rendered));
                Assert.Equal((ulong)engine.TotalFrames, info.total_frames);
                Assert.Equal((engine.ScreenWidth, engine.ScreenHeight), (info.width, info.height));
                Assert.Equal(engine.GetFrameBufferRgba(), Picture(s));
                short[] expected = engine.DequeueAudioSamples(int.MaxValue);
                Assert.Equal(expected.Length / 2, (int)info.audio_frames);
                int frames = s.DrainAudio(drained, out uint rate);
                Assert.Equal(expected, drained[..(frames * 2)]);
                Assert.Equal((uint)engine.AudioSampleRate, rate);
            }
        }

        // The frame call allocates nothing of its own: the engine's frame, the picture and the sound allocate nothing, and the call costs what the debugger's per-frame snapshots cost, measured on the calling thread - see EmuSen_Backend.md §14.
        [Fact]
        public void The_frame_call_allocates_nothing_but_the_debugger_s_snapshots()
        {
            using CrystalSession s = _f.Open();
            var info = new CrystalFrameInfo();
            var picture = new byte[1 << 20];
            var sound = new short[1 << 14];
            for (int i = 0; i < 200; i++)
            {
                s.Frame(ref info);
                s.CopyPicture(picture);
                s.DrainAudio(sound, out _);
            }
            long Measure(Action a)
            {
                long before = GC.GetAllocatedBytesForCurrentThread();
                for (int i = 0; i < 1000; i++) a();
                return GC.GetAllocatedBytesForCurrentThread() - before;
            }
            long frames = Measure(() => s.Frame(ref info));
            long pictures = Measure(() => s.CopyPicture(picture));
            long sounds = Measure(() => s.DrainAudio(sound, out _));

            using var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            engine.LoadRom(_f.Game("beside.tst"));
            var target = engine.CreateDebugTarget();
            for (int i = 0; i < 200; i++) engine.RunFrame();
            long runFrames = Measure(engine.RunFrame);
            long snapshots = Measure(target.RefreshProviders);

            Assert.Equal((0L, 0L, 0L), (runFrames, pictures, sounds));
            Assert.True(frames <= snapshots, $"1,000 frame calls allocated {frames} bytes, the debugger's snapshots alone {snapshots}");
        }
    }
}
