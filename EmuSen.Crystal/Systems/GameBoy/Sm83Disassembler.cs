using EmuSen.Crystal.Debugger;

namespace EmuSen.Crystal.Systems.GameBoy
{
    // A second opcode table, independent of the executing one, emusen-systems' gb::disasm - see Mercury_Debug.md §4, EmuSen_Backend.md §13.5.
    public static class Sm83Disassembler
    {
        // What an opcode with no instruction behind it prints as; the CPU throws on these - see Mercury_Cpu.md §6.1.
        public const string Illegal = "???";

        public static int LengthOf(byte opcode) => SystemCode.Disassemble("sm83", new[] { opcode }, 0, 0).Length;

        public static string MnemonicOf(byte opcode, byte suffix) => SystemCode.Disassemble("sm83", new[] { opcode, suffix }, 0, 0).Mnemonic;

        public static string FormatOperand(byte opcode, int address, byte low, byte high) => SystemCode.Disassemble("sm83", new[] { opcode, low, high }, address, 0).Operands;

        public static (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(byte opcode, int operand, int address) =>
            SystemCode.Disassemble("sm83", new[] { opcode }, address, operand) is { Kind: { } kind } i ? (kind, (int)i.Target) : null;
    }
}
