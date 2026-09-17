using System;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The software float, graded on the corpus's own vectors - see Mars_FpuMath.md.
    public class MarsFpuArithmeticTests
    {
        private const ulong FullMode = Cpu.StatusCop1Usable | Cpu.StatusFpuFullMode;

        private const uint Inexact = 1u << 2;
        private const uint Invalid = 1u << 6;

        // Each row is a hardware measurement: two operands, the sum, and the flags it sets.
        [Theory]
        [InlineData(0x00000000u, 0x40000000u, 0x40000000u, 0u)]
        [InlineData(0x3F800000u, 0x40A00000u, 0x40C00000u, 0u)]
        [InlineData(0xFF7FFFFFu, 0xBF800000u, 0xFF7FFFFFu, Inexact)]
        [InlineData(0x7F7FFFFFu, 0xBF800000u, 0x7F7FFFFFu, Inexact)]
        [InlineData(0x7F7FFFFFu, 0x3F800000u, 0x7F7FFFFFu, Inexact)]
        [InlineData(0xFF7FFFFFu, 0x7F7FFFFFu, 0x00000000u, 0u)]
        [InlineData(0x7F7FFFFFu, 0x00800000u, 0x7F7FFFFFu, Inexact)]
        [InlineData(0x7F800000u, 0xFF800000u, 0x7FBFFFFFu, Invalid)]
        public void A_single_precision_addition_matches_the_measured_result_and_flags(
            uint left, uint right, uint expected, uint flags)
        {
            var cpu = Compute(a => a.Cop1Format(0x10, 0x00, 4, 2, 3), left, right);

            Assert.Equal(expected, (uint)cpu.Fpr[4]);
            Assert.Equal(flags, cpu.Fcsr & 0x7C);
        }

        [Theory]
        [InlineData(0x41800000u, 0x40800000u, 0u)]
        [InlineData(0x40000000u, 0x3FB504F3u, Inexact)]
        [InlineData(0x00000000u, 0x00000000u, 0u)]
        [InlineData(0x80000000u, 0x80000000u, 0u)]
        [InlineData(0x7F800000u, 0x7F800000u, 0u)]
        [InlineData(0xBF800000u, 0x7FBFFFFFu, Invalid)]
        [InlineData(0x3F800000u, 0x3F800000u, 0u)]
        [InlineData(0x40900000u, 0x4007C3B6u, Inexact)]
        public void A_single_precision_square_root_matches_the_measured_result(
            uint operand, uint expected, uint flags)
        {
            var cpu = Compute(a => a.Cop1Format(0x10, 0x04, 4, 2, 0), operand, 0);

            Assert.Equal(expected, (uint)cpu.Fpr[4]);
            Assert.Equal(flags, cpu.Fcsr & 0x7C);
        }

        // The corpus's own boundary: the 64-bit conversions refuse a magnitude of 2^53 or more - see §6.3.
        [Theory]
        [InlineData(0x4010_0000_0000_0000UL, 4L)]
        [InlineData(0x432C_6BF5_2634_0000UL, 4_000_000_000_000_000L)]
        [InlineData(0x433F_FFFF_FFFF_FFFFUL, 9_007_199_254_740_991L)]
        [InlineData(0xC33F_FFFF_FFFF_FFFFUL, -9_007_199_254_740_991L)]
        public void A_double_below_two_to_the_fifty_third_converts_to_a_long(ulong operand, long expected)
        {
            var cpu = ComputeWide(a => a.Cop1Format(0x11, 0x25, 4, 2, 0), operand);

            Assert.Null(cpu.LastException);
            Assert.Equal(expected, (long)cpu.Fpr[4]);
        }

        // Not the target's range and not the source's: the unit simply stops at the double's mantissa.
        [Theory]
        [InlineData(0x4340_0000_0000_0000UL)]
        [InlineData(0xC340_0000_0000_0000UL)]
        [InlineData(0x7FEF_FFFF_FFFF_FFFFUL)]
        public void A_double_of_two_to_the_fifty_third_or_more_refuses_to_convert(ulong operand)
        {
            var cpu = ComputeWide(a => a.Cop1Format(0x11, 0x25, 4, 2, 0), operand);

            Assert.Equal(ExceptionCode.FloatingPoint, cpu.LastException!.Code);
            Assert.Equal(Cpu.FcsrCauseUnimplemented, cpu.Fcsr & Cpu.FcsrCauseUnimplemented);
        }

        // The same boundary from a single, which is what shows it belongs to the unit and not the format.
        [Theory]
        [InlineData(0x59FF_FFFEu, 9_007_198_180_999_168L)]
        [InlineData(0x59FF_FFFFu, 9_007_198_717_870_080L)]
        public void A_single_below_the_same_boundary_converts_to_a_long(uint operand, long expected)
        {
            var cpu = Compute(a => a.Cop1Format(0x10, 0x25, 4, 2, 0), operand, 0);

            Assert.Null(cpu.LastException);
            Assert.Equal(expected, (long)cpu.Fpr[4]);
        }

        [Theory]
        [InlineData(0x5A00_0000u)]
        [InlineData(0x5A00_0001u)]
        public void A_single_at_or_above_the_same_boundary_refuses_to_convert(uint operand)
        {
            var cpu = Compute(a => a.Cop1Format(0x10, 0x25, 4, 2, 0), operand, 0);

            Assert.Equal(ExceptionCode.FloatingPoint, cpu.LastException!.Code);
            Assert.Equal(Cpu.FcsrCauseUnimplemented, cpu.Fcsr & Cpu.FcsrCauseUnimplemented);
        }

        // The 32-bit family keeps its own smaller limit, which is the target's range and not this one.
        [Theory]
        [InlineData(0x41DF_FFFF_FFC0_0000UL, 2_147_483_647L)]
        [InlineData(0xC1E0_0000_0000_0000UL, -2_147_483_648L)]
        public void The_word_conversions_still_stop_at_the_word(ulong operand, long expected)
        {
            var cpu = ComputeWide(a => a.Cop1Format(0x11, 0x24, 4, 2, 0), operand);

            Assert.Null(cpu.LastException);
            Assert.Equal(expected, (int)cpu.Fpr[4]);
        }

        private static Cpu ComputeWide(Func<MipsAssembler, MipsAssembler> program, ulong operand)
        {
            var cpu = program(new MipsAssembler()).Build();

            cpu.Cop0[Cpu.StatusRegister] = FullMode;
            cpu.Fpr[2] = operand;
            cpu.Run(1);

            return cpu;
        }

        private static Cpu Compute(Func<MipsAssembler, MipsAssembler> program, uint left, uint right)
        {
            var cpu = program(new MipsAssembler()).Build();

            cpu.Cop0[Cpu.StatusRegister] = FullMode;
            cpu.Fpr[2] = left;
            cpu.Fpr[3] = right;
            cpu.Run(1);

            return cpu;
        }
    }
}
