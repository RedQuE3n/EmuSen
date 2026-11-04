using System;
using System.IO;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // A hosted core's folder of console firmware filled from the player's store, read-only, and the data it carries never asked for - see EmuSen_LibretroHost.md §10s.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class HostedFirmwareRuntimeTests : IDisposable
    {
        // Sapphire's declared name for the Americas BIOS; the test's file only has its size, and is no BIOS.
        private const string Americas = "ps2-0230a-20080220.bin";
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenHostedFirmware_" + Guid.NewGuid().ToString("N"));
        private string Cores => Path.Combine(_root, "Cores");
        private string Entry => Path.Combine(Cores, "libretro", "pcee2");
        private string BiosFolder => Path.Combine(Cores, "libretro", ".data", "pcee2", "system", "pcsx2", "bios");
        // A PlayStation 2 disc image of the test core's own content, the console's engine set to the core under PCEE2's stem.
        private string Game => Path.Combine(_root, "ps2", "game.iso");

        public HostedFirmwareRuntimeTests()
        {
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            LibretroEntryFixture.Lay(Entry, "", "pcee2");
            File.WriteAllText(Path.Combine(Entry, "pcee2_libretro.info"), "display_name = \"EmuSen - Test Core\"\ncorename = \"TestCore\"\nlicense = \"GPLv3\"\nsystemid = \"playstation2\"\ndatabase = \"Sony - PlayStation 2\"\nsupported_extensions = \"iso\"\nfirmware_count = 2\n" +
                "firmware0_desc = \"'pcsx2/bios' folder (any valid PS2 BIOS dump)\"\nfirmware0_path = \"pcsx2/bios\"\nfirmware0_opt = \"false\"\n" +
                "firmware1_desc = \"'pcsx2/resources' folder (shaders, GameIndex.yaml, fonts)\"\nfirmware1_path = \"pcsx2/resources\"\nfirmware1_opt = \"false\"\n");
            Directory.CreateDirectory(Path.GetDirectoryName(Game)!);
            File.WriteAllBytes(Game, "TCG1 a test game"u8.ToArray());
            LibretroEntryFixture.Environment("EMUSEN_CORE_RUNNER", Path.Combine(AppContext.BaseDirectory, "emusen-core-runner"));
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private CrystalRuntime Runtime()
        {
            var runtime = new CrystalRuntime(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory }, PlayerCores = new[] { Cores } });
            runtime.CoreApprove(Convert.ToHexStringLower(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(Path.Combine(Entry, "pcee2_libretro.so")))));
            runtime.SetSetting("PlayStation 2", "Engine", "EmuSen - Test Core");
            return runtime;
        }

        [Fact]
        public void With_no_bios_in_the_store_the_player_is_asked_for_one_bios_file_and_never_for_the_cores_own_data()
        {
            using CrystalRuntime runtime = Runtime();
            using JsonDocument needed = JsonDocument.Parse(runtime.FirmwareNeeded(Game));
            JsonElement only = Assert.Single(needed.RootElement.EnumerateArray());
            Assert.Equal(("ps2-0230j-20080220.bin", 4L << 20, "required"), (only.GetProperty("file").GetString(), only.GetProperty("size").GetInt64(), only.GetProperty("purpose").GetString()));
            Assert.Contains("'pcsx2/bios' folder", only.GetProperty("chip").GetString());
        }

        [Fact]
        public void The_stores_bios_is_linked_read_only_into_the_cores_folder_when_a_game_opens()
        {
            Directory.CreateDirectory(DataStore.Firmware);
            string bios = Path.Combine(DataStore.Firmware, Americas);
            File.WriteAllBytes(bios, new byte[4 << 20]);
            using CrystalRuntime runtime = Runtime();
            Assert.Equal("[]", runtime.FirmwareNeeded(Game));
            using CrystalSession session = runtime.Open(new CrystalOpen { Path = Game, NoBattery = true });
            var link = new FileInfo(Path.Combine(BiosFolder, Americas));
            Assert.Equal(bios, link.LinkTarget);
            Assert.Single(Directory.GetFileSystemEntries(BiosFolder));
        }
    }
}
