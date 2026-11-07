using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Crystal.Client;
using EmuSen.WiseMan.Bindings.Systems.Psx;
using EmuSen.Mistress.Library;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Shared by default once the import exists, and the import offered once before a game starts, copying each game's card and never moving it - see EmuSen_SaveMedia.md §9.
    [Collection(TestCollections.ProcessGlobals)]
    public class MemoryCardsOfferTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MemoryCardsOfferTests).GetTypeInfo().Assembly);

        private static readonly FieldInfo Confirm = typeof(MainWindow).GetField("ConfirmCardImport", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realConfirm = Confirm.GetValue(null)!;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMemoryCardsOffer_" + Guid.NewGuid().ToString("N"));
        private int _asked;

        public MemoryCardsOfferTests()
        {
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            Confirm.SetValue(null, _realConfirm);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private string Own(string game, string save, int block)
        {
            string path = Path.Combine(DataStore.Saves, "PlayStation", game + ".mcr");
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, SaveMediaManagerTests.WithSave(PsxCardFormat.Instance.Format()!, save, game, new[] { block }, 7));
            return path;
        }

        private string Rom()
        {
            string path = Path.Combine(_root, "Amber Saga.psexe");
            File.WriteAllBytes(path, TestPrograms.CardWriting(0x40, new byte[128]));
            return path;
        }

        private void Answer(bool copy) => Confirm.SetValue(null, (Func<MainWindow, string, string, Task<bool>>)((_, _, _) => { _asked++; return Task.FromResult(copy); }));

        private static Task Offer(MainWindow window, string rom) =>
            (Task)typeof(MainWindow).GetMethod("OfferCardImportAsync", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { rom })!;

        // The PlayStation's card and the Sega CD's two media, each read by its module, are Shared by default; a choice stored is kept.
        [Fact]
        public void A_medium_with_an_import_is_shared_by_default()
        {
            using var runtime = new CrystalRuntime();
            string Mode(string? stored)
            {
                var consoles = ConsolesConfig.Load();
                if (stored is null) consoles.Forget("PlayStation", CardModes.Key); else consoles.SetValue("PlayStation", CardModes.Key, stored);
                consoles.Save();
                return CardMedium.Of(runtime, "PlayStation").Single().Mode;
            }
            Assert.Equal(CardModes.Shared, Mode(null));
            Assert.Equal(CardModes.Shared, RuntimeCatalog.SettingsFor("PlayStation").Single(r => r.Key == CardModes.Key).Default);
            Assert.Equal(CardModes.PerGame, Mode(CardModes.PerGame));
            Assert.Equal(CardModes.PerGame, Mode("nonsense"));
            Mode(null);
            Assert.All(CardMedium.Of(runtime, "Sega CD"), m => Assert.Equal(CardModes.Shared, m.Mode));
        }

        // Accepted once: every game's saves copied onto Card 1, the games' cards unchanged, the offer recorded and not made again.
        [Fact]
        public Task The_import_is_offered_once_and_copies_every_games_saves() => Session.Dispatch(() =>
        {
            string amber = Own("Amber Saga", "BASLUS-00001A", 0), basalt = Own("Basalt Run", "BASLUS-00002B", 3);
            byte[] amberWas = File.ReadAllBytes(amber);
            Answer(copy: true);
            var window = new MainWindow();
            Offer(window, Rom()).GetAwaiter().GetResult();
            Offer(window, Rom()).GetAwaiter().GetResult();
            Assert.Equal(1, _asked);
            string shared = SaveLibrary.CardPathFor("PlayStation", "Card 1", ".mcr");
            Assert.Equal(new[] { "BASLUS-00001A", "BASLUS-00002B" }, PsxCardFormat.Instance.Read(File.ReadAllBytes(shared)).Saves.Select(s => s.Id));
            Assert.Equal(amberWas, File.ReadAllBytes(amber));
            Assert.True(File.Exists(basalt));
            Assert.Equal("true", GraphicsConfig.Load().Value("PlayStation", "CardsOffered"));
            window.Close();
        }, default);

        // Declined, nothing is copied and the offer is not made again; a player who chose a mode, or has no game's card, is never asked.
        [Fact]
        public Task Declined_or_needless_the_offer_changes_nothing() => Session.Dispatch(() =>
        {
            Answer(copy: false);
            var window = new MainWindow();
            Offer(window, Rom()).GetAwaiter().GetResult();
            Assert.Equal(0, _asked);
            Own("Amber Saga", "BASLUS-00001A", 0);
            var consoles = ConsolesConfig.Load();
            consoles.SetValue("PlayStation", CardModes.Key, CardModes.PerGame);
            consoles.Save();
            Offer(window, Rom()).GetAwaiter().GetResult();
            Assert.Equal(0, _asked);
            consoles.Forget("PlayStation", CardModes.Key);
            consoles.Save();
            Offer(window, Rom()).GetAwaiter().GetResult();
            Offer(window, Rom()).GetAwaiter().GetResult();
            Assert.Equal(1, _asked);
            Assert.False(File.Exists(SaveLibrary.CardPathFor("PlayStation", "Card 1", ".mcr")));
            window.Close();
        }, default);
    }
}
