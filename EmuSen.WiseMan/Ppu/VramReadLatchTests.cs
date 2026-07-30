using VenusPpu = EmuSen.Cores.Nintendo.Venus.Video.Ppu;

namespace EmuSen.WiseMan.Ppu
{
    // The $2139/$213A read latch - see Venus_PPU.md §2.3. Reads return a
    // prefetched word, not VRAM at the live address: the latch reloads from
    // the CURRENT address and only then does the counter step, so a read lags
    // one word behind. Serving VRAM directly is what displaced every
    // Super Metroid VRAM back-reference by one word.
    public class VramReadLatchTests
    {
        private const int Vmain = 0x2115;
        private const int Vmaddl = 0x2116;
        private const int Vmaddh = 0x2117;
        private const int Rdvraml = 0x2139;
        private const int Rdvramh = 0x213A;

        // Bit 7 set = step after the high-byte read; low 2 bits 0 = step 1.
        private const byte IncrementOnHigh = 0x80;
        private const byte IncrementOnLow = 0x00;

        private static VenusPpu NewPpu(byte vmain, int wordAddr, params (int Word, ushort Value)[] contents)
        {
            var ppu = new VenusPpu();
            foreach (var (word, value) in contents)
            {
                ppu.Vram[word * 2] = (byte)(value & 0xFF);
                ppu.Vram[(word * 2) + 1] = (byte)(value >> 8);
            }
            ppu.WriteRegister(Vmain, vmain);
            ppu.WriteRegister(Vmaddl, (byte)(wordAddr & 0xFF));
            ppu.WriteRegister(Vmaddh, (byte)(wordAddr >> 8));
            return ppu;
        }

        private static ushort ReadWord(VenusPpu ppu)
        {
            byte low = ppu.ReadRegister(Rdvraml);
            byte high = ppu.ReadRegister(Rdvramh);
            return (ushort)(low | (high << 8));
        }

        // Writing VMADD prefetches, so the very first read is already valid.
        [Fact]
        public void Writing_the_address_loads_the_latch()
        {
            var ppu = NewPpu(IncrementOnHigh, 0x0100, (0x0100, 0xBEEF));

            Assert.Equal(0xBEEF, ReadWord(ppu));
        }

        // The idiom Super Metroid's decompressor uses: set the address, read
        // twice, keep the second. Both reads must yield the addressed word -
        // the reload happens at the current address, before the step.
        [Fact]
        public void The_second_read_after_setting_the_address_still_returns_that_word()
        {
            var ppu = NewPpu(IncrementOnHigh, 0x0100, (0x0100, 0xBEEF), (0x0101, 0xCAFE));

            Assert.Equal(0xBEEF, ReadWord(ppu));
            Assert.Equal(0xBEEF, ReadWord(ppu));
            Assert.Equal(0xCAFE, ReadWord(ppu));
        }

        // A dummy read then N reads walks VRAM one word at a time.
        [Fact]
        public void Sequential_reads_after_one_dummy_read_walk_forward()
        {
            var ppu = NewPpu(IncrementOnHigh, 0x0000,
                (0x0000, 0x1111), (0x0001, 0x2222), (0x0002, 0x3333), (0x0003, 0x4444));

            ReadWord(ppu); // dummy

            Assert.Equal(0x1111, ReadWord(ppu));
            Assert.Equal(0x2222, ReadWord(ppu));
            Assert.Equal(0x3333, ReadWord(ppu));
        }

        [Fact]
        public void The_counter_steps_only_on_the_high_port_when_bit_seven_is_set()
        {
            var ppu = NewPpu(IncrementOnHigh, 0x0100, (0x0100, 0xBEEF));

            ppu.ReadRegister(Rdvraml);
            Assert.Equal(0x0100, ppu.CurrentVramAddr);

            ppu.ReadRegister(Rdvramh);
            Assert.Equal(0x0101, ppu.CurrentVramAddr);
        }

        [Fact]
        public void The_counter_steps_only_on_the_low_port_when_bit_seven_is_clear()
        {
            var ppu = NewPpu(IncrementOnLow, 0x0100, (0x0100, 0xBEEF));

            ppu.ReadRegister(Rdvraml);
            Assert.Equal(0x0101, ppu.CurrentVramAddr);

            ppu.ReadRegister(Rdvramh);
            Assert.Equal(0x0101, ppu.CurrentVramAddr);
        }

        // Reading the high port alone must not resurrect the low byte of a
        // word the latch never fetched.
        [Fact]
        public void Both_bytes_come_from_the_same_latched_word()
        {
            var ppu = NewPpu(IncrementOnHigh, 0x0000, (0x0000, 0x1234), (0x0001, 0x5678));

            Assert.Equal(0x34, ppu.ReadRegister(Rdvraml));
            Assert.Equal(0x12, ppu.ReadRegister(Rdvramh));
            Assert.Equal(0x34, ppu.ReadRegister(Rdvraml));
            Assert.Equal(0x12, ppu.ReadRegister(Rdvramh));
        }

        // A write to VMADD mid-sequence re-latches rather than leaving the
        // previous word visible.
        [Fact]
        public void Re_addressing_reloads_the_latch()
        {
            var ppu = NewPpu(IncrementOnHigh, 0x0000, (0x0000, 0x1111), (0x0040, 0x9999));

            Assert.Equal(0x1111, ReadWord(ppu));

            ppu.WriteRegister(Vmaddl, 0x40);
            ppu.WriteRegister(Vmaddh, 0x00);

            Assert.Equal(0x9999, ReadWord(ppu));
        }

        // The latch reload goes through the same VMAIN bits 2-3 rotation the
        // write path uses - see VramAddressTranslationTests.
        [Fact]
        public void The_latch_reload_honours_address_translation()
        {
            // Mode 1 rotates word $0001 to $0008.
            var ppu = NewPpu((byte)(IncrementOnHigh | (1 << 2)), 0x0001, (0x0008, 0x7A7A));

            Assert.Equal(0x7A7A, ReadWord(ppu));
        }
    }
}
