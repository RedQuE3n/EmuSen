using Gsu = EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx.SuperFx;

namespace EmuSen.WiseMan.Coprocessors
{
    // What DebugInstructionAddress names while a delay slot runs - see Venus_SuperFX.md §4.1a.
    public class SuperFxTraceAddressTests
    {
        private const ushort Clsr = 0x3039, R15Low = 0x301E, R15High = 0x301F;

        private static Gsu Started(byte[] program)
        {
            byte[] rom = new byte[0x10000];
            program.CopyTo(rom, 0);
            var gsu = new Gsu(rom, new byte[0x8000]);
            gsu.WriteRegister(Clsr, 0x01);
            gsu.WriteRegister(R15Low, 0x00);
            gsu.WriteRegister(R15High, 0x00);
            return gsu;
        }

        // R15 already names the target while the delay slot runs - see §4.1a.
        [Fact]
        public void A_delay_slot_reports_its_own_address_not_the_jump_target()
        {
            var gsu = Started(new byte[]
            {
                0x05, 0x04,       // $0000 BRA $0006
                0x4C,             // $0002 delay slot
                0x01,             // $0003
                0x01, 0x01,       // $0004
                0x01,             // $0006 target
                0x00,             // $0007 STOP
            });

            var seen = new List<int>();
            gsu.CoverageRecorder = pc => seen.Add(pc & 0xFFFF);
            gsu.Run(400);

            Assert.Equal(0x0000, seen[0]);
            Assert.Equal(0x0002, seen[1]);
            Assert.Equal(0x0006, seen[2]);
            Assert.DoesNotContain(0x0004, seen);
        }

        // Straight-line code has no pipeline subtlety; this is the control.
        [Fact]
        public void Sequential_instructions_report_consecutive_addresses()
        {
            var gsu = Started(new byte[] { 0x01, 0x01, 0x01, 0x00 });

            var seen = new List<int>();
            gsu.CoverageRecorder = pc => seen.Add(pc & 0xFFFF);
            gsu.Run(400);

            Assert.Equal(new[] { 0x0000, 0x0001, 0x0002, 0x0003 }, seen.Take(4));
        }
    }
}
