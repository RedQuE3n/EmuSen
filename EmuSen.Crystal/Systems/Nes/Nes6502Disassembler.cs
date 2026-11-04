using EmuSen.Crystal.Debugger;

namespace EmuSen.Crystal.Systems.Nes
{
    // The 2A03's instructions as text, independent of any engine's executing table, emusen-systems' nes::disasm - see Moon_Debug.md §5, EmuSen_Backend.md §13.5.
    public static class Nes6502Disassembler
    {
        public static string MnemonicOf(byte opcode) => SystemCode.Disassemble("6502", new[] { opcode }, 0, 0).Mnemonic;

        public static int LengthOf(byte opcode) => SystemCode.Disassemble("6502", new[] { opcode }, 0, 0).Length;

        public static string FormatOperand(byte opcode, int address, byte operandLow, byte operandHigh) =>
            SystemCode.Disassemble("6502", new[] { opcode, operandLow, operandHigh }, address, 0).Operands;

        public static (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(byte opcode, int operand) =>
            SystemCode.Disassemble("6502", new[] { opcode }, 0, operand) is { Kind: { } kind } i ? (kind, (int)i.Target) : null;
    }
}
