using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Which Sega CD BIOS a session boots, through the path a frontend takes: the firmware folder, CoreFactory and machine info - see EmuSen_Firmware.md §3.1.
    [Collection(TestCollections.ProcessGlobals)]
    public class NephriteFirmwareTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenNephriteFirmware_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public NephriteFirmwareTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            FirmwareLibrary.Directory = Path.Combine(_root, "Firmware");
            Directory.CreateDirectory(FirmwareLibrary.Directory);
            CoreDiscovery.UseDevelopment(false);
            CoreOptions.BatteryRamDisabled = true;
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            CoreDiscovery.UseDevelopment(null);
            FirmwareLibrary.ResetDirectory();
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // The disc as an image, or as a sheet, which asks for all three markets' files.
        private string Disc(bool sheet)
        {
            File.WriteAllBytes(Path.Combine(_root, "game.iso"), SyntheticMdRom.Disc("U"));
            if (!sheet) return Path.Combine(_root, "game.iso");
            string cue = Path.Combine(_root, "game.cue");
            File.WriteAllText(cue, "FILE \"game.iso\" BINARY\n  TRACK 01 MODE1/2048\n    INDEX 01 00:00:00\n");
            return cue;
        }

        // The lines the load writes to the console, which Mistress's per-game log holds.
        private static List<string> Load(string path)
        {
            var was = Console.Out;
            var text = new StringWriter();
            Console.SetOut(text);
            try
            {
                using var core = Assert.IsType<CoreEngine>(CoreFactory.Load(path).Core);
            }
            finally
            {
                Console.SetOut(was);
            }
            return text.ToString().Split('\n', StringSplitOptions.TrimEntries).Where(l => l.StartsWith("[Firmware]")).ToList();
        }

        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void A_bios_in_the_firmware_folder_under_the_cores_name_is_the_one_that_boots(bool sheet)
        {
            File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, "bios_CD_U.bin"), new byte[0x2_0000]);
            File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, "bios_CD_E.bin"), new byte[0x2_0000]);
            Assert.Equal(new[] { $"[Firmware] The American Sega CD's BIOS (131,072 bytes): the player's bios_CD_U.bin, in {FirmwareLibrary.Directory}" }, Load(Disc(sheet)));
        }

        [Fact]
        public void Without_one_the_session_says_it_runs_on_nephrites_own()
        {
            // A file of the wrong size is not the BIOS (EmuSen_Firmware.md §2.1).
            File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, "bios_CD_U.bin"), new byte[0x1_0000]);
            Assert.Equal(new[] { "[Firmware] The American Sega CD's BIOS (131,072 bytes): Nephrite's own replacement" }, Load(Disc(sheet: false)));
        }

        // A sheet's disc of one market boots another market's BIOS when only that one is installed, and the line names it.
        [Fact]
        public void Another_markets_bios_is_named_when_it_is_the_one_that_boots()
        {
            File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, "bios_CD_E.bin"), new byte[0x2_0000]);
            Assert.Equal(new[] { $"[Firmware] The European Mega-CD's BIOS (131,072 bytes): the player's bios_CD_E.bin, in {FirmwareLibrary.Directory}" }, Load(Disc(sheet: true)));
        }

        // Each entry of a folder with its length and time written, and the folder's own time, to show nothing was written there.
        private static List<string> Snapshot(string folder) =>
            Directory.GetFileSystemEntries(folder, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
                .Select(e => $"{e}|{(File.Exists(e) ? new FileInfo(e).Length : -1)}|{File.GetLastWriteTimeUtc(e):O}")
                .Prepend($"{folder}|{Directory.GetLastWriteTimeUtc(folder):O}").ToList();

        // The player's chosen folder, searched before home/Firmware: its BIOS boots on a tie of name and size, the line names the folder, and the folder is as it was after the session.
        [Fact]
        public void A_bios_in_the_chosen_folder_boots_the_log_names_the_folder_and_the_folder_is_never_written()
        {
            FirmwareLibrary.ResetDirectory();
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            string chosen = Path.Combine(_root, "BIOS");
            Directory.CreateDirectory(chosen);
            Directory.CreateDirectory(DataStore.Firmware);
            new AppSettings { FirmwareDirectory = chosen }.Save();
            File.WriteAllBytes(Path.Combine(DataStore.Firmware, "bios_CD_U.bin"), new byte[0x2_0000]);
            File.WriteAllBytes(Path.Combine(chosen, "bios_CD_U.bin"), Enumerable.Repeat((byte)0x5A, 0x2_0000).ToArray());
            File.WriteAllBytes(Path.Combine(chosen, "unrelated.bin"), new byte[16]);
            Directory.SetLastWriteTimeUtc(chosen, new DateTime(2026, 1, 2, 3, 4, 5, DateTimeKind.Utc));
            List<string> before = Snapshot(chosen);

            Assert.Equal(new[] { $"[Firmware] The American Sega CD's BIOS (131,072 bytes): the player's bios_CD_U.bin, in {chosen}" }, Load(Disc(sheet: true)));
            Assert.Equal(before, Snapshot(chosen));

            // Cleared, the setting leaves home/Firmware alone, and its file is the one named.
            new AppSettings().Save();
            Assert.Equal(new[] { $"[Firmware] The American Sega CD's BIOS (131,072 bytes): the player's bios_CD_U.bin, in {DataStore.Firmware}" }, Load(Disc(sheet: false)));
            Assert.Equal(before, Snapshot(chosen));
        }
    }
}
