using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;

namespace EmuSen.WiseMan.Mistress
{
    // The one console context the library and both cheat windows share - see EmuSen_Multicore.md §10.
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

        // A console the user actually picked must survive a round trip.
        [Fact]
        public void A_real_choice_is_kept()
        {
            string nes = CoreCatalog.ByExtension(".nes")!.DisplayName;
            new AppSettings { SelectedCore = nes }.Save();

            Assert.Equal(nes, AppSettings.Load().SelectedCore);
        }

        [Fact]
        public void The_filter_list_offers_all_consoles_first_then_every_core()
        {
            Assert.Equal(CoreCatalog.AllConsoles, CoreCatalog.FilterChoices[0]);
            Assert.Equal(CoreCatalog.Cores.Count + 1, CoreCatalog.FilterChoices.Count);

            foreach (var core in CoreCatalog.Cores)
            {
                Assert.Contains(core.DisplayName, CoreCatalog.FilterChoices);
            }
        }

        // Galaxia persists it and the catalog displays it, so the two spellings must not drift.
        [Fact]
        public void Galaxia_and_the_catalog_agree_on_the_all_consoles_label()
        {
            Assert.Equal(AppSettings.AllConsoles, CoreCatalog.AllConsoles);
        }

        [Fact]
        public void An_unknown_console_name_resolves_to_no_filter()
        {
            Assert.Null(CoreCatalog.ByDisplayName(CoreCatalog.AllConsoles));
            Assert.Null(CoreCatalog.ByDisplayName("Master System (Endou)"));
            Assert.Null(CoreCatalog.ByDisplayName(null));
        }

        // A code typed while NES is selected must not be parsed by an SNES codec.
        [Fact]
        public void Cheat_codecs_follow_the_selected_console()
        {
            string snes = CoreCatalog.ByExtension(".sfc")!.DisplayName;
            string nes = CoreCatalog.ByExtension(".nes")!.DisplayName;

            Assert.NotNull(CoreFactory.CheatCodecsFor(snes).AutoDetect);

            // Moon has no cheat-code format yet - see EmuSen_Multicore.md §4.
            Assert.Null(CoreFactory.CheatCodecsFor(nes).AutoDetect);
            Assert.Null(CoreFactory.CheatCodecsFor(nes).Explicit);
        }

        // With no console chosen there is no console to be wrong about.
        [Fact]
        public void All_consoles_falls_back_to_the_default_codecs()
        {
            Assert.NotNull(CoreFactory.CheatCodecsFor(CoreCatalog.AllConsoles).AutoDetect);
            Assert.NotNull(CoreFactory.CheatCodecsFor(null).Explicit);
        }

        // The console filter narrows the cheat database to that console's libretro folders.
        [Fact]
        public void Each_core_claims_at_least_one_cheat_system_folder()
        {
            foreach (var core in CoreCatalog.Cores)
            {
                Assert.NotEmpty(core.CheatSystemNames);
            }
        }

        [Fact]
        public void No_two_cores_claim_the_same_cheat_system_folder()
        {
            var all = CoreCatalog.Cores.SelectMany(c => c.CheatSystemNames).ToList();

            Assert.Equal(all.Count, all.Distinct(StringComparer.OrdinalIgnoreCase).Count());
        }
    }
}
