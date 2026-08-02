using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.WiseMan.DianaOS
{
    // RetroArch/libretro .cht import and export - see `man cheat`.
    public class ChtFileTests
    {
        // Stands in for a core's own code decoder. Matches the SNES layout
        // the libretro database actually uses for this system: 6 hex digits
        // of address then 2 of value.
        private sealed class HexCodec : ICheatCodeCodec
        {
            public string Name => "Test";
            public CheatCodeKind Kind => CheatCodeKind.RamPoke;
            public string? SpaceName => "CpuBus";

            public bool CanDecode(string code) => Normalize(code).Length == 8;

            public (int Address, byte Value) Decode(string code)
            {
                string hex = Normalize(code);
                if (hex.Length != 8) throw new FormatException(code);
                return (Convert.ToInt32(hex[..6], 16), Convert.ToByte(hex.Substring(6, 2), 16));
            }

            private static string Normalize(string code) => new((code ?? "").Where(Uri.IsHexDigit).ToArray());
        }

        // The shape a real libretro-database SNES file has.
        private const string SuperMarioWorld = """
            cheats = 2

            cheat0_desc = "Infinite Maximum Coins"
            cheat0_code = "7E0DBF63"
            cheat0_enable = false

            cheat1_desc = "Powerup Always ?? In Level"
            cheat1_code = "7E001900"
            cheat1_enable = false
            """;

        [Fact]
        public void A_real_libretro_database_file_imports()
        {
            ChtParseResult result = ChtFile.Parse(SuperMarioWorld, new HexCodec(), "CpuBus");

            Assert.Equal(0, result.Skipped);
            Assert.Equal(2, result.Cheats.Count);

            ChtCheat first = result.Cheats[0];
            Assert.Equal("Infinite Maximum Coins", first.Description);
            Assert.False(first.Enabled);
            Assert.Equal(0x7E0DBF, first.Writes.Single().Address);
            Assert.Equal(0x63u, first.Writes.Single().Value);
            Assert.Equal("CpuBus", first.Writes.Single().Space);
        }

        // The '+' form is how RetroArch expresses a multi-address cheat.
        [Fact]
        public void Codes_joined_with_plus_become_one_cheat_with_several_writes()
        {
            const string text = """
                cheats = 1
                cheat0_desc = "Max everything"
                cheat0_code = "7E0DBF63+7E0DC004+7E0019FF"
                cheat0_enable = true
                """;

            ChtParseResult result = ChtFile.Parse(text, new HexCodec(), "CpuBus");

            ChtCheat cheat = Assert.Single(result.Cheats);
            Assert.True(cheat.Enabled);
            Assert.Equal(3, cheat.Writes.Count);
            Assert.Equal(new[] { 0x7E0DBF, 0x7E0DC0, 0x7E0019 }, cheat.Writes.Select(w => w.Address));
            Assert.Equal(new uint[] { 0x63, 0x04, 0xFF }, cheat.Writes.Select(w => w.Value));
        }

        // The "retro" handler form, whose numbers are decimal, not hex.
        [Fact]
        public void The_explicit_field_form_reads_every_field_as_decimal()
        {
            const string text = """
                cheats = 1
                cheat0_desc = "Infinite health"
                cheat0_handler = 1
                cheat0_enable = true
                cheat0_cheat_type = 1
                cheat0_memory_search_size = 4
                cheat0_address = 8257471
                cheat0_value = 999
                cheat0_address_bit_position = 0
                cheat0_big_endian = false
                cheat0_repeat_count = 3
                cheat0_repeat_add_to_address = 2
                cheat0_repeat_add_to_value = 1
                """;

            ChtParseResult result = ChtFile.Parse(text, null, "CpuBus");

            CheatWrite write = Assert.Single(result.Cheats).Writes.Single();
            Assert.Equal(8257471, write.Address);
            Assert.Equal(999u, write.Value);
            Assert.Equal(2, write.EffectiveWidth);      // memory_search_size 4 = 16-bit
            Assert.Equal(CheatWriteType.Set, write.Type);
            Assert.Equal(3, write.EffectiveRepeatCount);
            Assert.Equal(2, write.RepeatAddAddress);
            Assert.Equal(1u, write.RepeatAddValue);
            Assert.Null(write.BitPosition);
            Assert.False(write.BigEndian);
        }

        [Theory]
        [InlineData(2, CheatWriteType.Increase)]
        [InlineData(3, CheatWriteType.Decrease)]
        public void Cheat_type_maps_onto_the_write_type(int chtType, CheatWriteType expected)
        {
            string text = $"""
                cheats = 1
                cheat0_desc = "x"
                cheat0_address = 16
                cheat0_value = 1
                cheat0_cheat_type = {chtType}
                """;

            Assert.Equal(expected, ChtFile.Parse(text, null, "CpuBus").Cheats.Single().Writes.Single().Type);
        }

        // cheat_type 0 means the entry exists but does nothing.
        [Fact]
        public void A_cheat_type_of_zero_is_skipped_rather_than_imported_as_a_no_op()
        {
            const string text = """
                cheats = 1
                cheat0_desc = "does nothing"
                cheat0_address = 16
                cheat0_value = 1
                cheat0_cheat_type = 0
                """;

            ChtParseResult result = ChtFile.Parse(text, null, "CpuBus");

            Assert.Empty(result.Cheats);
            Assert.Equal(1, result.Skipped);
        }

        [Fact]
        public void A_sub_byte_search_size_becomes_a_bit_write()
        {
            const string text = """
                cheats = 1
                cheat0_desc = "flag"
                cheat0_address = 64
                cheat0_value = 1
                cheat0_cheat_type = 1
                cheat0_memory_search_size = 0
                cheat0_address_bit_position = 5
                """;

            CheatWrite write = ChtFile.Parse(text, null, "CpuBus").Cheats.Single().Writes.Single();

            Assert.Equal(5, write.BitPosition);
        }

        [Fact]
        public void A_thirty_two_bit_search_size_becomes_a_four_byte_write()
        {
            const string text = """
                cheats = 1
                cheat0_desc = "score"
                cheat0_address = 100
                cheat0_value = 1
                cheat0_memory_search_size = 5
                """;

            Assert.Equal(4, ChtFile.Parse(text, null, "CpuBus").Cheats.Single().Writes.Single().EffectiveWidth);
        }

        [Fact]
        public void An_undecodable_code_is_skipped_and_the_rest_still_import()
        {
            const string text = """
                cheats = 2
                cheat0_desc = "broken"
                cheat0_code = "NOTHEX"
                cheat0_enable = false
                cheat1_desc = "fine"
                cheat1_code = "7E001900"
                cheat1_enable = false
                """;

            ChtParseResult result = ChtFile.Parse(text, new HexCodec(), "CpuBus");

            Assert.Equal("fine", Assert.Single(result.Cheats).Description);
            Assert.Equal(1, result.Skipped);
        }

        // A newer RetroArch field this build does not read must not make the
        // whole file unparseable.
        [Fact]
        public void Unknown_fields_and_comments_are_ignored()
        {
            const string text = """
                # a comment
                cheats = 1
                cheat0_desc = "fine"
                cheat0_code = "7E001900"
                cheat0_enable = true
                cheat0_rumble_type = 4
                cheat0_some_future_field = "whatever"
                """;

            Assert.Single(ChtFile.Parse(text, new HexCodec(), "CpuBus").Cheats);
        }

        [Fact]
        public void A_file_with_no_cheats_header_yields_nothing_rather_than_throwing()
        {
            Assert.Empty(ChtFile.Parse("nonsense\nmore nonsense", new HexCodec(), "CpuBus").Cheats);
            Assert.Empty(ChtFile.Parse("", new HexCodec(), "CpuBus").Cheats);
        }

        // --- Export ---

        [Fact]
        public void Export_round_trips_every_field_back_through_import()
        {
            var original = new List<ChtCheat>
            {
                new ChtCheat
                {
                    Description = "Infinite health",
                    Enabled = true,
                    Writes = new[]
                    {
                        new CheatWrite { Space = "CpuBus", Address = 0x7E0DBF, Value = 0x1234, Width = 2, Type = CheatWriteType.Decrease, BigEndian = true, RepeatCount = 5, RepeatAddAddress = 4, RepeatAddValue = 2 },
                    },
                },
            };

            string text = ChtFile.Write(original);
            ChtParseResult reparsed = ChtFile.Parse(text, null, "CpuBus");

            ChtCheat cheat = Assert.Single(reparsed.Cheats);
            Assert.Equal("Infinite health", cheat.Description);
            Assert.True(cheat.Enabled);

            CheatWrite write = cheat.Writes.Single();
            Assert.Equal(0x7E0DBF, write.Address);
            Assert.Equal(0x1234u, write.Value);
            Assert.Equal(2, write.EffectiveWidth);
            Assert.Equal(CheatWriteType.Decrease, write.Type);
            Assert.True(write.BigEndian);
            Assert.Equal(5, write.EffectiveRepeatCount);
            Assert.Equal(4, write.RepeatAddAddress);
            Assert.Equal(2u, write.RepeatAddValue);
        }

        [Fact]
        public void Export_writes_the_header_count_and_retroarch_field_names()
        {
            string text = ChtFile.Write(new List<ChtCheat>
            {
                new ChtCheat { Description = "one", Enabled = false, Writes = new[] { CheatWrite.Poke("CpuBus", 0x10, 1) } },
                new ChtCheat { Description = "two", Enabled = true, Writes = new[] { CheatWrite.Poke("CpuBus", 0x20, 2) } },
            });

            Assert.Contains("cheats = 2", text);
            Assert.Contains("cheat0_desc = \"one\"", text);
            Assert.Contains("cheat0_enable = false", text);
            Assert.Contains("cheat1_enable = true", text);
            Assert.Contains("cheat1_address = 32", text); // decimal, as RetroArch writes it
        }

        [Fact]
        public void A_multi_write_cheat_exports_its_extra_writes_as_a_code_string()
        {
            string text = ChtFile.Write(new List<ChtCheat>
            {
                new ChtCheat
                {
                    Description = "both",
                    Enabled = true,
                    Writes = new[] { CheatWrite.Poke("CpuBus", 0x7E0DBF, 0x63), CheatWrite.Poke("CpuBus", 0x7E0019, 0xFF) },
                },
            });

            Assert.Contains("cheat0_code = \"7E0DBF63+7E0019FF\"", text);
        }
    }
}
