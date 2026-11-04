using EmuSen.Crystal.Debugger;

namespace EmuSen.Crystal.Systems.N64
{
    // One decoded word, and the address it names when that is knowable from the word alone - see Mars_Disassembler.md §1.
    public readonly record struct MarsInstruction(string Mnemonic, string Operands, StaticReferenceKind? Kind, uint Target);

    // A pure decode of one VR4300 word, naming what Mars's interpreter executes and no more, emusen-mips' through the N64 pack - see Mars_Disassembler.md §3, EmuSen_Backend.md §13.5.
    public static class Vr4300Disassembler
    {
        public static MarsInstruction Decode(uint word, uint address)
        {
            var i = SystemCode.Disassemble("vr4300", new[] { (byte)(word >> 24), (byte)(word >> 16), (byte)(word >> 8), (byte)word }, unchecked((int)address), 0);
            return new MarsInstruction(i.Mnemonic, i.Operands, i.Kind, unchecked((uint)i.Target));
        }
    }
}
