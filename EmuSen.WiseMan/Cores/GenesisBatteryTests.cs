using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The Genesis's battery files through the frontend: a .srm round trip for save RAM and an EEPROM, the references' two-lane form read, a development build's save copied in, and --nobattery - see Nephrite_Native.md §32.
    [Collection(TestCollections.ProcessGlobals)]
    public class GenesisBatteryTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenGenesisBattery_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public GenesisBatteryTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(true);
            CoreOptions.BatteryRamDisabled = false;
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private string Rom(string name, byte[] image)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, image);
            return path;
        }

        // Wonder Boy in Monster World's board, by its serial: an X24C01 of 128 bytes.
        private static byte[] EepromCartridge()
        {
            byte[] rom = SyntheticMdRom.Cartridge();
            System.Text.Encoding.ASCII.GetBytes("GM G-4060 -00 ").CopyTo(rom, 0x180);
            return rom;
        }

        private static CoreEngine Load(string rom) => Assert.IsType<CoreEngine>(CoreFactory.Load(rom).Core);

        private static byte[] Space(CoreEngine core, int length) => Enumerable.Range(0, length).Select(i => core.ReadSpace("SRAM", i)).ToArray();

        [Theory]
        [InlineData(false, 512)]
        [InlineData(true, 128)]
        public void A_battery_file_round_trips_in_the_genesis_folder(bool eeprom, int length)
        {
            string rom = Rom(eeprom ? "monster.md" : "save.md", eeprom ? EepromCartridge() : SyntheticMdRom.Cartridge(saveBytes: 512));
            byte[] written = Enumerable.Range(0, length).Select(i => (byte)(i * 13 + 5)).ToArray();
            using (var core = Load(rom))
            {
                for (int i = 0; i < length; i++) core.WriteSpace("SRAM", i, written[i]);
                core.SaveSram();
            }
            string path = SaveLibrary.SramPathFor(rom, BatterySave.Genesis);
            Assert.Equal(written, File.ReadAllBytes(path));
            using var again = Load(rom);
            Assert.Equal(written, Space(again, length));
        }

        [Fact]
        public void A_references_two_lane_save_is_read_by_its_lane()
        {
            string rom = Rom("lanes.md", SyntheticMdRom.Cartridge(saveBytes: 512));
            byte[] lane = Enumerable.Range(0, 512).Select(i => (byte)(i ^ 0x5A)).ToArray();
            byte[] both = Enumerable.Repeat((byte)0xFF, 0x10000).ToArray();
            for (int i = 0; i < lane.Length; i++) both[2 * i + 1] = lane[i];
            string path = SaveLibrary.SramPathFor(rom, BatterySave.Genesis);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, both);
            using var core = Load(rom);
            Assert.Equal(lane, Space(core, 512));
            core.SaveSram();
            Assert.Equal(lane, File.ReadAllBytes(path));
        }

        [Fact]
        public void A_save_a_development_build_filed_under_md_is_copied_in_and_left()
        {
            string rom = Rom("old.md", SyntheticMdRom.Cartridge(saveBytes: 512));
            byte[] saved = Enumerable.Range(0, 512).Select(i => (byte)i).ToArray();
            string old = SaveLibrary.SramPathFor(rom, "MD");
            Directory.CreateDirectory(Path.GetDirectoryName(old)!);
            File.WriteAllBytes(old, saved);
            using var core = Load(rom);
            Assert.Equal(saved, Space(core, 512));
            Assert.Equal(saved, File.ReadAllBytes(SaveLibrary.SramPathFor(rom, BatterySave.Genesis)));
            Assert.Equal(saved, File.ReadAllBytes(old));
        }

        [Fact]
        public void With_no_battery_nothing_is_read_or_written()
        {
            string rom = Rom("none.md", SyntheticMdRom.Cartridge(saveBytes: 512));
            string path = SaveLibrary.SramPathFor(rom, BatterySave.Genesis);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, Enumerable.Repeat((byte)0x42, 512).ToArray());
            CoreOptions.BatteryRamDisabled = true;
            using var core = Load(rom);
            Assert.DoesNotContain(Space(core, 512), b => b == 0x42);
            core.WriteSpace("SRAM", 0, 0x99);
            core.SaveSram();
            Assert.All(File.ReadAllBytes(path), b => Assert.Equal(0x42, b));
        }
    }
}
