using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.Sms;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.DianaSh
{
    // The Master System pack's Pro Action Replay and Game Genie codecs against SMS Power!'s two pages, and the libretro database's files through ChtFile - see EmuSen_Cheats.md §11.
    public class SmsCheatFormatsTests
    {
        private readonly ITestOutputHelper _output;

        public SmsCheatFormatsTests(ITestOutputHelper output) => _output = output;

        private static readonly ICheatCodeCodec Par = SmsSystems.ProActionReplay(), Genie = SmsSystems.GameGenie();

        // SMS Power!'s "Pro Action Replay": 00AA-AADD, the first two digits of no effect, the address in the work RAM.
        [Fact]
        public void A_pro_action_replay_code_pokes_the_work_ram()
        {
            Assert.Equal((0x00DE, (byte)0x03), Par.Decode("00C0-DE03"));
            Assert.Equal((0x15BD, (byte)0x07), Par.Decode("20D5-BD07"));
            Assert.Equal((CheatCodeKind.RamPoke, SmsSystems.Ram), (Par.Kind, Par.SpaceName));
            Assert.False(Par.CanDecode("0080-0001"));
            Assert.Contains("from $C000", Assert.Throws<FormatException>(() => Par.Decode("0080-0001")).Message);
        }

        // SMS Power!'s "Game Genie": DDA-AAA-RRR, the address's last digit moved first and inverted, the reference rotated and XORed with $BA; Galoob's Game Boy cipher.
        [Fact]
        public void A_game_genie_code_patches_with_its_reference()
        {
            Assert.Equal((0x464A, (byte)0x3A), Genie.Decode("3A6-4AB-2A2"));
            long code = 0x3A64AB2A2;
            Assert.Equal((byte)((((code >> 2) & 0x03) | ((code >> 6) & 0x3C) | ((code << 6) & 0xC0)) ^ 0xBA), Genie.DecodeCompare("3A6-4AB-2A2"));
            Assert.Null(Genie.DecodeCompare("3A6-4AB"));
            Assert.Equal(CheatCodeKind.RomPatch, Genie.Kind);
            Assert.Equal(EmuSen.Crystal.Systems.GameBoy.GbGameGenieCodec.Decode("3A6-4AB-2A2"), Genie.Decode("3A6-4AB-2A2"));
            Assert.False(Genie.CanDecode("00C0-DE03"));
            Assert.False(Par.CanDecode("3A6-4AB-2A2"));
            Assert.False(Genie.CanDecode("00D5-BD??"));
            Assert.Equal(SystemPacks.For("sms")!.AutoDetectCodec, SystemPacks.For("sg1000")!.AutoDetectCodec);
        }

        private static string? Database => Environment.GetEnvironmentVariable("EMUSEN_SMS_CHEAT_DB") is { } d && Directory.Exists(d) ? d : null;

        // The libretro database's Master System and SG-1000 folders imported with both codecs, as Mistress's database window does; opt-in by the Cheats folder, measured for EmuSen_Cheats.md §11.
        [Fact]
        public void The_databases_master_system_and_sg_1000_files_import()
        {
            if (Database is not { } folder)
            {
                _output.WriteLine("skipped: EMUSEN_SMS_CHEAT_DB names the cheat database's folder holding Sega - Master System - Mark III and Sega - SG-1000");
                return;
            }
            foreach (string system in new[] { "Sega - Master System - Mark III", "Sega - SG-1000" })
            {
                int files = 0, entries = 0, pokes = 0, patches = 0, compared = 0;
                var refused = new Dictionary<string, int>();
                foreach (string file in Directory.EnumerateFiles(Path.Combine(folder, system), "*.cht"))
                {
                    string text = File.ReadAllText(file);
                    var parsed = ChtFile.Parse(text, Par, SmsSystems.Ram, Genie);
                    files++;
                    entries += parsed.Cheats.Count + parsed.Skipped;
                    pokes += parsed.Cheats.Count(c => c.Kind == CheatKind.RamPoke);
                    patches += parsed.Cheats.Count(c => c.Kind == CheatKind.RomPatch);
                    foreach (System.Text.RegularExpressions.Match m in System.Text.RegularExpressions.Regex.Matches(text, "cheat[0-9]+_code = \"([^\"]*)\""))
                    {
                        string part = m.Groups[1].Value.Split('+')[0].Trim();
                        if (Genie.CanDecode(part) && Genie.DecodeCompare(part) is not null) compared++;
                        if (!Par.CanDecode(part) && !Genie.CanDecode(part))
                        {
                            string shape = System.Text.RegularExpressions.Regex.Replace(part, "[0-9A-Fa-f]", "x");
                            refused[shape] = refused.GetValueOrDefault(shape) + 1;
                        }
                    }
                }
                _output.WriteLine($"{system}: {files} files, {pokes + patches} of {entries} entries load ({100.0 * (pokes + patches) / Math.Max(1, entries):F1}%), {pokes} pokes and {patches} patches; {compared} Game Genie codes carry a reference, which a .cht entry has nowhere to keep (EmuSen_Cheats.md §8)");
                foreach (var (shape, n) in refused.OrderByDescending(r => r.Value).Take(10)) _output.WriteLine($"{n,6}  '{shape}'");
                Assert.True(pokes > 0 && (patches > 0 || system == "Sega - SG-1000"), system);
            }
        }
    }
}
