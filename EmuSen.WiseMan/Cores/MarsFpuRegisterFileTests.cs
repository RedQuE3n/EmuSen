using System;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The coprocessor-1 register file, its two control registers and nothing numeric - see Mars_Fpu.md.
    public class MarsFpuRegisterFileTests
    {
        private const ulong FullMode = Cpu.StatusCop1Usable | Cpu.StatusFpuFullMode;
        private const ulong HalfMode = Cpu.StatusCop1Usable;

        [Fact]
        public void A_wide_move_survives_a_round_trip_through_the_register_file()
        {
            var cpu = Run(FullMode, 4, a => a
                .Lui(1, 0x1234).Ori(1, 1, 0x5678)
                .Dmtc1(1, 3)
                .Dmfc1(2, 3));

            Assert.Equal(0x1234_5678UL, cpu.Fpr[3]);
            Assert.Equal(0x1234_5678UL, cpu.Gpr[2]);
        }

        // The upper half of the register is not part of a word move, which a naive store would lose.
        [Fact]
        public void In_full_mode_a_word_write_leaves_the_upper_half_standing()
        {
            var cpu = Prepared(FullMode, 3, f => f[0] = 0x0000_1111_2222_3333, a => a
                .Lui(1, 0x3333).Ori(1, 1, 0x2222)
                .Mtc1(1, 0));

            Assert.Equal(0x0000_1111_3333_2222UL, cpu.Fpr[0]);
        }

        [Fact]
        public void In_full_mode_a_word_read_takes_the_lower_half()
        {
            var cpu = Prepared(FullMode, 1, f => f[1] = 0x4444_5555_6666_7777, a => a.Mfc1(2, 1));

            Assert.Equal(0x6666_7777u, (uint)cpu.Gpr[2]);
        }

        // Half mode is the whole of the FR bit: an odd index names the upper half of its pair.
        [Theory]
        [InlineData(0, 0x2222_3333u)]
        [InlineData(1, 0x0000_1111u)]
        [InlineData(2, 0xAAAA_BBBBu)]
        [InlineData(3, 0x8888_9999u)]
        public void In_half_mode_an_odd_index_reads_the_upper_half_of_its_pair(int index, uint expected)
        {
            var cpu = Prepared(HalfMode, 1, f =>
            {
                f[0] = 0x0000_1111_2222_3333;
                f[2] = 0x8888_9999_AAAA_BBBB;
            }, a => a.Mfc1(2, index));

            Assert.Equal(expected, (uint)cpu.Gpr[2]);
        }

        [Fact]
        public void In_half_mode_two_word_writes_fill_one_register_between_them()
        {
            var cpu = Prepared(HalfMode, 6, _ => { }, a => a
                .Lui(1, 0x3333).Ori(1, 1, 0x2222)
                .Mtc1(1, 0)
                .Lui(1, 0x7777).Ori(1, 1, 0x6666)
                .Mtc1(1, 1));

            Assert.Equal(0x7777_6666_3333_2222UL, cpu.Fpr[0]);
            Assert.Equal(0UL, cpu.Fpr[1]);
        }

        // A wide access in half mode drops the index's low bit, so the odd register cannot be reached.
        [Fact]
        public void In_half_mode_a_wide_write_to_an_odd_index_lands_on_the_even_register()
        {
            var cpu = Prepared(HalfMode, 3, f => f[5] = 0xDEAD_BEEF_DEAD_BEEF, a => a
                .Lui(1, 0x0123).Ori(1, 1, 0x4567)
                .Dmtc1(1, 5));

            Assert.Equal(0x0123_4567UL, cpu.Fpr[4]);
            Assert.Equal(0xDEAD_BEEF_DEAD_BEEFUL, cpu.Fpr[5]);
        }

        [Fact]
        public void In_half_mode_a_wide_read_of_an_odd_index_returns_the_even_register()
        {
            var cpu = Prepared(HalfMode, 1, f => f[4] = 0x0011_0011_2233_2233, a => a.Dmfc1(2, 5));

            Assert.Equal(0x0011_0011_2233_2233UL, cpu.Gpr[2]);
        }

        // The mode is a COP0 bit, so a program changes it the same way it changes anything else there.
        [Fact]
        public void A_program_can_change_the_register_file_mode_for_itself()
        {
            var cpu = Prepared(FullMode, 4, f => f[0] = 0x0000_1111_2222_3333, a => a
                .Lui(8, 0x2000)
                .Mtc0(8, Cpu.StatusRegister)
                .Nop()
                .Mfc1(2, 1));

            Assert.Equal(0x0000_1111u, (uint)cpu.Gpr[2]);
        }

        [Fact]
        public void The_implementation_register_reads_its_fixed_value_and_refuses_every_write()
        {
            var cpu = Run(FullMode, 4, a => a
                .Cfc1(2, Cpu.FpuImplementationRegister)
                .Lui(1, 0x0123)
                .Ctc1(1, Cpu.FpuImplementationRegister)
                .Cfc1(3, Cpu.FpuImplementationRegister));

            Assert.Equal(Cpu.FpuImplementation, (uint)cpu.Gpr[2]);
            Assert.Equal(Cpu.FpuImplementation, (uint)cpu.Gpr[3]);
        }

        // Twelve bits of the control word are not storage, and a handler reading them back must see zero.
        [Theory]
        [InlineData(0xFFFDF07Fu, 0x0181F07Fu)]
        [InlineData(0xFFFC0F83u, 0x01800F83u)]
        public void The_control_word_drops_the_bits_that_are_not_writable(uint written, uint expected)
        {
            var cpu = Run(FullMode, 3, a => a
                .Lui(1, (ushort)(written >> 16)).Ori(1, 1, (ushort)written)
                .Ctc1(1, Cpu.FpuControlStatusRegister));

            Assert.Equal(expected, cpu.Fcsr);
            Assert.Null(cpu.LastException);
        }

        [Fact]
        public void A_cause_without_its_enable_is_recorded_and_not_raised()
        {
            var cpu = Run(FullMode, 3, a => a
                .Lui(1, 0x0000).Ori(1, 1, 0x4000)
                .Ctc1(1, Cpu.FpuControlStatusRegister));

            Assert.Equal(0x4000u, cpu.Fcsr);
            Assert.Null(cpu.LastException);
        }

        // Writing a cause beside its enable fires the exception by hand, which is how the corpus reaches it.
        [Fact]
        public void A_cause_written_beside_its_enable_raises_a_floating_point_exception()
        {
            var cpu = Run(FullMode, 3, a => a
                .Lui(1, 0x0000).Ori(1, 1, 0x4200)
                .Ctc1(1, Cpu.FpuControlStatusRegister));

            Assert.Equal(ExceptionCode.FloatingPoint, cpu.LastException!.Code);
            Assert.Equal((ulong)ExceptionCode.FloatingPoint << 2, cpu.Cop0[Cpu.CauseRegister] & 0x7C);
            Assert.Equal(MipsAssembler.EntryPoint + 8, cpu.Cop0[Cpu.ExceptionPcRegister]);
        }

        // The control word keeps what the program wrote: this exception does not rewrite its own cause.
        [Fact]
        public void The_control_word_a_raising_write_left_behind_is_the_one_that_was_written()
        {
            var cpu = Run(FullMode, 3, a => a
                .Lui(1, 0x0000).Ori(1, 1, 0x4200)
                .Ctc1(1, Cpu.FpuControlStatusRegister));

            Assert.Equal(0x4200u, cpu.Fcsr);
        }

        [Fact]
        public void An_exception_that_names_no_coprocessor_leaves_the_cause_field_at_zero()
        {
            var cpu = new MipsAssembler().Syscall().Run(1);

            Assert.Equal(0UL, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseCoprocessor);
        }

        internal static Cpu Run(ulong status, int steps, Func<MipsAssembler, MipsAssembler> program) =>
            Prepared(status, steps, _ => { }, program);

        internal static Cpu Prepared(
            ulong status,
            int steps,
            Action<ulong[]> registers,
            Func<MipsAssembler, MipsAssembler> program,
            MarsBus? bus = null)
        {
            var cpu = program(new MipsAssembler()).Build(bus);

            cpu.Cop0[Cpu.StatusRegister] = status;
            registers(cpu.Fpr);
            cpu.Run(steps);

            return cpu;
        }
    }
}
