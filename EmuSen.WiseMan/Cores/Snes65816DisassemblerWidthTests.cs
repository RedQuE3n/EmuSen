using EmuSen.Cores.Nintendo.Venus.Processor;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.WiseMan.Cores
{
    // A wrong M/X guess shifts every later byte - see `man disasm`.
    public class Snes65816DisassemblerWidthTests
    {
        // Yoshi's Island $04:FDD8 - six instructions only when the index is 8-bit.
        private static readonly byte[] Bytes =
        {
            0xA0, 0x00, 0xC5, 0x39, 0x10, 0x02, 0xA0, 0x02, 0x84, 0x73, 0x85, 0x39,
        };

        private static System.Collections.Generic.List<DisassembledInstruction> Run(bool xFlagSet)
            => Snes65816Disassembler.Disassemble(
                a => a >= 0 && a < Bytes.Length ? Bytes[a] : (byte)0x00,
                address: 0, count: 6, eFlag: false, mFlagSet: false, xFlagSet: xFlagSet);

        [Fact]
        public void Eight_bit_index_decodes_the_real_instruction_stream()
        {
            var d = Run(xFlagSet: true);

            Assert.Equal(2, d[0].Bytes.Count);        // LDY #$00
            Assert.Equal(0x02, d[1].Address);         // CMP $39
            Assert.Equal(0x04, d[2].Address);         // BPL
            Assert.Equal(0x0A, d[5].Address);         // STA $39 - the write we traced
            Assert.Equal("STA", d[5].Mnemonic);
        }

        [Fact]
        public void Sixteen_bit_index_misaligns_the_same_bytes_without_looking_wrong()
        {
            var d = Run(xFlagSet: false);

            // LDY swallows the CMP's opcode and every later address is off by one.
            Assert.Equal(3, d[0].Bytes.Count);
            Assert.NotEqual(0x02, d[1].Address);
            Assert.DoesNotContain(d, i => i.Address == 0x0A && i.Mnemonic == "STA");
        }
    }
}
