using EmuSen.Hotaru;

namespace EmuSen.WiseMan.Hotaru
{
    // Which shell a launch gets - see EmuSen_Frontend_Driver.md §3c.
    public class LaunchModeTests
    {
        private static readonly string[] NoArgs = System.Array.Empty<string>();

        [Fact]
        public void A_desktop_launch_with_no_terminal_gets_the_window()
        {
            Assert.Equal(ShellMode.Window,
                LaunchMode.Decide(NoArgs, interactiveTerminal: false, displayAvailable: true));
        }

        // The case the whole feature exists for must not also change how a terminal launch behaves.
        [Fact]
        public void A_terminal_launch_on_a_desktop_still_gets_the_console()
        {
            Assert.Equal(ShellMode.Console,
                LaunchMode.Decide(NoArgs, interactiveTerminal: true, displayAvailable: true));
        }

        [Fact]
        public void A_server_gets_the_console_whether_or_not_a_terminal_is_attached()
        {
            Assert.Equal(ShellMode.Console,
                LaunchMode.Decide(NoArgs, interactiveTerminal: true, displayAvailable: false));
            Assert.Equal(ShellMode.Console,
                LaunchMode.Decide(NoArgs, interactiveTerminal: false, displayAvailable: false));
        }

        [Fact]
        public void The_console_flag_wins_over_a_windowed_launch()
        {
            Assert.Equal(ShellMode.Console,
                LaunchMode.Decide(new[] { "--console" }, interactiveTerminal: false, displayAvailable: true));
        }

        [Fact]
        public void The_window_flag_wins_over_a_terminal_launch()
        {
            Assert.Equal(ShellMode.Window,
                LaunchMode.Decide(new[] { "--shell-window" }, interactiveTerminal: true, displayAvailable: true));
        }

        // The flag is the escape hatch for a wrong heuristic, so it outranks the heuristic - see §3c.
        [Fact]
        public void The_window_flag_outranks_display_detection()
        {
            Assert.Equal(ShellMode.Window,
                LaunchMode.Decide(new[] { "--shell-window" }, interactiveTerminal: true, displayAvailable: false));
        }

        [Fact]
        public void A_flag_is_never_mistaken_for_the_rom_path()
        {
            Assert.Equal("/roms/game.smc", LaunchMode.RomPathFrom(new[] { "--console", "/roms/game.smc" }));
            Assert.Equal("/roms/game.smc", LaunchMode.RomPathFrom(new[] { "/roms/game.smc", "--shell-window" }));
            Assert.Null(LaunchMode.RomPathFrom(new[] { "--console" }));
            Assert.Null(LaunchMode.RomPathFrom(NoArgs));
        }
    }
}
