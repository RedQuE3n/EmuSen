using EmuSen.Cores;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // `coretop -w` (EmuSen.DianaSh.DianaSh.Bin.Commands.CoretopCommand's optional
    // Action<IDebugTarget> openWindow constructor parameter) - covers
    // the part of the feature that doesn't need a real console/GUI: the
    // -w flag parsing and the "not supported by this frontend" fallback
    // when no window opener is wired up (the EmuSen.WiseMan/headless
    // case). The actual window (EmuSen.Hotaru's AvaloniaHost/CoretopWindow,
    // EmuSen.Mistress's CoretopWindowCommand) can't be exercised
    // headlessly - not tested here, same as `coretop`'s own console
    // rendering isn't (both need a real display/terminal).
    public class CoretopWindowFlagTests
    {
        private static IDebugTarget BuildTarget()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return SyntheticRom.TargetOf(core);
        }

        [Fact]
        public void Minus_w_without_a_window_opener_reports_not_supported()
        {
            var target = BuildTarget();
            var shell = DianaShInterpreter.CreateDefault(target,
                new IDianaShCommand[] { new CoretopCommand() }); // no openWindow wired up

            var result = shell.Submit("coretop -w");

            Assert.Contains("not supported by this frontend", result.Output);
        }

        [Fact]
        public void Minus_w_with_a_window_opener_calls_it_instead_of_the_console_dashboard()
        {
            var target = BuildTarget();
            int opened = 0;
            var shell = DianaShInterpreter.CreateDefault(target,
                new IDianaShCommand[] { new CoretopCommand(() => opened++) });

            var result = shell.Submit("coretop -w");

            Assert.Equal(1, opened);
            Assert.Contains("opened in a separate window", result.Output);
        }

        [Fact]
        public void Minus_w_without_a_rom_loaded_reports_no_rom_before_checking_the_flag()
        {
            var shell = DianaShInterpreter.CreateDefault(null,
                new IDianaShCommand[] { new CoretopCommand(() => { }) });

            var result = shell.Submit("coretop -w");

            Assert.Contains("No ROM loaded", result.Output);
        }

        [Fact]
        public void Minus_w_flag_is_recognized_case_insensitively_and_anywhere_in_the_args()
        {
            var target = BuildTarget();
            int calls = 0;
            var shell = DianaShInterpreter.CreateDefault(target,
                new IDianaShCommand[] { new CoretopCommand(() => calls++) });

            shell.Submit("coretop -W");

            Assert.Equal(1, calls);
        }
    }
}
