using EmuSen.Cores.Nintendo.Venus.Debug;
using Step = EmuSen.Cores.Nintendo.Venus.Debug.CpuTraceDiff.Step;

namespace EmuSen.WiseMan.HeadlessDebug
{
    // The resyncing S-CPU trace differ - see EmuSen_Debugging_Tools_Reference_v5.md §3.40.
    public class CpuTraceDiffTests
    {
        private static Step At(uint addr, ushort a = 0) =>
            new Step(addr, 0xEA, CpuBinaryTrace.KindInstruction, a, 0, 0, 0x01FF, 0, 0, 0x30, false);

        // A straight run of distinct addresses, so nothing collapses by accident.
        private static Step[] Line(int count, uint from = 0x008000) =>
            Enumerable.Range(0, count).Select(i => At(from + (uint)i * 3)).ToArray();

        private static Step[] Spin(int iterations)
        {
            var loop = new List<Step>();
            for (int i = 0; i < iterations; i++)
            {
                loop.Add(At(0x00C000));
                loop.Add(At(0x00C003));
                loop.Add(At(0x00C005));
            }
            return loop.ToArray();
        }

        [Fact]
        public void Identical_streams_report_nothing()
        {
            var steps = Line(50);
            var result = CpuTraceDiff.Compare(steps, steps.ToArray());

            Assert.Empty(result.Findings);
        }

        [Fact]
        public void A_spin_loop_collapses_to_a_single_node()
        {
            var nodes = CpuTraceDiff.Collapse(Spin(200));

            Assert.Single(nodes);
            Assert.Equal(3, nodes[0].Length);
            Assert.Equal(200, nodes[0].Repeats);
        }

        // The whole reason for collapsing - see §3.40.
        [Fact]
        public void A_differing_loop_count_is_reported_as_a_count_not_a_desync()
        {
            var left = Line(4).Concat(Spin(431)).Concat(Line(4, 0x00D000)).ToArray();
            var right = Line(4).Concat(Spin(388)).Concat(Line(4, 0x00D000)).ToArray();

            var result = CpuTraceDiff.Compare(left, right);

            var f = Assert.Single(result.Findings);
            Assert.Equal(CpuTraceDiff.FindingKind.LoopCount, f.Kind);
            Assert.Contains("431x on the left", f.Detail);
            Assert.Contains("388x on the right", f.Detail);
        }

        [Fact]
        public void The_same_instruction_with_different_registers_is_a_register_finding()
        {
            var left = Line(20);
            var right = Line(20);
            right[7] = At(right[7].Addr, a: 0x1234);

            var result = CpuTraceDiff.Compare(left, right);

            var f = Assert.Single(result.Findings);
            Assert.Equal(CpuTraceDiff.FindingKind.Registers, f.Kind);
            Assert.Equal(7, f.LeftStep);
            Assert.Equal(7, f.RightStep);
        }

        // Mesen runs boot blocks we skip; without resync everything after reads as different.
        [Fact]
        public void An_inserted_block_resyncs_instead_of_poisoning_the_rest()
        {
            var shared = Line(40);
            var inserted = Line(12, 0x01A000);
            var left = shared.Take(10).Concat(inserted).Concat(shared.Skip(10)).ToArray();

            var result = CpuTraceDiff.Compare(left, shared);

            var f = Assert.Single(result.Findings);
            Assert.Equal(CpuTraceDiff.FindingKind.Structure, f.Kind);
            Assert.Contains("left executed 12 extra steps", f.Detail);
            Assert.Contains("then the paths rejoin", f.Detail);
        }

        // The finding that matters most: data goes wrong before control flow does.
        [Fact]
        public void A_register_divergence_after_an_inserted_block_is_still_found()
        {
            var shared = Line(40);
            var left = shared.Take(10).Concat(Line(12, 0x01A000)).Concat(shared.Skip(10)).ToArray();
            var right = shared.ToArray();
            right[30] = At(right[30].Addr, a: 0x00FF);

            var result = CpuTraceDiff.Compare(left, right);

            Assert.Equal(2, result.Findings.Count);
            Assert.Equal(CpuTraceDiff.FindingKind.Structure, result.Findings[0].Kind);
            var regs = result.Findings[1];
            Assert.Equal(CpuTraceDiff.FindingKind.Registers, regs.Kind);
            Assert.Equal(30, regs.RightStep);
            Assert.Equal(42, regs.LeftStep);
        }

        [Fact]
        public void An_interrupt_entry_never_aligns_with_a_plain_instruction()
        {
            var left = Line(10);
            var right = Line(10);
            right[5] = new Step(right[5].Addr, 0, CpuBinaryTrace.KindNmi, 0, 0, 0, 0x01FF, 0, 0, 0x30, false);

            var result = CpuTraceDiff.Compare(left, right);

            Assert.NotEmpty(result.Findings);
            Assert.Equal(CpuTraceDiff.FindingKind.Structure, result.Findings[0].Kind);
        }

        [Fact]
        public void A_shorter_side_that_otherwise_agrees_is_reported_as_truncated()
        {
            var left = Line(60);

            var result = CpuTraceDiff.Compare(left, left.Take(30).ToArray());

            var f = Assert.Single(result.Findings);
            Assert.Equal(CpuTraceDiff.FindingKind.TruncatedSide, f.Kind);
            Assert.Contains("right", f.Detail);
        }

        [Fact]
        public void Findings_come_back_in_execution_order()
        {
            var left = Line(60);
            var right = Line(60);
            right[40] = At(right[40].Addr, a: 0x0002);
            right[12] = At(right[12].Addr, a: 0x0001);

            var result = CpuTraceDiff.Compare(left, right);

            Assert.Equal(2, result.Findings.Count);
            Assert.Equal(12, result.Findings[0].LeftStep);
            Assert.Equal(40, result.Findings[1].LeftStep);
        }

        [Fact]
        public void A_blob_without_the_header_is_rejected_rather_than_misread()
        {
            Assert.Throws<InvalidDataException>(() => CpuTraceDiff.Parse(new byte[64]));
        }
    }

    // Its own class because CpuBinaryTrace is process-wide state, like DebugSettings.
    public class CpuBinaryTraceRoundTripTests
    {
        [Fact]
        public void What_the_core_records_is_what_the_differ_parses_back()
        {
            try
            {
                CpuBinaryTrace.Start();
                CpuBinaryTrace.Record(0x7E1234, 0xA9, CpuBinaryTrace.KindInstruction,
                    0x1122, 0x3344, 0x5566, 0x01F0, 0x0100, 0x7E, 0x24, true);
                CpuBinaryTrace.Record(0x008123, 0x00, CpuBinaryTrace.KindNmi,
                    0, 0, 0, 0x01FF, 0, 0, 0x30, false);
                CpuBinaryTrace.Stop();

                string path = Path.Combine(Path.GetTempPath(), Path.GetRandomFileName() + ".bin");
                CpuBinaryTrace.WriteTo(path);
                var steps = CpuTraceDiff.Load(path);
                File.Delete(path);

                Assert.Equal(2, steps.Length);
                Assert.Equal(0x7E1234u, steps[0].Addr);
                Assert.Equal(0xA9, steps[0].Opcode);
                Assert.Equal(0x1122, steps[0].A);
                Assert.Equal(0x3344, steps[0].X);
                Assert.Equal(0x5566, steps[0].Y);
                Assert.Equal(0x01F0, steps[0].S);
                Assert.Equal(0x0100, steps[0].D);
                Assert.Equal(0x7E, steps[0].Db);
                Assert.Equal(0x24, steps[0].P);
                Assert.True(steps[0].E);
                Assert.Equal(CpuBinaryTrace.KindNmi, steps[1].Kind);
            }
            finally
            {
                CpuBinaryTrace.Reset();
            }
        }

        // Gated on Enabled, so an ordinary run pays nothing and no steps leak between runs.
        [Fact]
        public void Nothing_is_recorded_while_the_trace_is_stopped()
        {
            try
            {
                CpuBinaryTrace.Start();
                CpuBinaryTrace.Stop();
                Assert.False(CpuBinaryTrace.Enabled);
                Assert.Equal(0, CpuBinaryTrace.Count);
            }
            finally
            {
                CpuBinaryTrace.Reset();
            }
        }
    }
}
