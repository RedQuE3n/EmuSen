using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // The two tools added for the Super Mario World title-screen
    // investigation: `memfind` (a byte-sequence scan, where `search` only
    // ever matched one scalar) and `bp write` (halt on a write, so the
    // state that produced it can be read) - see
    // EmuSen_Debugging_Tools_Reference_v5.md §3.25 and §3.26.
    public class MemFindAndDataBreakpointTests
    {
        private static SnesDebugTarget BuildTarget()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        private static void Poke(SnesDebugTarget target, int address, params byte[] bytes)
        {
            var wram = target.GetMemorySpaces().First(s => s.Name == "WRAM");
            for (int i = 0; i < bytes.Length; i++) wram.Write(address + i, bytes[i]);
        }

        [Fact]
        public void Memfind_reports_the_offset_of_a_byte_sequence()
        {
            var target = BuildTarget();
            Poke(target, 0x1000, 0xDE, 0xAD, 0xBE, 0xEF);

            var result = DianaOSInterpreter.CreateDefault(target).Submit("memfind WRAM DEADBEEF");

            Assert.Contains("0x1000", result.Output);
            Assert.Contains("1 match", result.Output);
        }

        [Fact]
        public void Memfind_accepts_separators_and_spaces_in_the_pattern()
        {
            var target = BuildTarget();
            Poke(target, 0x2000, 0xDE, 0xAD, 0xBE, 0xEF);
            var shell = DianaOSInterpreter.CreateDefault(target);

            Assert.Contains("0x2000", shell.Submit("memfind WRAM DE AD BE EF").Output);
            Assert.Contains("0x2000", shell.Submit("memfind WRAM DE:AD:BE:EF").Output);
        }

        [Fact]
        public void Memfind_treats_double_question_mark_as_a_wildcard()
        {
            var target = BuildTarget();
            Poke(target, 0x3000, 0x11, 0x5A, 0x33);

            var result = DianaOSInterpreter.CreateDefault(target).Submit("memfind WRAM 11??33");

            Assert.Contains("0x3000", result.Output);
        }

        [Fact]
        public void Memfind_says_so_when_nothing_matches()
        {
            var target = BuildTarget();

            var result = DianaOSInterpreter.CreateDefault(target).Submit("memfind WRAM DEADBEEFCAFEBABE");

            Assert.Contains("No match", result.Output);
        }

        [Fact]
        public void Memfind_rejects_a_half_byte_pattern()
        {
            var target = BuildTarget();

            var result = DianaOSInterpreter.CreateDefault(target).Submit("memfind WRAM ABC");

            Assert.Contains("whole hex byte pairs", result.Output);
        }

        // The trailing count is a hit limit, not part of the pattern - so a
        // pattern of "11" with a limit of "2" must not parse as "1122".
        [Fact]
        public void Memfind_stops_at_the_hit_limit_and_says_it_truncated()
        {
            var target = BuildTarget();
            Poke(target, 0x4000, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77);

            var result = DianaOSInterpreter.CreateDefault(target).Submit("memfind WRAM 77 2");

            Assert.Contains("More than 2", result.Output);
            Assert.Contains("0x4000", result.Output);
        }

        [Fact]
        public void Memfind_refuses_a_live_hardware_space()
        {
            var target = BuildTarget();

            var result = DianaOSInterpreter.CreateDefault(target).Submit("memfind CpuBus 00");

            Assert.Contains("refusing", result.Output);
        }

        [Fact]
        public void Data_breakpoint_fires_on_a_matching_write()
        {
            var registry = new BreakpointRegistry();
            registry.AddDataBreakpoint("VRAM", 0x2760);

            registry.NoteWrite("VRAM", 0x2760, 0x11);

            Assert.True(registry.ShouldBreak(0x008000));
        }

        [Fact]
        public void Data_breakpoint_ignores_another_address_or_space()
        {
            var registry = new BreakpointRegistry();
            registry.AddDataBreakpoint("VRAM", 0x2760);

            registry.NoteWrite("VRAM", 0x2761, 0x11);
            registry.NoteWrite("WRAM", 0x2760, 0x11);

            Assert.False(registry.ShouldBreak(0x008000));
        }

        [Fact]
        public void Data_breakpoint_with_a_value_only_fires_on_that_value()
        {
            var registry = new BreakpointRegistry();
            registry.AddDataBreakpoint("VRAM", 0x2760, 0x11);

            registry.NoteWrite("VRAM", 0x2760, 0x22);
            Assert.False(registry.ShouldBreak(0x008000));

            registry.NoteWrite("VRAM", 0x2760, 0x11);
            Assert.True(registry.ShouldBreak(0x008000));
        }

        // It arms once and is consumed once - a write must not leave the
        // core halting on every instruction that follows it.
        [Fact]
        public void Data_breakpoint_fires_only_once_per_write()
        {
            var registry = new BreakpointRegistry();
            registry.AddDataBreakpoint("WRAM", 0x40);

            registry.NoteWrite("WRAM", 0x40, 0x01);

            Assert.True(registry.ShouldBreak(0x008000));
            Assert.False(registry.ShouldBreak(0x008001));
        }

        [Fact]
        public void Bp_list_shows_data_breakpoints_alongside_address_ones()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("bp add 008000");
            shell.Submit("bp write WRAM 40 7F");

            string listing = shell.Submit("bp list").Output;

            Assert.Contains("$008000", listing);
            Assert.Contains("write WRAM 0x40 = 0x7F", listing);
        }

        [Fact]
        public void Bp_remove_removes_a_data_breakpoint_too()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            string added = shell.Submit("bp write WRAM 40").Output;
            Assert.Contains("#1", added);

            Assert.Contains("removed", shell.Submit("bp remove 1").Output);
            Assert.Contains("No active breakpoints", shell.Submit("bp list").Output);
        }

        // The whole point of the tool: a real write on the real bus arms it.
        [Fact]
        public void A_real_bus_write_arms_the_data_breakpoint()
        {
            var target = BuildTarget();
            target.Breakpoints.AddDataBreakpoint("WRAM", 0x1234, 0x5A);

            target.OnWrite("WRAM", 0x1234, 0x5A);

            Assert.True(target.Breakpoints.ShouldBreak(0x008000));
        }
    }
}
