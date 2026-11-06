using System;
using System.IO;
using System.Linq;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Systems;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The one console context the library and both cheat windows share - see EmuSen_Multicore.md §10.
    [Collection(TestCollections.ProcessGlobals)]
    public class ConsoleContextTests : IDisposable
    {
        private readonly string _configDir;

        public ConsoleContextTests()
        {
            _configDir = Path.Combine(Path.GetTempPath(), "EmuSenConsoleContextTests", Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = _configDir;
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_configDir)) Directory.Delete(_configDir, recursive: true); } catch { }
        }

        [Fact]
        public void A_fresh_config_starts_on_all_consoles()
        {
            Assert.Equal(AppSettings.AllConsoles, new AppSettings().SelectedCore);
        }

        // Honouring the inert old default would have hidden every NES game at once.
        [Fact]
        public void The_legacy_default_is_upgraded_rather_than_honoured()
        {
            new AppSettings { SelectedCore = AppSettings.LegacySelectedCoreDefault }.Save();

            Assert.Equal(AppSettings.AllConsoles, AppSettings.Load().SelectedCore);
        }

        // The upgrade is for a file written before the filter read the value; the same string saved since is a choice - see EmuSen_Multicore.md §10.3a.
        [Fact]
        public void The_super_nintendo_chosen_by_a_build_that_honours_it_is_kept()
        {
            new AppSettings { SelectedCore = AppSettings.LegacySelectedCoreDefault }.Save();
            AppSettings upgraded = AppSettings.Load();
            Assert.Equal(AppSettings.AllConsoles, upgraded.SelectedCore);

            upgraded.SelectedCore = AppSettings.LegacySelectedCoreDefault;
            upgraded.Save();

            Assert.Equal(AppSettings.LegacySelectedCoreDefault, AppSettings.Load().SelectedCore);
        }

        // A console the player actually picked must survive a round trip.
        [Fact]
        public void A_real_choice_is_kept()
        {
            string nes = ConsoleOf(".nes").DisplayName;
            new AppSettings { SelectedCore = nes }.Save();

            Assert.Equal(nes, AppSettings.Load().SelectedCore);
        }

        [Fact]
        public void The_filter_list_offers_all_consoles_first_then_every_shelf_with_the_color_after_the_game_boy()
        {
            Assert.Equal(RuntimeCatalog.AllConsoles, RuntimeCatalog.FilterChoices[0]);
            Assert.Equal(RuntimeCatalog.Shelves.Count + 1, RuntimeCatalog.FilterChoices.Count);

            foreach (var core in RuntimeCatalog.Consoles.Where(c => RuntimeCatalog.Shelves.Any(s => s.Core.Console == c.Console)))
            {
                Assert.Contains(core.DisplayName, RuntimeCatalog.FilterChoices);
            }

            ConsoleCore gameBoy = ConsoleOf(".gb");
            int at = RuntimeCatalog.FilterChoices.ToList().IndexOf(gameBoy.DisplayName);
            Assert.Equal(EngineNames.GameBoyColorShelf, RuntimeCatalog.FilterChoices[at + 1]);
            Assert.Equal(gameBoy.Console, RuntimeCatalog.ShelfByName(EngineNames.GameBoyColorShelf)!.Core.Console);
            Assert.Equal("GBC", RuntimeCatalog.ShelfByName(EngineNames.GameBoyColorShelf)!.Label);
        }

        // Galaxia persists it and the catalog displays it, so the two spellings must not drift.
        [Fact]
        public void Galaxia_and_the_catalog_agree_on_the_all_consoles_label()
        {
            Assert.Equal(AppSettings.AllConsoles, RuntimeCatalog.AllConsoles);
        }

        [Fact]
        public void An_unknown_console_name_resolves_to_no_filter()
        {
            Assert.Null(RuntimeCatalog.ByDisplayName(RuntimeCatalog.AllConsoles));
            Assert.Null(RuntimeCatalog.ByDisplayName("Game Gear (Endou)"));
            Assert.Null(RuntimeCatalog.ByDisplayName(null));
        }

        // A code typed while NES is selected must not be parsed by an SNES codec; a console's codecs are its system pack's, as a session of it answers them.
        [Fact]
        public void Cheat_codecs_follow_the_selected_console()
        {
            Assert.NotNull(Codecs("SNES").AutoDetect);

            // Both consoles decode now, and not with each other's formats - see EmuSen_Cheats.md §2.
            Assert.NotNull(Codecs("NES").AutoDetect);
            Assert.NotNull(Codecs("NES").Explicit);

            // A NES Game Genie code is letters the SNES device's alphabet has no room for.
            Assert.True(Codecs("NES").Explicit!.CanDecode("SXIOPO"));
            Assert.False(Codecs("SNES").Explicit!.CanDecode("SXIOPO"));
        }

        // The console filter narrows the cheat database to that console's libretro folders; the PlayStation 2 alone has none, the database holding no PlayStation 2 folder - see Sapphire_Plan.md §4.1 and §4.5.
        [Fact]
        public void Each_core_claims_at_least_one_cheat_system_folder()
        {
            foreach (var core in RuntimeCatalog.Consoles)
            {
                if (core.Console == "PlayStation 2") Assert.Empty(core.CheatSystems);
                else Assert.NotEmpty(core.CheatSystems);
            }
        }

        [Fact]
        public void No_two_cores_claim_the_same_cheat_system_folder()
        {
            var all = RuntimeCatalog.Consoles.SelectMany(c => c.CheatSystems).ToList();

            Assert.Equal(all.Count, all.Distinct(StringComparer.OrdinalIgnoreCase).Count());
        }

        private static ConsoleCore ConsoleOf(string extension) => RuntimeCatalog.Consoles.First(c => c.SupportsExtension(extension));

        private static (ICheatCodeCodec? AutoDetect, ICheatCodeCodec? Explicit) Codecs(string console)
        {
            SystemPack pack = SystemPacks.All.First(p => p.Entry.Console == console);
            return (pack.AutoDetectCodec?.Invoke(), pack.ExplicitCodec?.Invoke());
        }
    }
}
