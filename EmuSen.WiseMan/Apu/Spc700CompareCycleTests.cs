using EmuSen.Cores.Nintendo.Venus.Apu;

namespace EmuSen.WiseMan.Apu
{
    // Every direct-page compare is 3 cycles; CMP Y,dp claimed 4 - see Venus_APU.md §1.7.
    public class Spc700CompareCycleTests
    {
        private static Spc700 AtProgram(params byte[] code)
        {
            var spc = new Spc700();
            spc.Reset();
            spc.IplRomEnabled = false;
            spc.PC = 0x0200;
            for (int i = 0; i < code.Length; i++) spc.Ram[0x0200 + i] = code[i];
            return spc;
        }

        [Theory]
        [InlineData(0x64)] // CMP A, dp
        [InlineData(0x3E)] // CMP X, dp
        [InlineData(0x7E)] // CMP Y, dp
        public void Direct_page_compares_all_cost_three_cycles(byte opcode)
        {
            var spc = AtProgram(opcode, 0xF4);

            Assert.Equal(3, spc.PeekStepCycles());
        }

        // The IPL loader's per-byte transfer loop, which is what made the wrong
        // count visible: one extra cycle here slowed the boot upload enough to
        // land the game's first frame a whole frame late - see Venus_APU.md §1.7.
        [Fact]
        public void Ipl_transfer_loop_costs_the_documented_25_cycles()
        {
            int Cost(params byte[] code) => AtProgram(code).PeekStepCycles();

            int total = Cost(0x7E, 0xF4)   // CMP Y, $F4        3
                      + 2                  // BNE not taken     2
                      + Cost(0xE4, 0xF5)   // MOV A, $F5        3
                      + Cost(0xCB, 0xF4)   // MOV $F4, Y        4
                      + Cost(0xD7, 0x00)   // MOV [$00]+Y, A    7
                      + Cost(0xFC)         // INC Y             2
                      + 4;                 // BNE taken         4

            Assert.Equal(25, total);
        }
    }
}
