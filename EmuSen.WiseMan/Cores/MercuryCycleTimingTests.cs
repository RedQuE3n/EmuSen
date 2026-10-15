using EmuSen.Cores.Nintendo.Mercury.Cpu.Core;
using GbCpu = EmuSen.Cores.Nintendo.Mercury.Cpu.Core.Cpu;

namespace EmuSen.WiseMan.Cores
{
    // That the machine really runs *during* an instruction, not after it - see Mercury_Cpu.md §6.
    public class MercuryCycleTimingTests
    {
        // Records the order the CPU touches the bus in, which is the whole point of the change.
        private sealed class RecordingBus : ICpuBus
        {
            public readonly List<string> Trace = new();
            public readonly byte[] Memory = new byte[0x10000];

            public byte Read(ushort address)
            {
                Trace.Add($"R:{address:X4}");
                return Memory[address];
            }

            public void Write(ushort address, byte data)
            {
                Trace.Add($"W:{address:X4}");
                Memory[address] = data;
            }

            public void Tick(int cycles) => Trace.Add($"T:{cycles}");

            public void Stop() { }

            public byte PendingInterrupts => (byte)(Memory[0xFFFF] & Memory[0xFF0F] & 0x1F);

            public int Ticked
            {
                get
                {
                    int total = 0;
                    foreach (string entry in Trace)
                    {
                        if (entry.StartsWith("T:", StringComparison.Ordinal)) total += int.Parse(entry[2..]);
                    }

                    return total;
                }
            }
        }

        private static (GbCpu Cpu, RecordingBus Bus) Machine(params byte[] program)
        {
            var bus = new RecordingBus();
            program.CopyTo(bus.Memory, 0x0100);

            var cpu = new GbCpu(bus);
            cpu.Reset();
            return (cpu, bus);
        }

        // LD A,($C123): every one of the four machine cycles advances the machine before its transfer.
        [Fact]
        public void Each_memory_access_is_preceded_by_its_own_machine_cycle()
        {
            var (cpu, bus) = Machine(0xFA, 0x23, 0xC1);

            int cycles = cpu.Step(0, 0, out _);

            Assert.Equal(16, cycles);
            Assert.Equal(
                new[] { "T:4", "R:0100", "T:4", "R:0101", "T:4", "R:0102", "T:4", "R:C123" },
                bus.Trace);
        }

        // RET: one cycle of the sixteen is internal, and it follows the pops, so it settles at the end - see Mercury_Cpu.md §6.1, Mercury_Native.md §13.5.
        [Fact]
        public void Internal_cycles_settle_at_the_end_of_the_instruction()
        {
            var (cpu, bus) = Machine(0xC9);

            int cycles = cpu.Step(0, 0, out _);

            Assert.Equal(16, cycles);
            Assert.Equal(
                new[] { "T:4", "R:0100", "T:4", "R:FFFE", "T:4", "R:FFFF", "T:4" },
                bus.Trace);
        }

        // A read-modify-write on (HL) is three cycles and both halves are separately timed.
        [Fact]
        public void A_read_modify_write_ticks_between_its_read_and_its_write()
        {
            var (cpu, bus) = Machine(0x34);

            int cycles = cpu.Step(0, 0, out _);

            Assert.Equal(12, cycles);
            Assert.Equal(new[] { "T:4", "R:0100", "T:4", "R:014D", "T:4", "W:014D" }, bus.Trace);
        }

        // Nothing may be ticked twice or lost: the bus must advance by exactly what Step reports.
        [Theory]
        [InlineData(0x00)]
        [InlineData(0x34)]
        [InlineData(0xC5)]
        [InlineData(0xE1)]
        [InlineData(0xFA)]
        [InlineData(0xCD)]
        [InlineData(0x18)]
        [InlineData(0xE8)]
        [InlineData(0xF0)]
        [InlineData(0x76)]
        [InlineData(0x10)]
        public void The_bus_advances_by_exactly_what_the_step_reports(byte opcode)
        {
            var (cpu, bus) = Machine(opcode, 0x04, 0x01);

            int cycles = cpu.Step(0, 0, out _);

            Assert.Equal(cycles, bus.Ticked);
        }

        // Servicing an interrupt is 20 cycles with two stack writes inside them.
        [Fact]
        public void Servicing_an_interrupt_ticks_its_full_cost()
        {
            var (cpu, bus) = Machine(0x00);
            bus.Memory[0xFFFF] = 0x01;
            bus.Memory[0xFF0F] = 0x01;
            cpu.Ime = true;

            int cycles = cpu.Step(0x01, 0x01, out int serviced);

            Assert.Equal(0, serviced);
            Assert.Equal(20, cycles);
            Assert.Equal(20, bus.Ticked);
            Assert.Equal(0x0040, cpu.PC);
        }

        // Pan Docs' dispatch: two wait states, the push on the third and fourth machine cycles, the jump on the fifth - see Mercury_Native.md §13.3.
        [Fact]
        public void A_dispatch_pushes_on_its_third_and_fourth_machine_cycles()
        {
            var (cpu, bus) = Machine(0x00);
            bus.Memory[0xFFFF] = 0x01;
            bus.Memory[0xFF0F] = 0x01;
            cpu.Ime = true;

            cpu.Step(0x01, 0x01, out _);

            var ticksBefore = new List<int>();
            int sum = 0;
            foreach (string entry in bus.Trace)
            {
                if (entry.StartsWith("T:", StringComparison.Ordinal)) sum += int.Parse(entry[2..]);
                else ticksBefore.Add(sum);
            }
            Assert.Equal(new[] { 12, 16 }, ticksBefore);
            Assert.Equal(20, bus.Ticked);
        }

        // Where each access lands, as the T-cycles ticked before it, then the total.
        private static int[] AccessTimes(RecordingBus bus)
        {
            var times = new List<int>();
            int sum = 0;
            foreach (string entry in bus.Trace)
            {
                if (entry.StartsWith("T:", StringComparison.Ordinal)) sum += int.Parse(entry[2..]);
                else times.Add(sum);
            }
            times.Add(sum);
            return times.ToArray();
        }

        // mooneye's *_timing headers and GBCTR: the internal cycle that decrements SP comes before a push's writes, and RET cc's condition cycle before its pops - see Mercury_Native.md §13.5.
        [Theory]
        [InlineData(new byte[] { 0xC5 }, new[] { 4, 12, 16, 16 })]
        [InlineData(new byte[] { 0xCF }, new[] { 4, 12, 16, 16 })]
        [InlineData(new byte[] { 0xCD, 0x34, 0x12 }, new[] { 4, 8, 12, 20, 24, 24 })]
        [InlineData(new byte[] { 0xCC, 0x34, 0x12 }, new[] { 4, 8, 12, 20, 24, 24 })]
        [InlineData(new byte[] { 0xC8 }, new[] { 4, 12, 16, 20 })]
        public void Each_stack_access_lands_on_the_machine_cycle_the_hardware_gives_it(byte[] program, int[] times)
        {
            var (cpu, bus) = Machine(program);

            cpu.Step(0, 0, out _);

            Assert.Equal(times, AccessTimes(bus));
        }

        // A halted CPU still runs the machine, or nothing would ever wake it.
        [Fact]
        public void A_halted_cpu_still_advances_the_bus()
        {
            var (cpu, bus) = Machine(0x00);
            cpu.Halted = true;

            int cycles = cpu.Step(0, 0, out _);

            Assert.Equal(4, cycles);
            Assert.Equal(new[] { "T:4" }, bus.Trace);
        }
    }
}
