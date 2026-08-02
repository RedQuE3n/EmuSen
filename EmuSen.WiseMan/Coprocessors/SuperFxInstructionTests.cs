using Gsu = EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx.SuperFx;

namespace EmuSen.WiseMan.Coprocessors
{
    // The GSU instruction set, driven by assembling tiny programs into ROM and
    // running the chip standalone - see Venus_SuperFX.md §4. Far tighter
    // feedback than inspecting a framebuffer, and it pins the prefix and
    // delay-slot semantics that are easy to get subtly wrong.
    public class SuperFxInstructionTests
    {
        private const ushort Sfr = 0x3030, Clsr = 0x3039, Scmr = 0x303A, Scbr = 0x3038;
        private const ushort R15Low = 0x301E, R15High = 0x301F;

        private const byte FlagZ = 0x02, FlagCy = 0x04, FlagS = 0x08, FlagOv = 0x10;

        // Assembles <program> at ROM offset 0 and runs it until STOP or the
        // clock budget runs out.
        private static Gsu Run(params byte[] program)
        {
            byte[] rom = new byte[0x10000];
            program.CopyTo(rom, 0);

            var gsu = new Gsu(rom, new byte[0x8000]);
            gsu.WriteRegister(Clsr, 0x01); // one master clock per GSU cycle
            gsu.WriteRegister(R15Low, 0x00);
            gsu.WriteRegister(R15High, 0x00); // starts the GSU
            gsu.Run(20000);
            return gsu;
        }

        private static byte Flags(Gsu gsu) => gsu.ReadRegister(Sfr);

        private const byte Stop = 0x00;

        // IWT Rn, #imm16
        private static byte[] Iwt(int n, ushort value) => new[] { (byte)(0xF0 | n), (byte)value, (byte)(value >> 8) };

        // --- Immediates and register moves ---

        [Fact]
        public void Iwt_loads_a_sixteen_bit_immediate()
        {
            var gsu = Run([.. Iwt(1, 0x1234), Stop]);
            Assert.Equal(0x1234, gsu.R[1]);
        }

        [Fact]
        public void Ibt_sign_extends_its_byte_immediate()
        {
            var gsu = Run(0xA3, 0xFE, Stop); // IBT R3, #-2
            Assert.Equal(0xFFFE, gsu.R[3]);
        }

        // FROM Rn only redirects the source; the result still lands in R0.
        [Fact]
        public void From_selects_the_source_and_leaves_the_default_destination()
        {
            var gsu = Run([.. Iwt(1, 5), .. Iwt(2, 3), 0xB1, 0x52, Stop]);
            Assert.Equal(8, gsu.R[0]);
            Assert.Equal(5, gsu.R[1]);
        }

        // TO Rn only redirects the destination; the source stays R0.
        [Fact]
        public void To_selects_the_destination()
        {
            var gsu = Run([.. Iwt(0, 7), .. Iwt(2, 3), 0x14, 0x52, Stop]);
            Assert.Equal(10, gsu.R[4]);
        }

        // WITH Rn redirects both at once.
        [Fact]
        public void With_selects_source_and_destination_together()
        {
            var gsu = Run([.. Iwt(1, 5), .. Iwt(2, 3), 0x21, 0x52, Stop]);
            Assert.Equal(8, gsu.R[1]);
        }

        // With the B flag set by WITH, TO stops being a prefix and becomes a real move.
        [Fact]
        public void To_after_with_becomes_move()
        {
            var gsu = Run([.. Iwt(1, 0xABCD), 0x21, 0x13, Stop]);
            Assert.Equal(0xABCD, gsu.R[3]);
        }

        [Fact]
        public void Prefix_state_does_not_survive_the_instruction_it_applies_to()
        {
            // FROM R1, ADD R2 (-> R0), then a second ADD R2 with the source back at R0.
            var gsu = Run([.. Iwt(1, 5), .. Iwt(2, 3), 0xB1, 0x52, 0x52, Stop]);
            Assert.Equal(11, gsu.R[0]);
        }

        // --- Arithmetic ---

        [Fact]
        public void Add_sets_carry_on_wrap()
        {
            var gsu = Run([.. Iwt(0, 0xFFFF), .. Iwt(1, 2), 0x51, Stop]);
            Assert.Equal(1, gsu.R[0]);
            Assert.Equal(FlagCy, Flags(gsu) & FlagCy);
        }

        [Fact]
        public void Add_sets_overflow_on_signed_wrap()
        {
            var gsu = Run([.. Iwt(0, 0x7FFF), .. Iwt(1, 1), 0x51, Stop]);
            Assert.Equal(FlagOv, Flags(gsu) & FlagOv);
            Assert.Equal(FlagS, Flags(gsu) & FlagS);
        }

        // ALT2 turns the register operand into a 4-bit immediate.
        [Fact]
        public void Alt2_makes_add_take_an_immediate()
        {
            var gsu = Run([.. Iwt(0, 10), 0x3E, 0x55, Stop]);
            Assert.Equal(15, gsu.R[0]);
        }

        [Fact]
        public void Sub_sets_carry_when_no_borrow_is_needed()
        {
            var gsu = Run([.. Iwt(0, 10), .. Iwt(1, 3), 0x61, Stop]);
            Assert.Equal(7, gsu.R[0]);
            Assert.Equal(FlagCy, Flags(gsu) & FlagCy);
        }

        [Fact]
        public void Sub_clears_carry_when_it_borrows()
        {
            var gsu = Run([.. Iwt(0, 3), .. Iwt(1, 10), 0x61, Stop]);
            Assert.Equal(0, Flags(gsu) & FlagCy);
        }

        // ALT3 on the subtract slot is CMP: flags only, no write.
        [Fact]
        public void Alt3_makes_cmp_leave_the_destination_alone()
        {
            var gsu = Run([.. Iwt(0, 5), .. Iwt(1, 5), 0x3F, 0x61, Stop]);
            Assert.Equal(5, gsu.R[0]);
            Assert.Equal(FlagZ, Flags(gsu) & FlagZ);
        }

        [Fact]
        public void Inc_and_dec_operate_on_the_named_register()
        {
            var gsu = Run([.. Iwt(5, 100), 0xD5, 0xD5, 0xE5, Stop]);
            Assert.Equal(101, gsu.R[5]);
        }

        // --- Logic and shifts ---

        [Fact]
        public void And_bic_or_xor_select_on_the_alt_flags()
        {
            Assert.Equal(0x0F00, Run([.. Iwt(0, 0x0FF0), .. Iwt(1, 0xFF00), 0x71, Stop]).R[0]);          // AND
            Assert.Equal(0x00F0, Run([.. Iwt(0, 0x0FF0), .. Iwt(1, 0xFF00), 0x3D, 0x71, Stop]).R[0]);    // BIC
            Assert.Equal(0xFFF0, Run([.. Iwt(0, 0x0FF0), .. Iwt(1, 0xFF00), 0xC1, Stop]).R[0]);          // OR
            Assert.Equal(0xF0F0, Run([.. Iwt(0, 0x0FF0), .. Iwt(1, 0xFF00), 0x3D, 0xC1, Stop]).R[0]);    // XOR
        }

        [Fact]
        public void Not_inverts_every_bit()
        {
            var gsu = Run([.. Iwt(0, 0x0F0F), 0x4F, Stop]);
            Assert.Equal(0xF0F0, gsu.R[0]);
        }

        [Fact]
        public void Lsr_shifts_the_low_bit_into_carry()
        {
            var gsu = Run([.. Iwt(0, 0x0003), 0x03, Stop]);
            Assert.Equal(1, gsu.R[0]);
            Assert.Equal(FlagCy, Flags(gsu) & FlagCy);
        }

        [Fact]
        public void Asr_preserves_the_sign_bit()
        {
            var gsu = Run([.. Iwt(0, 0xFFF0), 0x96, Stop]);
            Assert.Equal(0xFFF8, gsu.R[0]);
        }

        // DIV2 is ASR with the one case that would leave -1 unchanged forced to 0.
        [Fact]
        public void Div2_rounds_minus_one_to_zero()
        {
            Assert.Equal(0, Run([.. Iwt(0, 0xFFFF), 0x3D, 0x96, Stop]).R[0]);
            Assert.Equal(0xFFFF, Run([.. Iwt(0, 0xFFFF), 0x96, Stop]).R[0]);
        }

        [Fact]
        public void Rol_and_ror_pass_through_carry()
        {
            Assert.Equal(0x0002, Run([.. Iwt(0, 0x0001), 0x04, Stop]).R[0]);
            Assert.Equal(0x0001, Run([.. Iwt(0, 0x0002), 0x97, Stop]).R[0]);
        }

        [Fact]
        public void Swap_lob_hib_and_sex_reshape_bytes()
        {
            Assert.Equal(0x3412, Run([.. Iwt(0, 0x1234), 0x4D, Stop]).R[0]);
            Assert.Equal(0x0034, Run([.. Iwt(0, 0x1234), 0x9E, Stop]).R[0]);
            Assert.Equal(0x0012, Run([.. Iwt(0, 0x1234), 0xC0, Stop]).R[0]);
            Assert.Equal(0xFF80, Run([.. Iwt(0, 0x0080), 0x95, Stop]).R[0]);
        }

        // --- Multiply ---

        [Fact]
        public void Mult_is_signed_and_umult_is_not()
        {
            // -2 * 3
            Assert.Equal(0xFFFA, Run([.. Iwt(0, 0x00FE), .. Iwt(1, 0x0003), 0x81, Stop]).R[0]);
            // 254 * 3
            Assert.Equal(762, Run([.. Iwt(0, 0x00FE), .. Iwt(1, 0x0003), 0x3D, 0x81, Stop]).R[0]);
        }

        // FMULT keeps the high word of a 16x16 product against R6.
        [Fact]
        public void Fmult_returns_the_high_word()
        {
            var gsu = Run([.. Iwt(0, 0x4000), .. Iwt(6, 0x4000), 0x9F, Stop]);
            Assert.Equal(0x1000, gsu.R[0]);
        }

        // LMULT additionally parks the low word in R4.
        [Fact]
        public void Lmult_also_keeps_the_low_word_in_r4()
        {
            var gsu = Run([.. Iwt(0, 0x1234), .. Iwt(6, 0x0100), 0x3D, 0x9F, Stop]);
            Assert.Equal(0x0012, gsu.R[0]);
            Assert.Equal(0x3400, gsu.R[4]);
        }

        // --- Control flow ---

        // Every branch runs the instruction after it before taking effect.
        [Fact]
        public void Branches_execute_their_delay_slot()
        {
            // BRA +2 ; INC R5 (delay slot) ; INC R5 (skipped) ; STOP
            var gsu = Run([.. Iwt(5, 0), 0x05, 0x02, 0xD5, 0xD5, Stop]);
            Assert.Equal(1, gsu.R[5]);
        }

        // A not-taken branch simply falls through, so the "delay slot" and the
        // instruction after it both run in program order.
        [Fact]
        public void A_not_taken_branch_still_runs_the_next_instruction()
        {
            // CMP R0(1) against R1(2) leaves Z clear, so BEQ is not taken.
            var gsu = Run([.. Iwt(5, 0), .. Iwt(0, 1), .. Iwt(1, 2), 0x3F, 0x61, 0x09, 0x02, 0xD5, 0xD5, Stop]);
            Assert.Equal(2, gsu.R[5]);
        }

        [Fact]
        public void Bne_is_taken_when_the_compare_differed()
        {
            var gsu = Run([.. Iwt(5, 0), .. Iwt(0, 1), .. Iwt(1, 2), 0x3F, 0x61, 0x08, 0x02, 0xD5, 0xD5, Stop]);
            Assert.Equal(1, gsu.R[5]);
        }

        [Fact]
        public void Jmp_transfers_control_to_a_register()
        {
            // R8 points past the INC that would otherwise run.
            var gsu = Run([.. Iwt(5, 0), .. Iwt(8, 11), 0x98, 0xD5, 0xD5, Stop]);
            Assert.Equal(1, gsu.R[5]);
            Assert.Equal(11, gsu.R[8]);
        }

        // LOOP decrements R12 and jumps to R13 until it hits zero.
        [Fact]
        public void Loop_repeats_until_the_counter_drains()
        {
            // R12 = 4 iterations, R13 = address of the INC at offset 9.
            var gsu = Run([.. Iwt(5, 0), .. Iwt(12, 4), .. Iwt(13, 9), 0xD5, 0x3C, 0x01, Stop]);
            Assert.Equal(4, gsu.R[5]);
            Assert.Equal(0, gsu.R[12]);
        }

        [Fact]
        public void Stop_clears_go_and_raises_the_interrupt()
        {
            var gsu = Run(Stop);
            Assert.False(gsu.Running);
            Assert.True(gsu.ScpuIrqPending);
        }

        [Fact]
        public void Reading_the_status_high_byte_acknowledges_the_interrupt()
        {
            var gsu = Run(Stop);
            Assert.True(gsu.ScpuIrqPending);
            gsu.ReadRegister(0x3031);
            Assert.False(gsu.ScpuIrqPending);
        }

        // --- Game Pak RAM ---

        [Fact]
        public void Sm_and_lm_round_trip_a_word_through_ram()
        {
            // SM ($0100), R1 then LM R2, ($0100)
            var gsu = Run([.. Iwt(1, 0xBEEF), 0x3E, 0xF1, 0x00, 0x01, 0x3D, 0xF2, 0x00, 0x01, Stop]);
            Assert.Equal(0xBEEF, gsu.R[2]);
        }

        [Fact]
        public void Stw_and_ldw_use_a_register_as_the_address()
        {
            // R3 = $0200; STW (R3) from R1; LDW (R3) into R2.
            var gsu = Run([.. Iwt(1, 0x1234), .. Iwt(3, 0x0200), 0xB1, 0x33, 0x14, 0x43, Stop]);
            Assert.Equal(0x1234, gsu.R[4]);
        }

        [Fact]
        public void Stb_and_ldb_touch_only_one_byte()
        {
            var gsu = Run([.. Iwt(1, 0x00AB), .. Iwt(3, 0x0300), 0xB1, 0x3D, 0x33, 0x14, 0x3D, 0x43, Stop]);
            Assert.Equal(0x00AB, gsu.R[4]);
        }

        // --- Plotting ---

        // One plotted pixel becomes one bit in each of the four bitplanes.
        [Fact]
        public void Plot_writes_a_pixel_into_the_framebuffer_bitplanes()
        {
            byte[] rom = new byte[0x10000];
            byte[] program =
            [
                .. Iwt(1, 0),      // x = 0
                .. Iwt(2, 0),      // y = 0
                .. Iwt(0, 0x0005), // colour 5
                0x4E,              // COLOR
                0x4C,              // PLOT
                Stop,
            ];
            program.CopyTo(rom, 0);

            var gsu = new Gsu(rom, new byte[0x8000]);
            gsu.WriteRegister(Clsr, 0x01);
            gsu.WriteRegister(Scmr, 0x01); // 16 colour, 128-line
            gsu.WriteRegister(Scbr, 0x00);
            gsu.WriteRegister(R15Low, 0x00);
            gsu.WriteRegister(R15High, 0x00);
            gsu.Run(20000);

            // Colour 5 = planes 0 and 2 set, leftmost pixel is bit 7.
            Assert.Equal(0x80, gsu.ReadRam(0));
            Assert.Equal(0x00, gsu.ReadRam(1));
            Assert.Equal(0x80, gsu.ReadRam(16));
            Assert.Equal(0x00, gsu.ReadRam(17));
        }

        [Fact]
        public void Plot_advances_x_so_a_run_of_pixels_fills_a_row()
        {
            byte[] rom = new byte[0x10000];
            byte[] program =
            [
                .. Iwt(1, 0),
                .. Iwt(2, 0),
                .. Iwt(0, 0x0001),
                0x4E,              // COLOR 1
                0x4C, 0x4C, 0x4C,  // PLOT x3
                Stop,
            ];
            program.CopyTo(rom, 0);

            var gsu = new Gsu(rom, new byte[0x8000]);
            gsu.WriteRegister(Clsr, 0x01);
            gsu.WriteRegister(Scmr, 0x01);
            gsu.WriteRegister(R15Low, 0x00);
            gsu.WriteRegister(R15High, 0x00);
            gsu.Run(20000);

            Assert.Equal(0xE0, gsu.ReadRam(0)); // three leftmost pixels
            Assert.Equal(3, gsu.R[1]);
        }

        // Colour 0 is skipped unless POR says to plot it.
        [Fact]
        public void Colour_zero_is_transparent_by_default()
        {
            byte[] rom = new byte[0x10000];
            byte[] program =
            [
                .. Iwt(1, 0), .. Iwt(2, 0), .. Iwt(0, 0x000F),
                0x4E, 0x4C,        // COLOR 15, PLOT
                .. Iwt(0, 0x0000),
                0x4E,              // COLOR 0
                .. Iwt(1, 0),
                0x4C,              // PLOT over the same pixel
                Stop,
            ];
            program.CopyTo(rom, 0);

            var gsu = new Gsu(rom, new byte[0x8000]);
            gsu.WriteRegister(Clsr, 0x01);
            gsu.WriteRegister(Scmr, 0x01);
            gsu.WriteRegister(R15Low, 0x00);
            gsu.WriteRegister(R15High, 0x00);
            gsu.Run(20000);

            Assert.Equal(0x80, gsu.ReadRam(0)); // still colour 15's bit
        }
    }
}
