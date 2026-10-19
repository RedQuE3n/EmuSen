using System;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text.Json;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Crystal.Managed;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // One test per row of EmuSen_Surfaces.md §4.1's divergence table, re-measured in EmuSen_Backend.md §14: each row's step is inside the frame call or one API function, so every client that drives a core through the API gives one answer.
    [Collection(TestCollections.ProcessGlobals)]
    public class DivergenceRowTests : IDisposable
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

        // An engine that declares rewind withheld, as Mars (C#) once did.
        private sealed class Withholding(CoreLibrary library) : CoreEngine(library), IEngineFeatures
        {
            public new EngineFeatures Features { get; } = new(RewindCapture: false, RewindWithheld: "withheld for the test");
        }

        private CrystalSession AttachWithholding()
        {
            var engine = new Withholding(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            string game = _f.Game("withheld.tst");
            engine.LoadRom(game);
            var runtime = (ManagedRuntime)GCHandle.FromIntPtr(_f.Runtime.Handle).Target!;
            uint id = runtime.Attach(new CoreBundle(engine, engine.CreateDebugTarget(), null, null, null), game, "TEST").Id;
            return _f.Runtime.SessionOf(id);
        }

        // Row 1: rewind capture is gated by the engine's opt-out in the frame call, whatever the client (Mistress gated it, Hotaru and Pharaoh did not).
        [CSharpImplementationFact("it attaches a C# engine that withholds rewind to the C# runtime; every v1 engine allows rewind, and the Rust answers rewind_capture true")]
        public void Row_1_rewind_capture_is_gated_by_the_engine_for_every_client()
        {
            CrystalSession withheld = AttachWithholding();
            withheld.Rewind(true, intervalFrames: 1);
            for (int i = 0; i < 10; i++) Frame(withheld);
            Assert.Equal(0, Json(withheld.RewindReel()).GetProperty("depth").GetInt32());
            Assert.False(Json(withheld.Info()).GetProperty("rewind_capture").GetBoolean());
            Assert.Equal("withheld for the test", Json(withheld.Info()).GetProperty("rewind_withheld").GetString());

            using CrystalSession allowed = _f.Open();
            allowed.Rewind(true, intervalFrames: 1);
            for (int i = 0; i < 10; i++) Frame(allowed);
            Assert.True(Json(allowed.RewindReel()).GetProperty("depth").GetInt32() > 0);
        }

        // Row 2: one post-load sequence, whichever way the load is asked for: the binding's call, the session's table (a shell's `state load`), a slot or the resume state.
        [Fact]
        public unsafe void Row_2_every_way_of_loading_a_state_runs_one_sequence()
        {
            using CrystalSession s = _f.Open();
            using CrystalSubscription events = _f.Runtime.Subscribe(s.Id, CrystalAbi.EventState);
            string path = Path.Combine(_f.Root, "one.state");
            for (int i = 0; i < 3; i++) Frame(s);
            s.SaveState(path);
            s.SaveSlot(1);
            s.SaveResume();
            var table = (EmuSen.Crystal.Session.EmuSenSessionV1*)s.ServiceTable();
            byte[] bytes = System.Text.Encoding.UTF8.GetBytes(path);
            Action[] loads =
            {
                () => s.LoadState(path),
                () => { fixed (byte* p = bytes) Assert.Equal(0, table->state_load(table->ctx, 0, p, (nuint)bytes.Length)); },
                () => s.LoadSlot(1),
                () => s.LoadResume(),
            };
            foreach (Action load in loads)
            {
                s.Rewind(true, intervalFrames: 1);
                for (int i = 0; i < 4; i++) Frame(s);
                Assert.True(Json(s.RewindReel()).GetProperty("depth").GetInt32() > 0);
                events.Drain();
                load();
                Assert.Equal(0, Json(s.RewindReel()).GetProperty("depth").GetInt32());
                Assert.Equal("loaded", Json(events.Drain()).EnumerateArray().Single().GetProperty("op").GetString());
            }
        }

        // Row 3: what the shell asked the host to do arrives as events every client can drain (Mistress handled one HostAction, Hotaru five, Pharaoh none).
        [Fact]
        public void Row_3_run_requests_reach_every_client_as_events()
        {
            using CrystalSession s = _f.Open();
            using CrystalSubscription run = _f.Runtime.Subscribe(s.Id, CrystalAbi.EventRun);
            s.Run(CrystalSessionAbi.RunPause);
            s.Run(CrystalSessionAbi.RunStep, argument: 1);
            s.Run(CrystalSessionAbi.RunResume);
            Assert.Equal(new[] { "paused", "stepped", "resumed" }, Json(run.Drain()).EnumerateArray().Select(e => e.GetProperty("op").GetString()));
        }

        // Row 4: a breakpoint halt is the frame call's to hold (Mistress never read IsHaltedAtBreakpoint; Hotaru and Pharaoh did).
        [Fact]
        public void Row_4_a_breakpoint_halt_holds_the_session_for_every_client()
        {
            using CrystalSession s = _f.Open();
            CrystalRuntime.WithText("{\"address\":4}", (p, l) => { unsafe { return CrystalRuntime.Check(EmuSen.Crystal.Client.Contract.Crystal.BpAdd(_f.Runtime.Handle, s.Id, 0, (byte*)p, l)); } });
            CrystalFrameInfo f = Frame(s);
            for (int i = 0; i < 10 && f.halted == 0; i++) f = Frame(s);
            Assert.Equal(1u, f.halted);
            ulong frames = f.total_frames;
            for (int i = 0; i < 3; i++) Assert.Equal((0u, frames), (Frame(s).ran, Frame(s).total_frames));
        }

        // Row 5: every client is handed the picture's whole shape: rows, aspect, lines and serial (Hotaru took width and height alone).
        [Fact]
        public void Row_5_the_hand_over_carries_the_picture_s_whole_shape()
        {
            string game = _f.Game();
            using CrystalSession s = _f.Open(game);
            using var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            engine.LoadRom(game);
            for (int i = 0; i < 5; i++)
            {
                engine.RunFrame();
                CrystalFrameInfo f = Frame(s);
                Assert.Equal(((ulong)engine.FrameSerial, engine.DisplayAspect, ((ICore)engine).DisplayLines, engine.RowRepeat), (f.picture_serial, f.display_aspect, f.display_lines, f.row_repeat));
            }
        }

        // Row 6: the picture comes with its rows as the console shows them unless the client asks for single rows, and a state's picture beside it is always the shown one (Mistress expanded repeated rows in screenshots, Hotaru and Pharaoh did not).
        [Fact]
        public void Row_6_pictures_come_rows_as_shown_unless_single_rows_are_asked()
        {
            using CrystalSession shown = _f.Open();
            using CrystalSession single = _f.Open(_f.Game("single.tst"), singleRows: true);
            CrystalFrameInfo a = Frame(shown), b = Frame(single);
            Assert.Equal(1, a.row_repeat);
            Assert.Equal(a.height, b.height * b.row_repeat);
            Assert.Equal((long)a.width * 4 * a.height, shown.PictureBytes);
            Assert.Equal((long)b.width * 4 * b.height, single.PictureBytes);
        }

        // Row 7, found while re-measuring: an engine's per-console settings reach it after the load for every client (Mistress applied them; Hotaru and Pharaoh ran a shim's defaults).
        [Fact]
        public void Row_7_stored_console_settings_reach_the_engine_for_every_client()
        {
            string game = _f.Game();
            using (CrystalSession probe = _f.Open(game))
            {
                string console = Json(probe.Info()).GetProperty("console").GetString()!;
                _f.Runtime.SetSetting(console, "Wide", "true");
            }
            using CrystalSession s = _f.Open(game);
            Frame(s);
            int wide = Frame(s).width;
            using var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            engine.LoadRom(game);
            engine.RunFrame();
            engine.RunFrame();
            Assert.Equal(engine.ScreenWidth * 2, wide);
        }
    }
}
