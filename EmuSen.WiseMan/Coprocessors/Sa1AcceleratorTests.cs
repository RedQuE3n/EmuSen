using Sa1Chip = EmuSen.Cores.Nintendo.Venus.Coprocessors.Sa1.Sa1;

namespace EmuSen.WiseMan.Coprocessors
{
    // The three accelerators the SA-1 adds on top of its CPU: the arithmetic
    // unit, the variable-length bit stream reader the decompressors are built
    // on, and normal block DMA - see Venus_SA1.md §6-§8.
    public class Sa1AcceleratorTests
    {
        private const ushort Mcnt = 0x2250, MaLow = 0x2251, MaHigh = 0x2252, MbLow = 0x2253, MbHigh = 0x2254;
        private const ushort ResultBase = 0x2306, Overflow = 0x230B;
        private const ushort Vbd = 0x2258, VdaLow = 0x2259, VdpLow = 0x230C, VdpHigh = 0x230D;
        private const ushort Dcnt = 0x2230, SdaLow = 0x2232, DdaLow = 0x2235, DtcLow = 0x2238;

        private static Sa1Chip Chip(byte[]? rom = null) => new(rom ?? new byte[0x100000], new byte[0x8000]);

        private static long ReadResult(Sa1Chip sa1)
        {
            long value = 0;
            for (int i = 4; i >= 0; i--) value = (value << 8) | sa1.ReadRegister((ushort)(ResultBase + i));
            return value;
        }

        private static void SetOperands(Sa1Chip sa1, ushort a, ushort b)
        {
            sa1.WriteRegister(MaLow, (byte)a);
            sa1.WriteRegister(MaHigh, (byte)(a >> 8));
            sa1.WriteRegister(MbLow, (byte)b);
            sa1.WriteRegister(MbHigh, (byte)(b >> 8)); // triggers
        }

        // --- Arithmetic unit ---

        [Fact]
        public void Multiply_is_signed_on_both_operands()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Mcnt, 0x00);
            SetOperands(sa1, 1000, 1000);
            Assert.Equal(1_000_000, ReadResult(sa1));
        }

        [Fact]
        public void Multiply_handles_a_negative_multiplicand()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Mcnt, 0x00);
            SetOperands(sa1, unchecked((ushort)-3), 5);
            Assert.Equal(unchecked((uint)-15), (uint)ReadResult(sa1));
        }

        [Fact]
        public void Multiply_handles_a_negative_multiplier()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Mcnt, 0x00);
            SetOperands(sa1, 5, unchecked((ushort)-3));
            Assert.Equal(unchecked((uint)-15), (uint)ReadResult(sa1));
        }

        // The divisor is the one operand that is unsigned - see Venus_SA1.md §6.1.
        [Fact]
        public void Divide_treats_a_high_bit_divisor_as_unsigned()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Mcnt, 0x01);
            SetOperands(sa1, 30000, 0x8000);

            long result = ReadResult(sa1);
            Assert.Equal(0, result & 0xFFFF);
            Assert.Equal(30000, (result >> 16) & 0xFFFF);
        }

        [Fact]
        public void Divide_puts_the_quotient_low_and_the_remainder_high()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Mcnt, 0x01);
            SetOperands(sa1, 1000, 7);

            long result = ReadResult(sa1);
            Assert.Equal(142, result & 0xFFFF);
            Assert.Equal(6, (result >> 16) & 0xFFFF);
        }

        // Hardware floors rather than truncating, so the remainder stays
        // non-negative - see Venus_SA1.md §6.1.
        [Fact]
        public void Divide_floors_a_negative_dividend()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Mcnt, 0x01);
            SetOperands(sa1, unchecked((ushort)-1000), 7);

            long result = ReadResult(sa1);
            Assert.Equal(unchecked((ushort)-143), (ushort)(result & 0xFFFF));
            Assert.Equal(1, (result >> 16) & 0xFFFF);
        }

        [Fact]
        public void Divide_by_zero_yields_zero_and_returns_the_dividend()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Mcnt, 0x01);
            SetOperands(sa1, 1234, 0);

            long result = ReadResult(sa1);
            Assert.Equal(0, result & 0xFFFF);
            Assert.Equal(1234, (result >> 16) & 0xFFFF);
        }

        [Fact]
        public void Accumulate_sums_successive_products()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Mcnt, 0x02);
            SetOperands(sa1, 100, 100);
            SetOperands(sa1, 200, 200);
            Assert.Equal(10_000 + 40_000, ReadResult(sa1));
        }

        [Fact]
        public void Writing_the_control_register_clears_the_accumulator()
        {
            var sa1 = Chip();
            sa1.WriteRegister(Mcnt, 0x02);
            SetOperands(sa1, 100, 100);
            sa1.WriteRegister(Mcnt, 0x02);
            Assert.Equal(0, ReadResult(sa1));
            Assert.Equal(0x00, sa1.ReadRegister(Overflow));
        }

        // --- Variable-length bit stream ---

        private static Sa1Chip BitStreamChip(params byte[] data)
        {
            byte[] rom = new byte[0x100000];
            data.CopyTo(rom, 0);
            return Chip(rom);
        }

        private static ushort ReadVdp(Sa1Chip sa1)
        {
            byte low = sa1.ReadRegister(VdpLow);
            byte high = sa1.ReadRegister(VdpHigh);
            return (ushort)(low | (high << 8));
        }

        private static void SetStreamAddress(Sa1Chip sa1, uint address)
        {
            sa1.WriteRegister(VdaLow, (byte)address);
            sa1.WriteRegister((ushort)(VdaLow + 1), (byte)(address >> 8));
            sa1.WriteRegister((ushort)(VdaLow + 2), (byte)(address >> 16));
        }

        [Fact]
        public void Bit_stream_reads_a_full_word_at_bit_zero()
        {
            var sa1 = BitStreamChip(0x34, 0x12);
            sa1.WriteRegister(Vbd, 0x80); // auto increment, length 16
            SetStreamAddress(sa1, 0);
            Assert.Equal(0x1234, ReadVdp(sa1));
        }

        [Fact]
        public void Auto_increment_advances_by_the_configured_field_width()
        {
            // 0b...0101 in the low nibble, then the next field above it.
            var sa1 = BitStreamChip(0xA5, 0x00);
            sa1.WriteRegister(Vbd, 0x84); // auto increment, 4-bit fields
            SetStreamAddress(sa1, 0);

            Assert.Equal(0x5, ReadVdp(sa1) & 0x0F);
            Assert.Equal(0xA, ReadVdp(sa1) & 0x0F);
        }

        // A 6-bit field starting at bit 6 spans bits 6-11, so it takes two bits
        // from the first byte and four from the second.
        [Fact]
        public void Fields_can_straddle_a_byte_boundary()
        {
            var sa1 = BitStreamChip(0x00, 0xFF);
            sa1.WriteRegister(Vbd, 0x86);
            SetStreamAddress(sa1, 0);

            Assert.Equal(0x00, ReadVdp(sa1) & 0x3F);
            Assert.Equal(0x3C, ReadVdp(sa1) & 0x3F);
        }

        [Fact]
        public void Loading_the_high_address_byte_restarts_the_stream_at_bit_zero()
        {
            var sa1 = BitStreamChip(0xA5, 0x00);
            sa1.WriteRegister(Vbd, 0x84);
            SetStreamAddress(sa1, 0);
            ReadVdp(sa1);

            SetStreamAddress(sa1, 0);
            Assert.Equal(0x5, ReadVdp(sa1) & 0x0F);
        }

        // --- Normal DMA ---

        [Fact]
        public void Normal_dma_copies_rom_into_iram()
        {
            byte[] rom = new byte[0x100000];
            for (int i = 0; i < 8; i++) rom[i] = (byte)(0xF0 + i);
            var sa1 = Chip(rom);

            sa1.WriteRegister(Dcnt, 0x80); // enable, ROM -> I-RAM
            sa1.WriteRegister(SdaLow, 0x00);
            sa1.WriteRegister((ushort)(SdaLow + 1), 0x80);
            sa1.WriteRegister((ushort)(SdaLow + 2), 0x00);
            sa1.WriteRegister(DtcLow, 8);
            sa1.WriteRegister((ushort)(DtcLow + 1), 0);
            sa1.WriteRegister(DdaLow, 0x10);
            sa1.WriteRegister((ushort)(DdaLow + 1), 0x00); // I-RAM destination triggers here

            for (int i = 0; i < 8; i++) Assert.Equal((byte)(0xF0 + i), sa1.IRam[0x10 + i]);
        }

        [Fact]
        public void Normal_dma_copies_iram_into_bw_ram()
        {
            var sa1 = Chip();
            for (int i = 0; i < 4; i++) sa1.IRam[i] = (byte)(0x11 * (i + 1));

            sa1.WriteRegister(Dcnt, 0x8A); // enable, I-RAM -> BW-RAM
            sa1.WriteRegister(SdaLow, 0x00);
            sa1.WriteRegister((ushort)(SdaLow + 1), 0x00);
            sa1.WriteRegister((ushort)(SdaLow + 2), 0x00);
            sa1.WriteRegister(DtcLow, 4);
            sa1.WriteRegister((ushort)(DtcLow + 1), 0);
            sa1.WriteRegister(DdaLow, 0x00);
            sa1.WriteRegister((ushort)(DdaLow + 1), 0x00);
            sa1.WriteRegister((ushort)(DdaLow + 2), 0x00); // BW-RAM destination triggers here

            for (int i = 0; i < 4; i++) Assert.Equal((byte)(0x11 * (i + 1)), sa1.ReadSa1((uint)(0x400000 + i)));
        }

        [Fact]
        public void Normal_dma_does_not_run_while_the_enable_bit_is_clear()
        {
            byte[] rom = new byte[0x100000];
            rom[0] = 0x5A;
            var sa1 = Chip(rom);

            sa1.WriteRegister(Dcnt, 0x00);
            sa1.WriteRegister(SdaLow, 0x00);
            sa1.WriteRegister((ushort)(SdaLow + 1), 0x80);
            sa1.WriteRegister(DtcLow, 1);
            sa1.WriteRegister(DdaLow, 0x00);
            sa1.WriteRegister((ushort)(DdaLow + 1), 0x00);

            Assert.Equal(0x00, sa1.IRam[0]);
        }
    }
}
