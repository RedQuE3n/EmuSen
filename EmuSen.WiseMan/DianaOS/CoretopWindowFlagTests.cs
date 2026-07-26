using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.Commands;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // `coretop -w` (EmuSen.DianaOS.Commands.CoretopCommand's optional
    // Action<IDebugTarget> openWindow constructor parameter) - covers
    // the part of the feature that doesn't need a real console/GUI: the
    // -w flag parsing and the "not supported by this frontend" fallback
    // when no window opener is wired up (the EmuSen.WiseMan/headless
    // case). The actual window (EmuSen.Hotaru's AvaloniaHost/CoretopWindow,
    // EmuSen.Mistress9's CoretopWindowCommand) can't be exercised
    // headlessly - not tested here, same as `coretop`'s own console
    // rendering isn't (both need a real display/terminal).
    public class CoretopWindowFlagTests
    {
        private static SnesDebugTarget BuildTarget()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        [Fact]
        public void Minus_w_without_a_window_opener_reports_not_supported()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target,
                new IDianaOSCommand[] { new CoretopCommand() }); // no openWindow wired up

            var result = shell.Submit("coretop -w");

            Assert.Contains("not supported by this frontend", result.Output);
        }

        [Fact]
        public void Minus_w_with_a_window_opener_calls_it_instead_of_the_console_dashboard()
        {
            var target = BuildTarget();
            IDebugTarget? received = null;
            var shell = DianaOSInterpreter.CreateDefault(target,
                new IDianaOSCommand[] { new CoretopCommand(t => received = t) });

            var result = shell.Submit("coretop -w");

            Assert.Same(target, received);
            Assert.Contains("opened in a separate window", result.Output);
        }

        [Fact]
        public void Minus_w_without_a_rom_loaded_reports_no_rom_before_checking_the_flag()
        {
            var shell = DianaOSInterpreter.CreateDefault(null,
                new IDianaOSCommand[] { new CoretopCommand(_ => { }) });

            var result = shell.Submit("coretop -w");

            Assert.Contains("No ROM loaded", result.Output);
        }

        [Fact]
        public void Minus_w_flag_is_recognized_case_insensitively_and_anywhere_in_the_args()
        {
            var target = BuildTarget();
            int calls = 0;
            var shell = DianaOSInterpreter.CreateDefault(target,
                new IDianaOSCommand[] { new CoretopCommand(_ => calls++) });

            shell.Submit("coretop -W");

            Assert.Equal(1, calls);
        }
    }
}
