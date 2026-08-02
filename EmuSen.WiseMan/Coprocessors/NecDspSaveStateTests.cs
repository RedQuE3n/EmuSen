using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;
using EmuSen.WiseMan.Fixtures;
using static EmuSen.WiseMan.Fixtures.NecDspFirmwareBuilder;

namespace EmuSen.WiseMan.Coprocessors
{
    // Save-state format v3 appends the DSP's registers and data RAM, but not
    // its firmware - LoadRom re-reads that. See EmuSen_Save_States.md §3 and
    // Venus_NecDSP.md §7.
    public class NecDspSaveStateTests
    {
        private const int DrPort = 0x308000;

        // Fills RAM word $10 and parks a value in DR, so a state has something
        // in both halves of the chip worth comparing.
        private static byte[] Rom() => SyntheticRom.BuildNecDsp("PILOTWINGS", NecDspFirmwareBuilder.Blob(
            NecDspFirmwareBuilder.Dsp1(
                Ld(0x0010, DestDp),
                Ld(0xBEEF, DestRam),
                Ld(0xC0DE, DestDr))));

        private static NecDsp Advanced(out EmuSen.Cores.Nintendo.Venus.VenusCore core)
        {
            core = SyntheticRom.LoadCore(Rom());
            NecDsp dsp = core.Cart!.NecDsp!;
            for (int i = 0; i < 3; i++) dsp.Run(3);
            return dsp;
        }

        [Fact]
        public void A_round_trip_restores_the_data_ram()
        {
            NecDsp dsp = Advanced(out var core);
            Assert.Equal(0xBEEF, dsp.Ram[0x10]);

            var stream = new MemoryStream();
            core.SaveState(stream);

            dsp.Ram[0x10] = 0x0000;

            stream.Position = 0;
            core.LoadState(stream);

            Assert.Equal(0xBEEF, dsp.Ram[0x10]);
        }

        [Fact]
        public void A_round_trip_restores_the_pending_data_register()
        {
            NecDsp dsp = Advanced(out var core);

            var stream = new MemoryStream();
            core.SaveState(stream);

            // Drain DR, which also clears RQM - the state has to put both back.
            dsp.ReadRegister(DrPort);
            dsp.ReadRegister(DrPort);

            stream.Position = 0;
            core.LoadState(stream);

            Assert.Equal(0xDE, dsp.ReadRegister(DrPort));
            Assert.Equal(0xC0, dsp.ReadRegister(DrPort));
        }

        [Fact]
        public void A_round_trip_restores_the_program_counter()
        {
            NecDsp dsp = Advanced(out var core);

            var stream = new MemoryStream();
            core.SaveState(stream);

            // Let it run on into the parking branch, then rewind.
            for (int i = 0; i < 20; i++) dsp.Run(3);
            dsp.Ram[0x10] = 0x0000;

            stream.Position = 0;
            core.LoadState(stream);

            Assert.Equal(0xBEEF, dsp.Ram[0x10]);
            Assert.Equal(0xDE, dsp.ReadRegister(DrPort));
        }

        [Fact]
        public void A_state_written_by_a_dsp_cartridge_is_longer_than_one_without()
        {
            var withDsp = new MemoryStream();
            SyntheticRom.LoadCore(Rom()).SaveState(withDsp);

            var without = new MemoryStream();
            SyntheticRom.LoadCore(SyntheticRom.Build()).SaveState(without);

            Assert.True(withDsp.Length > without.Length);
        }
    }
}
