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

        // PUSH BC: one cycle of the sixteen is internal, and it settles at the end - see Mercury_Cpu.md §6.1.
        [Fact]
        public void Internal_cycles_settle_at_the_end_of_the_instruction()
        {
            var (cpu, bus) = Machine(0xC5);

            int cycles = cpu.Step(0, 0, out _);

            Assert.Equal(16, cycles);
            Assert.Equal(
                new[] { "T:4", "R:0100", "T:4", "W:FFFD", "T:4", "W:FFFC", "T:4" },
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
            cpu.Ime = true;

            int cycles = cpu.Step(0x01, 0x01, out int serviced);

            Assert.Equal(0, serviced);
            Assert.Equal(20, cycles);
            Assert.Equal(20, bus.Ticked);
            Assert.Equal(0x0040, cpu.PC);
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
