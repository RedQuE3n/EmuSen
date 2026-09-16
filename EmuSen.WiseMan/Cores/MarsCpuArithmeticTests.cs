using System;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The two integer operations the hardware corpus caught reading the wrong operand width - see Mars_Cpu.md §14.
    public class MarsCpuArithmeticTests
    {
        // The corpus's own SRA vectors, which is where the 64-bit reading came from.
        [Theory]
        [InlineData(0x0000000012345678UL, 4, 0x0000000001234567UL)]
        [InlineData(0x0000000082345678UL, 0, 0xFFFFFFFF82345678UL)]
        [InlineData(0x0123456789ABCDEFUL, 4, 0x00000000789ABCDEUL)]
        [InlineData(0x00000008789ABCDEUL, 4, 0xFFFFFFFF8789ABCDUL)]
        public void An_arithmetic_right_shift_shifts_the_whole_register_before_truncating(
            ulong source, int amount, ulong expected)
        {
            var cpu = Prepared(g => g[2] = source, a => a.Sra(3, 2, amount));

            Assert.Equal(expected, cpu.Gpr[3]);
        }

        [Theory]
        [InlineData(0x0000000012345678UL, 4, 0x0000000001234567UL)]
        [InlineData(0x0000000082345678UL, 0, 0xFFFFFFFF82345678UL)]
        [InlineData(0x0123456789ABCDEFUL, 4, 0x00000000789ABCDEUL)]
        [InlineData(0x00000008789ABCDEUL, 4, 0xFFFFFFFF8789ABCDUL)]
        public void A_variable_arithmetic_right_shift_reads_the_same_width(ulong source, int amount, ulong expected)
        {
            var cpu = Prepared(g => { g[2] = source; g[4] = (ulong)amount; }, a => a.Srav(3, 2, 4));

            Assert.Equal(expected, cpu.Gpr[3]);
        }

        // The shift count is five bits, so the high bits of the register holding it are ignored.
        [Fact]
        public void A_variable_shift_takes_only_five_bits_of_its_count()
        {
            var cpu = Prepared(g => { g[2] = 0x0123456789ABCDEF; g[4] = 0xFFFF_FFFF_FFFF_FFE4; },
                a => a.Srav(3, 2, 4));

            Assert.Equal(0x00000000789ABCDEUL, cpu.Gpr[3]);
        }

        [Theory]
        // Every vector below is a hardware measurement from the corpus - see Mars_Cpu.md §14.
        [InlineData(0x0000000000000000UL, 0x0000000000000000UL, 0x0000000000000000UL, 0x0000000000000000UL)]
        [InlineData(0x0000000000000010UL, 0x0000000000000000UL, 0x0000000000000000UL, 0x0000000000000000UL)]
        [InlineData(0x0000000000000000UL, 0x0000000000000010UL, 0x0000000000000000UL, 0x0000000000000000UL)]
        [InlineData(0x0000000000000010UL, 0x0000000000000010UL, 0x0000000000000000UL, 0x0000000000000100UL)]
        [InlineData(0x00000000FFFFFFFFUL, 0x0000000000000010UL, 0x000000000000000FUL, 0xFFFFFFFFFFFFFFF0UL)]
        [InlineData(0x000000EEFFFFFFFFUL, 0x0000000000000010UL, 0x0000000000000EEFUL, 0xFFFFFFFFFFFFFFF0UL)]
        [InlineData(0x0000000000000010UL, 0x000000EEFFFFFFFFUL, 0xFFFFFFFFFFFFFFEFUL, 0xFFFFFFFFFFFFFFF0UL)]
        [InlineData(0x0000000012345678UL, 0x0000000000012334UL, 0x00000000000014B5UL, 0x0000000030EBF860UL)]
        [InlineData(0x0000000000012334UL, 0x0000000012345678UL, 0x00000000000014B5UL, 0x0000000030EBF860UL)]
        [InlineData(0x00000FF012345678UL, 0x0000000000012334UL, 0x0000000012212175UL, 0x0000000030EBF860UL)]
        [InlineData(0x0000000000012334UL, 0x00000FF012345678UL, 0x00000000000014B5UL, 0x0000000030EBF860UL)]
        [InlineData(0xFFFFFFFFFFFFFFFFUL, 0x0000000000000010UL, 0xFFFFFFFFFFFFFFFFUL, 0xFFFFFFFFFFFFFFF0UL)]
        [InlineData(0x0000000000000010UL, 0xFFFFFFFFFFFFFFFFUL, 0xFFFFFFFFFFFFFFFFUL, 0xFFFFFFFFFFFFFFF0UL)]
        [InlineData(0x0000000F70000001UL, 0x0000000000000010UL, 0x00000000000000F7UL, 0x0000000000000010UL)]
        [InlineData(0x0000000000000010UL, 0x0000000F70000001UL, 0xFFFFFFFFFFFFFFF7UL, 0x0000000000000010UL)]
        [InlineData(0x0000000F00000000UL, 0x0000000000000010UL, 0x00000000000000F0UL, 0x0000000000000000UL)]
        [InlineData(0x0000000000000010UL, 0x0000000F00000000UL, 0xFFFFFFFFFFFFFFF0UL, 0x0000000000000000UL)]
        [InlineData(0x0000000C00000000UL, 0x0000000000000001UL, 0x000000000000000CUL, 0x0000000000000000UL)]
        [InlineData(0x0000000000000001UL, 0x0000000C00000000UL, 0xFFFFFFFFFFFFFFFCUL, 0x0000000000000000UL)]
        [InlineData(0x0000000200000000UL, 0x0000000000000001UL, 0x0000000000000002UL, 0x0000000000000000UL)]
        [InlineData(0x0000000000000001UL, 0x0000000200000000UL, 0x0000000000000002UL, 0x0000000000000000UL)]
        [InlineData(0x0000000400000000UL, 0x0000000000000001UL, 0x0000000000000004UL, 0x0000000000000000UL)]
        [InlineData(0x0000000000000001UL, 0x0000000400000000UL, 0xFFFFFFFFFFFFFFFCUL, 0x0000000000000000UL)]
        [InlineData(0x0000000800000000UL, 0x0000000000000001UL, 0x0000000000000008UL, 0x0000000000000000UL)]
        [InlineData(0x0000000000000001UL, 0x0000000800000000UL, 0x0000000000000000UL, 0x0000000000000000UL)]
        [InlineData(0x0000004000000000UL, 0x0000000000000001UL, 0x0000000000000040UL, 0x0000000000000000UL)]
        [InlineData(0x0000000000000001UL, 0x0000004000000000UL, 0x0000000000000000UL, 0x0000000000000000UL)]
        public void A_signed_multiply_reads_thirty_five_bits_of_its_second_operand(
            ulong rs, ulong rt, ulong hi, ulong lo)
        {
            var cpu = Prepared(g => { g[2] = rs; g[3] = rt; }, a => a.Mult(2, 3));

            Assert.Equal(hi, cpu.Hi);
            Assert.Equal(lo, cpu.Lo);
        }

        // The bit that proves the width: 2^35 truncates to zero, and 2^34 reads as negative.
        [Theory]
        [InlineData(0x0000000800000000UL, 0x0000000000000000UL, 0x0000000000000000UL)]
        [InlineData(0x0000000400000000UL, 0xFFFFFFFFFFFFFFFCUL, 0x0000000000000000UL)]
        [InlineData(0x0000000200000000UL, 0x0000000000000002UL, 0x0000000000000000UL)]
        public void The_thirty_fifth_bit_of_the_second_operand_is_its_sign(ulong rt, ulong hi, ulong lo)
        {
            var cpu = Prepared(g => { g[2] = 1; g[3] = rt; }, a => a.Mult(2, 3));

            Assert.Equal(hi, cpu.Hi);
            Assert.Equal(lo, cpu.Lo);
        }

        // The first operand is not narrowed, so swapping the operands can change the answer.
        [Fact]
        public void A_signed_multiply_is_not_commutative()
        {
            var forward = Prepared(g => { g[2] = 0x000000EE_FFFFFFFF; g[3] = 0x10; }, a => a.Mult(2, 3));
            var reversed = Prepared(g => { g[2] = 0x10; g[3] = 0x000000EE_FFFFFFFF; }, a => a.Mult(2, 3));

            Assert.Equal(0x0000000000000EEFUL, forward.Hi);
            Assert.Equal(0xFFFFFFFFFFFFFFEFUL, reversed.Hi);
        }

        // Unchanged by this slice, and asserted so that the narrowing is not applied to the wrong opcode.
        [Fact]
        public void An_unsigned_multiply_still_reads_thirty_two_bits_of_both_operands()
        {
            var cpu = Prepared(g => { g[2] = 0x000000EE_FFFFFFFF; g[3] = 0x10; }, a => a.Multu(2, 3));

            Assert.Equal(0x000000000000000FUL, cpu.Hi);
            Assert.Equal(0xFFFFFFFFFFFFFFF0UL, cpu.Lo);
        }

        private static Cpu Prepared(Action<ulong[]> registers, Func<MipsAssembler, MipsAssembler> program)
        {
            var cpu = program(new MipsAssembler()).Build();

            registers(cpu.Gpr);
            cpu.Run(1);

            return cpu;
        }
    }
}
