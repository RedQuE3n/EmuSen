using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.Psx;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.DianaSh
{
    // The PlayStation pack's GameShark and Xploder codecs against psx-spx's "Cheat Devices" tables, applied through the cheat registry to a RAM of the test's own - see EmuSen_Cheats.md §10.
    public class PsxCheatFormatsTests
    {
        private readonly ITestOutputHelper _output;

        public PsxCheatFormatsTests(ITestOutputHelper output) => _output = output;

        private static readonly ICheatCodeCodec Either = PsxSystem.GameShark(), Xploder = PsxSystem.Xploder();

        // The codes applied once to a RAM holding `before`, as the runtime applies them between frames.
        private static byte[] Applied(string code, byte[] before)
        {
            var ram = (byte[])before.Clone();
            var registry = new CheatRegistry();
            registry.AddCheat(CheatKind.RamPoke, Either.DecodeWrites(code)!, null, "test");
            registry.ApplyAll((space, a) => space == PsxSystem.Ram ? ram[a] : (byte)0, (space, a, v) => { if (space == PsxSystem.Ram) ram[a] = v; });
            return ram;
        }

        private static byte[] Ram(params (int Address, byte Value)[] held)
        {
            var ram = new byte[0x20_0000];
            foreach (var (a, v) in held) ram[a] = v;
            return ram;
        }

        [Fact]
        public void The_writes_increments_and_decrements_land_little_endian_in_main_ram()
        {
            Assert.Equal(new byte[] { 0x63 }, Applied("3001E0DA 0063", Ram())[0x1E0DA..0x1E0DB]);
            Assert.Equal(new byte[] { 0x00, 0x3C }, Applied("8005FA8A+3C00", Ram())[0x5FA8A..0x5FA8C]);
            Assert.Equal(new byte[] { 0x12, 0x01 }, Applied("10001000 0102", Ram((0x1000, 0x10), (0x1001, 0x00)))[0x1000..0x1002]);
            Assert.Equal(new byte[] { 0x0E, 0x00 }, Applied("11001000 0002", Ram((0x1000, 0x10)))[0x1000..0x1002]);
            Assert.Equal(0x15, Applied("20001000 0005", Ram((0x1000, 0x10)))[0x1000]);
            Assert.Equal(0x0B, Applied("21001000 0005", Ram((0x1000, 0x10)))[0x1000]);
            Assert.Equal(0x63, Applied("301FFFFF 0063", Ram())[0x1F_FFFF]);
        }

        // psx-spx: D0/E0 "If dddd=[aaaaaa]", D1/E1 "<>", D2/E2 "If dddd<[aaaaaa]", D3/E3 "If dddd>[aaaaaa]", each then the next code.
        [Theory]
        [InlineData("E0001000 0005", 5, true)]
        [InlineData("E0001000 0005", 6, false)]
        [InlineData("E1001000 0005", 6, true)]
        [InlineData("E2001000 0005", 6, true)]
        [InlineData("E2001000 0005", 5, false)]
        [InlineData("E3001000 0005", 4, true)]
        [InlineData("E3001000 0005", 5, false)]
        [InlineData("D2001000 0105", 0x06, true)]
        public void A_test_runs_the_next_code_only_when_it_holds(string test, byte held, bool runs)
        {
            var ram = Applied(test + "+30002000 0077", Ram((0x1000, held), (0x1001, (byte)(test.StartsWith('D') ? 1 : 0))));
            Assert.Equal(runs ? 0x77 : 0x00, ram[0x2000]);
        }

        [Fact]
        public void A_slide_repeats_its_line_by_address_and_value_steps()
        {
            var w = Assert.Single(Either.DecodeWrites("50000502+0000+8001+E284+0101")!);
            Assert.Equal((5, 2, 0x0101u, 0x1E284, 0x0000u, 2), (w.RepeatCount, w.RepeatAddAddress, w.RepeatAddValue, w.Address, w.Value, w.EffectiveWidth));
            var ram = Applied("50000502+0000+8001+E284+0101", Ram());
            Assert.Equal(new byte[] { 0x00, 0x00, 0x01, 0x01, 0x02, 0x02, 0x03, 0x03, 0x04, 0x04 }, ram[0x1E284..0x1E28E]);
            Assert.Single(Either.DecodeWrites("00000000 0000+30001000 0001")!);
        }

        [Theory]
        [InlineData("C0001000 0001+30002000 0001", "turns every code on")]
        [InlineData("C1000000 4000+30002000 0001", "delays the codes")]
        [InlineData("C2001000 0010+80002000 0000", "copies a block")]
        [InlineData("D4000000 0800+30002000 0001", "tests the pad's buttons")]
        [InlineData("12001000 00000001", "not a PlayStation GameShark code")]
        [InlineData("D0001000 0005", "guards nothing")]
        [InlineData("50000502 0000+D0001000 0001", "must be followed by the 30 or 80 line")]
        public void A_code_emusen_does_not_apply_is_refused_with_its_reason(string code, string words)
        {
            var e = Assert.Throws<FormatException>(() => PsxCheatFormats.GameSharkWrites(code));
            _output.WriteLine(e.Message);
            Assert.Contains(words, e.Message);
        }

        // Inverses of psx-spx's decrypt_xplorer_cheat_code, written from it, so that each key's decryption is shown to undo an encryption.
        private static byte[] Encrypt(byte[] p, int key)
        {
            var x = (byte[])p.Clone();
            unchecked
            {
                switch (key)
                {
                    case 4:
                        x[1] = (byte)(p[1] ^ 0x25);
                        x[2] = (byte)(p[2] ^ (byte)(0xFA + (p[1] & 0x11)));
                        x[3] = (byte)(p[3] ^ (byte)(0xC0 + (p[2] & 0x11) + (p[1] ^ 0x12)));
                        x[4] = (byte)(p[4] ^ (byte)(0x7E + (p[3] & 0x11) + (p[2] ^ 0x12) + p[1]));
                        x[5] = (byte)(p[5] ^ (byte)(0x26 + (p[4] & 0x11) + (p[3] ^ 0x12) + p[2] + p[1]));
                        break;
                    case 5:
                        x[1] = (byte)(p[1] - 0x57); x[2] = (byte)(p[2] - 0x42); x[3] = (byte)(p[3] - 0x31); x[4] = (byte)(p[4] - 0x32); x[5] = (byte)(p[5] - 0x33);
                        break;
                    case 6:
                        for (int i = 1; i <= 5; i++) x[i] = (byte)((p[i] ^ i) - 0xAB);
                        break;
                    case 7:
                        x[1] = (byte)(p[1] - 0xF5 - (p[2] & 0x73) + (p[3] ^ 0x90) - p[4] - p[5]);
                        x[2] = (byte)(p[2] - 0x16 - (p[3] & 0x73) + (p[4] ^ 0x90) - p[5]);
                        x[3] = (byte)(p[3] - 0x5A - (p[4] & 0x73) + (p[5] ^ 0x90));
                        x[4] = (byte)(p[4] - 0xCB - (p[5] & 0x73));
                        x[5] = (byte)(p[5] - 0xCB);
                        break;
                }
            }
            x[0] = (byte)(p[0] | key);
            return x;
        }

        [Fact]
        public void Each_xploder_key_decrypts_what_it_encrypts_and_keys_1_to_3_are_refused()
        {
            var random = new Random(11);
            foreach (int key in new[] { 0, 4, 5, 6, 7 })
            {
                for (int n = 0; n < 2000; n++)
                {
                    var plain = new byte[6];
                    random.NextBytes(plain);
                    plain[0] &= 0xF8;
                    var e = Encrypt(plain, key);
                    var line = new PsxCheatFormats.Line((uint)(e[0] << 24 | e[1] << 16 | e[2] << 8 | e[3]), (ushort)(e[4] << 8 | e[5]));
                    var d = PsxCheatFormats.Decrypt(line);
                    Assert.Equal(((uint)(plain[0] << 24 | plain[1] << 16 | plain[2] << 8 | plain[3]), (ushort)(plain[4] << 8 | plain[5])), (d.Command, d.Value));
                }
            }
            Assert.Contains("encryption key 2", Assert.Throws<FormatException>(() => PsxCheatFormats.XploderWrites("32001000 0001")).Message);
        }

        [Fact]
        public void An_xploder_code_writes_tests_and_writes_32_bits_and_the_auto_detect_codec_tells_it_by_its_digits()
        {
            Assert.Equal(new byte[] { 0x34, 0x12, 0x00, 0x00 }, Applied("00001000 1234", Ram())[0x1000..0x1004]);
            Assert.Equal(0x77, Applied("70001000 0005+30002000 0077", Ram((0x1000, 5)))[0x2000]);
            Assert.Equal(0x00, Applied("90001000 0005+30002000 0077", Ram((0x1000, 5)))[0x2000]);
            var e = Encrypt(new byte[] { 0x30, 0x00, 0x10, 0x00, 0x00, 0x42 }, 5);
            string encrypted = $"{e[0]:X2}{e[1]:X2}{e[2]:X2}{e[3]:X2} {e[4]:X2}{e[5]:X2}";
            Assert.Equal((0x1000, (byte)0x42), Xploder.Decode(encrypted));
            Assert.True(PsxCheatFormats.IsGameShark("D0001000 0001+30001000 0001") && !PsxCheatFormats.IsGameShark("70001000 0005+30002000 0077"));
            foreach (string code in new[] { "40000000 0005+30001000 0001", "B0050002 0001+10001000 0001", "F0001000 0001+30001000 0001" })
                _output.WriteLine(Assert.Throws<FormatException>(() => Either.DecodeWrites(code)).Message);
        }

        [Fact]
        public void A_cht_file_of_the_database_imports_its_pokes()
        {
            const string text = "cheats = 3\n\ncheat0_desc = \"PAL2NTSC\"\ncheat0_code = \"8005FA8A+3C00\"\ncheat0_enable = false\n\n" +
                "cheat1_desc = \"Missions\"\ncheat1_code = \"50000502+0000+8001+E284+0101\"\ncheat1_enable = false\n\n" +
                "cheat2_desc = \"Delayed\"\ncheat2_code = \"C1000000+4000+3001E0DA+0063\"\ncheat2_enable = false\n";
            var parsed = ChtFile.Parse(text, Either, PsxSystem.Ram, Xploder);
            Assert.Equal((2, 1), (parsed.Cheats.Count, parsed.Skipped));
            Assert.All(parsed.Cheats, c => Assert.Equal(CheatKind.RamPoke, c.Kind));
            var pack = SystemPacks.For(PsxSystem.Id)!;
            Assert.Equal(("GameShark or Xploder", "Xploder", PsxSystem.Ram), (pack.AutoDetectCodec!().Name, pack.ExplicitCodec!().Name, pack.AutoDetectCodec!().SpaceName));
            Assert.Equal(new[] { "Sony - PlayStation" }, PsxSystem.Entry.CheatSystems);
        }

        private static string? Database => Environment.GetEnvironmentVariable("EMUSEN_PSX_CHEAT_DB") is { } d && Directory.Exists(d) ? d : null;

        // The libretro database's PlayStation folder decoded whole, by the codec its file names; opt-in by the folder, measured for EmuSen_Cheats.md §10.
        [Fact]
        public void The_databases_playstation_files_decode_and_each_refusal_is_named()
        {
            if (Database is not { } folder)
            {
                _output.WriteLine("skipped: EMUSEN_PSX_CHEAT_DB names the cheat database's Sony - PlayStation folder");
                return;
            }
            var counts = new Dictionary<string, (int Files, int Entries, int Loaded)>();
            var reasons = new Dictionary<string, int>();
            foreach (string file in Directory.EnumerateFiles(folder, "*.cht"))
            {
                string form = file.Contains("(Xploder)") ? "Xploder" : file.Contains("(Game Buster)") ? "Game Buster" : file.Contains("(GameShark)") ? "GameShark" : "other";
                var codec = form == "Xploder" ? Xploder : Either;
                string text = File.ReadAllText(file);
                var parsed = ChtFile.Parse(text, codec, PsxSystem.Ram);
                var c = counts.GetValueOrDefault(form);
                counts[form] = (c.Files + 1, c.Entries + parsed.Cheats.Count + parsed.Skipped, c.Loaded + parsed.Cheats.Count);
                foreach (System.Text.RegularExpressions.Match match in System.Text.RegularExpressions.Regex.Matches(text, "cheat[0-9]+_code = \"([^\"]*)\""))
                {
                    string code = match.Groups[1].Value;
                    try { codec.DecodeWrites(code); }
                    catch (FormatException e) { string key = e.Message.IndexOf("' ") is int at and >= 0 ? e.Message[(at + 2)..] : e.Message; key = key[..Math.Min(48, key.Length)]; reasons[key] = reasons.GetValueOrDefault(key) + 1; }
                }
            }
            foreach (var (form, (files, entries, loaded)) in counts) _output.WriteLine($"{form}: {files} files, {loaded} of {entries} entries load ({100.0 * loaded / Math.Max(1, entries):F1}%)");
            foreach (var (reason, n) in reasons.OrderByDescending(r => r.Value).Take(15)) _output.WriteLine($"{n,6}  {reason}");
            Assert.True(counts.Values.Sum(c => c.Loaded) > 0);
        }
    }
}
