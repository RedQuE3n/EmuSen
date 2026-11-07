using System;
using EmuSen.Crystal.Client;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // `coretop` as a front end adds it, over a session's table from the frontend API: the window opened over the session, the flag read in any case, refused in words without a window or a game - see `man coretop` and EmuSen_Backend.md §40.10.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoretopWindowFlagTests : IDisposable
    {
        private readonly CrystalFixture _f = new();

        public void Dispose() => _f.Dispose();

        private DianaShInterpreter Over(CrystalSession s, IDianaShCommand coretop) => DianaShInterpreter.CreateForService(s.ServiceTable(), new[] { coretop });

        [Fact]
        public void Minus_w_with_a_window_opener_over_a_session_opens_it()
        {
            using CrystalSession s = _f.Open();
            int opened = 0;
            var result = Over(s, new CoretopCommand(() => opened++, () => true)).Submit("coretop -w");
            Assert.Equal(1, opened);
            Assert.Contains("opened in a separate window", result.Output);
        }

        [Fact]
        public void Minus_w_without_a_window_opener_reports_not_supported()
        {
            using CrystalSession s = _f.Open();
            Assert.Contains("not supported by this frontend", Over(s, new CoretopCommand(() => true)).Submit("coretop -w").Output);
        }

        [Fact]
        public void Without_a_game_it_reports_no_rom_and_opens_nothing()
        {
            int opened = 0;
            var shell = DianaShInterpreter.CreateWithoutSession(extraCommands: new IDianaShCommand[] { new CoretopCommand(() => opened++, () => false) });
            Assert.Contains("No ROM loaded", shell.Submit("coretop -w").Output);
            Assert.Equal(0, opened);
        }

        [Fact]
        public void Minus_w_flag_is_recognized_case_insensitively_and_anywhere_in_the_args()
        {
            using CrystalSession s = _f.Open();
            int calls = 0;
            Over(s, new CoretopCommand(() => calls++, () => true)).Submit("coretop -W");
            Assert.Equal(1, calls);
        }
    }
}
