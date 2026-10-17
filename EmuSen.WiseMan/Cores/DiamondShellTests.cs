using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The PlayStation started with nothing in it through the host as Mistress starts it, and a disc put in and taken out afterwards - see Diamond_Shell.md §1.5, §4.3.
    [Collection(TestCollections.ProcessGlobals)]
    public class DiamondShellTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDiamondShell_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public DiamondShellTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            CoreDiscovery.UseDevelopment(false);
            CoreOptions.BatteryRamDisabled = false;
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static void Mode(string mode)
        {
            var config = GraphicsConfig.Load();
            config.SetValue("PlayStation", MemoryCards.ModeKey, mode);
            config.Save();
        }

        // A disc whose program writes to the card in slot 1 over SIO0, its replies left in RAM: proof it booted.
        private string Disc(string name)
        {
            string dir = Path.Combine(_root, "Discs");
            Directory.CreateDirectory(dir);
            return SyntheticPsxDisc.Build(exe: DiamondPlayersTests.CardWriting(0x40, Enumerable.Range(0, 128).Select(i => (byte)i).ToArray())).WriteSet(dir, name);
        }

        private static bool Booted(CoreEngine core) => Enumerable.Range(0x87, 3).Select(i => core.ReadSpace("RAM", 0x10_0000 + i)).SequenceEqual(new byte[] { 0x5C, 0x5D, 0x47 });

        private static void Run(CoreEngine core, int frames)
        {
            for (int f = 0; f < frames; f++) core.RunFrame();
        }

        // Read from the engines' info, never from a list of consoles: Diamond says psx starts empty, and no other engine here says so of its console.
        [Fact]
        public void The_playstation_is_offered_to_start_empty_by_its_engines_info()
        {
            Assert.True(CoreLibrary.Open(DiamondTests.LibraryPath).Info.Systems.Single(s => s.Id == "psx").StartsEmpty);
            Assert.Contains(CoreCatalog.ConsolesThatStartEmpty, c => c.Console == "PlayStation" && c.SystemId == "psx");
            Assert.Null(CoreCatalog.EmptyStartFor("Genesis"));
            Assert.Null(CoreCatalog.EmptyStartFor("SNES"));
            Assert.Throws<NotSupportedException>(() => CoreFactory.CreateConsole("SNES"));
        }

        // §1.5 and §9 Q5: started empty under Per game, the drive listed empty, the shared cards in the slots and locked under
        // the session's name, the open kernel's screen with no disc drawn, and a disc put in through the lid booted by the kernel.
        [Fact]
        public void A_console_started_empty_shows_its_screen_with_no_disc_and_boots_a_disc_put_in()
        {
            Mode(MemoryCards.PerGame);
            using var core = (CoreEngine)CoreFactory.CreateConsole("PlayStation");
            Assert.Equal(("psx", "", -1, 0), (core.StartedEmpty, core.RomPath, core.DiscIndex, core.Discs.Count));
            Assert.True(core.CanChangeDisc);
            Assert.False(core.Machine.Info.Drives[0].Loaded);
            Assert.Equal(ConsoleSession.NameOf("PlayStation"), MediumLock.HolderOf(SaveLibrary.CardPathFor("PlayStation", "Card 1", ".mcr")));

            Run(core, 420);
            byte[] frame = core.GetFrameBufferRgba();
            int at = (120 * core.ScreenWidth + 2) * 4;
            Assert.Equal(new byte[] { 16, 24, 41 }, frame[at..(at + 3)]);

            string cue = Disc("Amber Saga");
            core.InsertDisc(cue);
            Assert.Equal((0, 1, true), (core.DiscIndex, core.Discs.Count, core.LidOpen));
            for (int f = 0; f < 1800 && !Booted(core); f++) core.RunFrame();
            Assert.True(Booted(core), "the disc put in was booted by the kernel");
            Assert.False(core.LidOpen);

            core.EjectDisc();
            Run(core, 90);
            Assert.Equal((-1, false, false), (core.DiscIndex, core.LidOpen, core.Machine.Info.Drives[0].Loaded));
        }

        // §9 Q8: a state of the empty console loads over a disc and empties the drive again; one made with the disc in finds
        // the disc among the session's own when the drive is empty.
        [Fact]
        public void A_state_carries_the_drive_as_it_was_empty_or_with_its_disc()
        {
            using var core = (CoreEngine)CoreFactory.CreateConsole("PlayStation");
            Run(core, 120);
            using var empty = new MemoryStream();
            core.SaveState(empty);
            core.InsertDisc(Disc("Basalt Run"));
            Run(core, 240);
            using var withDisc = new MemoryStream();
            core.SaveState(withDisc);

            core.EjectDisc();
            Run(core, 90);
            withDisc.Position = 0;
            core.LoadState(withDisc);
            Assert.Equal(0, core.DiscIndex);
            Assert.True(core.Machine.Info.Drives[0].Loaded);

            empty.Position = 0;
            core.LoadState(empty);
            Assert.Equal(-1, core.DiscIndex);
            Assert.False(core.Machine.Info.Drives[0].Loaded);
        }

        // A file the core refuses goes into no drive: the lid stays open, the session's discs are as they were.
        [Fact]
        public void A_disc_the_core_refuses_is_not_kept()
        {
            using var core = (CoreEngine)CoreFactory.CreateConsole("PlayStation");
            string other = Path.Combine(_root, "not a disc.iso");
            File.WriteAllBytes(other, new byte[64 * 2048]);
            Assert.ThrowsAny<Exception>(() => core.InsertDisc(other));
            Assert.Equal((-1, 0, true), (core.DiscIndex, core.Discs.Count, core.LidOpen));
        }
    }
}
