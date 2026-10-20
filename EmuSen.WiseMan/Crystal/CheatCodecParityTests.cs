using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.Genesis;
using EmuSen.Crystal.Systems.Snes;
using EmuSen.Galaxia;

namespace EmuSen.WiseMan.Crystal
{
    // The cheat codecs in emusen-systems held to the C# codecs: every code of the cheat tests and the golden corpus, and generated codes, through every codec, each answer or refusal the same - see EmuSen_Backend.md §13.
    public class CheatCodecParityTests
    {
        private static readonly string Root = ConfigRoot.Managed.Directory;

        // The files whose codes the class takes: every cheat test, and the shell's recordings of `cheat`.
        private static readonly string[] Sources =
        {
            "EmuSen.WiseMan/Cores/CheatCodecAdapterTests.cs", "EmuSen.WiseMan/Cores/NesGameGenieCodecTests.cs", "EmuSen.WiseMan/Cores/MoonCheatTests.cs",
            "EmuSen.WiseMan/Cores/MarsCheatTests.cs", "EmuSen.WiseMan/Cores/MercuryRtCoreTests.cs", "EmuSen.WiseMan/Cores/MercuryDebugTargetTests.cs",
            "EmuSen.WiseMan/Cores/DiamondTests.cs", "EmuSen.WiseMan/Cores/MoonRtFrontendTests.cs", "EmuSen.WiseMan/Cores/MarsRtFrontendTests.cs",
            "EmuSen.WiseMan/DianaSh/GenesisCheatFormatsTests.cs", "EmuSen.WiseMan/DianaSh/SnesSystemPackTests.cs", "EmuSen.WiseMan/DianaSh/PsxCheatFormatsTests.cs",
            "EmuSen.WiseMan/DianaSh/CheatChtCommandTests.cs", "EmuSen.WiseMan/DianaSh/CheatMasterSwitchTests.cs", "EmuSen.WiseMan/DianaSh/CheatDatabaseTests.cs",
            "EmuSen.WiseMan/DianaSh/CheatPruneCommandTests.cs", "EmuSen.WiseMan/DianaSh/CheatDatabaseGamesTests.cs", "EmuSen.WiseMan/Mistress/ActiveCheatsWindowTests.cs",
            "EmuSen.WiseMan/Mistress/CheatDatabaseWindowTests.cs", "EmuSen.WiseMan/Mistress/ConsoleContextTests.cs", "EmuSen.WiseMan/Properties/CodecProperties.cs",
        };

        // Every string literal of the sources, and every argument after `cheat` in the golden corpus, as written.
        internal static IReadOnlyList<string> TestCodes()
        {
            var codes = new SortedSet<string>(StringComparer.Ordinal);
            foreach (string source in Sources)
                foreach (Match m in Regex.Matches(File.ReadAllText(Path.Combine(Root, source)), "\"((?:[^\"\\\\\\n]|\\\\.)*)\""))
                    codes.Add(Regex.Unescape(m.Groups[1].Value));
            foreach (string line in File.ReadAllLines(Path.Combine(Root, "EmuSen.WiseMan", "DianaSh", "Golden", "cheat.golden")).Where(l => l.StartsWith("$ cheat ", StringComparison.Ordinal)))
            {
                string[] words = line[8..].Split(' ', StringSplitOptions.RemoveEmptyEntries);
                foreach (string word in words) codes.Add(word);
                if (words.Length > 1) codes.Add(string.Join(' ', words[1..]));
            }
            return codes.ToArray();
        }

        private const string Hex = "0123456789ABCDEFabcdef";
        private static readonly string[] Separators = { " ", "-", ":", "+", "\n", "\t", " ", ",", "", " + ", "\r\n" };
        private static readonly string[] Odd = { "?", "x", "X", "y", "ſ", "ı", "１", "٣", "²", "Ⅳ", "ß", "́", "\U0001F600", "　", " ", "Z" };

        private static string Digits(Random r, int n, string alphabet = Hex) => new(Enumerable.Range(0, n).Select(_ => alphabet[r.Next(alphabet.Length)]).ToArray());

        private static string Joined(Random r, IEnumerable<string> parts) => string.Join(Separators[r.Next(Separators.Length)], parts);

        // Codes of every family's shape and of none, with the separators, placeholders and characters the codecs treat specially.
        internal static IReadOnlyList<string> GeneratedCodes()
        {
            var r = new Random(20261017);
            var codes = new List<string> { "", " ", "+", "++", " + ", " ", "-", ":" };
            for (int i = 0; i < 6000; i++)
            {
                string code = (i % 12) switch
                {
                    0 => Digits(r, r.Next(1, 30)),
                    1 => Joined(r, Enumerable.Range(0, r.Next(1, 4)).Select(_ => Digits(r, r.Next(1, 13)))),
                    2 => Digits(r, r.Next(5, 10), "APZLGITYEOXUKSVNapzlgityeoxuksvnſı -"),
                    3 => Digits(r, 4, "DF4709156BC8A23E" + Hex) + "-" + Digits(r, 4, "DF4709156BC8A23E" + Hex),
                    4 => Digits(r, 4, "ABCDEFGHJKLMNPRSTVWXYZ0123456789abcdIOQUſ") + Separators[r.Next(3)] + Digits(r, 4, "ABCDEFGHJKLMNPRSTVWXYZ0123456789"),
                    5 => (r.Next(2) == 0 ? "FF" : r.Next(2) == 0 ? "E0" : "00") + Digits(r, 4) + new[] { ":", " ", "-", "" }[r.Next(4)] + Digits(r, new[] { 1, 2, 3, 4, 5, 8, 9 }[r.Next(7)]),
                    6 => Joined(r, Enumerable.Range(0, r.Next(1, 5)).Select(_ => $"{r.Next(256):X2}{Digits(r, 6)} {Digits(r, 4)}")),
                    7 => Joined(r, Enumerable.Range(0, r.Next(1, 5)).Select(_ => $"{new[] { 0x80, 0x81, 0xA0, 0xA1, 0xD0, 0xD1, 0xD2, 0xD3, 0x50, 0x88, 0xF0, 0xEE, 0xDE, 0xCC, 0x30, 0xE0, 0x10, 0x11, 0x20, 0x21, 0xC0, 0xC2, 0xD4, 0x12, 0x70, 0x90, 0x00, 0x40, 0xB0, 0xF7 }[r.Next(30)]:X2}{Digits(r, 6)} {Digits(r, r.Next(1, 5))}")),
                    8 => $"5000{r.Next(4):X2}{r.Next(256):X2} {Digits(r, 4)}+{new[] { "80", "81", "A0", "30", "D0" }[r.Next(5)]}{Digits(r, 6)} {Digits(r, 4)}",
                    9 => Joined(r, new[] { Digits(r, 3), Digits(r, 3), Digits(r, r.Next(2, 4)) }),
                    10 => Digits(r, r.Next(4, 12)) + Odd[r.Next(Odd.Length)] + Digits(r, r.Next(0, 6)),
                    _ => Joined(r, Enumerable.Range(0, r.Next(1, 3)).Select(_ => Digits(r, 12))) + (r.Next(4) == 0 ? "?" : ""),
                };
                codes.Add(r.Next(10) == 0 ? "  " + code + " " : code);
            }
            return codes;
        }

        private static string Shown(Func<object?> answer)
        {
            try
            {
                return answer() switch
                {
                    null => "null",
                    IReadOnlyList<CheatWrite> writes => "[" + string.Join("; ", writes.Select(w =>
                        $"{w.Space} {w.Address} {w.Value} {w.Width} {w.Type} {w.BitPosition?.ToString() ?? "-"} {w.BigEndian} {w.RepeatCount} {w.RepeatAddAddress} {w.RepeatAddValue}")) + "]",
                    var other => other.ToString()!,
                };
            }
            catch (Exception e) { return $"threw {e.GetType().Name}: {e.Message}"; }
        }

        private static IEnumerable<(string Id, Func<ICheatCodeCodec> Managed)> Codecs() =>
            SystemPacks.ManagedCodecs.SelectMany(c => new[] { c.Auto, c.Explicit }).OfType<(string Id, Func<ICheatCodeCodec> Managed)>().DistinctBy(c => c.Id);

        private static List<string> Compare(IEnumerable<string> codes)
        {
            var differences = new List<string>();
            foreach (var (id, factory) in Codecs())
            {
                ICheatCodeCodec managed = factory(), native = new NativeCheatCodec(id, factory());
                foreach (string code in codes)
                {
                    foreach (var (what, ask) in new (string, Func<ICheatCodeCodec, object?>)[]
                    {
                        ("CanDecode", c => c.CanDecode(code)), ("Decode", c => c.Decode(code)), ("DecodeCompare", c => c.DecodeCompare(code)), ("DecodeWrites", c => c.DecodeWrites(code)),
                    })
                    {
                        string n = Shown(() => ask(native)), m = Shown(() => ask(managed));
                        if (n != m) differences.Add($"{id} {what} '{code}':\n  library: {n}\n  C#:      {m}");
                    }
                }
            }
            return differences;
        }

        [Fact]
        public void Every_codec_has_the_csharps_name_kind_space_and_place()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            foreach (var (id, factory) in Codecs())
            {
                ICheatCodeCodec managed = factory();
                SystemCode.CodecFacts facts = SystemCode.Codecs.Single(c => c.Id == id);
                Assert.Equal((managed.Name, managed.Kind, managed.SpaceName), (facts.Name, facts.Kind, facts.Space));
            }
            Assert.Equal(SystemPacks.ManagedCodecs.Select(c => (c.System, c.Auto?.Id, c.Explicit?.Id)), SystemCode.CodecPairs.Select(p => (p.System, p.Auto, p.Explicit)));
            Assert.Equal(SystemCode.Codecs.Select(c => c.Id).Order(StringComparer.Ordinal), Codecs().Select(c => c.Id).Order(StringComparer.Ordinal));
            foreach (var (system, auto, _) in SystemPacks.ManagedCodecs)
                Assert.Equal(auto?.Managed().SpaceName, SystemTable.CheatSpaces[system]);
        }

        [Fact]
        public void Each_pack_hands_out_the_codec_the_switch_chose()
        {
            foreach (SystemPack pack in SystemPacks.All)
                foreach (ICheatCodeCodec? codec in new[] { pack.AutoDetectCodec?.Invoke(), pack.ExplicitCodec?.Invoke() })
                    if (codec is not null) Assert.Equal(SystemTable.Active, codec is NativeCheatCodec);
            Assert.Equal(11, SystemPacks.All.Count(p => p.AutoDetectCodec is not null));
        }

        [Fact]
        public void Every_code_of_the_cheat_tests_decodes_alike_through_every_codec()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var codes = TestCodes();
            Assert.True(codes.Count > 900, $"{codes.Count} codes");
            var differences = Compare(codes);
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(15)));
        }

        [Fact]
        public void Generated_codes_decode_alike_through_every_codec()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var differences = Compare(GeneratedCodes());
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(15)));
        }

        // The two Game Genie encoders, the other way: every patch the cheat tests decode, and generated ones, encoded alike.
        [Fact]
        public void The_game_genie_encoders_agree()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1990);
            var differences = new List<string>();
            var patches = Enumerable.Range(0, 3000).Select(_ => (r.Next(int.MinValue, int.MaxValue), (uint)r.Next())).ToList();
            foreach (string code in TestCodes())
            {
                if (SnesCheatFormats.IsGameGenie(code)) { var (a, v) = SnesCheatFormats.DecodeGameGenie(code); patches.Add((a, v)); }
                if (GenesisCheatFormats.IsGameGenie(code)) { var (a, v) = GenesisCheatFormats.DecodeGameGenie(code); patches.Add((a, v)); }
            }
            foreach (var (address, value) in patches)
            {
                string snes = SnesCheatFormats.EncodeGameGenie(address, (byte)value), genesis = GenesisCheatFormats.EncodeGameGenie(address, (ushort)value);
                if (SystemCode.CheatEncode("snes.gg", address, value) != snes) differences.Add($"snes {address:X} {value:X}: {snes}");
                if (SystemCode.CheatEncode("md.gg", address, value) != genesis) differences.Add($"genesis {address:X} {value:X}: {genesis}");
            }
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(15)));
        }
    }
}
