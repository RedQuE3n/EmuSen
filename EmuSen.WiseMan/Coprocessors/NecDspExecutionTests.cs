using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;
using EmuSen.WiseMan.Fixtures;
using static EmuSen.WiseMan.Fixtures.NecDspFirmwareBuilder;

namespace EmuSen.WiseMan.Coprocessors
{
    // The uPD7725 interpreter: instruction decode, the ALU's flag rules, the
    // multiplier, and the DR/SR handshake the S-CPU talks through. See
    // Venus_NecDSP.md §4/§5. No core and no ROM file - the chip is driven
    // directly with hand-assembled firmware.
    public class NecDspExecutionTests
    {
        // The LoROM register window: $30:8000 is DR, $30:C000 is SR.
        private const int DrPort = 0x308000;
        private const int SrPort = 0x30C000;

        private static NecDsp Chip(params uint[] program) =>
            new(NecDspVariant.Dsp1, NecDspFirmwareBuilder.Dsp1(program), hiRom: false);

        // A master-clock budget generous enough to retire <count> instructions.
        // Anything past the assembled program parks on a self-branch, so an
        // over-generous budget can't change an outcome.
        private static void Steps(NecDsp dsp, int count)
        {
            for (int i = 0; i < count; i++) dsp.Run(3);
        }

        private static int ReadDr(NecDsp dsp) => dsp.ReadRegister(DrPort);

        private static int ReadSr(NecDsp dsp) => dsp.ReadRegister(SrPort);

        // Both halves of DR in one go, low byte first, as a game would take them.
        private static int ReadDr16(NecDsp dsp) => ReadDr(dsp) | (ReadDr(dsp) << 8);

        // --- The host handshake ---

        [Fact]
        public void Ld_writes_an_immediate_into_the_data_register()
        {
            var dsp = Chip(Ld(0x1234, DestDr));
            Steps(dsp, 1);

            Assert.Equal(0x1234, ReadDr16(dsp));
        }

        [Fact]
        public void Writing_dr_raises_the_request_flag_for_the_host()
        {
            var dsp = Chip(Ld(0x1234, DestDr));
            Assert.Equal(0x00, ReadSr(dsp));

            Steps(dsp, 1);

            // RQM is SR bit 15, and SR reads back as its high byte alone.
            Assert.Equal(0x80, ReadSr(dsp));
        }

        [Fact]
        public void Only_the_second_half_of_dr_clears_the_request_flag()
        {
            var dsp = Chip(Ld(0x1234, DestDr));
            Steps(dsp, 1);

            // After the low byte, RQM is still up and DRS has been raised to
            // mark that the host is mid-transfer.
            ReadDr(dsp);
            Assert.Equal(0x90, ReadSr(dsp));

            ReadDr(dsp);
            Assert.Equal(0x00, ReadSr(dsp));
        }

        [Fact]
        public void The_host_can_write_dr_and_the_firmware_reads_it_back()
        {
            var dsp = Chip(
                Op(source: SrcDrNoFlag, dest: DestA),
                Op(source: SrcA, dest: DestDr));

            dsp.WriteRegister(DrPort, 0xCD);
            dsp.WriteRegister(DrPort, 0xAB);
            Steps(dsp, 2);

            Assert.Equal(0xABCD, ReadDr16(dsp));
        }

        [Fact]
        public void Writes_to_the_status_register_are_ignored()
        {
            var dsp = Chip(Ld(0x1234, DestDr));
            Steps(dsp, 1);

            dsp.WriteRegister(SrPort, 0x00);

            Assert.Equal(0x80, ReadSr(dsp));
        }

        // --- ALU ---

        [Fact]
        public void Add_wraps_out_of_bit_15()
        {
            var dsp = Chip(
                Ld(0xFFFF, DestA),
                Ld(0x0002, DestK),
                Op(alu: AluAdd, source: SrcK),
                Op(source: SrcA, dest: DestDr));

            Steps(dsp, 4);

            Assert.Equal(0x0001, ReadDr16(dsp));
        }

        [Fact]
        public void Subtract_borrows_the_same_way()
        {
            var dsp = Chip(
                Ld(0x0001, DestA),
                Ld(0x0002, DestK),
                Op(alu: AluSub, source: SrcK),
                Op(source: SrcA, dest: DestDr));

            Steps(dsp, 4);

            Assert.Equal(0xFFFF, ReadDr16(dsp));
        }

        [Fact]
        public void Xor_of_a_value_with_itself_is_zero()
        {
            var dsp = Chip(
                Ld(0xBEEF, DestA),
                Ld(0xBEEF, DestK),
                Op(alu: AluXor, source: SrcK),
                Op(source: SrcA, dest: DestDr));

            Steps(dsp, 4);

            Assert.Equal(0x0000, ReadDr16(dsp));
        }

        [Fact]
        public void Arithmetic_right_shift_preserves_the_sign_bit()
        {
            var dsp = Chip(
                Ld(0x8000, DestA),
                Op(alu: AluShr),
                Op(source: SrcA, dest: DestDr));

            Steps(dsp, 3);

            Assert.Equal(0xC000, ReadDr16(dsp));
        }

        [Fact]
        public void Byte_swap_exchanges_the_two_halves_of_the_accumulator()
        {
            var dsp = Chip(
                Ld(0xAB12, DestA),
                Op(alu: AluSwap),
                Op(source: SrcA, dest: DestDr));

            Steps(dsp, 3);

            Assert.Equal(0x12AB, ReadDr16(dsp));
        }

        [Fact]
        public void The_opcode_picks_which_accumulator_the_alu_touches()
        {
            var dsp = Chip(
                Ld(0x0005, DestA),
                Ld(0x0007, DestB),
                Ld(0x0003, DestK),
                Op(alu: AluAdd, source: SrcK, accumulator: 1),
                Op(source: SrcA, dest: DestDr));

            Steps(dsp, 5);

            // A is untouched at 5; only B took the add.
            Assert.Equal(0x0005, ReadDr16(dsp));
        }

        [Fact]
        public void A_conditional_branch_reads_the_flags_the_alu_just_set()
        {
            // Zero out A, then branch on A's zero flag past the poison value.
            var dsp = Chip(
                Ld(0x0001, DestA),
                Ld(0x0001, DestK),
                Op(alu: AluSub, source: SrcK),
                Jp(JumpIfAccAZero, 5),
                Ld(0xDEAD, DestDr),
                Ld(0x0060, DestDr));

            Steps(dsp, 5);

            Assert.Equal(0x0060, ReadDr16(dsp));
        }

        // --- Multiplier ---

        [Fact]
        public void The_multiplier_relatches_after_every_instruction()
        {
            // 6 * 7 = 42, and N is the product shifted left one bit.
            var dsp = Chip(
                Ld(0x0006, DestK),
                Ld(0x0007, DestL),
                Op(alu: AluAdd, pSelect: PSelectN),
                Op(source: SrcA, dest: DestDr));

            Steps(dsp, 4);

            Assert.Equal(42 * 2, ReadDr16(dsp));
        }

        [Fact]
        public void The_multiplier_treats_its_inputs_as_signed()
        {
            var dsp = Chip(
                Ld(0xFFFF, DestK),  // -1
                Ld(0x0002, DestL),
                Op(alu: AluAdd, pSelect: PSelectM),
                Op(source: SrcA, dest: DestDr));

            Steps(dsp, 4);

            // -2 >> 15 sign-extends across the whole of M.
            Assert.Equal(0xFFFF, ReadDr16(dsp));
        }

        // --- Internal memories ---

        [Fact]
        public void Ram_writes_and_reads_go_through_the_data_pointer()
        {
            var dsp = Chip(
                Ld(0x0020, DestDp),
                Ld(0x5A5A, DestRam),
                Op(source: SrcRam, dest: DestDr));

            Steps(dsp, 3);

            Assert.Equal(0x5A5A, ReadDr16(dsp));
            Assert.Equal(0x5A5A, dsp.Ram[0x20]);
        }

        [Fact]
        public void Data_rom_is_addressed_by_rp()
        {
            var firmware = NecDspFirmwareBuilder.Dsp1WithData(
                new[]
                {
                    Ld(0x0002, DestRp),
                    Op(source: SrcDataRom, dest: DestDr),
                },
                new ushort[] { 0x1111, 0x2222, 0x3333 });

            var dsp = new NecDsp(NecDspVariant.Dsp1, firmware, hiRom: false);
            Steps(dsp, 2);

            Assert.Equal(0x3333, ReadDr16(dsp));
        }

        // --- The RQM idle loop ---

        [Fact]
        public void A_self_branch_on_rqm_parks_the_chip_instead_of_burning_cycles()
        {
            // Publish a value, then spin until the host takes it.
            var dsp = Chip(
                Ld(0x00FF, DestDr),
                Jp(JumpIfRqmSet, 1),
                Ld(0xDEAD, DestDr));

            Steps(dsp, 200);

            // Still parked on the first value, not run on to the second.
            Assert.Equal(0x00FF, ReadDr16(dsp));
        }

        [Fact]
        public void Draining_dr_releases_the_chip_from_the_idle_loop()
        {
            var dsp = Chip(
                Ld(0x00FF, DestDr),
                Jp(JumpIfRqmSet, 1),
                Ld(0xAAAA, DestDr));

            Steps(dsp, 50);
            ReadDr16(dsp);
            Steps(dsp, 50);

            Assert.Equal(0xAAAA, ReadDr16(dsp));
        }

        // --- Control flow ---

        [Fact]
        public void A_call_pushes_the_return_address_and_rt_pops_it()
        {
            var dsp = Chip(
                Ld(0x0077, DestK),
                Jp(JumpCall, 5),
                Op(source: SrcA, dest: DestDr),   // reached only after the return
                Jp(JumpAlways, 3),
                Jp(JumpAlways, 3),
                Rt(Op(source: SrcK, dest: DestA)));

            Steps(dsp, 4);

            Assert.Equal(0x0077, ReadDr16(dsp));
        }
    }
}
