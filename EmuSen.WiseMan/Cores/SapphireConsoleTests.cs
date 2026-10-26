using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Systems.Ps2;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The PlayStation 2 started with nothing in it through the host as Mistress starts it, offered to players, refused with no BIOS as a game is, and a disc put in and taken out - see Sapphire_Plan.md §8 (G10), Sapphire_Native.md §30.15 and §49.5.
    [Collection(TestCollections.ProcessGlobals)]
    public class SapphireConsoleTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSapphireConsole_" + Guid.NewGuid().ToString("N"));

        public SapphireConsoleTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            FirmwareLibrary.Directory = Path.Combine(_root, "Firmware");
            Directory.CreateDirectory(FirmwareLibrary.Directory);
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            FirmwareLibrary.Directory = null;
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // A stand-in of the BIOS's name and size, every byte zero, made here: no firmware is read or shipped.
        private void StandIn() => File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory!, Ps2System.Bios[1].Name), new byte[Ps2System.Bios[1].Size]);

        // Read from the engines' info, never a list of consoles, in a player's build (G10 offered, Sapphire_Native.md §49.1).
        [Fact]
        public void The_playstation_2_is_offered_to_start_empty_by_its_engines_info()
        {
            Assert.True(CoreLibrary.Open(SapphireTests.LibraryPath).Info.Systems.Single().StartsEmpty);
            Assert.Contains(CoreCatalog.ConsolesThatStartEmpty, c => c.Console == "PlayStation 2" && c.SystemId == "ps2");
            Assert.NotNull(CoreCatalog.EmptyStartFor("PlayStation 2"));
        }

        // A game with no BIOS of its market is refused in the core's words as the empty start is, never a black screen; a European disc names the European BIOS (Sapphire_Native.md §49.5).
        [Fact]
        public void With_no_bios_a_game_is_refused_in_the_cores_words()
        {
            string dir = Path.Combine(_root, "Discs");
            Directory.CreateDirectory(dir);
            string iso = Path.Combine(dir, "Basalt Run.iso");
            File.WriteAllBytes(iso, SyntheticPs2Disc.Dvd().Iso);
            var refused = Assert.Throws<ConsoleFirmwareException>(() => CoreFactory.Load(iso));
            Assert.StartsWith("Basalt Run.iso cannot start: PlayStation 2 BIOS (Americas) (ps2-0230a-20080220.bin) is not in the firmware folder.", refused.Message);
            Assert.Contains("runs nothing", refused.Message);
            string program = Path.Combine(dir, "program.elf");
            File.WriteAllBytes(program, SyntheticPs2Elf.Build());
            Assert.Throws<ConsoleFirmwareException>(() => CoreFactory.Load(program));
            StandIn();
            using var core = (CoreEngine)CoreFactory.Load(iso).Core;
            Assert.Contains(core.Machine.Info.Firmware, f => f.Source == "file");
        }

        // With no BIOS the start is refused in the core's own words, never a black screen; the machine is not kept.
        [Fact]
        public void With_no_bios_the_start_is_refused_in_the_cores_words()
        {
            var refused = Assert.Throws<ConsoleFirmwareException>(() => CoreFactory.CreateConsole("PlayStation 2"));
            Assert.StartsWith("PlayStation 2 cannot start with nothing in it: PlayStation 2 BIOS (Americas) (ps2-0230a-20080220.bin) is not in the firmware folder.", refused.Message);
            Assert.Contains("runs nothing", refused.Message);
            Assert.Null(MediumLock.HolderOf(SaveLibrary.CardPathFor("PlayStation 2", "Card 1", ".ps2")));
        }

        // An .m3u of two discs: the first in the drive at power-on, Change Disc through the tray, and each disc's state bringing its own disc back, as the PlayStation's do - see Sapphire_Native.md §49.4.
        [Fact]
        public void An_m3u_changes_its_discs_through_the_tray_and_a_state_brings_its_own_disc_back()
        {
            StandIn();
            string dir = Path.Combine(_root, "ps2");
            Directory.CreateDirectory(dir);
            File.WriteAllBytes(Path.Combine(dir, "Twin Peaks (Disc 1).iso"), SyntheticPs2Disc.Dvd().Iso);
            File.WriteAllBytes(Path.Combine(dir, "Twin Peaks (Disc 2).iso"), SyntheticPs2Disc.Dvd(config: "BOOT2 = cdrom0:\\EMUS_000.03;1\r\nVER = 1.00\r\nVMODE = NTSC\r\n").Iso);
            string m3u = Path.Combine(dir, "Twin Peaks.m3u");
            File.WriteAllText(m3u, "Twin Peaks (Disc 1).iso\nTwin Peaks (Disc 2).iso\n");
            Assert.Equal(new[] { "Twin Peaks.m3u" }, ImageSets.Shelved(Directory.EnumerateFiles(dir).ToList()).Select(Path.GetFileName));
            Assert.Equal("PlayStation 2", CoreCatalog.ConsoleForRom(m3u));

            using var core = (CoreEngine)CoreFactory.Load(m3u).Core;
            Assert.True(core.CanChangeDisc);
            Assert.Equal((2, 0, false, true), (core.Discs.Count, core.DiscIndex, core.LidOpen, core.Machine.Info.Drives[0].Loaded));
            for (int f = 0; f < 10; f++) core.RunFrame();
            using var first = new MemoryStream();
            core.SaveState(first);

            core.ChangeDisc(1);
            Assert.Equal((1, true), (core.DiscIndex, core.LidOpen));
            for (int f = 0; f < 70; f++) core.RunFrame();
            Assert.False(core.LidOpen, "the tray closes a second of the game's time after the disc goes in");
            using var second = new MemoryStream();
            core.SaveState(second);

            first.Position = 0;
            core.LoadState(first);
            Assert.Equal((0, false), (core.DiscIndex, core.LidOpen));
            second.Position = 0;
            core.LoadState(second);
            Assert.Equal(1, core.DiscIndex);
        }

        // Started empty with the tray closed, a synthetic disc inserted reaching the recogniser and the drive, a program refused, Eject emptying it.
        [Fact]
        public void A_console_started_empty_takes_a_disc_through_the_tray_and_ejects_it()
        {
            StandIn();
            using var core = (CoreEngine)CoreFactory.CreateConsole("PlayStation 2");
            Assert.Equal(("ps2", "", -1, true), (core.StartedEmpty, core.RomPath, core.DiscIndex, core.CanChangeDisc));
            var drive = core.Machine.Info.Drives.Single();
            Assert.Equal(("dvd", false, false), (drive.Kind, drive.Loaded, drive.LidOpen));
            Assert.Contains(core.Machine.Info.Firmware, f => f.Source == "file");
            for (int f = 0; f < 10; f++) core.RunFrame();

            string dir = Path.Combine(_root, "Discs");
            Directory.CreateDirectory(dir);
            string iso = Path.Combine(dir, "Basalt Run.iso");
            File.WriteAllBytes(iso, SyntheticPs2Disc.Dvd().Iso);
            core.InsertDisc(iso);
            for (int f = 0; f < 90; f++) core.RunFrame();
            Assert.Equal((0, true, false), (core.DiscIndex, core.Machine.Info.Drives[0].Loaded, core.LidOpen));

            string program = Path.Combine(dir, "program.elf");
            File.WriteAllBytes(program, SyntheticPs2Elf.Build());
            var refused = Assert.ThrowsAny<Exception>(() => core.InsertDisc(program));
            Assert.Contains("not put in the drive", refused.Message);

            core.EjectDisc();
            for (int f = 0; f < 90; f++) core.RunFrame();
            Assert.Equal((-1, false, false), (core.DiscIndex, core.LidOpen, core.Machine.Info.Drives[0].Loaded));
        }
    }
}
