using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Cores.Nintendo.Venus.Processor;

namespace EmuSen.WiseMan.Cores
{
    // Master-clock cost per instruction, against Mesen's own - see Venus_CPU.md §8.8.
    public class Snes65816CycleCostTests
    {
        // Uniform 8 master clocks, so a 6-clock internal cycle is distinguishable.
        private sealed class SlowBus : ICpuBus
        {
            public readonly byte[] Flat = new byte[0x1000000];
            public byte Read8(uint address) => Flat[address & 0xFFFFFF];
            public void Write8(uint address, byte data) => Flat[address & 0xFFFFFF] = data;
            public int GetAccessSpeedCycles(uint address) => 8;
            public int TakePendingDmaCycles() => 0;
        }

        private static int Cost(bool wide16A, bool wide16X, params byte[] code)
        {
            var bus = new SlowBus();
            Cpu cpu;
            var realOut = Console.Out;
            Console.SetOut(TextWriter.Null);
            try { cpu = new Cpu(bus); }
            finally { Console.SetOut(realOut); }

            cpu.E = false;
            cpu.P = (byte)((wide16A ? 0 : (byte)CpuFlags.M) | (wide16X ? 0 : (byte)CpuFlags.X));
            cpu.PB = 0;
            cpu.PC = 0x8000;
            cpu.S = 0x01FF;
            cpu.D = 0;
            for (int i = 0; i < code.Length; i++) bus.Flat[0x8000 + i] = code[i];
            return cpu.Step();
        }

        // 1 opcode fetch at 8, then one internal cycle that touches no bus at 6.
        [Theory]
        [InlineData(0x78)] // SEI
        [InlineData(0x18)] // CLC
        [InlineData(0xC8)] // INY
        [InlineData(0xCA)] // DEX
        [InlineData(0xAA)] // TAX
        [InlineData(0xEA)] // NOP
        [InlineData(0x1A)] // INC A
        [InlineData(0x0A)] // ASL A
        public void Register_only_opcodes_cost_a_fetch_plus_a_six_clock_internal_cycle(byte opcode)
        {
            Assert.Equal(14, Cost(false, false, opcode));
        }

        [Fact]
        public void Xba_pays_for_two_internal_cycles_not_two_bus_accesses()
        {
            Assert.Equal(20, Cost(false, false, 0xEB));
        }

        [Theory]
        [InlineData(0xC2)] // REP
        [InlineData(0xE2)] // SEP
        public void Rep_and_sep_are_two_fetches_plus_one_internal_cycle(byte opcode)
        {
            Assert.Equal(22, Cost(false, false, opcode, 0x20));
        }

        // The 8-bit table said 16 for both, which ran the machine fast - see §8.8.
        [Fact]
        public void A_sixteen_bit_immediate_costs_the_extra_fetch_it_really_performs()
        {
            Assert.Equal(16, Cost(wide16A: false, wide16X: false, 0xA9, 0x34));       // LDA #$34
            Assert.Equal(24, Cost(wide16A: true, wide16X: false, 0xA9, 0x34, 0x12));  // LDA #$1234
        }

        [Fact]
        public void A_sixteen_bit_index_load_costs_the_extra_fetch_too()
        {
            Assert.Equal(16, Cost(false, wide16X: false, 0xA2, 0x34));       // LDX #$34
            Assert.Equal(24, Cost(false, wide16X: true, 0xA2, 0x34, 0x12));  // LDX #$1234
        }

        // Three fetches, then one store per operand byte.
        [Fact]
        public void A_sixteen_bit_store_writes_twice_and_is_charged_twice()
        {
            Assert.Equal(32, Cost(wide16A: false, wide16X: false, 0x8D, 0x00, 0x10));
            Assert.Equal(40, Cost(wide16A: true, wide16X: false, 0x8D, 0x00, 0x10));
        }

        // Read-modify-write touches the operand twice, so 16-bit costs two extra.
        [Fact]
        public void A_sixteen_bit_read_modify_write_costs_two_extra_cycles()
        {
            Assert.Equal(48, Cost(wide16A: false, wide16X: false, 0xEE, 0x00, 0x10)); // INC $1000
            Assert.Equal(64, Cost(wide16A: true, wide16X: false, 0xEE, 0x00, 0x10));
        }

        // The largest divergence the differ found: 9,545 taken branches, each a cycle short.
        [Fact]
        public void A_taken_branch_costs_one_internal_cycle_more_than_a_skipped_one()
        {
            var bus = new SlowBus();
            int Branch(bool zeroSet)
            {
                Cpu cpu;
                var realOut = Console.Out;
                Console.SetOut(TextWriter.Null);
                try { cpu = new Cpu(bus); }
                finally { Console.SetOut(realOut); }

                cpu.E = false;
                cpu.P = zeroSet ? (byte)CpuFlags.Z : (byte)0;
                cpu.PB = 0;
                cpu.PC = 0x8000;
                bus.Flat[0x8000] = 0xD0; // BNE
                bus.Flat[0x8001] = 0xFE; // back to itself
                return cpu.Step();
            }

            Assert.Equal(16, Branch(zeroSet: true));   // not taken: two fetches
            Assert.Equal(22, Branch(zeroSet: false));  // taken: plus one internal cycle
        }
    }
}
