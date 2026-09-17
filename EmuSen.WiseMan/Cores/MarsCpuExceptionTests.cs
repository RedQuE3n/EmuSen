using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // What an exception does to the machine, now that it goes somewhere - see Mars_Cpu.md §11.
    public class MarsCpuExceptionTests
    {
        private const ulong GeneralVector = Cpu.VectorBase + Cpu.VectorOffsetGeneral;

        [Fact]
        public void A_system_call_records_its_cause_and_jumps_to_the_handler()
        {
            var cpu = new MipsAssembler().Nop().Syscall().Run(2);

            Assert.Equal((ulong)ExceptionCode.Syscall << 2, cpu.Cop0[Cpu.CauseRegister] & 0x7C);
            Assert.Equal(Cpu.StatusExceptionLevel, cpu.Cop0[Cpu.StatusRegister] & Cpu.StatusExceptionLevel);
            Assert.Equal(GeneralVector, cpu.Pc);
        }

        // The return address is the faulting instruction itself, not the one after it.
        [Fact]
        public void The_saved_address_points_at_the_instruction_that_faulted()
        {
            var cpu = new MipsAssembler().Nop().Nop().Syscall().Run(3);

            Assert.Equal(MipsAssembler.EntryPoint + 8, cpu.Cop0[Cpu.ExceptionPcRegister]);
        }

        // In a delay slot it points at the branch, because returning to the slot alone would lose the jump.
        [Fact]
        public void A_fault_in_a_delay_slot_saves_the_branch_and_says_so()
        {
            var cpu = new MipsAssembler()
                .Beq(0, 0, 4)
                .Syscall()
                .Run(2);

            Assert.Equal(MipsAssembler.EntryPoint, cpu.Cop0[Cpu.ExceptionPcRegister]);
            Assert.Equal(Cpu.CauseBranchDelay, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseBranchDelay);
        }

        [Fact]
        public void An_ordinary_fault_clears_the_delay_slot_flag_again()
        {
            var cpu = new MipsAssembler()
                .Beq(0, 0, 4).Syscall()
                .Run(2);
            cpu.Cop0[Cpu.StatusRegister] &= ~Cpu.StatusExceptionLevel;

            var second = new MipsAssembler().Nop().Syscall().Run(2);

            Assert.Equal(0UL, second.Cop0[Cpu.CauseRegister] & Cpu.CauseBranchDelay);
        }

        [Fact]
        public void An_address_error_records_the_address_that_caused_it()
        {
            var cpu = new MipsAssembler()
                .Lui(1, 0x8000).Ori(1, 1, 0x0801)
                .Lw(2, 1, 0)
                .Run(3);

            Assert.Equal(ExceptionCode.AddressErrorLoad, cpu.LastException!.Code);
            Assert.Equal(0xFFFF_FFFF_8000_0801UL, cpu.Cop0[Cpu.BadVirtualAddressRegister]);
        }

        // A segment with no translation behind it faults through the refill door rather than the general one.
        [Fact]
        public void An_untranslated_segment_uses_the_refill_vector()
        {
            var cpu = new MipsAssembler()
                .Lui(1, 0x0010)
                .Lw(2, 1, 0)
                .Run(2);

            Assert.Equal(ExceptionCode.TlbLoad, cpu.LastException!.Code);
            Assert.Equal(Cpu.VectorBase + Cpu.VectorOffsetTlbRefill, cpu.Pc);
        }

        [Fact]
        public void A_fault_while_already_handling_one_uses_the_general_vector_and_keeps_the_first_address()
        {
            var bus = new MarsBus();

            // The handler at the general vector faults again on its first instruction.
            bus.Write32(0x180, new MipsAssembler().Syscall().ToArray()[0]);

            var cpu = new MipsAssembler().Nop().Syscall().Run(3, bus);

            Assert.Equal(MipsAssembler.EntryPoint + 4, cpu.Cop0[Cpu.ExceptionPcRegister]);
            Assert.Equal(GeneralVector, cpu.Pc);
        }

        [Fact]
        public void The_bootstrap_flag_moves_the_handler_somewhere_else_entirely()
        {
            var cpu = new MipsAssembler().Nop().Run(1);
            cpu.Cop0[Cpu.StatusRegister] |= Cpu.StatusBootstrapVectors;
            cpu.Pc = MipsAssembler.EntryPoint;
            cpu.NextPc = cpu.Pc + 4;

            var bus = cpu.Bus;
            bus.Write32(0, new MipsAssembler().Syscall().ToArray()[0]);
            cpu.Step();

            Assert.Equal(Cpu.VectorBaseBootstrap + Cpu.VectorOffsetGeneral, cpu.Pc);
        }

        // The whole round trip: fault, run a handler, return, carry on with the next instruction.
        [Fact]
        public void A_handler_can_return_to_where_the_fault_happened()
        {
            var bus = new MarsBus();

            // At the general vector: record the arrival, step the saved address past the fault, and return.
            var handler = new MipsAssembler()
                .Addiu(5, 0, 0x77)
                .Mfc0(6, Cpu.ExceptionPcRegister)
                .Addiu(6, 6, 4)
                .Mtc0(6, Cpu.ExceptionPcRegister)
                .Eret()
                .ToArray();

            for (int i = 0; i < handler.Length; i++) bus.Write32(0x180 + (uint)(i * 4), handler[i]);

            var cpu = new MipsAssembler()
                .Syscall()
                .Addiu(4, 0, 0x42)
                .Run(7, bus);

            Assert.Equal(0x77UL, cpu.Gpr[5]);
            Assert.Equal(0x42UL, cpu.Gpr[4]);
            Assert.Equal(0UL, cpu.Cop0[Cpu.StatusRegister] & Cpu.StatusExceptionLevel);
        }
    }
}
