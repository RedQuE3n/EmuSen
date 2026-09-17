using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The asynchronous half: a counter, an aggregator, and the two gates between them and the CPU - see Mars_Cpu.md §12.
    public class MarsCpuInterruptTests
    {
        private const ulong TimerMask = 1UL << 15;
        private const ulong RcpMask = 1UL << 10;
        private const ulong GeneralVector = Cpu.VectorBase + Cpu.VectorOffsetGeneral;

        private static Cpu Machine(MemoryBus? bus = null)
        {
            var cpu = new MipsAssembler().Nop().Run(0, bus ?? new MemoryBus());
            cpu.Pc = MipsAssembler.EntryPoint;
            cpu.NextPc = cpu.Pc + 4;
            return cpu;
        }

        private static void RunNops(Cpu cpu, int count)
        {
            for (int i = 0; i < count; i++) cpu.Step();
        }

        // Execution continues into the handler, so the vector is only on the program counter for one step.
        private static bool StepUntilInterrupted(Cpu cpu, int limit)
        {
            for (int i = 0; i < limit; i++)
            {
                cpu.Step();
                if (cpu.LastException is not null) return true;
            }

            return false;
        }

        [Fact]
        public void The_counter_raises_its_line_when_it_reaches_the_comparison_value()
        {
            var cpu = Machine();
            cpu.Cop0[Cpu.CompareRegister] = 3;

            RunNops(cpu, 10);

            Assert.Equal(Cpu.CauseInterruptTimer, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseInterruptTimer);
        }

        // The case the interval test exists for: one instruction can step the counter past the value entirely.
        [Fact]
        public void A_counter_that_jumps_over_the_comparison_value_still_raises_it()
        {
            var bus = new MemoryBus();
            var cpu = new MipsAssembler()
                .Addiu(1, 0, 7).Addiu(2, 0, 3)
                .Mult(1, 2)
                .Run(0, bus);

            cpu.Pc = MipsAssembler.EntryPoint;
            cpu.NextPc = cpu.Pc + 4;

            // Two adds put the counter at 1; the multiply charges six cycles and lands it at 4.
            cpu.Cop0[Cpu.CompareRegister] = 2;
            cpu.Run(3);

            Assert.Equal(4u, bus.Count);
            Assert.Equal(Cpu.CauseInterruptTimer, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseInterruptTimer);
        }

        [Fact]
        public void A_raised_line_does_nothing_while_it_is_masked_off()
        {
            var cpu = Machine();
            cpu.Cop0[Cpu.CompareRegister] = 3;
            cpu.Cop0[Cpu.StatusRegister] = Cpu.StatusInterruptEnable;

            RunNops(cpu, 10);

            Assert.Equal(Cpu.CauseInterruptTimer, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseInterruptTimer);
            Assert.Null(cpu.LastException);
        }

        [Fact]
        public void A_masked_in_line_interrupts_once_the_enable_bit_is_set()
        {
            var cpu = Machine();
            cpu.Cop0[Cpu.CompareRegister] = 3;
            cpu.Cop0[Cpu.StatusRegister] = Cpu.StatusInterruptEnable | TimerMask;

            Assert.True(StepUntilInterrupted(cpu, 10));
            Assert.Equal(ExceptionCode.Interrupt, cpu.LastException!.Code);
            Assert.Equal(GeneralVector, cpu.Pc);
        }

        [Fact]
        public void The_enable_bit_alone_gates_it()
        {
            var cpu = Machine();
            cpu.Cop0[Cpu.CompareRegister] = 3;
            cpu.Cop0[Cpu.StatusRegister] = TimerMask;

            RunNops(cpu, 10);

            Assert.Null(cpu.LastException);
        }

        // Without this a handler would be interrupted by the line it was entered to service.
        [Fact]
        public void An_exception_already_in_progress_blocks_the_next_interrupt()
        {
            var cpu = Machine();
            cpu.Cop0[Cpu.CompareRegister] = 3;
            cpu.Cop0[Cpu.StatusRegister] = Cpu.StatusInterruptEnable | TimerMask | Cpu.StatusExceptionLevel;

            RunNops(cpu, 10);

            Assert.Null(cpu.LastException);
        }

        [Fact]
        public void Writing_the_comparison_value_is_what_lowers_the_line()
        {
            var cpu = Machine();
            cpu.Cop0[Cpu.CompareRegister] = 3;
            RunNops(cpu, 10);

            var program = new MipsAssembler().Addiu(1, 0, 0x7000).Mtc0(1, Cpu.CompareRegister);
            var bus = program.LoadInto(cpu.Bus);
            cpu.Pc = MipsAssembler.EntryPoint;
            cpu.NextPc = cpu.Pc + 4;
            cpu.Cop0[Cpu.StatusRegister] = 0;
            cpu.Run(2);

            Assert.Equal(0UL, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseInterruptTimer);
        }

        [Fact]
        public void A_device_reaches_the_cpu_through_the_aggregator()
        {
            var bus = new MemoryBus();
            bus.Mi.Mask = MiInterrupt.PeripheralInterface;
            bus.Mi.Raise(MiInterrupt.PeripheralInterface);

            var cpu = Machine(bus);
            cpu.Cop0[Cpu.StatusRegister] = Cpu.StatusInterruptEnable | RcpMask;
            cpu.Step();

            Assert.Equal(ExceptionCode.Interrupt, cpu.LastException!.Code);
            Assert.Equal(Cpu.CauseInterruptRcp, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseInterruptRcp);
        }

        [Fact]
        public void An_unmasked_device_never_reaches_the_line_at_all()
        {
            var bus = new MemoryBus();
            bus.Mi.Raise(MiInterrupt.PeripheralInterface);

            var cpu = Machine(bus);
            cpu.Cop0[Cpu.StatusRegister] = Cpu.StatusInterruptEnable | RcpMask;
            RunNops(cpu, 3);

            Assert.Null(cpu.LastException);
            Assert.Equal(0UL, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseInterruptRcp);
        }

        // Level-triggered: clearing the device's flag lowers the CPU's line with no further action.
        [Fact]
        public void Clearing_the_device_lowers_the_line_again()
        {
            var bus = new MemoryBus();
            bus.Mi.Mask = MiInterrupt.PeripheralInterface;
            bus.Mi.Raise(MiInterrupt.PeripheralInterface);

            var cpu = Machine(bus);
            cpu.Step();
            Assert.Equal(Cpu.CauseInterruptRcp, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseInterruptRcp);

            bus.Mi.Clear(MiInterrupt.PeripheralInterface);
            cpu.Step();

            Assert.Equal(0UL, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseInterruptRcp);
        }

        [Fact]
        public void A_finished_cartridge_transfer_is_one_of_those_devices()
        {
            var bus = new MemoryBus { Cart = Rom() };
            bus.Mi.Mask = MiInterrupt.PeripheralInterface;

            bus.Write32(MemoryMap.PiBase + PiInterface.DramAddress, 0x1000);
            bus.Write32(MemoryMap.PiBase + PiInterface.CartAddress, MemoryMap.CartDomain1Address2);
            bus.Write32(MemoryMap.PiBase + PiInterface.WriteLength, 0x40 - 1);

            var cpu = Machine(bus);
            cpu.Cop0[Cpu.StatusRegister] = Cpu.StatusInterruptEnable | RcpMask;
            cpu.Step();

            Assert.Equal(ExceptionCode.Interrupt, cpu.LastException!.Code);
        }

        [Fact]
        public void The_mask_register_takes_a_pair_of_bits_for_each_device()
        {
            var bus = new MemoryBus();

            bus.Write32(MemoryMap.MiBase + MiInterface.InterruptMask, 1u << 9);
            Assert.Equal(MiInterrupt.PeripheralInterface, bus.Mi.Mask);

            bus.Write32(MemoryMap.MiBase + MiInterface.InterruptMask, 1u << 8);
            Assert.Equal(MiInterrupt.None, bus.Mi.Mask);
        }

        private static EmuSen.Cores.Nintendo.Mars.Rom.RomImage Rom() =>
            EmuSen.Cores.Nintendo.Mars.Rom.RomImage.FromImage(SyntheticN64Rom.Build());
    }
}
