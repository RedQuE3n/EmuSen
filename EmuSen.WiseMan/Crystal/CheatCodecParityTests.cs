using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.WiseMan.Bindings.Cheats;
using EmuSen.WiseMan.Bindings.Systems;

namespace EmuSen.WiseMan.Crystal
{
    // The cheat codecs in emusen-systems held to the C# codecs' answers as recorded before they were deleted: every code of the cheat tests and the golden corpus, and generated codes, through every codec, each answer or refusal - see EmuSen_Backend.md §13, §40.1.
    public class CheatCodecParityTests
    {
        // The codes of the cheat tests and of the golden corpus's `cheat` file as they stood on 2026-11-04, kept in the golden whose keys they are.
        internal static IReadOnlyList<string> TestCodes() =>
            ParityGolden.Read("CheatCodesOfTheTests").Select(l => l.Key).Where(k => k.StartsWith("nes.raw ", StringComparison.Ordinal)).Select(k => k["nes.raw ".Length..]).ToArray();

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

        // Every codec once, in the order of the table's pairs.
        private static IEnumerable<string> Codecs() => SystemCode.CodecPairs.SelectMany(p => new[] { p.Auto, p.Explicit }).OfType<string>().Distinct();

        private static void Record(IReadOnlyList<string> codes, ParityGolden golden, bool keyByCode = true)
        {
            foreach (string id in Codecs())
            {
                ICheatCodeCodec codec = new NativeCheatCodec(id);
                for (int at = 0; at < codes.Count; at++)
                {
                    string code = codes[at];
                    string answers = string.Join(" | ", new Func<object?>[] { () => codec.CanDecode(code), () => codec.Decode(code), () => codec.DecodeCompare(code), () => codec.DecodeWrites(code) }.Select(Shown));
                    if (keyByCode) golden.Add($"{id} {code}", answers);
                    else golden.AddChunked(id, answers, 50);
                }
            }
        }

        [Fact]
        public void Every_code_of_the_cheat_tests_decodes_as_recorded_through_every_codec()
        {
            var codes = TestCodes();
            Assert.True(codes.Count > 900, $"{codes.Count} codes");
            var golden = new ParityGolden("CheatCodesOfTheTests");
            Record(codes, golden);
            golden.Check(9000);
        }

        [Fact]
        public void Generated_codes_decode_as_recorded_through_every_codec()
        {
            var golden = new ParityGolden("CheatCodesGenerated");
            Record(GeneratedCodes(), golden, keyByCode: false);
            golden.Check(1000);
        }

        // A lone surrogate crosses as U+FFFD, which no codec takes as a digit, so a code holding one decodes as the code holding U+FFFD - see EmuSen_Backend.md §40.1.
        [Fact]
        public void A_code_holding_a_lone_surrogate_decodes_as_one_holding_the_replacement()
        {
            foreach (string id in Codecs())
                foreach (string code in new[] { "7E0DBE:\uD800", "\uDC00AAAA-AAAA", "800F\uD83D1234 0001" })
                {
                    string text = ParityGolden.Unescape(code);
                    ICheatCodeCodec codec = new NativeCheatCodec(id);
                    Assert.Equal(Shown(() => codec.CanDecode(EmuSen.Galaxia.Native.PlatformText.Mend(text))) + Shown(() => codec.DecodeWrites(EmuSen.Galaxia.Native.PlatformText.Mend(text))),
                        Shown(() => codec.CanDecode(text)) + Shown(() => codec.DecodeWrites(text)));
                }
        }

        // The two Game Genie encoders, the other way: every patch the cheat tests decoded on 2026-11-04, and generated ones, encoded as recorded.
        [Fact]
        public void The_game_genie_encoders_encode_as_recorded()
        {
            var golden = new ParityGolden("GameGenieEncoders");
            foreach (var (key, _) in ParityGolden.Read("GameGenieEncoders"))
            {
                string[] parts = key.Split(' ');
                int address = int.Parse(parts[0]);
                uint value = uint.Parse(parts[1]);
                golden.Add(key, $"{SystemCode.CheatEncode("snes.gg", address, value)} | {SystemCode.CheatEncode("md.gg", address, value)}");
            }
            golden.Check(3000);
        }
    }
}
