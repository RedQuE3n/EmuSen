using VenusPpu = EmuSen.Cores.Nintendo.Venus.Video.Ppu;

namespace EmuSen.WiseMan.Ppu
{
    // VMAIN ($2115) bits 2-3, the VRAM address translation modes - see
    // Venus_PPU.md §2.2. These rotate the low bits of the word address the
    // data ports touch while leaving the address counter itself alone, so a
    // game can upload interleaved bitplane data with a plain linear
    // increment. Ignoring them entirely is what scrambled FFMQ's BG3 tile
    // upload into full-screen garbage.
    public class VramAddressTranslationTests
    {
        private const int Vmain = 0x2115;
        private const int Vmaddl = 0x2116;
        private const int Vmaddh = 0x2117;
        private const int Vmdatal = 0x2118;
        private const int Vmdatah = 0x2119;

        // Bit 7 set = increment after the high-byte write; low 2 bits 0 = step 1.
        private static byte Vmain8(int translationMode) => (byte)(0x80 | (translationMode << 2));

        private static VenusPpu NewPpu(int translationMode, int wordAddr)
        {
            var ppu = new VenusPpu();
            ppu.WriteRegister(Vmain, Vmain8(translationMode));
            ppu.WriteRegister(Vmaddl, (byte)(wordAddr & 0xFF));
            ppu.WriteRegister(Vmaddh, (byte)(wordAddr >> 8));
            return ppu;
        }

        private static void WriteWord(VenusPpu ppu, byte low, byte high)
        {
            ppu.WriteRegister(Vmdatal, low);
            ppu.WriteRegister(Vmdatah, high);
        }

        // Where a 16-bit write actually landed, as a word address.
        private static int FindWrittenWord(VenusPpu ppu, byte low, byte high)
        {
            for (int w = 0; w < 0x8000; w++)
            {
                if (ppu.Vram[w * 2] == low && ppu.Vram[w * 2 + 1] == high) return w;
            }
            return -1;
        }

        [Theory]
        [InlineData(0x0000)]
        [InlineData(0x0001)]
        [InlineData(0x1234)]
        [InlineData(0x7FFF)]
        public void Mode_0_writes_straight_through(int addr)
        {
            var ppu = NewPpu(0, addr);

            WriteWord(ppu, 0xAB, 0xCD);

            Assert.Equal(0xAB, ppu.Vram[(addr * 2) & 0xFFFF]);
            Assert.Equal(0xCD, ppu.Vram[((addr * 2) + 1) & 0xFFFF]);
        }

        // Mode 1 rotates the low 8 bits: aaaaaaaaBBBccccc -> aaaaaaaacccccBBB.
        [Theory]
        [InlineData(0x0000, 0x0000)]
        [InlineData(0x0001, 0x0008)] // ccccc=1 moves up 3 bits
        [InlineData(0x0020, 0x0001)] // BBB=1 moves down to the bottom
        [InlineData(0x001F, 0x00F8)]
        [InlineData(0x00FF, 0x00FF)] // all low 8 bits set - rotation is a no-op
        [InlineData(0x3001, 0x3008)] // FFMQ's own BG3 upload range
        [InlineData(0x1240, 0x1202)]
        public void Mode_1_rotates_the_low_eight_bits(int addr, int expected)
        {
            var ppu = NewPpu(1, addr);

            WriteWord(ppu, 0xAB, 0xCD);

            Assert.Equal(expected, FindWrittenWord(ppu, 0xAB, 0xCD));
        }

        // Mode 2 rotates the low 9 bits, mode 3 the low 10.
        [Theory]
        [InlineData(2, 0x0001, 0x0008)]
        [InlineData(2, 0x0040, 0x0001)]
        [InlineData(2, 0x01FF, 0x01FF)]
        [InlineData(3, 0x0001, 0x0008)]
        [InlineData(3, 0x0080, 0x0001)]
        [InlineData(3, 0x03FF, 0x03FF)]
        public void Modes_2_and_3_rotate_nine_and_ten_bits(int mode, int addr, int expected)
        {
            var ppu = NewPpu(mode, addr);

            WriteWord(ppu, 0xAB, 0xCD);

            Assert.Equal(expected, FindWrittenWord(ppu, 0xAB, 0xCD));
        }

        // The high bits above the rotated field must survive untouched,
        // otherwise an upload lands in a completely unrelated VRAM region.
        [Fact]
        public void Translation_leaves_the_high_address_bits_alone()
        {
            var ppu = NewPpu(1, 0x6D21);

            WriteWord(ppu, 0x11, 0x22);

            Assert.Equal(0x6D << 8, FindWrittenWord(ppu, 0x11, 0x22) & 0xFF00);
        }

        // The counter is NOT translated - only the address the data port
        // dereferences is. Getting this backwards would make the rotation
        // compound on itself every write.
        [Fact]
        public void The_address_counter_advances_untranslated()
        {
            var ppu = NewPpu(1, 0x3000);

            WriteWord(ppu, 0x01, 0x02);
            WriteWord(ppu, 0x03, 0x04);

            Assert.Equal(0x3002, ppu.CurrentVramAddr);
        }

        // The scatter pattern the whole feature exists for: 8 consecutive
        // writes with step 1 land 8 words apart, which is exactly one 2bpp
        // tile's stride - that is how a linear DMA fills bitplanes.
        [Fact]
        public void Consecutive_writes_in_mode_1_scatter_by_eight_words()
        {
            var ppu = NewPpu(1, 0x0000);

            for (int i = 0; i < 8; i++) WriteWord(ppu, (byte)(0xF0 | i), 0x00);

            for (int i = 0; i < 8; i++)
            {
                Assert.Equal(0xF0 | i, ppu.Vram[(i * 8) * 2]);
            }
        }

        [Fact]
        public void Reads_use_the_same_translated_address_as_writes()
        {
            var ppu = NewPpu(1, 0x0001);
            WriteWord(ppu, 0x5A, 0xA5);

            // VRAM has to be in place before VMADD is written - writing it is
            // what loads the read latch. See Venus_PPU.md §2.3.
            var reader = new VenusPpu();
            System.Array.Copy(ppu.Vram, reader.Vram, ppu.Vram.Length);
            reader.WriteRegister(Vmain, Vmain8(1));
            reader.WriteRegister(Vmaddl, 0x01);
            reader.WriteRegister(Vmaddh, 0x00);

            Assert.Equal(0x5A, reader.ReadRegister(0x2139));
            Assert.Equal(0xA5, reader.ReadRegister(0x213A));
        }
    }
}
