using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.DianaSh.DianaSh.Lib;

namespace EmuSen.WiseMan.DianaSh
{
    // Each command's read-only flag as the shell lists it: commands with a read and a write sub-verb list false, since the flag cannot vary per line - see EmuSen_DianaSh.md §6.5.
    public class IsReadOnlyClassificationTests
    {
        private static Dictionary<string, bool> Listed()
        {
            DianaShInterpreter shell = DianaShInterpreter.CreateWithoutSession();
            return shell.NativeShellOf.Commands(shell.NativeSessionId).ToDictionary(c => c.Name, c => c.ReadOnly);
        }

        [Theory]
        [InlineData("mem", "regs", "sprites", "pal", "channels", "spaces", "disasm", "tile", "tilemap", "callers", "writers", "readers", "dump", "clear", "echo", "test", "[")]
        public void Plain_read_commands_report_read_only(params string[] names)
        {
            var listed = Listed();
            Assert.All(names, n => Assert.True(listed[n], n));
        }

        [Fact]
        public void Diff_is_read_only_but_its_sibling_snapshot_command_is_not()
        {
            var listed = Listed();
            Assert.True(listed["diff"]);
            Assert.False(listed["snapshot"]);
        }

        [Theory]
        [InlineData("write", "load", "mute", "cd", "mv", "nano", "step", "resume", "shutdown")]
        public void Plain_mutating_commands_report_not_read_only(params string[] names)
        {
            var listed = Listed();
            Assert.All(names, n => Assert.False(listed[n], n));
            Assert.False(new CoreCommand(new Dictionary<string, CoreChoice>()).IsReadOnly);
            Assert.False(new StateCommand(_ => { }, _ => { }, () => "").IsReadOnly);
        }

        [Theory]
        [InlineData("watch", "bp", "framelog", "cheat", "log", "trace", "search", "history")]
        public void Mixed_readwrite_subverb_commands_conservatively_report_not_read_only(params string[] names)
        {
            var listed = Listed();
            Assert.All(names, n => Assert.False(listed[n], n));
        }

        // A live coretop blocks its thread until Ctrl+C, so it is queued like nano, never run on the fast path.
        [Fact]
        public void Coretop_is_not_fast_path_eligible_despite_never_writing_core_state()
        {
            Assert.False(Listed()["coretop"]);
            Assert.False(new CoretopCommand(() => true).IsReadOnly);
        }
    }
}
