using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using System.Text.RegularExpressions;
using EmuSen.WiseMan.Bindings.Systems;
using EmuSen.Galaxia;

namespace EmuSen.WiseMan.Crystal
{
    // The systems table held to the C# literal entries as recorded before they were deleted, field by field, and its schema to SystemEntry - see EmuSen_Backend.md §12, §40.1.
    public class SystemTableParityTests
    {
        private static readonly PropertyInfo[] Properties = typeof(SystemEntry).GetProperties(BindingFlags.Public | BindingFlags.Instance).Where(p => p.Name != "EqualityContract").ToArray();

        private static bool IsCode(PropertyInfo p) => typeof(Delegate).IsAssignableFrom(p.PropertyType);

        private static string Show(object? value) => value switch
        {
            null => "null",
            string s => $"\"{s}\"",
            IEnumerable<string> list => "[" + string.Join(", ", list) + "]",
            double d => d.ToString("R", System.Globalization.CultureInfo.InvariantCulture),
            _ => value.ToString() ?? "",
        };

        // Each pack's entry, media, byte order and codecs, under the keys the C#'s were recorded under.
        [Fact]
        public void The_table_s_facts_match_the_golden()
        {
            var golden = new ParityGolden("SystemTable");
            foreach (SystemPack pack in SystemPacks.All)
            {
                SystemEntry e = pack.Entry;
                foreach (var p in Properties.Where(p => !IsCode(p))) golden.Add($"{e.Id}.{p.Name}", Show(p.GetValue(e)));
                golden.Add($"{e.Id}.recognises", (e.Recognise is not null).ToString());
            }
            foreach (SystemPack pack in SystemPacks.All) golden.Add($"{pack.Entry.Id}.byte_order", SystemTable.ByteOrders[pack.Entry.Id] ?? "null");
            foreach (SystemTable.RowMedium row in SystemTable.Media)
            {
                var m = SystemPacks.For(row.System)!.Media!.Single(x => x.Id == row.Id);
                golden.Add($"{row.System}.medium {m.Id}", $"{m.Title}|{m.Noun}|{m.Extension}");
            }
            foreach (SystemCode.CodecPair pair in SystemCode.CodecPairs)
            {
                golden.Add($"{pair.System}.codecs", $"{pair.Auto ?? "null"}|{pair.Explicit ?? "null"}");
                golden.Add($"{pair.System}.cheat_space", SystemTable.CheatSpaces[pair.System] ?? "null");
                foreach (string id in new[] { pair.Auto, pair.Explicit }.OfType<string>())
                {
                    SystemCode.CodecFacts c = SystemCode.Codecs.Single(x => x.Id == id);
                    golden.Add($"{pair.System}.codec {id}", $"{c.Name}|{c.Kind}|{c.Space ?? "null"}");
                }
            }
            golden.Check(100);
        }

        [Fact]
        public void The_library_gives_a_table()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            Assert.Equal(14, SystemTable.Rows.Count);
        }

        // Each pack's codecs are the library's by the pair its row names, and its cheat space is the table's - see EmuSen_Backend.md §13.2.
        [Fact]
        public void Each_pack_hands_out_the_library_s_codecs_by_its_pair()
        {
            foreach (SystemPack pack in SystemPacks.All)
            {
                SystemCode.CodecPair? pair = SystemCode.CodecPairs.SingleOrDefault(p => p.System == pack.Entry.Id);
                Assert.Equal(pair?.Auto, (pack.AutoDetectCodec?.Invoke() as NativeCheatCodec)?.Id);
                Assert.Equal(pair?.Explicit, (pack.ExplicitCodec?.Invoke() as NativeCheatCodec)?.Id);
                Assert.Equal(SystemTable.CheatSpaces[pack.Entry.Id], pack.AutoDetectCodec?.Invoke().SpaceName);
            }
        }

        // The byte order each data file states, big for the N64 and the Genesis family - see EmuSen_Backend.md §18.7, §18.9.
        [Fact]
        public void Each_pack_s_byte_order_is_its_data_file_s()
        {
            Assert.True(SystemPacks.BigEndian("n64") && SystemPacks.BigEndian("md") && SystemPacks.BigEndian("mcd") && SystemPacks.BigEndian("32x"));
            Assert.False(SystemPacks.BigEndian("nes") || SystemPacks.BigEndian("psx") || SystemPacks.BigEndian("unknown"));
        }

        [Fact]
        public void The_table_and_the_packs_name_the_same_systems_in_one_order()
        {
            Assert.Equal(SystemTable.Rows.Select(e => e.Id), SystemPacks.All.Select(p => p.Entry.Id));
            Assert.Equal(SystemTable.Rows.Select(e => e.Id), SystemCode.Code.Select(c => c.Id));
        }

        [Fact]
        public void Every_data_property_has_a_key_and_every_key_of_the_table_a_property()
        {
            var data = Properties.Where(p => !IsCode(p)).Select(p => p.Name).Order(StringComparer.Ordinal).ToArray();
            Assert.Equal(data, SystemTable.Keys.Select(k => k.Property).Order(StringComparer.Ordinal).ToArray());
            using var document = JsonDocument.Parse(SystemTable.Json);
            foreach (JsonElement row in document.RootElement.EnumerateArray())
                Assert.Equal(SystemTable.Keys.Select(k => k.Key).Concat(SystemTable.PackKeys).ToArray(), row.EnumerateObject().Select(m => m.Name).ToArray());
        }

        // A pack's entry carries the library's OpenVGDB rule, and its recogniser only where the library has one for the system - see EmuSen_Backend.md §13.3.
        [Fact]
        public void Each_pack_s_entry_carries_the_library_s_code()
        {
            foreach (SystemPack pack in SystemPacks.All)
            {
                SystemCode.SystemCodeFacts facts = SystemCode.Code.Single(c => c.Id == pack.Entry.Id);
                Assert.Equal(facts.Recognises, pack.Entry.Recognise is not null);
                Assert.Equal(pack.Entry.Id is "nes" or "gb" or "gbc", facts.Header);
                byte[] file = { 1, 2, 3 };
                Assert.Equal(SystemCode.OpenVgdbBytes(pack.Entry.Id, file), pack.Entry.OpenVgdbBytes(file));
            }
        }

        [Fact]
        public void The_header_declares_the_exports_the_facades_call_and_no_other()
        {
            string header = File.ReadAllText(Path.Combine(ConfigRoot.Directory, "Platform", "include", "emusen_platform.h"));
            var declared = Regex.Matches(header, @"^\w+ \*?(emusen_systems_\w+)\(", RegexOptions.Multiline).Select(m => m.Groups[1].Value).Order(StringComparer.Ordinal);
            Assert.Equal(SystemCode.Exports.Append("emusen_systems_libretro_cores").Order(StringComparer.Ordinal), declared);
        }
    }
}
