using System;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // A shell over a session's table, as emusen-shell's game window makes one: resume and step are asked of the session's run control and come back from its run events, on the Rust shell and on the C# - see EmuSen_Backend.md §14.11.
    [Collection(TestCollections.ProcessGlobals)]
    public class ShellOverSessionTests : IDisposable
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

        private static unsafe CrystalFrameInfo Halted(CrystalSession s)
        {
            Frame(s);
            CrystalRuntime.WithText("{\"address\":5}", (p, l) => CrystalRuntime.Check(EmuSen.Crystal.Client.Contract.Crystal.BpAdd(s.Runtime.Handle, s.Id, 0, (byte*)p, l)));
            CrystalFrameInfo f = Frame(s);
            for (int i = 0; i < 10 && f.halted == 0; i++) f = Frame(s);
            Assert.Equal(1u, f.halted);
            return f;
        }

        [Fact]
        public void Resume_typed_runs_the_halted_session_and_says_so()
        {
            using CrystalSession s = _f.Open();
            Halted(s);
            using CrystalSubscription run = _f.Runtime.Subscribe(s.Id, CrystalAbi.EventRun);
            var shell = DianaShInterpreter.CreateForService(s.ServiceTable());
            (_, _, HostAction? action) = shell.Submit("resume");
            Assert.IsType<HostAction.Resume>(action);
            Assert.Equal("running", Json(s.Info()).GetProperty("run").GetString());
            Assert.Contains(Json(run.Drain()).EnumerateArray(), e => e.GetProperty("op").GetString() == "resumed");
        }

        [Fact]
        public void Step_typed_runs_the_halted_session_one_step()
        {
            using CrystalSession s = _f.Open();
            ulong at = Halted(s).total_frames;
            var shell = DianaShInterpreter.CreateForService(s.ServiceTable());
            (_, _, HostAction? action) = shell.Submit("step");
            Assert.IsType<HostAction.Step>(action);
            CrystalFrameInfo after = Frame(s);
            bool stayedPut = after.ran == 0 && after.halted == 1 && after.halted_address == 5 && after.total_frames == at;
            Assert.False(stayedPut, "the session did not run after the step");
        }

        // A run the frontend asked of the session itself, outside any line, is not the next line's to report.
        [Fact]
        public void A_run_the_frontend_asked_itself_is_not_a_line_s()
        {
            using CrystalSession s = _f.Open();
            Halted(s);
            var shell = DianaShInterpreter.CreateForService(s.ServiceTable());
            s.Run(CrystalSessionAbi.RunResume);
            (_, _, HostAction? action) = shell.Submit("spaces");
            Assert.Null(action);
        }

        // `tmux new` over a session's table makes a shell over the same table.
        [Fact]
        public void A_tmux_session_is_over_the_same_table()
        {
            using CrystalSession s = _f.Open();
            Frame(s);
            var sessions = new DianaShSessionManager();
            var shell = DianaShInterpreter.CreateForService(s.ServiceTable(), null, sessions);
            sessions.RegisterInitial(new DianaShSession("main", shell));
            shell.Execute("tmux new second");
            DianaShInterpreter second = sessions.Current!.Interpreter;
            Assert.NotSame(shell, second);
            second.Execute("watch add RAM 3 1");
            Assert.Single(Json(s.WatchList()).EnumerateArray());
        }
    }
}
