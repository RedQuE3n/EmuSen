using EmuSen.DianaOS;

namespace EmuSen.WiseMan.DianaOS
{
    // DianaOSInterpreter.TryGetReadOnlyFastPath - the pure, parse-only
    // check a live (non-halted) console (see EmuSen.Hotaru's GameWindow
    // console reader thread) uses to decide whether a typed line can run
    // immediately off the emulation thread, or must be queued for its
    // next tick. Never executes anything - these assertions would still
    // hold even against a null IDebugTarget.
    public class FastPathClassificationTests
    {
        private static DianaOSInterpreter NewShell() => DianaOSInterpreter.CreateDefault(null);

        [Theory]
        [InlineData("mem read WRAM 0")]
        [InlineData("  regs  ")]
        [InlineData("echo hi")]
        public void Single_read_only_commands_classify_as_fast_path(string line)
        {
            Assert.True(NewShell().TryGetReadOnlyFastPath(line, out _));
        }

        [Theory]
        [InlineData("write WRAM 0 1")]
        [InlineData("watch add WRAM 0")]
        [InlineData("watch list")] // mixed-verb command: conservatively false even for its read sub-verb
        [InlineData("quit")] // aliases to shutdown, mutating
        [InlineData("c")] // aliases to resume, mutating
        [InlineData("s")] // aliases to step, mutating
        [InlineData("unknowncommand")]
        [InlineData("help")] // handled directly in Dispatch, not a registry entry
        [InlineData("source foo.dosh")]
        public void Mutating_or_unresolvable_commands_do_not_classify_as_fast_path(string line)
        {
            Assert.False(NewShell().TryGetReadOnlyFastPath(line, out _));
        }

        [Theory]
        [InlineData("mem read WRAM 0 | grep 1")]
        [InlineData("regs && step")]
        [InlineData("regs; write WRAM 0 1")]
        [InlineData("if true; then mem; fi")]
        [InlineData("for i in 1 2 3; do echo $i; done")]
        public void Compound_or_control_flow_lines_do_not_classify_as_fast_path(string line)
        {
            Assert.False(NewShell().TryGetReadOnlyFastPath(line, out _));
        }

        [Fact]
        public void Unterminated_construct_does_not_classify_as_fast_path()
        {
            Assert.False(NewShell().TryGetReadOnlyFastPath("for i in 1 2 3", out _));
        }

        [Fact]
        public void History_reference_does_not_classify_as_fast_path()
        {
            Assert.False(NewShell().TryGetReadOnlyFastPath("!!", out _));
        }

        [Fact]
        public void Blank_line_does_not_classify_as_fast_path()
        {
            Assert.False(NewShell().TryGetReadOnlyFastPath("   ", out _));
        }
    }
}
