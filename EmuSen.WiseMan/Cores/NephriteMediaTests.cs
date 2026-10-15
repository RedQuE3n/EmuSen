using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The Sega CD's save media through the host: its RAM cartridge removable, its backup RAM the console's, each medium's own mode and files - see EmuSen_SaveMedia.md §7.3.
    [Collection(TestCollections.ProcessGlobals)]
    public class NephriteMediaTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenNephriteMedia_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public NephriteMediaTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            CoreDiscovery.UseDevelopment(true);
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

        private string Disc(string name)
        {
            string path = Path.Combine(_root, name + ".iso");
            File.WriteAllBytes(path, SyntheticMdRom.Disc());
            return path;
        }

        private static CoreLibrary Library => CoreLibrary.Open(NephriteTests.LibraryPath);

        private static IReadOnlyList<SaveMedium> Media => MemoryCards.MediaOf("Sega CD", Library.Info.Systems.Single(s => s.Id == "mcd"));

        // Core info's two slots become two media, each the first of its kind, named by the Genesis pack.
        [Fact]
        public void The_sega_cd_has_a_cartridge_and_its_own_backup_ram()
        {
            Assert.Equal(new[] { ("mcd.backup", CoreBatteryKind.Console, ".brm", "Internal Backup RAM", "ConsoleMemory", "Backup RAM 1", "Inside the console"), ("mcd.ramcart", CoreBatteryKind.Removable, ".crm", "Backup RAM Cartridge", "Cards", "Cartridge 1", "Cartridge slot") },
                Media.Select(m => (m.Id, m.Kind, m.Extension, m.Title, m.ModeKey, m.DefaultName(0), m.SlotLabel(0))));
            Assert.Equal(new[] { "ConsoleMemory", "ConsoleMemory1", "Cards", "Card1" }, Media.SelectMany(m => { var (mode, slots) = MemoryCards.RowsFor(m); return slots.Prepend(mode); }).Select(r => r.Key));
        }

        // Shared, the backup RAM is one file in Console for every game, and a state of one game leaves it as it stands; per game a state puts the game's back.
        [Theory]
        [InlineData(MemoryCards.PerGame)]
        [InlineData(MemoryCards.Shared)]
        public void A_state_leaves_the_shared_backup_ram_and_puts_back_a_games_own(string mode)
        {
            var choices = MemoryCards.Stored(Media, key => key == MemoryCards.ConsoleModeKey ? mode : null);
            string shared = SaveLibrary.MediumPathFor("Sega CD", consoleKind: true, "Backup RAM 1", ".brm"), own = Path.Combine(DataStore.Saves, "Sega CD", "Alpha.brm");
            string file = mode == MemoryCards.Shared ? shared : own;
            Directory.CreateDirectory(Path.GetDirectoryName(file)!);
            File.WriteAllBytes(file, Enumerable.Repeat((byte)0x11, 0x2000).ToArray());
            using var state = new MemoryStream();
            using (var first = new CoreEngine(Library) { Media = choices })
            {
                first.LoadRom(Disc("Alpha"));
                Assert.Equal("Sega CD", first.Console);
                Assert.Equal(0x11, first.Machine.Battery(0).Data[5]);
                first.SaveState(state);
            }
            File.WriteAllBytes(file, Enumerable.Repeat((byte)0x33, 0x2000).ToArray());
            using var again = new CoreEngine(Library) { Media = choices };
            again.LoadRom(Disc("Alpha"));
            Assert.Equal(0x33, again.Machine.Battery(0).Data[5]);
            state.Position = 0;
            again.LoadState(state);
            Assert.Equal(mode == MemoryCards.Shared ? 0x33 : 0x11, again.Machine.Battery(0).Data[5]);
            Assert.Equal(mode == MemoryCards.PerGame, File.Exists(own));
        }
    }
}
