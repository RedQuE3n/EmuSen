using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;

namespace EmuSen.WiseMan.Coprocessors
{
    // Splitting a firmware dump, and recognising one appended to a ROM file.
    // See Venus_NecDSP.md §2.
    public class NecDspFirmwareTests
    {
        private static readonly NecDspProfile Dsp1 = NecDspProfile.For(NecDspVariant.Dsp1);

        [Fact]
        public void A_dsp1_dump_is_6kb_of_program_plus_2kb_of_data()
        {
            Assert.Equal(0x1800, Dsp1.ProgramBytes);
            Assert.Equal(0x800, Dsp1.DataRomBytes);
            Assert.Equal(0x2000, Dsp1.FirmwareBytes);
        }

        [Fact]
        public void The_data_half_is_read_as_little_endian_words()
        {
            byte[] blob = new byte[Dsp1.FirmwareBytes];
            blob[Dsp1.ProgramBytes] = 0xCD;
            blob[Dsp1.ProgramBytes + 1] = 0xAB;

            NecDspFirmware firmware = NecDspFirmware.FromBlob(Dsp1, blob)!;

            Assert.Equal(0xABCD, firmware.DataRom[0]);
            Assert.Equal(Dsp1.DataRomBytes / 2, firmware.DataRom.Length);
        }

        [Fact]
        public void The_program_half_is_taken_verbatim()
        {
            byte[] blob = new byte[Dsp1.FirmwareBytes];
            blob[0] = 0x11;
            blob[Dsp1.ProgramBytes - 1] = 0x22;

            NecDspFirmware firmware = NecDspFirmware.FromBlob(Dsp1, blob)!;

            Assert.Equal(Dsp1.ProgramBytes, firmware.Program.Length);
            Assert.Equal(0x11, firmware.Program[0]);
            Assert.Equal(0x22, firmware.Program[^1]);
        }

        [Fact]
        public void A_dump_of_the_wrong_length_is_rejected_rather_than_padded()
        {
            Assert.Null(NecDspFirmware.FromBlob(Dsp1, new byte[Dsp1.FirmwareBytes - 1]));
            Assert.Null(NecDspFirmware.FromBlob(Dsp1, new byte[Dsp1.FirmwareBytes + 1]));
        }

        [Theory]
        [InlineData(0x8000 + 0x2000, 0x2000)]   // 32KB ROM plus a uPD7725 dump
        [InlineData(0x80000 + 0x2000, 0x2000)]  // 512KB, the usual DSP-1 size
        [InlineData(0x100000 + 0xD000, 0xD000)] // 1MB plus a uPD96050 dump
        public void An_appended_firmware_is_recognised_from_the_file_size(int romSize, int expected)
        {
            Assert.Equal(expected, NecDspFirmware.EmbeddedSize(romSize));
        }

        [Theory]
        [InlineData(0x8000)]
        [InlineData(0x80000)]
        [InlineData(0x100000)]
        public void A_plain_power_of_two_rom_carries_no_firmware(int romSize)
        {
            Assert.Equal(0, NecDspFirmware.EmbeddedSize(romSize));
        }
    }
}
