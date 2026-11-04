namespace EmuSen.Crystal.Systems.N64
{
    // A pure decode of one RSP word, every address an IMEM or DMEM offset, emusen-mips' through the N64 pack - see Mars_Disassembler.md §5, EmuSen_Backend.md §18.7.
    public static class RspDisassembler
    {
        public static MarsInstruction Decode(uint word, uint address)
        {
            var i = SystemCode.Disassemble("rsp", new[] { (byte)(word >> 24), (byte)(word >> 16), (byte)(word >> 8), (byte)word }, unchecked((int)address), 0);
            return new MarsInstruction(i.Mnemonic, i.Operands, i.Kind, unchecked((uint)i.Target));
        }
    }
}
