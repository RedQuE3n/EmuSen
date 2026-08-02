using EmuSen.Common.Firmware;
using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.WiseMan.Fixtures;
using static EmuSen.WiseMan.Fixtures.NecDspFirmwareBuilder;

namespace EmuSen.WiseMan.Coprocessors
{
    // What the SNES core reports it needs before anything is loaded - the
    // half of the firmware contract a frontend acts on. See
    // EmuSen_Firmware.md §1 and Venus_NecDSP.md §2.
    public class NecDspFirmwareRequestTests : IDisposable
    {
        private readonly string _dir;

        // An empty library, so "is it installed" never depends on whatever
        // dumps happen to be on the machine running this.
        public NecDspFirmwareRequestTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), $"wiseman_fwreq_{Guid.NewGuid():N}");
            Directory.CreateDirectory(_dir);
            FirmwareLibrary.Directory = _dir;
        }

        public void Dispose()
        {
            FirmwareLibrary.ResetDirectory();
            try { Directory.Delete(_dir, recursive: true); } catch (IOException) { }
        }

        private static string RomFile(byte[] rom)
        {
            string path = Path.Combine(Path.GetTempPath(), $"wiseman_fwreq_{Guid.NewGuid():N}.sfc");
            File.WriteAllBytes(path, rom);
            return path;
        }

        // --- Per-variant metadata ---

        [Theory]
        [InlineData(NecDspVariant.Dsp1, "DSP1", "dsp1.rom", 0x2000)]
        [InlineData(NecDspVariant.Dsp1B, "DSP1B", "dsp1b.rom", 0x2000)]
        [InlineData(NecDspVariant.Dsp4, "DSP4", "dsp4.rom", 0x2000)]
        [InlineData(NecDspVariant.St010, "ST010", "st010.rom", 0xD000)]
        [InlineData(NecDspVariant.St011, "ST011", "st011.rom", 0xD000)]
        public void Each_variant_asks_for_its_own_dump(NecDspVariant variant, string chip, string file, int size)
        {
            FirmwareRequest request = NecDspFirmware.RequestFor(variant);

            Assert.Equal("SNES", request.CoreName);
            Assert.Equal(chip, request.ChipName);
            Assert.Equal(file, request.FileName);
            Assert.Equal(size, request.Size);
        }

        // --- Reported from the header, without loading ---

        [Fact]
        public void A_dsp_cartridge_reports_the_dump_it_needs()
        {
            string path = RomFile(SyntheticRom.BuildNecDsp("PILOTWINGS"));

            var required = Cartridge.FirmwareRequirements(path);

            Assert.Equal("dsp1.rom", Assert.Single(required).FileName);
            File.Delete(path);
        }

        [Fact]
        public void A_cartridge_carrying_its_own_firmware_asks_for_nothing()
        {
            byte[] appended = NecDspFirmwareBuilder.Blob(NecDspFirmwareBuilder.Dsp1(Ld(0x1234, DestDr)));
            string path = RomFile(SyntheticRom.BuildNecDsp("PILOTWINGS", appended));

            Assert.Empty(Cartridge.FirmwareRequirements(path));
            File.Delete(path);
        }

        [Fact]
        public void A_plain_cartridge_asks_for_nothing()
        {
            string path = RomFile(SyntheticRom.Build());

            Assert.Empty(Cartridge.FirmwareRequirements(path));
            File.Delete(path);
        }

        [Fact]
        public void An_obc1_cartridge_asks_for_nothing()
        {
            // The OBC1 has no mask ROM to dump - see Venus_OBC1.md §1.
            string path = RomFile(SyntheticRom.BuildObc1());

            Assert.Empty(Cartridge.FirmwareRequirements(path));
            File.Delete(path);
        }

        [Fact]
        public void A_missing_rom_file_reports_nothing_rather_than_throwing()
        {
            Assert.Empty(Cartridge.FirmwareRequirements(Path.Combine(_dir, "does-not-exist.sfc")));
        }

        // --- End to end through the session ---

        [Fact]
        public void The_session_reports_the_dump_as_missing_when_the_library_is_empty()
        {
            string path = RomFile(SyntheticRom.BuildNecDsp("PILOTWINGS"));

            var missing = EmuSen.Common.EmulatorSession.MissingFirmwareFor(path);

            Assert.Equal("dsp1.rom", Assert.Single(missing).FileName);
            File.Delete(path);
        }

        [Fact]
        public void Installing_the_dump_settles_the_request_and_builds_the_chip()
        {
            string path = RomFile(SyntheticRom.BuildNecDsp("PILOTWINGS"));
            FirmwareRequest request = NecDspFirmware.RequestFor(NecDspVariant.Dsp1);

            // Stand in for the file a user picks in the frontend's dialog.
            string picked = RomFile(NecDspFirmwareBuilder.Blob(NecDspFirmwareBuilder.Dsp1(Ld(0x4321, DestDr))));
            Assert.True(FirmwareLibrary.Install(request, picked));

            Assert.Empty(EmuSen.Common.EmulatorSession.MissingFirmwareFor(path));

            // And the core now actually comes up with the chip present.
            var core = SyntheticRom.LoadCore(File.ReadAllBytes(path));
            Assert.NotNull(core.Cart!.NecDsp);

            core.Cart.NecDsp!.Run(3);
            Assert.Equal(0x21, core.Cart.NecDsp.ReadRegister(0x308000));

            File.Delete(path);
            File.Delete(picked);
        }
    }
}
