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
