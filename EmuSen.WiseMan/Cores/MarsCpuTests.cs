using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The VR4300's integer core, driven by assembled programs - see Mars_Cpu.md.
    public class MarsCpuTests
    {
        [Fact]
        public void A_thirty_two_bit_result_occupies_the_whole_register_sign_extended()
        {
            var cpu = new MipsAssembler().Addiu(1, 0, -1).Run(1);

            Assert.Equal(0xFFFF_FFFF_FFFF_FFFFUL, cpu.Gpr[1]);
        }

        [Fact]
        public void Register_zero_stays_zero_however_hard_it_is_written()
        {
            var cpu = new MipsAssembler().Addiu(0, 0, 5).Ori(0, 0, 9).Run(2);

            Assert.Equal(0UL, cpu.Gpr[0]);
        }

        // The property the throwing design buys: a trapping instruction leaves no half-written state.
        [Fact]
        public void An_overflowing_add_traps_and_writes_nothing()
        {
            var cpu = new MipsAssembler()
                .Lui(1, 0x7FFF).Ori(1, 1, 0xFFFF)
                .Addi(2, 1, 1)
                .Run(3);

            Assert.NotNull(cpu.LastException);
            Assert.Equal(ExceptionCode.Overflow, cpu.LastException!.Code);
            Assert.Equal(0UL, cpu.Gpr[2]);
        }

        [Fact]
        public void The_unsigned_form_of_the_same_addition_wraps_instead()
        {
            var cpu = new MipsAssembler()
                .Lui(1, 0x7FFF).Ori(1, 1, 0xFFFF)
                .Addiu(2, 1, 1)
                .Run(3);

            Assert.Null(cpu.LastException);
            Assert.Equal(0xFFFF_FFFF_8000_0000UL, cpu.Gpr[2]);
        }

        [Fact]
        public void The_sixty_four_bit_add_keeps_the_upper_half()
        {
            var cpu = new MipsAssembler()
                .Addiu(1, 0, 1).Dsll32(1, 1, 0)
                .Daddiu(2, 1, 5)
                .Run(3);

            Assert.Equal(0x0000_0001_0000_0005UL, cpu.Gpr[2]);
        }

        [Fact]
        public void Signed_and_unsigned_comparison_disagree_about_a_negative_number()
        {
            var cpu = new MipsAssembler()
                .Addiu(1, 0, -1)
                .Slt(2, 1, 0)
                .Sltu(3, 1, 0)
                .Run(3);

            Assert.Equal(1UL, cpu.Gpr[2]);
            Assert.Equal(0UL, cpu.Gpr[3]);
        }

        // The upper half is not shifted in, and the 32-bit result is sign-extended back out.
        [Fact]
        public void A_shift_ignores_the_upper_half_of_its_source_and_sign_extends_its_result()
        {
            var cpu = new MipsAssembler()
                .Addiu(1, 0, 1).Dsll32(1, 1, 0).Ori(1, 1, 0x8000)
                .Sll(2, 1, 16)
                .Run(4);

            Assert.Equal(0xFFFF_FFFF_8000_0000UL, cpu.Gpr[2]);
        }

        [Fact]
        public void Multiply_lands_in_the_two_result_registers_sign_extended()
        {
            var cpu = new MipsAssembler()
                .Addiu(1, 0, -2).Addiu(2, 0, 3)
                .Mult(1, 2).Mflo(3).Mfhi(4)
                .Run(5);

            Assert.Equal(0xFFFF_FFFF_FFFF_FFFAUL, cpu.Gpr[3]);
            Assert.Equal(0xFFFF_FFFF_FFFF_FFFFUL, cpu.Gpr[4]);
        }

        [Fact]
        public void Dividing_by_zero_has_a_defined_answer_rather_than_an_exception()
        {
            var cpu = new MipsAssembler()
                .Addiu(1, 0, 7)
                .Div(1, 0).Mflo(2).Mfhi(3)
                .Run(4);

            Assert.Null(cpu.LastException);
            Assert.Equal(0xFFFF_FFFF_FFFF_FFFFUL, cpu.Gpr[2]);
            Assert.Equal(7UL, cpu.Gpr[3]);
        }

        [Fact]
        public void The_instruction_behind_a_taken_branch_still_runs()
        {
            var cpu = new MipsAssembler()
                .Beq(0, 0, 2)
                .Addiu(1, 0, 11)
                .Addiu(2, 0, 22)
                .Addiu(3, 0, 33)
                .Run(3);

            Assert.Equal(11UL, cpu.Gpr[1]);
            Assert.Equal(0UL, cpu.Gpr[2]);
            Assert.Equal(33UL, cpu.Gpr[3]);
        }

        // The whole point of the likely forms: an untaken one throws its delay slot away.
        [Fact]
        public void A_likely_branch_that_is_not_taken_discards_the_instruction_behind_it()
        {
            var cpu = new MipsAssembler()
                .Addiu(1, 0, 1)
                .Beql(1, 0, 2)
                .Addiu(2, 0, 22)
                .Addiu(3, 0, 33)
                .Run(3);

            Assert.Equal(0UL, cpu.Gpr[2]);
            Assert.Equal(33UL, cpu.Gpr[3]);
        }

        [Fact]
        public void An_ordinary_branch_that_is_not_taken_still_runs_the_instruction_behind_it()
        {
            var cpu = new MipsAssembler()
                .Addiu(1, 0, 1)
                .Beq(1, 0, 2)
                .Addiu(2, 0, 22)
                .Run(3);

            Assert.Equal(22UL, cpu.Gpr[2]);
        }

        // The trap in this family: the link happens whether or not the branch is taken.
        [Fact]
        public void A_linking_branch_writes_the_return_address_even_when_it_does_not_branch()
        {
            var cpu = new MipsAssembler()
                .Addiu(1, 0, -1)
                .Bgezal(1, 4)
                .Nop()
                .Run(3);

            Assert.NotEqual(0UL, cpu.Gpr[31]);
        }

        [Fact]
        public void A_call_returns_past_its_own_delay_slot()
        {
            var cpu = new MipsAssembler()
                .Jal(0x0000_0020)
                .Nop()
                .Run(2);

            Assert.Equal(MipsAssembler.EntryPoint + 8, cpu.Gpr[31]);
        }

        [Fact]
        public void Loads_and_stores_reach_memory_and_sign_extend_as_asked()
        {
            var bus = new MarsBus();
            bus.Write32(0x800, 0xFFFF_8001);

            var cpu = new MipsAssembler()
                .Lui(1, 0x8000).Ori(1, 1, 0x0800)
                .Lw(2, 1, 0)
                .Lwu(3, 1, 0)
                .Lb(4, 1, 0)
                .Lbu(5, 1, 0)
                .Run(6, bus);

            Assert.Equal(0xFFFF_FFFF_FFFF_8001UL, cpu.Gpr[2]);
            Assert.Equal(0x0000_0000_FFFF_8001UL, cpu.Gpr[3]);
            Assert.Equal(0xFFFF_FFFF_FFFF_FFFFUL, cpu.Gpr[4]);
            Assert.Equal(0xFFUL, cpu.Gpr[5]);
        }

        [Fact]
        public void A_store_round_trips_through_the_bus()
        {
            var bus = new MarsBus();

            var cpu = new MipsAssembler()
                .Lui(1, 0x8000).Ori(1, 1, 0x0900)
                .Addiu(2, 0, 0x1234)
                .Sw(2, 1, 0)
                .Run(4, bus);

            Assert.Null(cpu.LastException);
            Assert.Equal(0x1234u, bus.Read32(0x900));
        }

        [Fact]
        public void A_misaligned_load_traps_before_it_writes_its_register()
        {
            var cpu = new MipsAssembler()
                .Lui(1, 0x8000).Ori(1, 1, 0x0801)
                .Addiu(2, 0, 0x55)
                .Lw(2, 1, 0)
                .Run(4);

            Assert.NotNull(cpu.LastException);
            Assert.Equal(ExceptionCode.AddressErrorLoad, cpu.LastException!.Code);
            Assert.Equal(0x55UL, cpu.Gpr[2]);
        }

        [Fact]
        public void A_system_call_raises_its_own_exception_code()
        {
            var cpu = new MipsAssembler().Syscall().Run(1);

            Assert.Equal(ExceptionCode.Syscall, cpu.LastException!.Code);
        }

        // The corpus calls one of these unconditionally before anything else it does.
        [Fact]
        public void The_emulator_extension_opcodes_are_ignored_rather_than_refused()
        {
            var cpu = new MipsAssembler()
                .CoprocessorZeroExtension(0x2C)
                .Addiu(1, 0, 7)
                .Run(2);

            Assert.Null(cpu.LastException);
            Assert.Equal(7UL, cpu.Gpr[1]);
        }

        [Fact]
        public void A_coprocessor_operation_below_that_range_still_refuses()
        {
            var cpu = new MipsAssembler().CoprocessorZeroExtension(0x1F).Run(1);

            Assert.Equal(ExceptionCode.ReservedInstruction, cpu.LastException!.Code);
        }

        // The clock design end to end: nothing increments Count, and it still advances.
        [Fact]
        public void The_count_register_advances_because_instructions_ran()
        {
            var cpu = new MipsAssembler()
                .Nop().Nop().Nop().Nop()
                .Mfc0(1, Cpu.CountRegister)
                .Run(5);

            Assert.Equal(2UL, cpu.Gpr[1]);
        }

        // Rebased, but not stopped: the clock runs under the write, so an immediate read has moved on.
        [Fact]
        public void Writing_the_count_register_rebases_it_without_stopping_it()
        {
            var cpu = new MipsAssembler()
                .Addiu(1, 0, 100)
                .Mtc0(1, Cpu.CountRegister)
                .Mfc0(2, Cpu.CountRegister)
                .Run(3);

            Assert.InRange(cpu.Gpr[2], 100UL, 101UL);
        }

        // The only cycle counts the vendor manual tabulates, charged where it tabulates them.
        [Fact]
        public void A_multiply_costs_what_the_manual_says_and_an_add_costs_one()
        {
            var bus = new MarsBus();
            new MipsAssembler().Addiu(1, 0, 3).Addiu(2, 0, 4).Run(2, bus);
            long afterTwoAdds = bus.Cycles;

            var second = new MarsBus();
            new MipsAssembler().Addiu(1, 0, 3).Addiu(2, 0, 4).Mult(1, 2).Run(3, second);

            Assert.Equal(2, afterTwoAdds);
            Assert.Equal(2 + 1 + Cpu.MultiplyStall, second.Cycles);
        }
    }
}
