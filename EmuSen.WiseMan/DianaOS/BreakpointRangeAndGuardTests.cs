using System.Linq;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.WiseMan.DianaOS
{
    // Ranges, read breakpoints, forbid ranges, uninit reads and the depth guard - see `man bp`.
    public class BreakpointRangeAndGuardTests
    {
        [Fact]
        public void A_range_breakpoint_fires_anywhere_inside_it()
        {
            var bp = new BreakpointRegistry();
            bp.AddBreakpoint(0x818000, 0x818FFF, null);

            Assert.False(bp.ShouldBreak(0x817FFF));
            Assert.True(bp.ShouldBreak(0x818000));
            Assert.True(bp.ShouldBreak(0x818ABC));
            Assert.True(bp.ShouldBreak(0x818FFF));
            Assert.False(bp.ShouldBreak(0x819000));
        }

        [Fact]
        public void A_range_break_names_the_address_that_matched_and_the_range()
        {
            var bp = new BreakpointRegistry();
            bp.AddBreakpoint(0x818000, 0x818FFF, null);

            bp.ShouldBreak(0x818ABC);

            Assert.Contains("$818ABC", bp.LastBreakReason);
            Assert.Contains("$818000-$818FFF", bp.LastBreakReason);
        }

        [Fact]
        public void A_reversed_range_is_accepted_in_either_order()
        {
            var bp = new BreakpointRegistry();
            bp.AddBreakpoint(0x818FFF, 0x818000, null);

            Assert.True(bp.ShouldBreak(0x818500));
        }

        [Fact]
        public void A_single_address_breakpoint_still_matches_only_itself()
        {
            var bp = new BreakpointRegistry();
            bp.AddBreakpoint(0x808000);

            Assert.True(bp.ShouldBreak(0x808000));
            Assert.False(bp.ShouldBreak(0x808001));
        }

        [Fact]
        public void A_write_range_catches_any_byte_in_the_struct()
        {
            var bp = new BreakpointRegistry();
            bp.AddDataBreakpoint("WRAM", 0x0100, 0x01FF, value: -1, onRead: false, changedOnly: false, condition: null);

            bp.NoteWrite("WRAM", 0x0180, 0x42);

            Assert.True(bp.ShouldBreak(0x808000));
            Assert.Contains("0x180", bp.LastBreakReason);
        }

        [Fact]
        public void A_read_breakpoint_ignores_writes_and_a_write_breakpoint_ignores_reads()
        {
            var reads = new BreakpointRegistry();
            reads.AddDataBreakpoint("WRAM", 0x13C6, 0x13C6, -1, onRead: true, changedOnly: false, condition: null);
            reads.NoteWrite("WRAM", 0x13C6, 0x01);
            Assert.False(reads.ShouldBreak(0x808000));

            reads.NoteRead("WRAM", 0x13C6, 0x01);
            Assert.True(reads.ShouldBreak(0x808000));

            var writes = new BreakpointRegistry();
            writes.AddDataBreakpoint("WRAM", 0x13C6, 0x13C6, -1, onRead: false, changedOnly: false, condition: null);
            writes.NoteRead("WRAM", 0x13C6, 0x01);
            Assert.False(writes.ShouldBreak(0x808000));
        }

        [Fact]
        public void A_read_break_reason_says_read()
        {
            var bp = new BreakpointRegistry();
            bp.AddDataBreakpoint("WRAM", 0x13C6, 0x13C6, -1, onRead: true, changedOnly: false, condition: null);

            bp.NoteRead("WRAM", 0x13C6, 0x07);
            bp.ShouldBreak(0x808000);

            Assert.Contains("read", bp.LastBreakReason);
        }

        [Fact]
        public void Changed_only_ignores_a_rewrite_of_the_same_value()
        {
            var bp = new BreakpointRegistry();
            bp.AddDataBreakpoint("WRAM", 0x0DB3, 0x0DB3, -1, onRead: false, changedOnly: true, condition: null);

            // The first write has nothing to compare against, so it counts.
            bp.NoteWrite("WRAM", 0x0DB3, 0x01);
            Assert.True(bp.ShouldBreak(0x808000));

            bp.NoteWrite("WRAM", 0x0DB3, 0x01);
            bp.NoteWrite("WRAM", 0x0DB3, 0x01);
            Assert.False(bp.ShouldBreak(0x808000));

            bp.NoteWrite("WRAM", 0x0DB3, 0x02);
            Assert.True(bp.ShouldBreak(0x808000));
        }

        [Fact]
        public void Changed_only_tracks_each_address_of_a_range_separately()
        {
            var bp = new BreakpointRegistry();
            bp.AddDataBreakpoint("WRAM", 0x0100, 0x01FF, -1, onRead: false, changedOnly: true, condition: null);

            bp.NoteWrite("WRAM", 0x0100, 0xAA);
            Assert.True(bp.ShouldBreak(0x808000));

            // A different address, so its own first write is still a change.
            bp.NoteWrite("WRAM", 0x0101, 0xAA);
            Assert.True(bp.ShouldBreak(0x808000));

            bp.NoteWrite("WRAM", 0x0100, 0xAA);
            Assert.False(bp.ShouldBreak(0x808000));
        }

        [Fact]
        public void A_plain_write_breakpoint_still_fires_on_an_unchanged_rewrite()
        {
            var bp = new BreakpointRegistry();
            bp.AddDataBreakpoint("WRAM", 0x0DB3, 0x0DB3, -1, onRead: false, changedOnly: false, condition: null);

            bp.NoteWrite("WRAM", 0x0DB3, 0x01);
            Assert.True(bp.ShouldBreak(0x808000));
            bp.NoteWrite("WRAM", 0x0DB3, 0x01);
            Assert.True(bp.ShouldBreak(0x808000));
        }

        [Fact]
        public void An_uninitialized_read_halts_but_a_written_byte_does_not()
        {
            var bp = new BreakpointRegistry();
            bp.ArmUninitializedReadBreak("WRAM", 0x2000);

            bp.NoteWrite("WRAM", 0x0010, 0x55);
            bp.NoteRead("WRAM", 0x0010, 0x55);
            Assert.False(bp.ShouldBreak(0x808000));

            bp.NoteRead("WRAM", 0x0020, 0xFF);
            Assert.True(bp.ShouldBreak(0x808000));
            Assert.Contains("uninitialized", bp.LastBreakReason);
        }

        [Fact]
        public void An_uninitialized_read_reports_each_address_only_once()
        {
            var bp = new BreakpointRegistry();
            bp.ArmUninitializedReadBreak("WRAM", 0x2000);

            bp.NoteRead("WRAM", 0x0020, 0xFF);
            Assert.True(bp.ShouldBreak(0x808000));

            bp.NoteRead("WRAM", 0x0020, 0xFF);
            Assert.False(bp.ShouldBreak(0x808000));
        }

        [Fact]
        public void Uninitialized_reads_only_track_the_armed_space()
        {
            var bp = new BreakpointRegistry();
            bp.ArmUninitializedReadBreak("WRAM", 0x2000);

            bp.NoteRead("VRAM", 0x0020, 0xFF);

            Assert.False(bp.ShouldBreak(0x808000));
        }

        [Fact]
        public void Disarming_uninitialized_reads_stops_them_halting()
        {
            var bp = new BreakpointRegistry();
            bp.ArmUninitializedReadBreak("WRAM", 0x2000);
            bp.DisarmUninitializedReadBreak();

            bp.NoteRead("WRAM", 0x0020, 0xFF);

            Assert.False(bp.IsUninitializedReadBreakArmed);
            Assert.False(bp.ShouldBreak(0x808000));
        }

        [Fact]
        public void A_forbid_range_suppresses_a_breakpoint_inside_it()
        {
            var bp = new BreakpointRegistry();
            bp.AddBreakpoint(0x0080C8);
            bp.AddForbidRange(0x0080C4, 0x0080FF);

            Assert.False(bp.ShouldBreak(0x0080C8));
        }

        [Fact]
        public void A_forbid_range_suppresses_a_pending_data_break_rather_than_deferring_it()
        {
            var bp = new BreakpointRegistry();
            bp.AddDataBreakpoint("WRAM", 0x0100, 0x0100, -1, onRead: false, changedOnly: false, condition: null);
            bp.AddForbidRange(0x0080C4, 0x0080FF);

            bp.NoteWrite("WRAM", 0x0100, 0x42);
            Assert.False(bp.ShouldBreak(0x0080C8));

            // Dropped, not queued - or it would fire on the next instruction outside the range.
            Assert.False(bp.ShouldBreak(0x808000));
        }

        [Fact]
        public void A_forbid_range_does_not_block_a_step()
        {
            var bp = new BreakpointRegistry();
            bp.AddForbidRange(0x0080C4, 0x0080FF);
            bp.ArmSingleStep();

            Assert.True(bp.ShouldBreak(0x0080C8));
        }

        [Fact]
        public void A_disabled_forbid_range_stops_suppressing()
        {
            var bp = new BreakpointRegistry();
            bp.AddBreakpoint(0x0080C8);
            int forbid = bp.AddForbidRange(0x0080C4, 0x0080FF);

            Assert.False(bp.ShouldBreak(0x0080C8));
            bp.SetEnabled(forbid, false);
            Assert.True(bp.ShouldBreak(0x0080C8));
        }

        [Fact]
        public void A_forbid_range_is_listed_and_removable()
        {
            var bp = new BreakpointRegistry();
            int id = bp.AddForbidRange(0x0080C4, 0x0080FF);

            Assert.Single(bp.GetForbidRanges());
            Assert.True(bp.RemoveBreakpoint(id));
            Assert.Empty(bp.GetForbidRanges());
        }

        [Fact]
        public void The_depth_guard_needs_a_call_stack()
        {
            var bp = new BreakpointRegistry();

            Assert.False(bp.ArmDepthGuard(4));
        }

        [Fact]
        public void The_depth_guard_halts_once_the_stack_gets_too_deep()
        {
            var stack = new CallStackRegistry();
            var bp = new BreakpointRegistry { CallStack = stack };
            Assert.True(bp.ArmDepthGuard(2));

            stack.NotePush(0x808000, 0x808100, CallFrameKind.Call);
            stack.NotePush(0x808100, 0x808200, CallFrameKind.Call);
            Assert.False(bp.ShouldBreak(0x808200));

            stack.NotePush(0x808200, 0x808300, CallFrameKind.Call);
            Assert.True(bp.ShouldBreak(0x808300));
            Assert.Contains("depth", bp.LastBreakReason);
        }

        [Fact]
        public void The_depth_guard_disarms_itself_after_firing()
        {
            var stack = new CallStackRegistry();
            var bp = new BreakpointRegistry { CallStack = stack };
            bp.ArmDepthGuard(1);

            stack.NotePush(0x808000, 0x808100, CallFrameKind.Call);
            stack.NotePush(0x808100, 0x808200, CallFrameKind.Call);
            Assert.True(bp.ShouldBreak(0x808200));

            // Still deep, but it must not retrigger on every following instruction.
            Assert.False(bp.ShouldBreak(0x808201));
            Assert.Equal(-1, bp.DepthGuard);
        }

        [Fact]
        public void Enabling_and_disabling_reaches_data_breakpoints_too()
        {
            var bp = new BreakpointRegistry();
            int id = bp.AddDataBreakpoint("WRAM", 0x0100, 0x0100, -1, onRead: false, changedOnly: false, condition: null);

            Assert.True(bp.SetEnabled(id, false));
            bp.NoteWrite("WRAM", 0x0100, 0x42);
            Assert.False(bp.ShouldBreak(0x808000));

            Assert.True(bp.SetEnabled(id, true));
            bp.NoteWrite("WRAM", 0x0100, 0x42);
            Assert.True(bp.ShouldBreak(0x808000));
        }

        [Fact]
        public void The_listed_shape_of_a_data_breakpoint_carries_its_range_and_flags()
        {
            var bp = new BreakpointRegistry();
            bp.AddDataBreakpoint("WRAM", 0x0100, 0x01FF, 0x42, onRead: true, changedOnly: false, condition: "a > 1");

            var listed = bp.GetDataBreakpoints().Single();

            Assert.Equal(0x0100, listed.Address);
            Assert.Equal(0x01FF, listed.EndAddress);
            Assert.Equal(0x42, listed.Value);
            Assert.True(listed.OnRead);
            Assert.Equal("a > 1", listed.Condition);
        }
    }
}
