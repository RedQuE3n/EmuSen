using System;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The API's groups one by one on the C# implementation: input, speed, rewind, states, settings, firmware, save media, events, run control and the twins - see EmuSen_Backend.md §14.
    [Collection(TestCollections.ProcessGlobals)]
    public class ManagedCrystalGroupTests : IDisposable
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

        private static byte Ram(CrystalSession s, uint address)
        {
            Span<byte> one = stackalloc byte[1];
            s.ReadSpace("RAM", address, one);
            return one[0];
        }

        // The test core counts frames with button A held at RAM 3; the latch sends what was held as the next frame starts.
        [Fact]
        public void A_button_held_through_the_API_reaches_the_machine_at_the_next_frame()
        {
            using CrystalSession s = _f.Open();
            Frame(s);
            byte before = Ram(s, 3);
            s.Button(0, PadButton.A, true);
            Assert.Equal(before, Ram(s, 3));
            Frame(s);
            Frame(s);
            Assert.Equal(before + 2, Ram(s, 3));
            s.Button(0, PadButton.A, false);
            Frame(s);
            Assert.Equal(before + 2, Ram(s, 3));
            Assert.Equal(CrystalSessionAbi.NotSupported, Assert.Throws<CrystalException>(() => s.Button(0, (PadButton)40, true)).Status);
        }

        // Speed decides what is drawn and played and hands the pacer its interval; the pacing stays the client's - see EmuSen_Backend.md §3.4, SR6.
        [Fact]
        public void Speed_decides_drawing_sound_and_the_interval_the_pacer_reads()
        {
            using CrystalSession s = _f.Open();
            CrystalFrameInfo normal = Frame(s);
            Assert.Equal((100u, 1u, 1u), (normal.speed_percent, normal.play_audio, normal.rendered));
            Assert.InRange(normal.frame_interval_ns, (ulong)(1e9 / normal.frame_rate_hz) - 1000, (ulong)(1e9 / normal.frame_rate_hz) + 1000);
            s.Speed(100, turboPercent: 300);
            s.SpeedMode(CrystalAbi.SpeedTurbo);
            var drawn = Enumerable.Range(0, 9).Select(_ => Frame(s)).ToArray();
            Assert.All(drawn, f => Assert.Equal((300u, 0u), (f.speed_percent, f.play_audio)));
            Assert.Equal(3, drawn.Count(f => f.rendered == 1));
            s.SpeedMode(CrystalAbi.SpeedNormal);
            s.Speed(0);
            CrystalFrameInfo flat = Frame(s);
            Assert.Equal((0u, 0ul), (flat.speed_percent, flat.frame_interval_ns));
            s.Speed(100, drawNone: true);
            Assert.All(Enumerable.Range(0, 5).Select(_ => Frame(s)), f => Assert.Equal((1u, 0u), (f.ran, f.rendered)));
        }

        // A step back is in place of a frame: the machine goes back a snapshot, the picture shows it, and the history shortens.
        [Fact]
        public void Rewind_takes_snapshots_steps_back_and_reports_its_reel()
        {
            using CrystalSession s = _f.Open();
            s.Rewind(true, intervalFrames: 2);
            var pictures = new System.Collections.Generic.List<byte[]>();
            for (int i = 0; i < 10; i++)
            {
                Frame(s);
                pictures.Add(Picture(s));
            }
            JsonElement reel = Json(s.RewindReel());
            Assert.True(reel.GetProperty("enabled").GetBoolean());
            int depth = reel.GetProperty("depth").GetInt32();
            Assert.True(depth >= 4, $"depth {depth}");
            var info = new CrystalFrameInfo();
            Assert.Equal(1, s.RewindStep(1, ref info));
            Assert.Equal(1u, info.rendered);
            Assert.Equal(pictures[7], Picture(s));
            Assert.True(Json(s.RewindReel()).GetProperty("depth").GetInt32() < depth);
            s.Rewind(false);
            Assert.Equal(0, Json(s.RewindReel()).GetProperty("depth").GetInt32());
        }

        // A state saved and loaded by path, by slot and as the resume state, each in the player's state folder; a load clears rewind and says so - see EmuSen_Backend.md §3.12's C7, F4.
        [Fact]
        public void States_go_to_the_player_s_folder_and_a_load_runs_the_one_sequence()
        {
            string folder = Path.Combine(_f.Root, "My States");
            var settings = AppSettings.Load();
            settings.StateDirectory = folder;
            settings.Save();
            string game = _f.Game("Some Game.tst");
            using CrystalSession s = _f.Open(game);
            using CrystalSubscription events = _f.Runtime.Subscribe(s.Id, CrystalAbi.EventState | CrystalAbi.EventRewind);
            Assert.Equal(Path.Combine(folder, "Some Game.state"), s.DefaultStatePath());
            for (int i = 0; i < 5; i++) Frame(s);
            byte[] saved = Picture(s);
            s.SaveSlot(3);
            Assert.True(File.Exists(Path.Combine(folder, "Some Game.slot3.state")));
            Assert.True(File.Exists(Path.Combine(folder, "Some Game.slot3.png")));
            s.Rewind(true, intervalFrames: 1);
            for (int i = 0; i < 5; i++) Frame(s);
            Assert.NotEqual(saved, Picture(s));
            s.LoadSlot(3);
            Assert.Equal(saved, Picture(s));
            Assert.Equal(0, Json(s.RewindReel()).GetProperty("depth").GetInt32());
            JsonElement[] drained = Json(events.Drain()).EnumerateArray().ToArray();
            Assert.Equal(new[] { "saved", "loaded" }, drained.Where(e => e.GetProperty("kind").GetString() == "state").Select(e => e.GetProperty("op").GetString()));
            JsonElement slots = Json(s.StateSlots());
            Assert.Equal(9, slots.GetArrayLength());
            Assert.True(slots.EnumerateArray().Single(e => e.GetProperty("slot").GetInt32() == 3).GetProperty("exists").GetBoolean());
            s.SaveResume();
            Assert.True(File.Exists(Path.Combine(folder, "Some Game.resume.state")));
            Assert.Equal(CrystalSessionAbi.Absent, Assert.Throws<CrystalException>(() => s.LoadSlot(5)).Status);
            Assert.Equal(CrystalSessionAbi.BadArgument, Assert.Throws<CrystalException>(() => s.SaveSlot(9)).Status);
        }

        // Game over console over the schema's default; a value the schema refuses is refused; a running session takes a run-time value between frames.
        [Fact]
        public void Settings_are_read_in_layers_and_a_change_reaches_the_running_session()
        {
            using CrystalSession s = _f.Open();
            string console = Json(s.Info()).GetProperty("console").GetString()!;
            Assert.Contains("Wide", Json(s.SettingsSchema()).EnumerateArray().Select(r => r.GetProperty("key").GetString()));
            Assert.Equal("false", s.Setting("Wide"));
            Frame(s);
            int narrow = Frame(s).width;
            s.SetSetting("Wide", "true", game: false);
            Assert.Equal("true", ConsolesConfig.Load().Value(console, "Wide"));
            Assert.Equal("true", s.Setting("Wide", game: false));
            Frame(s);
            Assert.Equal(narrow * 2, Frame(s).width);
            s.SetSetting("Wide", "false", game: true);
            Assert.Equal("false", s.Setting("Wide"));
            Assert.Equal("true", s.Setting("Wide", game: false));
            Assert.True(File.Exists(Path.Combine(_f.Root, "Config", "games", "test", "game.json")));
            s.SetSetting("Wide", null, game: true);
            Assert.Equal("true", s.Setting("Wide"));
            Assert.Equal(CrystalSessionAbi.BadArgument, Assert.Throws<CrystalException>(() => s.SetSetting("Rate", "44100", game: false)).Status);
            Assert.Equal(CrystalSessionAbi.NoSuchId, Assert.Throws<CrystalException>(() => s.SetSetting("Nothing", "1", game: false)).Status);
            Assert.Equal("true", _f.Runtime.Setting(console, "Wide"));
        }

        // A game's own settings are /etc/EmuSen/games/<system>/<file stem>.json, so two consoles' games of one name keep two files (decided 2026-10-17).
        [Fact]
        public void Two_consoles_games_of_one_name_keep_two_settings_files()
        {
            string nes = EmuSen.Crystal.Managed.ManagedRuntime.GameSettingsPath("nes", "/roms/Tetris.nes");
            string gb = EmuSen.Crystal.Managed.ManagedRuntime.GameSettingsPath("gb", "/roms/Tetris.gb");
            Assert.Equal(Path.Combine(_f.Root, "Config", "games", "nes", "Tetris.json"), nes);
            Assert.Equal(Path.Combine(_f.Root, "Config", "games", "gb", "Tetris.json"), gb);
            Assert.NotEqual(nes, gb);
        }

        [Fact]
        public void Firmware_save_media_and_the_card_manager_answer_without_a_session()
        {
            string game = _f.Game();
            Assert.Equal(0, Json(_f.Runtime.FirmwareNeeded(game)).GetArrayLength());
            JsonElement overview = Json(_f.Runtime.FirmwareOverview());
            Assert.True(overview.GetProperty("folders").GetArrayLength() >= 1);
            using CrystalSession s = _f.Open(game);
            Assert.Equal(JsonValueKind.Array, Json(s.SaveMedia()).GetProperty("media").ValueKind);
            Assert.Equal(JsonValueKind.Array, Json(_f.Runtime.CardsOp(CrystalAbi.CardsList, "{\"console\":\"PlayStation\"}")).ValueKind);
            Assert.Equal(CrystalSessionAbi.Absent, Assert.Throws<CrystalException>(() => _f.Runtime.CardsOp(CrystalAbi.CardsOpen, "{\"path\":\"a.txt\"}")).Status);
            Assert.Equal(CrystalSessionAbi.NotSupported, Assert.Throws<CrystalException>(() => _f.Runtime.CardsOp(99, "{}")).Status);
            Assert.Equal(CrystalSessionAbi.BadArgument, Assert.Throws<CrystalException>(() => _f.Runtime.CardsOp(CrystalAbi.CardsList, "not json")).Status);
        }

        [Fact]
        public void The_systems_the_engines_and_a_route_answer_from_the_table_and_the_sidecars()
        {
            JsonElement[] systems = Json(_f.Runtime.Systems()).EnumerateArray().ToArray();
            Assert.Equal("nes", systems[0].GetProperty("id").GetString());
            Assert.Contains(systems, s => s.GetProperty("id").GetString() == "test" && s.GetProperty("engines").EnumerateArray().Any(e => e.GetString() == "V1TestCore (test)"));
            Assert.Contains(Json(_f.Runtime.Engines()).EnumerateArray(), e => e.GetProperty("id").GetString() == "v1-test-core");
            JsonElement route = Json(_f.Runtime.Route(_f.Game()));
            Assert.Equal("test", route.GetProperty("system").GetString());
            JsonElement nowhere = Json(_f.Runtime.Route(Path.Combine(_f.Root, "x.xyz")));
            Assert.Equal(JsonValueKind.Null, nowhere.GetProperty("system").ValueKind);
            Assert.False(string.IsNullOrEmpty(nowhere.GetProperty("reason").GetString()));
        }

        // A subscriber's queue is its own and bounded; a full one drops its oldest and says how many in the next drain.
        [Fact]
        public void Events_are_drained_and_an_overflow_says_what_it_dropped()
        {
            using CrystalSubscription all = _f.Runtime.Subscribe();
            using (CrystalSession s = _f.Open())
            {
                for (int i = 0; i < CrystalSubscriptionCapacity + 10; i++) s.SaveState(Path.Combine(_f.Root, "s.state"));
            }
            JsonElement[] drained = Json(all.Drain()).EnumerateArray().ToArray();
            Assert.Equal("dropped", drained[0].GetProperty("kind").GetString());
            Assert.True(drained[0].GetProperty("count").GetInt32() > 0);
            Assert.Equal("closed", drained[^1].GetProperty("op").GetString());
            Assert.Empty(Json(all.Drain()).EnumerateArray());
            Assert.False(all.Wait(10));
        }

        private const int CrystalSubscriptionCapacity = 256;

        // The provider's refusal of PAUSE ends: run control pauses and resumes the session, and the frame advances nothing while it is held - see EmuSen_Backend.md §3.11.
        [Fact]
        public void Run_control_pauses_and_resumes_the_frame()
        {
            using CrystalSession s = _f.Open();
            Frame(s);
            s.Run(CrystalSessionAbi.RunPause);
            CrystalFrameInfo held = Frame(s);
            Assert.Equal((0u, 1u), (held.ran, held.paused));
            ulong frames = held.total_frames;
            Assert.Equal(frames, Frame(s).total_frames);
            Assert.Equal("paused", Json(s.Info()).GetProperty("run").GetString());
            s.Run(CrystalSessionAbi.RunResume);
            CrystalFrameInfo again = Frame(s);
            Assert.Equal((1u, 0u, frames + 1), (again.ran, again.paused, again.total_frames));
        }

        // A change from another thread wakes the paused frame thread parked in wait - see EmuSen_Backend.md §3.3, §3.12's C1.
        [Fact]
        public async Task A_resume_from_another_thread_wakes_the_parked_frame_thread()
        {
            using CrystalSession s = _f.Open();
            Frame(s);
            s.Run(CrystalSessionAbi.RunPause);
            Frame(s);
            Assert.False(s.Wait(20));
            Task<bool> parked = Task.Run(() => s.Wait(10_000));
            await Task.Delay(50);
            await Task.Run(() => s.Run(CrystalSessionAbi.RunResume));
            Assert.True(await parked);
            Assert.Equal(1u, Frame(s).ran);
        }

        // A breakpoint halts the session inside a frame, and nothing advances until run control resumes it - see EmuSen_Backend.md §3.12's C2.
        [Fact]
        public void A_breakpoint_halts_the_frame_until_run_control_resumes_it()
        {
            using CrystalSession s = _f.Open();
            using CrystalSubscription run = _f.Runtime.Subscribe(s.Id, CrystalAbi.EventRun);
            for (int i = 0; i < 3; i++) Frame(s);
            Assert.True(WithBreakpoint(s, 5) > 0);
            CrystalFrameInfo f = Frame(s);
            for (int i = 0; i < 10 && f.halted == 0; i++) f = Frame(s);
            Assert.Equal((0u, 1u, 5ul), (f.ran, f.halted, f.halted_address));
            ulong at = f.total_frames;
            Assert.Equal(at, Frame(s).total_frames);
            Assert.Equal("halted", Json(s.Info()).GetProperty("run").GetString());
            Assert.Contains(Json(run.Drain()).EnumerateArray(), e => e.GetProperty("op").GetString() == "halted");
            s.Run(CrystalSessionAbi.RunResume);
            Assert.Equal(1u, Frame(s).ran);
        }

        private static unsafe long WithBreakpoint(CrystalSession s, uint address) =>
            CrystalRuntime.WithText($"{{\"address\":{address}}}", (p, l) => CrystalRuntime.Check(EmuSen.Crystal.Client.Contract.Crystal.BpAdd(s.Runtime.Handle, s.Id, 0, (byte*)p, l)));

        // A twin and the session's table are one function: the same bytes, the same refusals, changes made through one seen through the other - see EmuSen_Backend.md §3.8, SR4.
        [Fact]
        public unsafe void A_twin_and_the_table_answer_alike()
        {
            using CrystalSession s = _f.Open();
            Frame(s);
            var table = (EmuSen.Crystal.Session.EmuSenSessionV1*)s.ServiceTable();
            byte[] FromTable(Func<nint, nuint, long> call) => CrystalRuntime.Bytes((o, n) => call((nint)o, n));
            byte[] viaTable = FromTable((o, n) => table->info(table->ctx, s.Id, (byte*)o, n));
            Assert.Equal(Encoding.UTF8.GetBytes(s.ServiceInfo()), viaTable);
            Assert.Equal(s.Id, Json(s.ServiceInfo()).GetProperty("id").GetUInt32());
            long id = s.WatchAdd("RAM", 3, 1, CrystalSessionAbi.WatchWrite);
            byte[] watches = FromTable((o, n) => table->watch_list(table->ctx, 0, (byte*)o, n));
            Assert.Contains(Json(Encoding.UTF8.GetString(watches)).EnumerateArray(), w => w.GetProperty("id").GetInt64() == id);
            Assert.Equal(CrystalSessionAbi.NoSuchSpace, Assert.Throws<CrystalException>(() => s.WatchAdd("NOPE", 0, 1, 0)).Status);
            Assert.Contains("NOPE", Assert.Throws<CrystalException>(() => s.WatchAdd("NOPE", 0, 1, 0)).Message);
            var readOnly = (EmuSen.Crystal.Session.EmuSenSessionV1*)s.ServiceTable(readOnly: true);
            byte[] space = Encoding.UTF8.GetBytes("RAM");
            fixed (byte* sp = space)
                Assert.Equal(CrystalSessionAbi.ReadOnly, readOnly->watch_add(readOnly->ctx, 0, sp, (nuint)space.Length, 0, 1, 0));
            Assert.Equal(CrystalSessionAbi.ReadOnly, readOnly->run_control(readOnly->ctx, 0, CrystalSessionAbi.RunPause, 0, 0));
            Assert.Equal(0, table->run_control(table->ctx, 0, CrystalSessionAbi.RunPause, 0, 0));
            Assert.Equal(1u, Frame(s).paused);
        }

        // A shell handed the session's table reaches that session: a watch its command adds is the session's, and its state load runs the one sequence - see EmuSen_Backend.md §14.8.
        [Fact]
        public void A_shell_handed_the_session_s_table_reaches_the_session()
        {
            using CrystalSession s = _f.Open();
            Frame(s);
            var shell = EmuSen.DianaSh.DianaSh.Bin.DianaShInterpreter.CreateForService(s.ServiceTable());
            Assert.Contains("RAM", shell.Execute("spaces"));
            shell.Execute("watch add RAM 3 1");
            Assert.Single(Json(s.WatchList()).EnumerateArray());
            Assert.Throws<ArgumentNullException>(() => EmuSen.DianaSh.DianaSh.Bin.DianaShInterpreter.CreateForService(0));
        }

        // A struct shorter than 1.0's is refused; a longer one has its tail left as the client wrote it - EmuSen_CoreAPI.md §4.2's rule, one layer up.
        [Fact]
        public unsafe void A_frame_struct_of_another_size_is_refused_or_its_tail_kept()
        {
            using CrystalSession s = _f.Open();
            byte* buffer = stackalloc byte[160];
            new Span<byte>(buffer, 160).Fill(0xA5);
            *(uint*)buffer = 160;
            Assert.Equal(0, EmuSen.Crystal.Client.Contract.Crystal.Frame(s.Runtime.Handle, s.Id, (CrystalFrameInfo*)buffer));
            Assert.Equal(1u, ((CrystalFrameInfo*)buffer)->ran);
            Assert.Equal(160u, *(uint*)buffer);
            Assert.True(new Span<byte>(buffer + sizeof(CrystalFrameInfo), 160 - sizeof(CrystalFrameInfo)).ToArray().All(b => b == 0xA5));
            *(uint*)buffer = 96;
            Assert.Equal(CrystalAbi.BadStruct, EmuSen.Crystal.Client.Contract.Crystal.Frame(s.Runtime.Handle, s.Id, (CrystalFrameInfo*)buffer));
        }

        // A second thread's frame while one is inside a frame is refused, never run beside it.
        [Fact]
        public async Task Frames_from_two_threads_at_once_are_refused_not_run_together()
        {
            using CrystalSession s = _f.Open();
            int busy = 0, ran = 0;
            using var stop = new CancellationTokenSource(TimeSpan.FromSeconds(2));
            async Task Run() => await Task.Run(() =>
            {
                while (!stop.IsCancellationRequested && Volatile.Read(ref busy) == 0)
                {
                    try
                    {
                        Frame(s);
                        Interlocked.Increment(ref ran);
                    }
                    catch (CrystalException e) when (e.Status == CrystalAbi.Busy) { Interlocked.Increment(ref busy); }
                }
            });
            await Task.WhenAll(Run(), Run());
            Assert.True(busy > 0, $"{ran} frames ran and none was refused");
        }
    }
}
