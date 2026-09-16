using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Who may reach coprocessor 1, what it refuses, and its four transfers - see Mars_Fpu.md §3 and §7.
    public class MarsFpuAccessTests
    {
        private const ulong FullMode = Cpu.StatusCop1Usable | Cpu.StatusFpuFullMode;
        private const ulong HalfMode = Cpu.StatusCop1Usable;

        private const uint Scratch = 0x0000_0100;

        [Fact]
        public void A_move_with_the_coprocessor_switched_off_is_refused()
        {
            var cpu = MarsFpuRegisterFileTests.Run(Cpu.StatusFpuFullMode, 1, a => a.Mfc1(2, 0));

            Assert.Equal(ExceptionCode.CoprocessorUnusable, cpu.LastException!.Code);
            Assert.Equal((ulong)ExceptionCode.CoprocessorUnusable << 2, cpu.Cop0[Cpu.CauseRegister] & 0x7C);
        }

        // A handler has to know which coprocessor asked, and the Cause field is the only place that says.
        [Fact]
        public void A_refused_access_names_coprocessor_one_in_the_cause_register()
        {
            var cpu = MarsFpuRegisterFileTests.Run(0, 1, a => a.Mfc1(2, 0));

            Assert.Equal(1UL << 28, cpu.Cop0[Cpu.CauseRegister] & Cpu.CauseCoprocessor);
        }

        [Theory]
        [InlineData(0x31)]
        [InlineData(0x35)]
        [InlineData(0x39)]
        [InlineData(0x3D)]
        public void The_coprocessor_transfers_are_refused_with_it_switched_off_too(uint opcode)
        {
            var cpu = MarsFpuRegisterFileTests.Run(0, 1, a => a.Word((opcode << 26) | (2u << 16)));

            Assert.Equal(ExceptionCode.CoprocessorUnusable, cpu.LastException!.Code);
        }

        // Reserved here means the unit decoded the word and refused it, which is not what the CPU does.
        [Theory]
        [InlineData(0x03)]
        [InlineData(0x07)]
        [InlineData(0x09)]
        [InlineData(0x13)]
        [InlineData(0x1F)]
        public void A_reserved_sub_opcode_is_a_floating_point_fault_and_not_a_reserved_instruction(int rs)
        {
            var cpu = MarsFpuRegisterFileTests.Run(FullMode, 1, a => a.Cop1(rs, 2, 0));

            Assert.Equal(ExceptionCode.FloatingPoint, cpu.LastException!.Code);
            Assert.Equal(Cpu.FcsrCauseUnimplemented, cpu.Fcsr & Cpu.FcsrCauseUnimplemented);
        }

        // The unimplemented cause arrives alone, and the sticky flags beside it are not disturbed.
        [Fact]
        public void An_unimplemented_operation_clears_the_maskable_causes_and_keeps_the_flags()
        {
            var cpu = MarsFpuRegisterFileTests.Run(FullMode, 4, a => a
                .Lui(1, 0x0000).Ori(1, 1, 0xF07C)
                .Ctc1(1, Cpu.FpuControlStatusRegister)
                .Cop1(0x03, 2, 0));

            Assert.Equal(0u, cpu.Fcsr & Cpu.FcsrMaskableCauses);
            Assert.Equal(Cpu.FcsrCauseUnimplemented, cpu.Fcsr & Cpu.FcsrCauseUnimplemented);
            Assert.Equal(0x7Cu, cpu.Fcsr & 0x7Cu);
        }

        // Usability is answered before the decode, so the reserved word never reaches the unit at all.
        [Fact]
        public void An_unusable_coprocessor_answers_for_its_reserved_encodings_before_the_unit_does()
        {
            var cpu = MarsFpuRegisterFileTests.Run(0, 1, a => a.Cop1(0x03, 2, 0));

            Assert.Equal(ExceptionCode.CoprocessorUnusable, cpu.LastException!.Code);
            Assert.Equal(0u, cpu.Fcsr);
        }

        [Fact]
        public void A_word_load_in_full_mode_fills_the_lower_half_and_leaves_the_upper_one()
        {
            var cpu = Loaded(FullMode, 0x0123_4567_89AB_CDEF, 3, f => f[0] = 0x0000_1111_2222_3333, a => a
                .Lui(1, 0x8000)
                .Lwc1(0, 1, (short)(Scratch + 4)));

            Assert.Equal(0x0000_1111_89AB_CDEFUL, cpu.Fpr[0]);
        }

        // The same half-select as a word move, which is what makes one register reachable by two indices.
        [Fact]
        public void Two_word_loads_in_half_mode_fill_one_register_from_two_addresses()
        {
            var cpu = Loaded(HalfMode, 0x0123_4567_89AB_CDEF, 3, _ => { }, a => a
                .Lui(1, 0x8000)
                .Lwc1(0, 1, (short)(Scratch + 4))
                .Lwc1(1, 1, (short)Scratch));

            Assert.Equal(0x0123_4567_89AB_CDEFUL, cpu.Fpr[0]);
        }

        [Fact]
        public void Two_word_stores_in_half_mode_write_one_register_to_two_addresses()
        {
            var bus = new MarsBus();
            MarsFpuRegisterFileTests.Prepared(HalfMode, 3, f => f[0] = 0x0000_1111_2222_3333, a => a
                .Lui(1, 0x8000)
                .Swc1(0, 1, (short)(Scratch + 4))
                .Swc1(1, 1, (short)Scratch), bus);

            Assert.Equal(0x0000_1111_2222_3333UL, bus.Read64(Scratch));
        }

        [Fact]
        public void A_wide_load_in_half_mode_ignores_the_low_bit_of_its_index()
        {
            var cpu = Loaded(HalfMode, 0xFEDC_BA98_7654_3210, 2, f => f[1] = 0, a => a
                .Lui(1, 0x8000)
                .Ldc1(1, 1, (short)Scratch));

            Assert.Equal(0xFEDC_BA98_7654_3210UL, cpu.Fpr[0]);
            Assert.Equal(0UL, cpu.Fpr[1]);
        }

        [Fact]
        public void A_wide_store_in_half_mode_ignores_the_low_bit_of_its_index()
        {
            var bus = new MarsBus();
            MarsFpuRegisterFileTests.Prepared(HalfMode, 2, f => f[0] = 0x0000_1111_2222_3333, a => a
                .Lui(1, 0x8000)
                .Sdc1(1, 1, (short)Scratch), bus);

            Assert.Equal(0x0000_1111_2222_3333UL, bus.Read64(Scratch));
        }

        // A transfer faults on alignment the same way an integer one does, before anything is written.
        [Fact]
        public void A_misaligned_wide_load_faults_without_touching_the_register()
        {
            var cpu = Loaded(FullMode, 0xFEDC_BA98_7654_3210, 2, f => f[2] = 0xAAAA, a => a
                .Lui(1, 0x8000)
                .Ldc1(2, 1, (short)(Scratch + 4)));

            Assert.Equal(ExceptionCode.AddressErrorLoad, cpu.LastException!.Code);
            Assert.Equal(0xAAAAUL, cpu.Fpr[2]);
        }

        private static Cpu Loaded(
            ulong status,
            ulong memory,
            int steps,
            System.Action<ulong[]> registers,
            System.Func<MipsAssembler, MipsAssembler> program)
        {
            var bus = new MarsBus();
            bus.Write64(Scratch, memory);

            return MarsFpuRegisterFileTests.Prepared(status, steps, registers, program, bus);
        }
    }
}
