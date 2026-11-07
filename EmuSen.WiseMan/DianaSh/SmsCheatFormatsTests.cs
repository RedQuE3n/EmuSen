using EmuSen.WiseMan.Bindings.Cheats;
using EmuSen.WiseMan.Bindings.Systems;
using EmuSen.WiseMan.Bindings.Systems.Sms;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.DianaSh
{
    // The Master System pack's Pro Action Replay and Game Genie codecs against SMS Power!'s two pages - see EmuSen_Cheats.md §11.
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
            Assert.Equal(EmuSen.WiseMan.Bindings.Systems.GameBoy.GameBoyCheatCodecs.GameGenie().Decode("3A6-4AB-2A2"), Genie.Decode("3A6-4AB-2A2"));
            Assert.False(Genie.CanDecode("00C0-DE03"));
            Assert.False(Par.CanDecode("3A6-4AB-2A2"));
            Assert.False(Genie.CanDecode("00D5-BD??"));
            Assert.Equal(SystemPacks.For("sms")!.AutoDetectCodec, SystemPacks.For("sg1000")!.AutoDetectCodec);
        }
    }
}
