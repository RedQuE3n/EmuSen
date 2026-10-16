using EmuSen.Cores;
using System.Linq;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.Crystal.Debugger;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // The coprocessor register-window log and its poll detection - see `man copflow`.
    public class RegisterFlowTests
    {
        private static RegisterFlowRegistry Armed(int size = 0x100)
        {
            var flow = new RegisterFlowRegistry();
            flow.Arm("CoprocessorRegisters", size);
            return flow;
        }

        // A clear after `off` empties the log it says it clears - see EmuSen_DianaSh.md §16 Q19.
        [Fact]
        public void Clear_empties_a_disarmed_window_and_leaves_it_disarmed()
        {
            var flow = Armed();
            flow.Note(0x30, 0x01, isWrite: true, "cpu");
            flow.Disarm();

            flow.Clear();

            Assert.Equal(0, flow.TotalAccesses);
            Assert.Empty(flow.Tail(10));
            Assert.False(flow.IsArmed);
        }

        [Fact]
        public void A_disarmed_registry_records_nothing()
        {
            var flow = new RegisterFlowRegistry();

            flow.Note(0x30, 0x01, isWrite: true, "cpu");

            Assert.Equal(0, flow.TotalAccesses);
        }

        [Fact]
        public void Reads_and_writes_are_tallied_separately_per_register()
        {
            var flow = Armed();

            flow.Note(0x30, 0x01, isWrite: true, "cpu");
            flow.Note(0x30, 0x01, isWrite: false, "cpu");
            flow.Note(0x30, 0x02, isWrite: false, "cpu");

            var stat = flow.Stats().Single(s => s.Register == 0x30);
            Assert.Equal(1, stat.Writes);
            Assert.Equal(2, stat.Reads);
            Assert.Equal(2, stat.DistinctValues);
            Assert.Equal(0x02, stat.LastValue);
        }

        [Fact]
        public void Stats_rank_the_busiest_register_first()
        {
            var flow = Armed();
            flow.Note(0x10, 0x00, isWrite: false, "cpu");
            for (int i = 0; i < 5; i++) flow.Note(0x20, 0x00, isWrite: false, "cpu");

            Assert.Equal(0x20, flow.Stats()[0].Register);
        }

        [Fact]
        public void Tail_returns_the_most_recent_accesses_oldest_first()
        {
            var flow = Armed();
            for (int i = 0; i < 5; i++) flow.Note(0x30, (byte)i, isWrite: false, "cpu");

            var tail = flow.Tail(3);

            Assert.Equal(3, tail.Count);
            Assert.Equal(new byte[] { 2, 3, 4 }, tail.Select(e => e.Value));
            Assert.Equal(3, tail[0].Sequence);
        }

        // The ring must wrap without reordering or duplicating entries.
        [Fact]
        public void Tail_stays_in_order_after_the_ring_wraps()
        {
            var flow = new RegisterFlowRegistry();
            flow.Arm("CoprocessorRegisters", 0x100, capacity: 16);
            for (int i = 0; i < 40; i++) flow.Note(0x30, (byte)i, isWrite: false, "cpu");

            var tail = flow.Tail(16);

            Assert.Equal(16, tail.Count);
            Assert.Equal(25, tail[0].Sequence);
            Assert.Equal(40, tail[^1].Sequence);
            Assert.Equal(39, tail[^1].Value);
        }

        [Fact]
        public void Tail_never_returns_more_than_was_recorded()
        {
            var flow = Armed();
            flow.Note(0x30, 0x00, isWrite: false, "cpu");

            Assert.Single(flow.Tail(50));
        }

        // The whole point: a spin on an unchanging status bit.
        [Fact]
        public void Repeated_reads_of_an_unchanging_value_count_as_a_poll_run()
        {
            var flow = Armed();

            for (int i = 0; i < 100; i++) flow.Note(0x30, 0x00, isWrite: false, "cpu");

            Assert.Equal(100, flow.LongestPollRun);
            Assert.Equal(0x30, flow.LongestPollRegister);
            Assert.Equal(100, flow.CurrentPollRun);
        }

        [Fact]
        public void A_changed_value_breaks_the_poll_run()
        {
            var flow = Armed();

            for (int i = 0; i < 10; i++) flow.Note(0x30, 0x00, isWrite: false, "cpu");
            flow.Note(0x30, 0x80, isWrite: false, "cpu");

            Assert.Equal(10, flow.LongestPollRun);
            Assert.Equal(1, flow.CurrentPollRun);
        }

        // A write is progress by either side, so it always breaks the run.
        [Fact]
        public void A_write_breaks_the_poll_run()
        {
            var flow = Armed();

            for (int i = 0; i < 10; i++) flow.Note(0x30, 0x00, isWrite: false, "cpu");
            flow.Note(0x31, 0x01, isWrite: true, "cpu");

            Assert.Equal(10, flow.LongestPollRun);
            Assert.Equal(0, flow.CurrentPollRun);
        }

        [Fact]
        public void Reads_of_different_registers_do_not_accumulate_one_run()
        {
            var flow = Armed();

            for (int i = 0; i < 10; i++)
            {
                flow.Note(0x30, 0x00, isWrite: false, "cpu");
                flow.Note(0x31, 0x00, isWrite: false, "cpu");
            }

            Assert.Equal(1, flow.LongestPollRun);
        }

        [Fact]
        public void An_out_of_range_register_is_ignored_rather_than_throwing()
        {
            var flow = Armed(0x10);

            flow.Note(0x999, 0x00, isWrite: false, "cpu");

            Assert.Equal(0, flow.TotalAccesses);
        }

        [Fact]
        public void Clear_keeps_the_registry_armed_but_forgets_everything()
        {
            var flow = Armed();
            for (int i = 0; i < 10; i++) flow.Note(0x30, 0x00, isWrite: false, "cpu");

            flow.Clear();

            Assert.True(flow.IsArmed);
            Assert.Equal(0, flow.TotalAccesses);
            Assert.Equal(0, flow.LongestPollRun);
        }

        [Fact]
        public void Entries_carry_the_frame_they_happened_on()
        {
            var flow = Armed();
            flow.FrameNumberProvider = () => 42;

            flow.Note(0x30, 0x00, isWrite: false, "cpu");

            Assert.Equal(42, flow.Tail(1)[0].FrameNumber);
        }

    }
}
