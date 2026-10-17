using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // The card mode: each game's own cards, one card shared by every game, or named cards chosen per slot, and a save state that never rolls a shared card back - see Diamond_Cards.md.
    [Collection(TestCollections.ProcessGlobals)]
    public class DiamondCardsTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDiamondCards_" + Guid.NewGuid().ToString("N"));
        private readonly string _library;

        public DiamondCardsTests(ITestOutputHelper output)
        {
            _output = output;
            _library = Path.Combine(_root, "Library");
            Directory.CreateDirectory(_library);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static readonly byte[] SaveA = Enumerable.Range(0, 128).Select(i => (byte)(i * 3 + 1)).ToArray();
        private static readonly byte[] SaveB = Enumerable.Range(0, 128).Select(i => (byte)(0xB0 ^ i)).ToArray();

        // Two games, each writing its save to its own frame of the card in slot 1.
        private (string A, string B) Games()
        {
            string a = Path.Combine(_library, "Amber Saga.psexe"), b = Path.Combine(_library, "Basalt Run.psexe");
            File.WriteAllBytes(a, DiamondPlayersTests.CardWriting(0x40, SaveA));
            File.WriteAllBytes(b, DiamondPlayersTests.CardWriting(0x50, SaveB));
            return (a, b);
        }

        private static void Mode(string mode, string? slot1 = null)
        {
            var config = GraphicsConfig.Load();
            config.SetValue("PlayStation", MemoryCards.ModeKey, mode);
            if (slot1 is not null) config.SetValue("PlayStation", MemoryCards.SlotKeys[0], slot1);
            config.Save();
        }

        // The game played until its save is written, the host writing the card as Mistress does.
        private static CoreEngine Played(string game)
        {
            var core = (CoreEngine)CoreFactory.Load(game).Core;
            for (int f = 0; f < 30; f++) core.RunFrame();
            Assert.Equal(new byte[] { 0x5C, 0x5D, 0x47 }, Enumerable.Range(0x87, 3).Select(i => core.ReadSpace("RAM", 0x10_0000 + i)).ToArray());
            core.SaveSram();
            return core;
        }

        private static byte[] Frame(byte[] card, int frame) => card[(frame * 128)..((frame + 1) * 128)];

        private static string Card(string name) => SaveLibrary.CardPathFor("PlayStation", name, ".mcr");

        private static string Own(string game) => Path.Combine(DataStore.Saves, "PlayStation", Path.GetFileNameWithoutExtension(game) + ".mcr");

        // Core info marks both of Diamond's cards removable, which is what offers the rows - see EmuSen_CoreAPI.md §6.27.
        [Fact]
        public void The_playstations_cards_are_removable_and_its_tab_offers_the_mode()
        {
            var listed = MemoryCards.RemovableFor("PlayStation");
            Assert.Equal(new[] { (0u, ".mcr", true), (4u, ".2.mcr", true) }, listed.Select(b => (b.Which, b.Suffix, b.Removable)));
            var rows = MemoryCards.SettingsFor("PlayStation");
            Assert.Equal(new[] { "Cards", "Card1", "Card2" }, rows.Select(r => r.Key));
            Assert.Equal(new[] { "game", "shared", "named" }, rows[0].Choices);
            Assert.Equal("shared", rows[0].Default);
            Assert.Equal(Enumerable.Range(1, 8).Select(n => $"Card {n}"), rows[1].Choices);
            Directory.CreateDirectory(Path.GetDirectoryName(Card("Blue"))!);
            File.WriteAllBytes(Card("Blue"), new byte[131_072]);
            Assert.Equal("Blue", MemoryCards.SettingsFor("PlayStation")[1].Choices![^1]);
            Assert.Empty(MemoryCards.SettingsFor("SNES"));
            Assert.Equal(CardChoice.PerGame, MemoryCards.Read(key => key == MemoryCards.ModeKey ? "nonsense" : null));
            Assert.Equal("Card 2", MemoryCards.Read(key => key == MemoryCards.SlotKeys[1] ? "../escape" : MemoryCards.Named).Slot2);
        }

        // Per game, as before: each game's save is in its own file, and the next game's card is blank.
        [Fact]
        public void Per_game_a_save_stays_with_its_game()
        {
            var (a, b) = Games();
            Mode(MemoryCards.PerGame);
            using (Played(a)) { }
            Assert.Equal(SaveA, Frame(File.ReadAllBytes(Own(a)), 0x40));
            using var next = (CoreEngine)CoreFactory.Load(b).Core;
            Assert.All(next.Machine.Battery(0).Data, x => Assert.Equal(0, x));
            Assert.False(Directory.Exists(Path.Combine(DataStore.Saves, "PlayStation", SaveLibrary.CardsFolder)), "nothing of the shared mode is made");
        }

        // Shared: the save game A made is on the card game B finds, B's beside it, and no game's own file is written.
        [Fact]
        public void Shared_a_save_in_one_game_is_on_the_card_of_the_next()
        {
            var (a, b) = Games();
            Mode(MemoryCards.Shared);
            using (Played(a)) { }
            Assert.False(File.Exists(Own(a)));
            Assert.Equal(SaveA, Frame(File.ReadAllBytes(Card("Card 1")), 0x40));
            using (var next = Played(b)) Assert.Equal(SaveA, Frame(next.Machine.Battery(0).Data, 0x40));
            byte[] card = File.ReadAllBytes(Card("Card 1"));
            Assert.Equal(SaveA, Frame(card, 0x40));
            Assert.Equal(SaveB, Frame(card, 0x50));
            Mode(MemoryCards.PerGame);
            using var back = (CoreEngine)CoreFactory.Load(b).Core;
            Assert.All(back.Machine.Battery(0).Data, x => Assert.Equal(0, x));
        }

        // Named cards: the card follows the slot's name, not the game, as a card moved between consoles does.
        [Fact]
        public void Named_cards_follow_the_slot_not_the_game()
        {
            var (a, b) = Games();
            Mode(MemoryCards.Named, "Blue");
            using (Played(a)) { }
            Mode(MemoryCards.Named, "Red");
            using (var red = Played(b)) Assert.Equal(new byte[128], Frame(red.Machine.Battery(0).Data, 0x40));
            Mode(MemoryCards.Named, "Blue");
            using var blue = (CoreEngine)CoreFactory.Load(b).Core;
            Assert.Equal(SaveA, Frame(blue.Machine.Battery(0).Data, 0x40));
            Assert.Equal(new byte[128], Frame(blue.Machine.Battery(0).Data, 0x50));
            Assert.Equal(SaveB, Frame(File.ReadAllBytes(Card("Red")), 0x50));
        }

        // The hazard of Diamond_Cards.md §1 through the host: A's state, loaded after B saved on the shared card, leaves B's save on it.
        [Fact]
        public void A_state_of_one_game_leaves_the_shared_card_as_it_stands()
        {
            var (a, b) = Games();
            Mode(MemoryCards.Shared);
            using var state = new MemoryStream();
            using (var first = Played(a)) first.SaveState(state);
            using (Played(b)) { }
            using (var again = (CoreEngine)CoreFactory.Load(a).Core)
            {
                state.Position = 0;
                again.LoadState(state);
                for (int f = 0; f < 5; f++) again.RunFrame();
                again.SaveSram();
            }
            byte[] card = File.ReadAllBytes(Card("Card 1"));
            Assert.Equal(SaveA, Frame(card, 0x40));
            Assert.Equal(SaveB, Frame(card, 0x50));
        }

        // Rewind, a state loaded from before a save and the card written as at the next frame's end: per game the save is gone from the game's card; shared, it stays.
        [Theory]
        [InlineData(MemoryCards.PerGame)]
        [InlineData(MemoryCards.Shared)]
        public void Rewinding_past_a_save_takes_it_off_only_a_games_own_card(string mode)
        {
            var (a, _) = Games();
            Mode(mode);
            using var core = (CoreEngine)CoreFactory.Load(a).Core;
            using var before = new MemoryStream();
            core.SaveSnapshot(before);
            for (int f = 0; f < 30; f++) core.RunFrame();
            core.SaveSram();
            before.Position = 0;
            core.LoadState(before);
            core.SaveSram();
            string file = mode == MemoryCards.Shared ? Card("Card 1") : Own(a);
            byte[] kept = Frame(File.ReadAllBytes(file), 0x40);
            _output.WriteLine($"{mode}: frame 40h after the rewind {Convert.ToHexString(kept[..8])}");
            Assert.Equal(mode == MemoryCards.Shared ? SaveA : new byte[128], kept);
        }
    }
}
