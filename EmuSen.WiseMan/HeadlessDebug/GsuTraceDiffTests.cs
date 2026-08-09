using EmuSen.Cores.Debug;
using EmuSen.Cores.Nintendo.Venus.Debug;
using Gsu = EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx.SuperFx;

namespace EmuSen.WiseMan.HeadlessDebug
{
    // The GSU half of the trace differ - see EmuSen_Debugging_Tools_Reference_v5.md §3.41.
    public class GsuTraceDiffTests
    {
        private const ushort Clsr = 0x3039, R15Low = 0x301E, R15High = 0x301F;

        private static GsuTraceDiff.Step[] Line(int count, uint from = 0x090000, ushort sfr = 0x0020)
        {
            var regs = new ushort[count * 16];
            var steps = new GsuTraceDiff.Step[count];
            for (int i = 0; i < count; i++)
            {
                steps[i] = new GsuTraceDiff.Step(from + (uint)i, 0x01, sfr, 0, 0, regs, i * 16, 2);
            }
            return steps;
        }

        private static GsuTraceDiff.Step[] Copy(GsuTraceDiff.Step[] source)
        {
            var regs = new ushort[source.Length * 16];
            var steps = new GsuTraceDiff.Step[source.Length];
            for (int i = 0; i < source.Length; i++)
            {
                for (int k = 0; k < 16; k++) regs[i * 16 + k] = source[i].Regs[source[i].RegBase + k];
                steps[i] = source[i] with { Regs = regs, RegBase = i * 16 };
            }
            return steps;
        }

        [Fact]
        public void Parse_rejects_a_blob_that_is_not_a_gsu_trace()
        {
            Assert.Throws<InvalidDataException>(() => GsuTraceDiff.Parse(new byte[64]));
            Assert.Throws<InvalidDataException>(() => GsuTraceDiff.Parse(CpuBinaryTrace.Magic));
        }

        [Fact]
        public void Identical_streams_report_nothing()
        {
            var left = Line(60);
            var result = GsuTraceDiff.Compare(left, Copy(left));

            Assert.Empty(result.Findings);
        }

        [Fact]
        public void A_differing_register_is_a_register_finding()
        {
            var left = Line(40);
            var right = Copy(left);
            right[9].Regs[right[9].RegBase + 4] = 0x1234;

            var result = GsuTraceDiff.Compare(left, right);

            var f = Assert.Single(result.Findings);
            Assert.Equal(TraceDiff.FindingKind.Registers, f.Kind);
            Assert.Equal(9, f.LeftStep);
        }

        // R15 is the address under two conventions, so it is never a register finding - see §3.41.
        [Fact]
        public void R15_is_not_compared()
        {
            var left = Line(40);
            var right = Copy(left);
            right[9].Regs[right[9].RegBase + 15] = 0xBEEF;

            Assert.Empty(GsuTraceDiff.Compare(left, right).Findings);
        }

        // Go, ROM-pending and IRQ are outside the compared mask - see §3.41.
        [Fact]
        public void The_run_rompending_and_irq_bits_are_not_compared()
        {
            var left = Line(40, sfr: 0x0020);
            var right = Line(40, sfr: 0x8040);

            Assert.Empty(GsuTraceDiff.Compare(left, right).Findings);
        }

        [Fact]
        public void An_arithmetic_flag_is_compared()
        {
            var left = Line(40, sfr: 0x0020);
            var right = Line(40, sfr: 0x0022);

            var f = GsuTraceDiff.Compare(left, right).First;
            Assert.NotNull(f);
            Assert.Equal(TraceDiff.FindingKind.Registers, f.Value.Kind);
            Assert.Equal(0, f.Value.LeftStep);
        }

        // The recorded address is the opcode's own byte, which for a delay slot is not R15 (§4.1a).
        [Fact]
        public void A_recorded_delay_slot_carries_its_own_address()
        {
            byte[] rom = new byte[0x10000];
            new byte[]
            {
                0x05, 0x04, // $0000 BRA $0006
                0x4C,       // $0002 delay slot
                0x01,       // $0003
                0x01, 0x01, // $0004
                0x01,       // $0006 target
                0x00,       // $0007 STOP
            }.CopyTo(rom, 0);

            var gsu = new Gsu(rom, new byte[0x8000]);
            gsu.WriteRegister(Clsr, 0x01);
            GsuBinaryTrace.Start();
            try
            {
                gsu.WriteRegister(R15Low, 0x00);
                gsu.WriteRegister(R15High, 0x00);
                gsu.Run(4000);
            }
            finally
            {
                GsuBinaryTrace.Stop();
            }

            string path = Path.Combine(Path.GetTempPath(), Path.GetRandomFileName() + ".bin");
            try
            {
                GsuBinaryTrace.WriteTo(path);
                var steps = GsuTraceDiff.Load(path);

                Assert.Equal(0x000000u, steps[0].Addr);
                Assert.Equal(0x000002u, steps[1].Addr);
                Assert.Equal(0x000006u, steps[2].Addr);
                Assert.Equal(0x4C, steps[1].Opcode);
            }
            finally
            {
                File.Delete(path);
                GsuBinaryTrace.Reset();
            }
        }

        // Nothing recorded unless armed, so a normal run pays nothing for this.
        [Fact]
        public void Nothing_is_recorded_while_the_sink_is_off()
        {
            GsuBinaryTrace.Reset();
            byte[] rom = new byte[0x10000];
            new byte[] { 0x01, 0x01, 0x00 }.CopyTo(rom, 0);

            var gsu = new Gsu(rom, new byte[0x8000]);
            gsu.WriteRegister(Clsr, 0x01);
            gsu.WriteRegister(R15Low, 0x00);
            gsu.WriteRegister(R15High, 0x00);
            gsu.Run(4000);

            Assert.Equal(0, GsuBinaryTrace.Count);
        }
    }
}
