using System;
using System.IO;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;

namespace EmuSen.WiseMan.Galaxia
{
    // Saved cheat sets, and the hex-text on-disk shape that makes them
    // editable from the shell - see EmuSen_Config_Reference.md §3.4.
    public class CheatFileTests : IDisposable
    {
        private readonly string _dir;

        public CheatFileTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "EmuSenCheats_" + Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = _dir;
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        [Fact]
        public void Both_cheat_kinds_survive_a_registry_round_trip()
        {
            var saved = new CheatRegistry();
            saved.AddRamPoke("WRAM", 0x0019, 0x09, "99 lives");
            saved.AddRomPatch(0x00C05F, 0xEA, 0x1F, "no clip", enabled: false);

            ConfigFile<CheatFile> file = CheatFile.For("zelda");
            Assert.True(file.Save(saved.ToCheatFile()));

            var reloaded = new CheatRegistry();
            (int loaded, int skipped) = reloaded.LoadFrom(file.Load()!);

            Assert.Equal(2, loaded);
            Assert.Equal(0, skipped);

            CheatInfo poke = reloaded.GetCheats().Single(c => c.Kind == CheatKind.RamPoke);
            Assert.Equal("WRAM", poke.SpaceName);
            Assert.Equal(0x0019, poke.Address);
            Assert.Equal(0x09u, poke.Value);
            Assert.Equal("99 lives", poke.Description);
            Assert.True(poke.Enabled);

            CheatInfo patch = reloaded.GetCheats().Single(c => c.Kind == CheatKind.RomPatch);
            Assert.Equal(0x00C05F, patch.Address);
            Assert.Equal(0xEAu, patch.Value);
            Assert.Equal((byte?)0x1F, patch.Compare);
            Assert.False(patch.Enabled);
        }

        // Nobody writes a SNES address in decimal, so the file holds hex text.
        [Fact]
        public void Addresses_are_written_as_hex_text()
        {
            var registry = new CheatRegistry();
            registry.AddRamPoke("WRAM", 0x7E0019, 0x09, "lives");
            CheatFile.For("hex").Save(registry.ToCheatFile());

            string json = File.ReadAllText(Path.Combine(_dir, "cheats", "hex.json"));

            Assert.Contains("7E0019", json);
            Assert.DoesNotContain("8257561", json);
        }

        [Fact]
        public void An_unconditional_rom_patch_writes_no_compare_byte()
        {
            var registry = new CheatRegistry();
            registry.AddRomPatch(0x00C05F, 0xEA, null, "unconditional");

            CheatFileEntry entry = registry.ToCheatFile().Cheats.Single();

            Assert.Null(entry.Compare);
            Assert.True(entry.TryParseCompare(out byte? compare));
            Assert.Null(compare);
        }

        [Theory]
        [InlineData("7E0019")]
        [InlineData("0x7E0019")]
        [InlineData("$7E0019")]
        [InlineData("  7e0019  ")]
        public void Hand_written_addresses_are_read_the_way_people_write_them(string address)
        {
            // Moved onto the per-write carrier when a cheat became a list of writes.
            Assert.True(CheatFileWrite.TryHex(address, out uint parsed));
            Assert.Equal(0x7E0019u, parsed);
        }

        [Fact]
        public void A_dash_means_no_compare_byte()
        {
            Assert.True(new CheatFileEntry { Compare = "-" }.TryParseCompare(out byte? compare));
            Assert.Null(compare);
        }

        // One typo shouldn't cost the user the rest of a hand-written file.
        [Fact]
        public void A_broken_entry_is_skipped_and_the_rest_still_load()
        {
            Directory.CreateDirectory(Path.Combine(_dir, "cheats"));
            File.WriteAllText(Path.Combine(_dir, "cheats", "mixed.json"), """
                {
                  "cheats": [
                    { "kind": "RamPoke", "space": "WRAM", "address": "0019", "value": "09", "description": "good" },
                    { "kind": "RamPoke", "space": "WRAM", "address": "ZZZZ", "value": "09", "description": "typo" },
                    { "kind": "RomPatch", "address": "00C05F", "value": "EA", "compare": "QQ", "description": "bad compare" }
                  ]
                }
                """);

            var registry = new CheatRegistry();
            (int loaded, int skipped) = registry.LoadFrom(CheatFile.For("mixed").Load()!);

            Assert.Equal(1, loaded);
            Assert.Equal(2, skipped);
            Assert.Equal("good", registry.GetCheats().Single().Description);
        }

        [Fact]
        public void Kind_is_matched_case_insensitively()
        {
            Assert.True(new CheatFileEntry { Kind = "rompatch" }.IsRomPatch);
            Assert.False(new CheatFileEntry { Kind = "rampoke" }.IsRomPatch);
        }

        // The name reaches this from a shell argument, so it must not be able
        // to walk out of the cheats directory.
        [Theory]
        [InlineData("")]
        [InlineData("   ")]
        [InlineData("..")]
        [InlineData(".")]
        [InlineData("../escape")]
        [InlineData("sub/dir")]
        public void Names_that_could_escape_the_directory_are_refused(string name)
        {
            Assert.False(CheatFile.IsValidName(name));
        }

        [Fact]
        public void Ordinary_names_are_accepted()
        {
            Assert.True(CheatFile.IsValidName("zelda"));
            Assert.True(CheatFile.IsValidName("Super Metroid"));
        }

        [Fact]
        public void ListNames_reports_saved_files_and_is_empty_before_any_exist()
        {
            Assert.Empty(CheatFile.ListNames());

            CheatFile.For("zelda").Save(new CheatFile());
            CheatFile.For("metroid").Save(new CheatFile());

            Assert.Equal(new[] { "metroid", "zelda" }, CheatFile.ListNames());
        }
    }
}
