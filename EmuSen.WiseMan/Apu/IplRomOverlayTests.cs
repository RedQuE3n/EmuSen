using EmuSen.Cores.Nintendo.Venus.Apu;

namespace EmuSen.WiseMan.Apu
{
    // The $FFC0-$FFFF IPL ROM is a read-only overlay over ordinary APU RAM,
    // not an image stamped into it - see Venus_APU.md §1.1 and §2.7 for the
    // Super Metroid boot hang that came from getting this wrong.
    public class IplRomOverlayTests
    {
        private const ushort IplBase = 0xFFC0;

        // First two bytes of the stock IPL: MOV X,#$EF.
        private const byte IplFirstByte = 0xCD;

        // CMP Y,$F4 - the transfer loop's "next index arrived?" test, and the
        // byte Super Metroid's upload used to overwrite.
        private const ushort TransferCompare = 0xFFDA;
        private const byte CmpYDp = 0x7E;

        [Fact]
        public void Reset_exposes_the_ipl_rom_with_the_overlay_enabled()
        {
            var spc = new Spc700();
            spc.Reset();

            Assert.True(spc.IplRomEnabled);
            Assert.Equal(0xFFC0, spc.PC);
            Assert.Equal(IplFirstByte, spc.Read8(IplBase));
            Assert.Equal(CmpYDp, spc.Read8(TransferCompare));
        }

        [Fact]
        public void Writes_under_the_overlay_reach_ram_without_changing_what_is_fetched()
        {
            var spc = new Spc700();
            spc.Reset();

            spc.Write8(TransferCompare, 0xE2); // SET1 dp.7 - the corrupting byte

            Assert.Equal(CmpYDp, spc.Read8(TransferCompare));
            Assert.Equal(0xE2, spc.Ram[TransferCompare]);
        }

        [Fact]
        public void Clearing_control_bit_7_reveals_the_ram_underneath()
        {
            var spc = new Spc700();
            spc.Reset();
            spc.Write8(TransferCompare, 0xE2);

            spc.Write8(0x00F1, 0x00);

            Assert.False(spc.IplRomEnabled);
            Assert.Equal(0xE2, spc.Read8(TransferCompare));
        }

        [Fact]
        public void Setting_control_bit_7_brings_the_overlay_back()
        {
            var spc = new Spc700();
            spc.Reset();
            spc.Write8(0x00F1, 0x00);

            spc.Write8(0x00F1, 0x80);

            Assert.True(spc.IplRomEnabled);
            Assert.Equal(IplFirstByte, spc.Read8(IplBase));
        }

        // Timer bits share $F1 with the overlay bit; neither may clobber the other.
        [Fact]
        public void Enabling_timers_does_not_disturb_the_overlay_bit()
        {
            var spc = new Spc700();
            spc.Reset();

            spc.Write8(0x00F1, 0x87);

            Assert.True(spc.IplRomEnabled);
            Assert.Equal(IplFirstByte, spc.Read8(IplBase));
        }
    }
}
