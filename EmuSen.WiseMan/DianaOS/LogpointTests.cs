using System.Linq;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.WiseMan.DianaOS
{
    // Breakpoints that record and carry on instead of halting - see `man bp`.
    public class LogpointTests
    {
        private static BreakpointRegistry Registry()
            => new() { LogEvaluator = expression => ($"<{expression}>", null) };

        [Fact]
        public void A_logpoint_records_instead_of_halting()
        {
            var bp = Registry();
            bp.AddBreakpoint(0x808000, 0x808000, null, "a, x");

            Assert.False(bp.ShouldBreak(0x808000));
            Assert.Equal(1, bp.LogEntriesRecorded);
            Assert.Equal("<a, x>", bp.LogTail(10).Single().Text);
        }

        [Fact]
        public void A_logpoint_keeps_counting_hits()
        {
            var bp = Registry();
            int id = bp.AddBreakpoint(0x808000, 0x808000, null, "a");

            for (int i = 0; i < 5; i++) bp.ShouldBreak(0x808000);

            Assert.Equal(5, bp.GetBreakpoints().Single(b => b.Id == id).HitCount);
            Assert.Equal(5, bp.LogEntriesRecorded);
        }

        [Fact]
        public void A_breakpoint_without_a_log_expression_still_halts()
        {
            var bp = Registry();
            bp.AddBreakpoint(0x808000);

            Assert.True(bp.ShouldBreak(0x808000));
            Assert.Equal(0, bp.LogEntriesRecorded);
        }

        [Fact]
        public void A_logpoint_still_honors_its_condition()
        {
            var bp = Registry();
            bp.ConditionEvaluator = _ => (false, null);
            bp.AddBreakpoint(0x808000, 0x808000, "x == 7", "a");

            Assert.False(bp.ShouldBreak(0x808000));
            Assert.Equal(0, bp.LogEntriesRecorded);
        }

        [Fact]
        public void A_logged_range_records_the_address_that_matched()
        {
            var bp = Registry();
            bp.AddBreakpoint(0x818000, 0x818FFF, null, "a");

            bp.ShouldBreak(0x818ABC);

            Assert.Equal(0x818ABC, bp.LogTail(10).Single().Address);
        }

        [Fact]
        public void A_logging_breakpoint_does_not_stop_a_later_halting_one()
        {
            var bp = Registry();
            bp.AddBreakpoint(0x808000, 0x808000, null, "a");
            bp.AddBreakpoint(0x808000);

            // The logpoint records, and the scan carries on to the halting breakpoint.
            Assert.True(bp.ShouldBreak(0x808000));
            Assert.Equal(1, bp.LogEntriesRecorded);
        }

        [Fact]
        public void A_write_logpoint_records_at_the_write_and_never_halts()
        {
            var bp = Registry();
            bp.AddDataBreakpoint("WRAM", 0x0DB3, 0x0DB3, -1, onRead: false, changedOnly: false, condition: null, logExpression: "a");

            bp.NoteWrite("WRAM", 0x0DB3, 0x42);

            Assert.Equal(1, bp.LogEntriesRecorded);
            Assert.False(bp.ShouldBreak(0x808000));
        }

        [Fact]
        public void A_read_logpoint_records_too()
        {
            var bp = Registry();
            bp.AddDataBreakpoint("WRAM", 0x0DB3, 0x0DB3, -1, onRead: true, changedOnly: false, condition: null, logExpression: "a");

            bp.NoteRead("WRAM", 0x0DB3, 0x42);

            Assert.Equal(1, bp.LogEntriesRecorded);
            Assert.False(bp.ShouldBreak(0x808000));
        }

        [Fact]
        public void An_evaluator_error_is_recorded_rather_than_dropping_the_logpoint()
        {
            var bp = new BreakpointRegistry { LogEvaluator = _ => (string.Empty, "unknown symbol 'q'") };
            bp.AddBreakpoint(0x808000, 0x808000, null, "q");

            bp.ShouldBreak(0x808000);
            bp.ShouldBreak(0x808000);

            Assert.Equal(2, bp.LogEntriesRecorded);
            Assert.Equal("unknown symbol 'q'", bp.LogTail(10).First().Text);
        }

        [Fact]
        public void Without_an_evaluator_the_raw_expression_is_recorded()
        {
            var bp = new BreakpointRegistry();
            bp.AddBreakpoint(0x808000, 0x808000, null, "a, x");

            bp.ShouldBreak(0x808000);

            Assert.Equal("a, x", bp.LogTail(10).Single().Text);
        }

        [Fact]
        public void The_log_is_a_ring_that_keeps_the_most_recent_entries()
        {
            var bp = new BreakpointRegistry { LogEvaluator = _ => ("x", null) };
            bp.AddBreakpoint(0x808000, 0x808000, null, "a");

            for (int i = 0; i < 5000; i++) bp.ShouldBreak(0x808000);

            Assert.Equal(5000, bp.LogEntriesRecorded);
            Assert.Equal(4096, bp.LogTail(10000).Count);
        }

        [Fact]
        public void Tail_returns_the_newest_entries_oldest_first()
        {
            int n = 0;
            var bp = new BreakpointRegistry { LogEvaluator = _ => ((n++).ToString(), null) };
            bp.AddBreakpoint(0x808000, 0x808000, null, "a");

            for (int i = 0; i < 10; i++) bp.ShouldBreak(0x808000);

            var tail = bp.LogTail(3);
            Assert.Equal(new[] { "7", "8", "9" }, tail.Select(e => e.Text));
        }

        [Fact]
        public void Clearing_the_log_forgets_the_entries_and_the_total()
        {
            var bp = Registry();
            bp.AddBreakpoint(0x808000, 0x808000, null, "a");
            bp.ShouldBreak(0x808000);

            bp.ClearLog();

            Assert.Equal(0, bp.LogEntriesRecorded);
            Assert.Empty(bp.LogTail(10));
        }

        [Fact]
        public void A_logpoint_inside_a_forbid_range_records_nothing()
        {
            var bp = Registry();
            bp.AddBreakpoint(0x0080C8, 0x0080C8, null, "a");
            bp.AddForbidRange(0x0080C4, 0x0080FF);

            Assert.False(bp.ShouldBreak(0x0080C8));
            Assert.Equal(0, bp.LogEntriesRecorded);
        }
    }
}
