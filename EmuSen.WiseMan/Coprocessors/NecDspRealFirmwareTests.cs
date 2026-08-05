using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;
using EmuSen.DianaOS.DianaOS.Etc;

namespace EmuSen.WiseMan.Coprocessors
{
    // The only tests here that run Nintendo's own firmware rather than
    // hand-assembled code. That firmware is not in this repo and never will
    // be (see Venus_NecDSP.md §2), so each of these returns early when
    // home/Firmware is empty - they verify a real dump when a developer
    // has one and cost nothing when they don't.
    public class NecDspRealFirmwareTests
    {
        // The DSP-1/2/3/4's LoROM window. The ST01x pair sits somewhere else
        // entirely - see Venus_NecDSP.md §3.
        private const int DrPort = 0x308000;
        private const int SrPort = 0x30C000;

        private static int StatusPort(NecDspVariant variant) =>
            NecDspProfile.IsSt01x(variant) ? 0x600001 : SrPort;

        private static NecDsp? TryLoad(NecDspVariant variant)
        {
            NecDspProfile profile = NecDspProfile.For(variant);
            NecDspFirmware? firmware = NecDspFirmware.FromFirmwareDirectory(profile);
            return firmware == null ? null : new NecDsp(variant, firmware, hiRom: false);
        }

        // Enough budget that the firmware reaches its next idle loop.
        private static void Settle(NecDsp dsp)
        {
            for (int i = 0; i < 500; i++) dsp.Run(50);
        }

        [Theory]
        [InlineData(NecDspVariant.Dsp1)]
        [InlineData(NecDspVariant.Dsp1B)]
        [InlineData(NecDspVariant.Dsp2)]
        [InlineData(NecDspVariant.Dsp3)]
        [InlineData(NecDspVariant.Dsp4)]
        [InlineData(NecDspVariant.St010)]
        [InlineData(NecDspVariant.St011)]
        public void Real_firmware_boots_and_parks_asking_the_host_for_work(NecDspVariant variant)
        {
            NecDsp? dsp = TryLoad(variant);
            if (dsp == null) return;

            Settle(dsp);

            // Every one of these firmwares reaches an RQM idle loop at boot.
            // A zero here means it never got there; anything without bit 15
            // means it is off executing something it shouldn't be.
            Assert.Equal(0x80, dsp.ReadRegister(StatusPort(variant)) & 0x80);
        }

        [Fact]
        public void The_dsp1b_boots_into_8_bit_transfer_mode()
        {
            NecDsp? dsp = TryLoad(NecDspVariant.Dsp1B);
            if (dsp == null) return;

            Settle(dsp);

            // $84 high byte = RQM | DRC: waiting for the host, one byte per
            // DR access - see Venus_NecDSP.md §3.2.
            Assert.Equal(0x84, dsp.ReadRegister(SrPort));
        }

        // The DSP-1's command $00. Operands and result are 1.15 fixed point,
        // so the answer is (a * b) >> 15, not the raw integer product.
        [Theory]
        [InlineData(16384, 8192, 4096)]
        [InlineData(0x7FFF, 0x7FFF, 32766)]
        [InlineData(-16384, 8192, -4096)]
        [InlineData(1000, 2000, 61)]
        [InlineData(12345, -4321, -1628)]
        [InlineData(256, 128, 1)]
        [InlineData(3000, -7, -1)]
        public void Real_dsp1b_firmware_computes_the_documented_multiply(short a, short b, short expected)
        {
            NecDsp? dsp = TryLoad(NecDspVariant.Dsp1B);
            if (dsp == null) return;

            Settle(dsp);
            Send(dsp, 0x00);
            Send(dsp, (byte)a);
            Send(dsp, (byte)(a >> 8));
            Send(dsp, (byte)b);
            Send(dsp, (byte)(b >> 8));

            int low = Receive(dsp);
            int high = Receive(dsp);

            Assert.Equal(expected, (short)(low | (high << 8)));
        }

        private static void Send(NecDsp dsp, byte value)
        {
            Settle(dsp);
            dsp.WriteRegister(DrPort, value);
        }

        private static byte Receive(NecDsp dsp)
        {
            Settle(dsp);
            return dsp.ReadRegister(DrPort);
        }

        [Fact]
        public void The_firmware_directory_is_where_the_man_page_says_it_is()
        {
            Assert.EndsWith(Path.Combine("home", "Firmware"), DianaOSSandbox.FirmwareDirectory);
        }
    }
}
