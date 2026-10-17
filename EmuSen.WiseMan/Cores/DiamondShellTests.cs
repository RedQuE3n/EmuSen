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

        // Started empty, shown, given a disc and emptied - see Diamond_Shell.md §11.2.
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
            Assert.Equal(new byte[] { 8, 8, 16 }, frame[at..(at + 3)]);

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

        // A state carries the drive empty or with its disc - see Diamond_Shell.md §11.2.
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

        private static void Press(CoreEngine core, EmuSen.Galaxia.Input.PadButton button)
        {
            core.SetButton(0, button, true);
            Run(core, 4);
            core.SetButton(0, button, false);
            Run(core, 4);
        }

        // The shell's copy and delete read back by the host's own reader of the format - see Diamond_Shell.md §4.3.
        [Fact]
        public void The_shells_copy_and_delete_are_read_back_by_the_hosts_format_module()
        {
            Mode(MemoryCards.Shared);
            var config = GraphicsConfig.Load();
            config.SetValue("PlayStation", "card2", "card");
            config.Save();
            var format = EmuSen.Crystal.Systems.Psx.PsxCardFormat.Instance;
            byte[] one = EmuSen.WiseMan.Crystal.SaveMediaManagerTests.WithSave(EmuSen.WiseMan.Crystal.SaveMediaManagerTests.WithSave(format.Format(), "BASLUS-00001AMBER", "Amber Saga", new[] { 0 }, 0xA1), "BASLUS-00002BASALT", "Basalt Run", new[] { 1, 2 }, 0xB2);
            string card1 = SaveLibrary.CardPathFor("PlayStation", "Card 1", ".mcr"), card2 = SaveLibrary.CardPathFor("PlayStation", "Card 2", ".mcr");
            Directory.CreateDirectory(Path.GetDirectoryName(card1)!);
            File.WriteAllBytes(card1, one);
            File.WriteAllBytes(card2, format.Format());
            using (var core = (CoreEngine)CoreFactory.CreateConsole("PlayStation"))
            {
                Run(core, 900);
                foreach (var b in new[] { EmuSen.Galaxia.Input.PadButton.B, EmuSen.Galaxia.Input.PadButton.Down, EmuSen.Galaxia.Input.PadButton.B, EmuSen.Galaxia.Input.PadButton.B })
                    Press(core, b);
                Run(core, 900);
                foreach (var b in new[] { EmuSen.Galaxia.Input.PadButton.Up, EmuSen.Galaxia.Input.PadButton.B, EmuSen.Galaxia.Input.PadButton.Down, EmuSen.Galaxia.Input.PadButton.B, EmuSen.Galaxia.Input.PadButton.Up, EmuSen.Galaxia.Input.PadButton.B })
                    Press(core, b);
                Run(core, 300);
                core.SaveSram();
            }
            var first = format.Read(File.ReadAllBytes(card1));
            var second = format.Read(File.ReadAllBytes(card2));
            Assert.Equal(new[] { ("BASLUS-00002BASALT", 2) }, first.Saves.Select(s => (s.Id, s.Units)));
            Assert.Equal(new[] { ("BASLUS-00002BASALT", "Basalt Run", 2) }, second.Saves.Select(s => (s.Id, s.Title, s.Units)));
            Assert.Equal((13, 13), (first.FreeUnits, second.FreeUnits));
        }

        // A disc of music alone goes in the drive through Insert Disc - see Diamond_Shell.md §5.1.
        [Fact]
        public void An_audio_disc_goes_in_the_drive()
        {
            string dir = Path.Combine(_root, "Music");
            Directory.CreateDirectory(dir);
            File.WriteAllBytes(Path.Combine(dir, "Tones.bin"), Enumerable.Range(0, 300 * 2352).Select(i => (byte)(i / 4 % 64)).ToArray());
            string cue = Path.Combine(dir, "Tones.cue");
            File.WriteAllText(cue, "FILE \"Tones.bin\" BINARY\n  TRACK 01 AUDIO\n    INDEX 01 00:00:00\n");
            using var core = (CoreEngine)CoreFactory.CreateConsole("PlayStation");
            Run(core, 60);
            core.InsertDisc(cue);
            Run(core, 90);
            Assert.Equal((0, false, true), (core.DiscIndex, core.LidOpen, core.Machine.Info.Drives[0].Loaded));
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
