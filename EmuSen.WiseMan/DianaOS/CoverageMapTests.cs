using System.Linq;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.WiseMan.DianaOS
{
    // Differential coverage, discovered routines and the saved map - see `man cov`.
    public class CoverageMapTests
    {
        private static CoverageRegistry Armed()
        {
            var coverage = new CoverageRegistry();
            coverage.Arm();
            return coverage;
        }

        [Fact]
        public void A_disarmed_registry_records_nothing()
        {
            var coverage = new CoverageRegistry();

            coverage.Record(0x808000);
            coverage.RecordEntryPoint(0x808100, CallFrameKind.Call);

            Assert.False(coverage.WasExecuted(0x808000));
            Assert.Equal(0, coverage.EntryPointCount);
        }

        [Fact]
        public void New_since_mark_reports_only_what_ran_after_the_mark()
        {
            var coverage = Armed();
            coverage.Record(0x808000);
            coverage.Record(0x808001);

            coverage.Mark();
            coverage.Record(0x808002);

            var fresh = coverage.NewSinceMark(0x808000, 0x10, 32);

            Assert.Equal(new[] { 0x808002 }, fresh);
            Assert.Equal(1, coverage.CountNewSinceMark(0x808000, 0x10));
            Assert.Equal(3, coverage.CountExecuted(0x808000, 0x10));
        }

        [Fact]
        public void Re_running_marked_code_is_not_new()
        {
            var coverage = Armed();
            coverage.Record(0x808000);
            coverage.Mark();
            coverage.Record(0x808000);

            Assert.Empty(coverage.NewSinceMark(0x808000, 0x10, 32));
        }

        [Fact]
        public void Marking_twice_moves_the_baseline_forward()
        {
            var coverage = Armed();
            coverage.Record(0x808000);
            coverage.Mark();
            coverage.Record(0x808001);
            coverage.Mark();
            coverage.Record(0x808002);

            Assert.Equal(new[] { 0x808002 }, coverage.NewSinceMark(0x808000, 0x10, 32));
        }

        [Fact]
        public void Without_a_mark_nothing_is_new()
        {
            var coverage = Armed();
            coverage.Record(0x808000);

            Assert.False(coverage.HasMark);
            Assert.Empty(coverage.NewSinceMark(0x808000, 0x10, 32));
        }

        [Fact]
        public void Clearing_drops_the_mark_and_the_routines()
        {
            var coverage = Armed();
            coverage.Record(0x808000);
            coverage.RecordEntryPoint(0x808100, CallFrameKind.Call);
            coverage.Mark();

            coverage.Clear();

            Assert.False(coverage.HasMark);
            Assert.Equal(0, coverage.EntryPointCount);
            Assert.False(coverage.WasExecuted(0x808000));
        }

        [Fact]
        public void Entry_points_tally_repeat_entries_and_keep_the_first_kind()
        {
            var coverage = Armed();
            coverage.RecordEntryPoint(0x808100, CallFrameKind.Call);
            coverage.RecordEntryPoint(0x808100, CallFrameKind.Call);
            coverage.RecordEntryPoint(0x80FF00, CallFrameKind.Nmi);

            var funcs = coverage.EntryPoints(32);

            Assert.Equal(2, coverage.EntryPointCount);
            Assert.Equal(0x808100, funcs[0].Address);
            Assert.Equal(2, funcs[0].Entries);
            Assert.Equal(CallFrameKind.Nmi, funcs.Single(f => f.Address == 0x80FF00).Kind);
        }

        [Fact]
        public void Entry_points_come_back_busiest_first()
        {
            var coverage = Armed();
            coverage.RecordEntryPoint(0x808100, CallFrameKind.Call);
            for (int i = 0; i < 5; i++) coverage.RecordEntryPoint(0x808200, CallFrameKind.Call);

            Assert.Equal(0x808200, coverage.EntryPoints(32)[0].Address);
        }

        [Fact]
        public void Entry_points_in_a_range_come_back_in_address_order()
        {
            var coverage = Armed();
            coverage.RecordEntryPoint(0x808300, CallFrameKind.Call);
            coverage.RecordEntryPoint(0x808100, CallFrameKind.Call);
            coverage.RecordEntryPoint(0x908100, CallFrameKind.Call);

            Assert.Equal(new[] { 0x808100, 0x808300 }, coverage.EntryPointsIn(0x808000, 0x1000));
        }

        [Fact]
        public void The_call_stack_seam_feeds_entry_point_discovery()
        {
            var coverage = Armed();
            var stack = new CallStackRegistry { EntryPointObserver = coverage.RecordEntryPoint };

            stack.NotePush(0x808000, 0x808100, CallFrameKind.Call);

            Assert.Equal(1, coverage.EntryPointCount);
            Assert.Equal(0x808100, coverage.EntryPoints(1)[0].Address);
        }

        [Fact]
        public void An_exported_map_round_trips_through_import()
        {
            var coverage = Armed();
            coverage.Record(0x808000);
            coverage.Record(0x818ABC);
            coverage.RecordEntryPoint(0x808100, CallFrameKind.Call);

            var restored = Armed();
            Assert.True(restored.Import(coverage.Export()));

            Assert.True(restored.WasExecuted(0x808000));
            Assert.True(restored.WasExecuted(0x818ABC));
            Assert.False(restored.WasExecuted(0x808001));
            Assert.Equal(0x808100, restored.EntryPoints(1)[0].Address);
        }

        [Fact]
        public void Importing_merges_rather_than_replaces()
        {
            var first = Armed();
            first.Record(0x808000);
            first.RecordEntryPoint(0x808100, CallFrameKind.Call);
            var saved = first.Export();

            var second = Armed();
            second.Record(0x809000);
            second.RecordEntryPoint(0x808100, CallFrameKind.Call);
            second.RecordEntryPoint(0x809100, CallFrameKind.Call);

            Assert.True(second.Import(saved));

            Assert.True(second.WasExecuted(0x808000));
            Assert.True(second.WasExecuted(0x809000));
            Assert.Equal(2, second.EntryPointCount);
            Assert.Equal(2, second.EntryPoints(32).Single(f => f.Address == 0x808100).Entries);
        }

        [Fact]
        public void A_map_that_is_not_one_is_refused_rather_than_merged()
        {
            var coverage = Armed();

            Assert.False(coverage.Import(new byte[] { 1, 2, 3 }));
            Assert.False(coverage.Import(new byte[64]));
        }

        [Fact]
        public void Statistics_report_executed_total_and_new()
        {
            var coverage = Armed();
            coverage.Record(0x808000);
            coverage.Mark();
            coverage.Record(0x808001);

            var stats = coverage.Statistics(0x808000, 0x10);

            Assert.Equal(2, stats.Executed);
            Assert.Equal(0x10, stats.Total);
            Assert.Equal(1, stats.NewSinceMark);
        }
    }
}
