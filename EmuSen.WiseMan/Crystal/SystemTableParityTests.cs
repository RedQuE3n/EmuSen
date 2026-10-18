using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using System.Text.RegularExpressions;
using EmuSen.Crystal.Systems;
using EmuSen.Galaxia;

namespace EmuSen.WiseMan.Crystal
{
    // Each C# literal entry held to its data file in emusen-systems, field by field, whichever EMUSEN_SYSTEMS_NATIVE chose - see EmuSen_Backend.md §12.
    public class SystemTableParityTests
    {
        private static readonly PropertyInfo[] Properties = typeof(SystemEntry).GetProperties(BindingFlags.Public | BindingFlags.Instance).Where(p => p.Name != "EqualityContract").ToArray();

        private static bool IsCode(PropertyInfo p) => typeof(Delegate).IsAssignableFrom(p.PropertyType);

        // Every field in which two entries differ, by name with both values; code must be the very same delegate.
        public static List<string> Differences(SystemEntry expected, SystemEntry actual)
        {
            var differences = new List<string>();
            foreach (var p in Properties)
            {
                object? a = p.GetValue(expected), b = p.GetValue(actual);
                bool same = IsCode(p) ? ReferenceEquals(a, b)
                    : a is IEnumerable<string> x && b is IEnumerable<string> y ? x.SequenceEqual(y, StringComparer.Ordinal)
                    : Equals(a, b);
                if (!same) differences.Add($"{expected.Id}.{p.Name}: {Show(a)} and {Show(b)}");
            }
            return differences;
        }

        private static string Show(object? value) => value switch
        {
            null => "null",
            string s => $"\"{s}\"",
            IEnumerable<string> list => "[" + string.Join(", ", list) + "]",
            double d => d.ToString("R", System.Globalization.CultureInfo.InvariantCulture),
            _ => value.ToString() ?? "",
        };

        [Fact]
        public void The_library_gives_a_table()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            Assert.Equal(9, SystemTable.Rows.Count);
        }

        [Fact]
        public void Every_literal_equals_its_data_file_field_by_field()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var differences = new List<string>();
            foreach (SystemEntry literal in SystemPacks.Literals)
            {
                if (SystemTable.FromTable(literal) is not { } row) { differences.Add($"{literal.Id}: no data file"); continue; }
                differences.AddRange(Differences(literal, row));
            }
            Assert.True(differences.Count == 0, "the data files differ from the literals:\n" + string.Join("\n", differences));
        }

        // The byte order each data file states is the pack literal's, big for the N64 and unstated for the Genesis family - see EmuSen_Backend.md §18.7.
        [Fact]
        public void Each_pack_s_byte_order_is_its_data_file_s()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            foreach (SystemPack pack in SystemPacks.All) Assert.Equal(pack.ByteOrder, SystemTable.ByteOrders[pack.Entry.Id]);
            Assert.True(SystemPacks.BigEndian("n64"));
            Assert.False(SystemPacks.BigEndian("nes") || SystemPacks.BigEndian("md") || SystemPacks.BigEndian("unknown"));
        }

        [Fact]
        public void The_table_the_literals_and_the_packs_name_the_same_systems_in_one_order()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var literals = SystemPacks.Literals.Select(e => e.Id).ToArray();
            Assert.Equal(literals, SystemTable.Rows.Select(e => e.Id).ToArray());
            Assert.Equal(literals, SystemPacks.All.Select(p => p.Entry.Id).ToArray());
        }

        [Fact]
        public void Every_data_property_has_a_key_and_every_key_of_the_table_a_property()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var data = Properties.Where(p => !IsCode(p)).Select(p => p.Name).Order(StringComparer.Ordinal).ToArray();
            Assert.Equal(data, SystemTable.Keys.Select(k => k.Property).Order(StringComparer.Ordinal).ToArray());
            using var document = JsonDocument.Parse(SystemTable.Json!);
            foreach (JsonElement row in document.RootElement.EnumerateArray())
                Assert.Equal(SystemTable.Keys.Select(k => k.Key).Concat(SystemTable.PackKeys).ToArray(), row.EnumerateObject().Select(m => m.Name).ToArray());
        }

        // The library's code in place of the literal's where the switch chose the table, and a recogniser only where the literal has one - see EmuSen_Backend.md §13.3.
        [Fact]
        public void Each_pack_holds_the_entry_the_switch_chose_with_the_code_it_chose()
        {
            foreach (var (pack, literal) in SystemPacks.All.Zip(SystemPacks.Literals))
            {
                if (SystemTable.Active)
                {
                    Assert.NotSame(literal, pack.Entry);
                    Assert.Equal(new[] { $"{literal.Id}.OpenVgdbBytes", $"{literal.Id}.Recognise" }.Take(literal.Recognise is null ? 1 : 2),
                        Differences(literal, pack.Entry).Select(d => d[..d.IndexOf(':')]));
                    Assert.Equal(literal.Recognise is null, pack.Entry.Recognise is null);
                }
                else Assert.Same(literal, pack.Entry);
            }
            Assert.Equal(Environment.GetEnvironmentVariable(SystemTable.Variable) != "0" && SystemTable.Ready, SystemTable.Active);
        }

        // The comparison itself must see a change in any one data field, or the test above proves nothing.
        [Fact]
        public void The_comparison_tells_a_change_in_each_field()
        {
            SystemEntry nes = SystemPacks.Literals[0];
            var changed = new SystemEntry[]
            {
                nes with { Id = "fc" }, nes with { Name = "Famicom" }, nes with { Console = "FC" }, nes with { Manufacturer = "Sharp" }, nes with { ReleaseYear = 1984 },
                nes with { Extensions = new[] { ".nes", ".unf" } }, nes with { CheatSystems = Array.Empty<string>() }, nes with { OpenVgdbSystems = new[] { "FC" } },
                nes with { CoverAspect = 1.430000001 }, nes with { EsdeSystem = "famicom" }, nes with { EsdeFullName = "Famicom" }, nes with { SelfDelimitingImages = true },
                nes with { AttachedTo = "NES" }, nes with { ScreenScraperSystem = 4 }, nes with { OpenVgdbBytes = f => f }, nes with { Recognise = _ => null },
            };
            Assert.Equal(Properties.Length, changed.Length);
            foreach (var entry in changed) Assert.Single(Differences(nes, entry));
            Assert.Empty(Differences(nes, nes with { Extensions = nes.Extensions.ToArray() }));
        }

        [Fact]
        public void The_header_declares_the_exports_the_facades_call_and_no_other()
        {
            string header = File.ReadAllText(Path.Combine(ConfigRoot.Managed.Directory, "Platform", "include", "emusen_platform.h"));
            Assert.Contains("int64_t emusen_systems_table(uint8_t *out, size_t len);", header);
            var declared = Regex.Matches(header, @"^\w+ \*?(emusen_systems_\w+)\(", RegexOptions.Multiline).Select(m => m.Groups[1].Value).Order(StringComparer.Ordinal);
            Assert.Equal(SystemCode.Exports.Append("emusen_systems_table").Order(StringComparer.Ordinal), declared);
        }
    }
}
