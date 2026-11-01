using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Crystal.Systems.Ps2;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // The PlayStation 2's memory cards are the console's, the same files under the hosted engine and Sapphire - see EmuSen_LibretroHost.md §10j.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class HostedPs2CardTests : IDisposable
    {
        private const string Console = "PlayStation 2";
        private readonly ITestOutputHelper _output;
        // The extensions pcee2's own .info lists, by which the Rust runtime matches a game to a hosted core (EmuSen_LibretroHost.md §4.1a).
        private readonly LibretroEntryFixture _entry = new(systemId: "playstation2", stem: "pcee2", extensions: "iso|chd|cso|zso|gz|bin|mdf|nrg|elf|irx");

        public HostedPs2CardTests(ITestOutputHelper output) => _output = output;

        public void Dispose() => _entry.Dispose();

        private static object Shape(SaveMedium m) =>
            (m.Id, m.Kind, m.Extension, m.Title, string.Join(" ", m.Slots.Select(s => $"{s.Suffix}:{s.Label}")), m.PathOf(m.DefaultName(0)), MemoryCards.DefaultMode(m), string.Join(" | ", MemoryCards.CardsOf(m).Select(c => $"{c.Path} {c.Label}")));

        [Fact]
        public void The_hosted_engine_and_sapphire_list_the_same_card_files()
        {
            string drawer = SaveLibrary.CardPathFor(Console, "Card 1", ".ps2");
            Directory.CreateDirectory(Path.GetDirectoryName(drawer)!);
            File.WriteAllBytes(drawer, Ps2CardFormat.Instance.Format()!);
            File.WriteAllBytes(Path.Combine(DataStore.Saves, Console, "Amber Saga.ps2"), Ps2CardFormat.Instance.Format()!);
            File.WriteAllBytes(Path.Combine(DataStore.Saves, Console, "Amber Saga.2.ps2"), Ps2CardFormat.Instance.Format()!);
            var hosted = _entry.Found.Info.Systems.Single();
            var sapphire = CoreLibrary.Open(SapphireTests.LibraryPath).Info.Systems.Single();
            var shapes = new[] { hosted, sapphire }.Select(s => Shape(MemoryCards.MediaOf(Console, s).Single(m => m.Id == Ps2CardFormat.Id))).ToArray();
            _output.WriteLine(shapes[0].ToString());
            Assert.Equal(shapes[1], shapes[0]);
            Assert.Contains("Amber Saga, Slot 2 (the game's own)", shapes[0].ToString());
        }

        [Fact]
        public void A_shared_card_reaches_the_hosted_core_and_its_save_comes_back_to_the_cards_file()
        {
            string drawer = SaveLibrary.CardPathFor(Console, "Card 1", ".ps2");
            Directory.CreateDirectory(Path.GetDirectoryName(drawer)!);
            byte[] card = Ps2CardFormatTests.WithSave(Ps2CardFormat.Instance.Format()!, "BASLUS-20001AMBER", "Amber Saga", 40, 1);
            File.WriteAllBytes(drawer, card);
            var disc = SyntheticPs2Disc.Dvd();
            "TCG1"u8.CopyTo(disc.Iso);
            string path = disc.WriteSet(_entry.Root, "Hosted");
            string slot1 = Path.Combine(_entry.Cores, LibretroEntries.Folder, ".data", "pcee2", "system", "pcsx2", "memcards", "Mcd001.ps2");
            using (var runtime = new CrystalRuntime(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory }, PlayerCores = new[] { _entry.Cores } }))
            {
                _output.WriteLine($"runtime: {CrystalRuntime.Implementation}");
                using CrystalSession s = runtime.Open(new CrystalOpen { Path = path, Engine = _entry.Found.EngineName });
                var info = new CrystalFrameInfo { size = (uint)System.Runtime.InteropServices.Marshal.SizeOf<CrystalFrameInfo>() };
                s.Frame(ref info);
                Assert.Equal(card, File.ReadAllBytes(slot1));
                File.WriteAllBytes(slot1, Ps2CardFormatTests.WithSave(card, "BESLES-50002BASALT", "Basalt Run", 300, 2));
                for (int i = 0; i < 30; i++) s.Frame(ref info);
            }
            var listing = Ps2CardFormat.Instance.Read(File.ReadAllBytes(drawer));
            Assert.Equal(new[] { "Amber Saga", "Basalt Run" }, listing.Saves.Select(e => e.Title).Order().ToArray());
        }
    }
}
