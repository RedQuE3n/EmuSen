using System;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The twelve conditional traps, which compare all sixty-four bits - see Mars_Cpu.md §15.
    public class MarsCpuTrapTests
    {
        private const ulong Kseg0 = 0xFFFF_FFFF_8000_0000;

        // Every vector here is the corpus's own, which is why the odd-looking pairs are the interesting ones.
        [Theory]
        [InlineData(0x0000_0000_0000_0095UL, 0x0000_0000_0000_0096UL, true)]
        [InlineData(0xFFFF_FFFF_FFFF_FFFFUL, 0x0000_0000_0000_0000UL, true)]
        [InlineData(0x0000_0095_0000_0096UL, 0x0000_0096_0000_0095UL, true)]
        [InlineData(0xFFFF_FFFF_0000_0000UL, 0x0000_0000_0000_0000UL, true)]
        [InlineData(0xFFFF_FFFF_0000_0000UL, 0xFFFF_FFFF_F000_0000UL, true)]
        [InlineData(0xBADD_ECAF_15C0_FFEEUL, 0xBADD_ECAF_15C0_FFEEUL, false)]
        [InlineData(0x0000_0000_0000_0000UL, 0xFFFF_FFFF_FFFF_FFFFUL, false)]
        [InlineData(0x0000_0000_0000_0000UL, 0xFFFF_FFFF_0000_0000UL, false)]
        [InlineData(0xFFFF_FFFF_F000_0000UL, 0xFFFF_FFFF_0000_0000UL, false)]
        public void Trap_if_less_than_reads_both_operands_as_signed(ulong left, ulong right, bool trapped) =>
            Expect(trapped, left, right, a => a.Tlt(2, 3));

        [Theory]
        [InlineData(0x0000_0000_0000_0095UL, 0x0000_0000_0000_0096UL, false)]
        [InlineData(0x0000_0095_0000_0096UL, 0x0000_0096_0000_0095UL, false)]
        [InlineData(0xFFFF_FFFF_0000_0000UL, 0xFFFF_FFFF_F000_0000UL, false)]
        [InlineData(0xBADD_ECAF_15C0_FFEEUL, 0xBADD_ECAF_15C0_FFEEUL, true)]
        [InlineData(0x0000_0000_0000_0000UL, 0xFFFF_FFFF_FFFF_FFFFUL, true)]
        [InlineData(0xFFFF_FFFF_F000_0000UL, 0xFFFF_FFFF_0000_0000UL, true)]
        public void Trap_if_greater_or_equal_is_the_exact_inverse(ulong left, ulong right, bool trapped) =>
            Expect(trapped, left, right, a => a.Tge(2, 3));

        // The pair that separates unsigned from signed: as signed the first operand is the smaller one.
        [Theory]
        [InlineData(0xFFFF_FFFF_0000_0122UL, 0x0FFF_FFFF_0000_0123UL, false)]
        [InlineData(0xFFFF_FFFF_0000_0000UL, 0xFFFF_FFFF_F000_0000UL, true)]
        [InlineData(0x0000_0000_0000_0100UL, 0x0000_0000_0000_0101UL, true)]
        [InlineData(0x0000_0100_0000_0100UL, 0x0000_0101_0000_00FFUL, true)]
        [InlineData(0x0000_0100_0000_0100UL, 0x0000_00FF_0000_0101UL, false)]
        public void Trap_if_less_than_unsigned_reads_both_operands_as_unsigned(
            ulong left, ulong right, bool trapped) =>
            Expect(trapped, left, right, a => a.Tltu(2, 3));

        [Theory]
        [InlineData(0x0FFF_FFFF_0000_0123UL, 0xFFFF_FFFF_0000_0122UL, false)]
        [InlineData(0x0000_0000_0000_0000UL, 0x0000_0000_0000_0000UL, true)]
        [InlineData(0xFFFF_FFFF_F000_0000UL, 0xFFFF_FFFF_0000_0000UL, true)]
        public void Trap_if_greater_or_equal_unsigned_is_the_exact_inverse(
            ulong left, ulong right, bool trapped) =>
            Expect(trapped, left, right, a => a.Tgeu(2, 3));

        [Theory]
        [InlineData(0x0000_0000_0000_0000UL, 0x0000_0000_0000_0000UL, true)]
        [InlineData(0x0000_0100_0000_0000UL, 0x0000_0100_0000_0000UL, true)]
        [InlineData(0x0000_0100_0000_0000UL, 0x0000_0101_0000_0000UL, false)]
        [InlineData(0x0000_0100_0000_0100UL, 0x0000_00FF_0000_0101UL, false)]
        public void Trap_if_equal_compares_the_whole_register(ulong left, ulong right, bool trapped) =>
            Expect(trapped, left, right, a => a.Teq(2, 3));

        [Theory]
        [InlineData(0x0000_0000_0000_0000UL, 0x0000_0000_0000_0000UL, false)]
        [InlineData(0x0000_0100_0000_0000UL, 0x0000_0100_0000_0000UL, false)]
        [InlineData(0x0000_0100_0000_0000UL, 0x0000_0101_0000_0000UL, true)]
        [InlineData(0x0000_0100_0000_0100UL, 0x0000_00FF_0000_0101UL, true)]
        public void Trap_if_not_equal_compares_the_whole_register(ulong left, ulong right, bool trapped) =>
            Expect(trapped, left, right, a => a.Tne(2, 3));

        // The immediate sign-extends to sixty-four bits before an unsigned comparison reads it - see §15.1.
        [Theory]
        [InlineData(0xFFFF_FFFF_FFFF_FFFFUL, (short)-2, false)]
        [InlineData(0xFFFF_FFFF_FFFF_FFFEUL, (short)-2, false)]
        [InlineData(0xFFFF_FFFF_FFFF_FFFDUL, (short)-2, true)]
        [InlineData(0x0000_0000_0000_0000UL, (short)-2, true)]
        [InlineData(0x0000_0000_FFFF_FFFFUL, (short)-2, true)]
        [InlineData(0x8000_0000_0000_0000UL, (short)0, false)]
        [InlineData(0x0000_0000_0000_0001UL, (short)2, true)]
        [InlineData(0x0000_0000_0000_0002UL, (short)2, false)]
        public void Trap_if_less_than_immediate_unsigned_sign_extends_then_compares_unsigned(
            ulong value, short immediate, bool trapped) =>
            Expect(trapped, value, 0, a => a.Tltiu(2, immediate));

        [Theory]
        [InlineData(0xFFFF_FFFF_FFFF_FFFFUL, (short)-2, true)]
        [InlineData(0xFFFF_FFFF_FFFF_FFFDUL, (short)-2, false)]
        [InlineData(0x0000_0000_0000_0000UL, (short)-2, false)]
        public void Trap_if_greater_or_equal_immediate_unsigned_is_the_exact_inverse(
            ulong value, short immediate, bool trapped) =>
            Expect(trapped, value, 0, a => a.Tgeiu(2, immediate));

        [Theory]
        [InlineData(0x8000_0000_0000_0000UL, (short)0, true)]
        [InlineData(0x0000_0000_FFFF_FFFFUL, (short)0, false)]
        [InlineData(0x0000_0000_0000_0001UL, (short)2, true)]
        [InlineData(0xFFFF_FFFF_0000_0002UL, (short)2, true)]
        [InlineData(0xFFFF_FFFF_FFFF_FFFFUL, (short)-2, false)]
        [InlineData(0x0000_0000_FFFF_FFFDUL, (short)-2, false)]
        [InlineData(0xFFFF_FFFF_FFFF_FFFDUL, (short)-2, true)]
        public void Trap_if_less_than_immediate_reads_both_as_signed(
            ulong value, short immediate, bool trapped) =>
            Expect(trapped, value, 0, a => a.Tlti(2, immediate));

        [Theory]
        [InlineData(0x8000_0000_0000_0000UL, (short)0, false)]
        [InlineData(0x0000_0000_FFFF_FFFFUL, (short)0, true)]
        [InlineData(0xFFFF_FFFF_FFFF_FFFFUL, (short)-2, true)]
        [InlineData(0x0000_0000_FFFF_FFFDUL, (short)-2, true)]
        [InlineData(0xFFFF_FFFF_FFFF_FFFDUL, (short)-2, false)]
        public void Trap_if_greater_or_equal_immediate_is_the_exact_inverse(
            ulong value, short immediate, bool trapped) =>
            Expect(trapped, value, 0, a => a.Tgei(2, immediate));

        [Theory]
        [InlineData(0xFFFF_FFFF_FFFF_FFFEUL, (short)-2, true)]
        [InlineData(0x0000_0000_FFFF_FFFEUL, (short)-2, false)]
        [InlineData(0x0000_0000_0000_0002UL, (short)2, true)]
        public void Trap_if_equal_immediate_compares_the_sign_extended_immediate(
            ulong value, short immediate, bool trapped) =>
            Expect(trapped, value, 0, a => a.Teqi(2, immediate));

        [Theory]
        [InlineData(0xFFFF_FFFF_FFFF_FFFEUL, (short)-2, false)]
        [InlineData(0x0000_0000_FFFF_FFFEUL, (short)-2, true)]
        [InlineData(0x0000_0000_0000_0002UL, (short)2, false)]
        public void Trap_if_not_equal_immediate_compares_the_sign_extended_immediate(
            ulong value, short immediate, bool trapped) =>
            Expect(trapped, value, 0, a => a.Tnei(2, immediate));

        // The whole point of the family is the handler, so the fault has to name itself correctly.
        [Fact]
        public void A_taken_trap_reports_itself_as_a_trap_and_saves_its_own_address()
        {
            var cpu = Run(0, 0, a => a.Teq(2, 3));

            Assert.Equal(ExceptionCode.Trap, cpu.LastException!.Code);
            Assert.Equal(0x34UL, cpu.Cop0[Cpu.CauseRegister] & 0xFF);
            Assert.Equal(Kseg0 + 12, cpu.Cop0[Cpu.ExceptionPcRegister]);
        }

        // Rustc emits this after every division, which is how the corpus first ran into the gap - see §15.
        [Fact]
        public void An_untaken_trap_costs_nothing_and_execution_carries_on()
        {
            var cpu = Run(1, 0, a => a.Teq(2, 3).Addiu(4, 0, 0x1234), steps: 5);

            Assert.Null(cpu.LastException);
            Assert.Equal(0x1234UL, cpu.Gpr[4]);
        }

        // Both cases of the corpus's pair: the slot behind a branch is a slot however the branch went.
        [Theory]
        [InlineData(true)]
        [InlineData(false)]
        public void A_trap_in_a_delay_slot_saves_the_branch_and_flags_the_slot(bool taken)
        {
            var cpu = Run(0, 0, a => taken ? a.Beq(0, 0, 1).Teq(2, 3) : a.Bne(0, 0, 1).Teq(2, 3), steps: 5);

            Assert.Equal(ExceptionCode.Trap, cpu.LastException!.Code);
            Assert.Equal(Kseg0 + 12, cpu.Cop0[Cpu.ExceptionPcRegister]);
            Assert.Equal(Cpu.CauseBranchDelay, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseBranchDelay);
        }

        private static void Expect(bool trapped, ulong left, ulong right, Func<MipsAssembler, MipsAssembler> trap)
        {
            var cpu = Run(left, right, trap);

            if (trapped) Assert.Equal(ExceptionCode.Trap, cpu.LastException!.Code);
            else Assert.Null(cpu.LastException);
        }

        // The operands arrive by load, because a sixty-four bit literal is four instructions of noise.
        private static Cpu Run(
            ulong left, ulong right, Func<MipsAssembler, MipsAssembler> trap, int steps = 4)
        {
            var bus = new MarsBus();
            bus.Write64(0x100, left);
            bus.Write64(0x108, right);

            return trap(new MipsAssembler().Lui(1, 0x8000).Ld(2, 1, 0x100).Ld(3, 1, 0x108)).Run(steps, bus);
        }
    }
}
