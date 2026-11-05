using System;
using System.Linq;
using EmuSen.Crystal.Client;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Mistress
{
    // How a tile names the engine a console runs on, from the console's choice and the runtime's own cores listing, never from a table of names - see EmuSen_Interface.md §24.6.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class RuntimeCatalogEngineTests
    {
        private static CrystalRuntime Runtime(LibretroEntryFixture entry) =>
            new(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory }, PlayerCores = new[] { entry.Cores } });

        [Fact]
        public void The_cores_listing_names_a_libretro_core_by_its_corename_and_an_EmuSen_core_by_its_info_name()
        {
            using var entry = new LibretroEntryFixture(systemId: "playstation2");
            using CrystalRuntime runtime = Runtime(entry);
            var names = RuntimeCatalog.EnginesIn(runtime.Cores(), RuntimeCatalog.MakerOfSystem);
            Assert.Equal(new EngineShown("TestCore", true) { Mark = EngineMarks.Libretro }, names[entry.Found.EngineName]);
            Assert.All(names.Values, n => Assert.False(n.Name.EndsWith("RT", StringComparison.Ordinal), n.Name));
            Assert.Contains(names, n => !n.Value.Libretro && n.Key != n.Value.Name && n.Key.StartsWith(n.Value.Name, StringComparison.Ordinal));
        }

        // Each engine's mark is its own: the badge for a libretro core, an astronomical symbol for a core named after a body, else its maker's body glyph (EmuSen_Interface.md §24.6).
        [Fact]
        public void Each_engine_of_the_listing_carries_its_own_mark()
        {
            using var entry = new LibretroEntryFixture(systemId: "playstation2");
            using CrystalRuntime runtime = Runtime(entry);
            using var cores = System.Text.Json.JsonDocument.Parse(runtime.Cores());
            var names = RuntimeCatalog.EnginesIn(runtime.Cores(), RuntimeCatalog.MakerOfSystem);
            int own = 0;
            foreach (var row in cores.RootElement.EnumerateArray())
            {
                if (row.GetProperty("engine").GetString() is not { } engine) continue;
                EngineShown shown = names[engine];
                if (shown.Libretro) { Assert.Same(EngineMarks.Libretro, shown.Mark); continue; }
                own++;
                string? maker = row.GetProperty("systems").EnumerateArray().Select(x => RuntimeCatalog.MakerOfSystem(x.GetString()!)).FirstOrDefault(m => m is { Length: > 0 });
                if (maker == "Sega") Assert.Same(EngineMarks.Shard, shown.Mark);
                else if (maker == "Sony") Assert.Same(EngineMarks.Gem, shown.Mark);
                else Assert.True(shown.Mark!.Stroke is { Length: > 0 } && shown.Mark != EngineMarks.Planet, $"{shown.Name} has its own symbol");
                Assert.Null(shown.Mark!.Pixels);
            }
            Assert.True(own >= 8, $"{own} EmuSen cores listed");
            Assert.Equal(4, names.Values.Where(n => !n.Libretro && n.Mark is { } m && m != EngineMarks.Shard && m != EngineMarks.Gem && m != EngineMarks.Planet).Select(n => n.Mark).Distinct().Count());
        }

        // Dreamcast runs on a hosted Flycast today: its choice, read from the runtime, is named as the core's info names it, with the libretro tag.
        [Fact]
        public void Dreamcast_on_Flycast_reads_FLYCAST_with_the_LIBRETRO_tag()
        {
            using var entry = new LibretroEntryFixture(systemId: "dreamcast", stem: "flycast", corename: "Flycast");
            using CrystalRuntime runtime = Runtime(entry);
            string chosen = runtime.Setting("Dreamcast", RuntimeCatalog.EngineKey)!;
            Assert.Equal(entry.Found.EngineName, chosen);
            EngineShown shown = RuntimeCatalog.Shown(chosen, RuntimeCatalog.EnginesIn(runtime.Cores(), RuntimeCatalog.MakerOfSystem));
            Assert.Equal(new EngineShown("Flycast", true) { Mark = EngineMarks.Libretro }, shown);
            Assert.Equal(("FLYCAST", "LIBRETRO"), (shown.Name.ToUpperInvariant(), ConsolesScreen.TagOf(shown).ToUpperInvariant()));
            Assert.Equal("runs on Flycast, a libretro core", ConsolesScreen.RunsOn(shown));
        }

        [Fact]
        public void A_name_without_the_port_s_mark_is_kept_and_a_row_with_no_engine_is_skipped()
        {
            var names = RuntimeCatalog.EnginesIn("""
                [{"engine":"Kunzite","name":"Kunzite","libretro":null},{"engine":null,"name":null,"libretro":null},
                 {"engine":"Beetle","name":"Beetle PSX HW","libretro":{"corename":null}},{"engine":"ART","name":"ART","libretro":null}]
                """);
            Assert.Equal(3, names.Count);
            Assert.Equal(new EngineShown("Kunzite", false) { Mark = EngineMarks.Planet }, names["Kunzite"]);
            Assert.Equal(new EngineShown("Beetle PSX HW", true) { Mark = EngineMarks.Libretro }, names["Beetle"]);
            Assert.Equal(new EngineShown("ART", false) { Mark = EngineMarks.Planet }, names["ART"]);
            Assert.Equal(new EngineShown("Unlisted (Rust)", false) { Mark = EngineMarks.Planet }, RuntimeCatalog.Shown("Unlisted (Rust)", names));
        }
    }
}
