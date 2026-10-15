using System.Diagnostics;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The runtime's save media for any console: media grouped from core info, each medium's mode and keys, its files, the flags and the lock - see EmuSen_SaveMedia.md §3.
    [Collection(TestCollections.ProcessGlobals)]
    public class SaveMediaTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSaveMedia_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public SaveMediaTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            CoreOptions.BatteryRamDisabled = false;
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static CoreSystem TestSystem => CoreAdapterTests.Full.Info.Systems.Single();

        private string Rom(string name, params byte[] payload)
        {
            string path = Path.Combine(_root, name + ".tst");
            File.WriteAllBytes(path, CoreAdapterTests.Image(payload));
            return path;
        }

        // The test core lists a card and the console's memory: two media, each kind's first, so 1.7's keys and the console kind's.
        [Fact]
        public void A_cores_slots_are_grouped_into_media_with_their_kinds_keys()
        {
            var media = MemoryCards.MediaOf("TEST", TestSystem);
            Assert.Equal(new[] { ("test.card", CoreBatteryKind.Removable, ".card", "Cards", "Card1"), ("test.memory", CoreBatteryKind.Console, ".mem", "ConsoleMemory", "ConsoleMemory1") },
                media.Select(m => (m.Id, m.Kind, m.Extension, m.ModeKey, m.SlotKey(0))));
            Assert.Equal(("Card 1", "Memory 1", "Card slot", "Inside the console"), (media[0].DefaultName(0), media[1].DefaultName(0), media[0].SlotLabel(0), media[1].SlotLabel(0)));
            Assert.Equal(Path.Combine(DataStore.Saves, "TEST", "Console", "Memory 1.mem"), media[1].PathOf("Memory 1"));
        }

        // A core of 1.7 names no medium: each kind and extension is one medium, and a second of a kind takes its id's keys.
        [Fact]
        public void A_core_of_1_7_is_grouped_by_kind_and_extension()
        {
            string json = CoreAdapterTests.Full.Info.Text;
            json = json.Replace(",\"medium\":\"test.card\",\"label\":\"Card slot\"", "").Replace("{\"which\":2,\"suffix\":\".mem\",\"kind\":\"console\",\"medium\":\"test.memory\",\"label\":\"Inside the console\"}",
                "{\"which\":2,\"suffix\":\".2.card\",\"kind\":\"removable\"},{\"which\":3,\"suffix\":\".pak\",\"kind\":\"removable\"}");
            var media = MemoryCards.MediaOf("TEST", CoreDescriptorReader.Info(json).Systems.Single());
            Assert.Equal(new[] { ("removable.card", 2, "Cards", "Card2"), ("removable.pak", 1, "Media.removable.pak", "Media.removable.pak.1") },
                media.Select(m => (m.Id, m.Slots.Count, m.ModeKey, m.SlotKey(m.Slots.Count - 1))));
            Assert.Equal("Slot 2", media[0].SlotLabel(1));
        }

        // Shared, the console's memory is one file for every game, given at create and held in Console; per game each game's own.
        [Theory]
        [InlineData(MemoryCards.PerGame)]
        [InlineData(MemoryCards.Shared)]
        public void The_consoles_memory_is_one_file_for_every_game_when_shared(string mode)
        {
            var memory = MemoryCards.MediaOf("TEST", TestSystem)[1];
            byte[] pattern = Enumerable.Range(0, 16).Select(i => (byte)(0x40 + i)).ToArray();
            Directory.CreateDirectory(Path.GetDirectoryName(memory.PathOf("Memory 1"))!);
            File.WriteAllBytes(memory.PathOf("Memory 1"), pattern);
            var media = new Dictionary<string, CardChoice> { [memory.Id] = new(mode, Array.Empty<string>()) };
            foreach (string game in new[] { "First", "Second" })
            {
                using var engine = new CoreEngine(CoreAdapterTests.Full) { Media = media };
                engine.LoadRom(Rom(game, 0xCE, 1, 2));
                Assert.Equal(mode == MemoryCards.Shared ? pattern : new byte[16], engine.Machine.Battery(2).Data);
            }
        }

        // The lock: held by a machine for its life, shared by the program's machines, refused to the card manager and to another program, named by its holder.
        [Fact]
        public void A_shared_file_is_locked_while_a_game_holds_it()
        {
            var card = MemoryCards.MediaOf("TEST", TestSystem)[0];
            string path = card.PathOf("Card 1");
            var media = new Dictionary<string, CardChoice> { [card.Id] = new(MemoryCards.Shared, Array.Empty<string>()) };
            using (var engine = new CoreEngine(CoreAdapterTests.Full) { Media = media })
            {
                engine.LoadRom(Rom("Holder", 0xCD));
                Assert.Equal("Holder", MediumLock.HolderOf(path));
                Assert.Null(MediumLock.TryTake(path, "the card manager"));
                Assert.Throws<IOException>(() => new FileStream(path + MediumLock.Suffix, FileMode.Open, FileAccess.Read, FileShare.ReadWrite));
                if (FlockAt() is { } flock)
                {
                    using var other = Process.Start(new ProcessStartInfo(flock, new[] { "-n", path + MediumLock.Suffix, "true" }))!;
                    other.WaitForExit();
                    Assert.Equal(1, other.ExitCode);
                }
                using (var second = new CoreEngine(CoreAdapterTests.Full) { Media = media })
                {
                    second.LoadRom(Rom("Next", 0xCD));
                    Assert.Null(second.MediaNotice);
                    Assert.Equal("Next", MediumLock.HolderOf(path));
                }
                Assert.Equal("Holder", MediumLock.HolderOf(path));
                engine.ReleaseMedia();
                Assert.Null(MediumLock.HolderOf(path));
                Assert.False(File.Exists(path + MediumLock.Suffix), "the lock file goes with the lock");
            }
            using var manager = MediumLock.TryTake(path, "the card manager");
            Assert.NotNull(manager);
            using var game = new CoreEngine(CoreAdapterTests.Full) { Media = media };
            game.LoadRom(Rom("Late", 0xCD));
            Assert.Contains("in use by the card manager", game.MediaNotice);
            Assert.Equal("Late", MediumLock.HolderOf(Path.Combine(DataStore.Saves, "TEST", "Late.card")));
        }

        private static string? FlockAt() => new[] { "/usr/bin/flock", "/bin/flock" }.FirstOrDefault(File.Exists);
    }
}
