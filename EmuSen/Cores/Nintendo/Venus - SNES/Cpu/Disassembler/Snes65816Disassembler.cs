using System;
using System.Collections.Generic;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    // Addressing modes, used only to drive operand length + formatting -
    // NOT reused from Cpu's execution-side AddrMode delegates (Cpu.cs),
    // which compute target addresses with real side effects (advancing PC,
    // consuming cycles) and aren't safe or meaningful to invoke just to
    // find out "how many bytes does this instruction take". This is a
    // deliberately separate, read-only data table.
    internal enum Snes65816AddrMode
    {
        Implied, Accumulator, Immediate8, ImmediateA, ImmediateXY,
        DirectPage, DirectPageX, DirectPageY,
        DirectPageIndirect, DirectPageIndirectX, DirectPageIndirectY,
        DirectPageIndirectLong, DirectPageIndirectLongY,
        Absolute, AbsoluteX, AbsoluteY,
        AbsoluteIndirect, AbsoluteIndirectLong, AbsoluteIndirectX,
        AbsoluteLong, AbsoluteLongX,
        StackRelative, StackRelativeIndirectY,
        Relative8, Relative16,
        BlockMove,
    }

    // Standalone 65816 disassembler, deliberately separate from Cpu's
    // execution opcode table - see Venus_CPU.md §1 and §7, and
    // EmuSen_Debugging_Tools_Reference_v5.md §3.7 for the full verification
    // status and caveats.
    public static class Snes65816Disassembler
    {
        private static readonly (string Mnemonic, Snes65816AddrMode Mode)[] Table = new (string, Snes65816AddrMode)[256]
        {
            /*00*/ ("BRK", Snes65816AddrMode.Immediate8),
            /*01*/ ("ORA", Snes65816AddrMode.DirectPageIndirectX),
            /*02*/ ("COP", Snes65816AddrMode.Immediate8),
            /*03*/ ("ORA", Snes65816AddrMode.StackRelative),
            /*04*/ ("TSB", Snes65816AddrMode.DirectPage),
            /*05*/ ("ORA", Snes65816AddrMode.DirectPage),
            /*06*/ ("ASL", Snes65816AddrMode.DirectPage),
            /*07*/ ("ORA", Snes65816AddrMode.DirectPageIndirectLong),
            /*08*/ ("PHP", Snes65816AddrMode.Implied),
            /*09*/ ("ORA", Snes65816AddrMode.ImmediateA),
            /*0A*/ ("ASL", Snes65816AddrMode.Accumulator),
            /*0B*/ ("PHD", Snes65816AddrMode.Implied),
            /*0C*/ ("TSB", Snes65816AddrMode.Absolute),
            /*0D*/ ("ORA", Snes65816AddrMode.Absolute),
            /*0E*/ ("ASL", Snes65816AddrMode.Absolute),
            /*0F*/ ("ORA", Snes65816AddrMode.AbsoluteLong),

            /*10*/ ("BPL", Snes65816AddrMode.Relative8),
            /*11*/ ("ORA", Snes65816AddrMode.DirectPageIndirectY),
            /*12*/ ("ORA", Snes65816AddrMode.DirectPageIndirect),
            /*13*/ ("ORA", Snes65816AddrMode.StackRelativeIndirectY),
            /*14*/ ("TRB", Snes65816AddrMode.DirectPage),
            /*15*/ ("ORA", Snes65816AddrMode.DirectPageX),
            /*16*/ ("ASL", Snes65816AddrMode.DirectPageX),
            /*17*/ ("ORA", Snes65816AddrMode.DirectPageIndirectLongY),
            /*18*/ ("CLC", Snes65816AddrMode.Implied),
            /*19*/ ("ORA", Snes65816AddrMode.AbsoluteY),
            /*1A*/ ("INC", Snes65816AddrMode.Accumulator),
            /*1B*/ ("TCS", Snes65816AddrMode.Implied),
            /*1C*/ ("TRB", Snes65816AddrMode.Absolute),
            /*1D*/ ("ORA", Snes65816AddrMode.AbsoluteX),
            /*1E*/ ("ASL", Snes65816AddrMode.AbsoluteX),
            /*1F*/ ("ORA", Snes65816AddrMode.AbsoluteLongX),

            /*20*/ ("JSR", Snes65816AddrMode.Absolute),
            /*21*/ ("AND", Snes65816AddrMode.DirectPageIndirectX),
            /*22*/ ("JSL", Snes65816AddrMode.AbsoluteLong),
            /*23*/ ("AND", Snes65816AddrMode.StackRelative),
            /*24*/ ("BIT", Snes65816AddrMode.DirectPage),
            /*25*/ ("AND", Snes65816AddrMode.DirectPage),
            /*26*/ ("ROL", Snes65816AddrMode.DirectPage),
            /*27*/ ("AND", Snes65816AddrMode.DirectPageIndirectLong),
            /*28*/ ("PLP", Snes65816AddrMode.Implied),
            /*29*/ ("AND", Snes65816AddrMode.ImmediateA),
            /*2A*/ ("ROL", Snes65816AddrMode.Accumulator),
            /*2B*/ ("PLD", Snes65816AddrMode.Implied),
            /*2C*/ ("BIT", Snes65816AddrMode.Absolute),
            /*2D*/ ("AND", Snes65816AddrMode.Absolute),
            /*2E*/ ("ROL", Snes65816AddrMode.Absolute),
            /*2F*/ ("AND", Snes65816AddrMode.AbsoluteLong),

            /*30*/ ("BMI", Snes65816AddrMode.Relative8),
            /*31*/ ("AND", Snes65816AddrMode.DirectPageIndirectY),
            /*32*/ ("AND", Snes65816AddrMode.DirectPageIndirect),
            /*33*/ ("AND", Snes65816AddrMode.StackRelativeIndirectY),
            /*34*/ ("BIT", Snes65816AddrMode.DirectPageX),
            /*35*/ ("AND", Snes65816AddrMode.DirectPageX),
            /*36*/ ("ROL", Snes65816AddrMode.DirectPageX),
            /*37*/ ("AND", Snes65816AddrMode.DirectPageIndirectLongY),
            /*38*/ ("SEC", Snes65816AddrMode.Implied),
            /*39*/ ("AND", Snes65816AddrMode.AbsoluteY),
            /*3A*/ ("DEC", Snes65816AddrMode.Accumulator),
            /*3B*/ ("TSC", Snes65816AddrMode.Implied),
            /*3C*/ ("BIT", Snes65816AddrMode.AbsoluteX),
            /*3D*/ ("AND", Snes65816AddrMode.AbsoluteX),
            /*3E*/ ("ROL", Snes65816AddrMode.AbsoluteX),
            /*3F*/ ("AND", Snes65816AddrMode.AbsoluteLongX),

            /*40*/ ("RTI", Snes65816AddrMode.Implied),
            /*41*/ ("EOR", Snes65816AddrMode.DirectPageIndirectX),
            /*42*/ ("WDM", Snes65816AddrMode.Immediate8),
            /*43*/ ("EOR", Snes65816AddrMode.StackRelative),
            /*44*/ ("MVP", Snes65816AddrMode.BlockMove),
            /*45*/ ("EOR", Snes65816AddrMode.DirectPage),
            /*46*/ ("LSR", Snes65816AddrMode.DirectPage),
            /*47*/ ("EOR", Snes65816AddrMode.DirectPageIndirectLong),
            /*48*/ ("PHA", Snes65816AddrMode.Implied),
            /*49*/ ("EOR", Snes65816AddrMode.ImmediateA),
            /*4A*/ ("LSR", Snes65816AddrMode.Accumulator),
            /*4B*/ ("PHK", Snes65816AddrMode.Implied),
            /*4C*/ ("JMP", Snes65816AddrMode.Absolute),
            /*4D*/ ("EOR", Snes65816AddrMode.Absolute),
            /*4E*/ ("LSR", Snes65816AddrMode.Absolute),
            /*4F*/ ("EOR", Snes65816AddrMode.AbsoluteLong),

            /*50*/ ("BVC", Snes65816AddrMode.Relative8),
            /*51*/ ("EOR", Snes65816AddrMode.DirectPageIndirectY),
            /*52*/ ("EOR", Snes65816AddrMode.DirectPageIndirect),
            /*53*/ ("EOR", Snes65816AddrMode.StackRelativeIndirectY),
            /*54*/ ("MVN", Snes65816AddrMode.BlockMove),
            /*55*/ ("EOR", Snes65816AddrMode.DirectPageX),
            /*56*/ ("LSR", Snes65816AddrMode.DirectPageX),
            /*57*/ ("EOR", Snes65816AddrMode.DirectPageIndirectLongY),
            /*58*/ ("CLI", Snes65816AddrMode.Implied),
            /*59*/ ("EOR", Snes65816AddrMode.AbsoluteY),
            /*5A*/ ("PHY", Snes65816AddrMode.Implied),
            /*5B*/ ("TCD", Snes65816AddrMode.Implied),
            /*5C*/ ("JMP", Snes65816AddrMode.AbsoluteLong),
            /*5D*/ ("EOR", Snes65816AddrMode.AbsoluteX),
            /*5E*/ ("LSR", Snes65816AddrMode.AbsoluteX),
            /*5F*/ ("EOR", Snes65816AddrMode.AbsoluteLongX),

            /*60*/ ("RTS", Snes65816AddrMode.Implied),
            /*61*/ ("ADC", Snes65816AddrMode.DirectPageIndirectX),
            /*62*/ ("PER", Snes65816AddrMode.Relative16),
            /*63*/ ("ADC", Snes65816AddrMode.StackRelative),
            /*64*/ ("STZ", Snes65816AddrMode.DirectPage),
            /*65*/ ("ADC", Snes65816AddrMode.DirectPage),
            /*66*/ ("ROR", Snes65816AddrMode.DirectPage),
            /*67*/ ("ADC", Snes65816AddrMode.DirectPageIndirectLong),
            /*68*/ ("PLA", Snes65816AddrMode.Implied),
            /*69*/ ("ADC", Snes65816AddrMode.ImmediateA),
            /*6A*/ ("ROR", Snes65816AddrMode.Accumulator),
            /*6B*/ ("RTL", Snes65816AddrMode.Implied),
            /*6C*/ ("JMP", Snes65816AddrMode.AbsoluteIndirect),
            /*6D*/ ("ADC", Snes65816AddrMode.Absolute),
            /*6E*/ ("ROR", Snes65816AddrMode.Absolute),
            /*6F*/ ("ADC", Snes65816AddrMode.AbsoluteLong),

            /*70*/ ("BVS", Snes65816AddrMode.Relative8),
            /*71*/ ("ADC", Snes65816AddrMode.DirectPageIndirectY),
            /*72*/ ("ADC", Snes65816AddrMode.DirectPageIndirect),
            /*73*/ ("ADC", Snes65816AddrMode.StackRelativeIndirectY),
            /*74*/ ("STZ", Snes65816AddrMode.DirectPageX),
            /*75*/ ("ADC", Snes65816AddrMode.DirectPageX),
            /*76*/ ("ROR", Snes65816AddrMode.DirectPageX),
            /*77*/ ("ADC", Snes65816AddrMode.DirectPageIndirectLongY),
            /*78*/ ("SEI", Snes65816AddrMode.Implied),
            /*79*/ ("ADC", Snes65816AddrMode.AbsoluteY),
            /*7A*/ ("PLY", Snes65816AddrMode.Implied),
            /*7B*/ ("TDC", Snes65816AddrMode.Implied),
            /*7C*/ ("JMP", Snes65816AddrMode.AbsoluteIndirectX),
            /*7D*/ ("ADC", Snes65816AddrMode.AbsoluteX),
            /*7E*/ ("ROR", Snes65816AddrMode.AbsoluteX),
            /*7F*/ ("ADC", Snes65816AddrMode.AbsoluteLongX),

            /*80*/ ("BRA", Snes65816AddrMode.Relative8),
            /*81*/ ("STA", Snes65816AddrMode.DirectPageIndirectX),
            /*82*/ ("BRL", Snes65816AddrMode.Relative16),
            /*83*/ ("STA", Snes65816AddrMode.StackRelative),
            /*84*/ ("STY", Snes65816AddrMode.DirectPage),
            /*85*/ ("STA", Snes65816AddrMode.DirectPage),
            /*86*/ ("STX", Snes65816AddrMode.DirectPage),
            /*87*/ ("STA", Snes65816AddrMode.DirectPageIndirectLong),
            /*88*/ ("DEY", Snes65816AddrMode.Implied),
            /*89*/ ("BIT", Snes65816AddrMode.ImmediateA),
            /*8A*/ ("TXA", Snes65816AddrMode.Implied),
            /*8B*/ ("PHB", Snes65816AddrMode.Implied),
            /*8C*/ ("STY", Snes65816AddrMode.Absolute),
            /*8D*/ ("STA", Snes65816AddrMode.Absolute),
            /*8E*/ ("STX", Snes65816AddrMode.Absolute),
            /*8F*/ ("STA", Snes65816AddrMode.AbsoluteLong),

            /*90*/ ("BCC", Snes65816AddrMode.Relative8),
            /*91*/ ("STA", Snes65816AddrMode.DirectPageIndirectY),
            /*92*/ ("STA", Snes65816AddrMode.DirectPageIndirect),
            /*93*/ ("STA", Snes65816AddrMode.StackRelativeIndirectY),
            /*94*/ ("STY", Snes65816AddrMode.DirectPageX),
            /*95*/ ("STA", Snes65816AddrMode.DirectPageX),
            /*96*/ ("STX", Snes65816AddrMode.DirectPageY),
            /*97*/ ("STA", Snes65816AddrMode.DirectPageIndirectLongY),
            /*98*/ ("TYA", Snes65816AddrMode.Implied),
            /*99*/ ("STA", Snes65816AddrMode.AbsoluteY),
            /*9A*/ ("TXS", Snes65816AddrMode.Implied),
            /*9B*/ ("TXY", Snes65816AddrMode.Implied),
            /*9C*/ ("STZ", Snes65816AddrMode.Absolute),
            /*9D*/ ("STA", Snes65816AddrMode.AbsoluteX),
            /*9E*/ ("STZ", Snes65816AddrMode.AbsoluteX),
            /*9F*/ ("STA", Snes65816AddrMode.AbsoluteLongX),

            /*A0*/ ("LDY", Snes65816AddrMode.ImmediateXY),
            /*A1*/ ("LDA", Snes65816AddrMode.DirectPageIndirectX),
            /*A2*/ ("LDX", Snes65816AddrMode.ImmediateXY),
            /*A3*/ ("LDA", Snes65816AddrMode.StackRelative),
            /*A4*/ ("LDY", Snes65816AddrMode.DirectPage),
            /*A5*/ ("LDA", Snes65816AddrMode.DirectPage),
            /*A6*/ ("LDX", Snes65816AddrMode.DirectPage),
            /*A7*/ ("LDA", Snes65816AddrMode.DirectPageIndirectLong),
            /*A8*/ ("TAY", Snes65816AddrMode.Implied),
            /*A9*/ ("LDA", Snes65816AddrMode.ImmediateA),
            /*AA*/ ("TAX", Snes65816AddrMode.Implied),
            /*AB*/ ("PLB", Snes65816AddrMode.Implied),
            /*AC*/ ("LDY", Snes65816AddrMode.Absolute),
            /*AD*/ ("LDA", Snes65816AddrMode.Absolute),
            /*AE*/ ("LDX", Snes65816AddrMode.Absolute),
            /*AF*/ ("LDA", Snes65816AddrMode.AbsoluteLong),

            /*B0*/ ("BCS", Snes65816AddrMode.Relative8),
            /*B1*/ ("LDA", Snes65816AddrMode.DirectPageIndirectY),
            /*B2*/ ("LDA", Snes65816AddrMode.DirectPageIndirect),
            /*B3*/ ("LDA", Snes65816AddrMode.StackRelativeIndirectY),
            /*B4*/ ("LDY", Snes65816AddrMode.DirectPageX),
            /*B5*/ ("LDA", Snes65816AddrMode.DirectPageX),
            /*B6*/ ("LDX", Snes65816AddrMode.DirectPageY),
            /*B7*/ ("LDA", Snes65816AddrMode.DirectPageIndirectLongY),
            /*B8*/ ("CLV", Snes65816AddrMode.Implied),
            /*B9*/ ("LDA", Snes65816AddrMode.AbsoluteY),
            /*BA*/ ("TSX", Snes65816AddrMode.Implied),
            /*BB*/ ("TYX", Snes65816AddrMode.Implied),
            /*BC*/ ("LDY", Snes65816AddrMode.AbsoluteX),
            /*BD*/ ("LDA", Snes65816AddrMode.AbsoluteX),
            /*BE*/ ("LDX", Snes65816AddrMode.AbsoluteY),
            /*BF*/ ("LDA", Snes65816AddrMode.AbsoluteLongX),

            /*C0*/ ("CPY", Snes65816AddrMode.ImmediateXY),
            /*C1*/ ("CMP", Snes65816AddrMode.DirectPageIndirectX),
            /*C2*/ ("REP", Snes65816AddrMode.Immediate8),
            /*C3*/ ("CMP", Snes65816AddrMode.StackRelative),
            /*C4*/ ("CPY", Snes65816AddrMode.DirectPage),
            /*C5*/ ("CMP", Snes65816AddrMode.DirectPage),
            /*C6*/ ("DEC", Snes65816AddrMode.DirectPage),
            /*C7*/ ("CMP", Snes65816AddrMode.DirectPageIndirectLong),
            /*C8*/ ("INY", Snes65816AddrMode.Implied),
            /*C9*/ ("CMP", Snes65816AddrMode.ImmediateA),
            /*CA*/ ("DEX", Snes65816AddrMode.Implied),
            /*CB*/ ("WAI", Snes65816AddrMode.Implied),
            /*CC*/ ("CPY", Snes65816AddrMode.Absolute),
            /*CD*/ ("CMP", Snes65816AddrMode.Absolute),
            /*CE*/ ("DEC", Snes65816AddrMode.Absolute),
            /*CF*/ ("CMP", Snes65816AddrMode.AbsoluteLong),

            /*D0*/ ("BNE", Snes65816AddrMode.Relative8),
            /*D1*/ ("CMP", Snes65816AddrMode.DirectPageIndirectY),
            /*D2*/ ("CMP", Snes65816AddrMode.DirectPageIndirect),
            /*D3*/ ("CMP", Snes65816AddrMode.StackRelativeIndirectY),
            /*D4*/ ("PEI", Snes65816AddrMode.DirectPageIndirect),
            /*D5*/ ("CMP", Snes65816AddrMode.DirectPageX),
            /*D6*/ ("DEC", Snes65816AddrMode.DirectPageX),
            /*D7*/ ("CMP", Snes65816AddrMode.DirectPageIndirectLongY),
            /*D8*/ ("CLD", Snes65816AddrMode.Implied),
            /*D9*/ ("CMP", Snes65816AddrMode.AbsoluteY),
            /*DA*/ ("PHX", Snes65816AddrMode.Implied),
            /*DB*/ ("STP", Snes65816AddrMode.Implied),
            /*DC*/ ("JML", Snes65816AddrMode.AbsoluteIndirectLong),
            /*DD*/ ("CMP", Snes65816AddrMode.AbsoluteX),
            /*DE*/ ("DEC", Snes65816AddrMode.AbsoluteX),
            /*DF*/ ("CMP", Snes65816AddrMode.AbsoluteLongX),

            /*E0*/ ("CPX", Snes65816AddrMode.ImmediateXY),
            /*E1*/ ("SBC", Snes65816AddrMode.DirectPageIndirectX),
            /*E2*/ ("SEP", Snes65816AddrMode.Immediate8),
            /*E3*/ ("SBC", Snes65816AddrMode.StackRelative),
            /*E4*/ ("CPX", Snes65816AddrMode.DirectPage),
            /*E5*/ ("SBC", Snes65816AddrMode.DirectPage),
            /*E6*/ ("INC", Snes65816AddrMode.DirectPage),
            /*E7*/ ("SBC", Snes65816AddrMode.DirectPageIndirectLong),
            /*E8*/ ("INX", Snes65816AddrMode.Implied),
            /*E9*/ ("SBC", Snes65816AddrMode.ImmediateA),
            /*EA*/ ("NOP", Snes65816AddrMode.Implied),
            /*EB*/ ("XBA", Snes65816AddrMode.Implied),
            /*EC*/ ("CPX", Snes65816AddrMode.Absolute),
            /*ED*/ ("SBC", Snes65816AddrMode.Absolute),
            /*EE*/ ("INC", Snes65816AddrMode.Absolute),
            /*EF*/ ("SBC", Snes65816AddrMode.AbsoluteLong),

            /*F0*/ ("BEQ", Snes65816AddrMode.Relative8),
            /*F1*/ ("SBC", Snes65816AddrMode.DirectPageIndirectY),
            /*F2*/ ("SBC", Snes65816AddrMode.DirectPageIndirect),
            /*F3*/ ("SBC", Snes65816AddrMode.StackRelativeIndirectY),
            /*F4*/ ("PEA", Snes65816AddrMode.Absolute),
            /*F5*/ ("SBC", Snes65816AddrMode.DirectPageX),
            /*F6*/ ("INC", Snes65816AddrMode.DirectPageX),
            /*F7*/ ("SBC", Snes65816AddrMode.DirectPageIndirectLongY),
            /*F8*/ ("SED", Snes65816AddrMode.Implied),
            /*F9*/ ("SBC", Snes65816AddrMode.AbsoluteY),
            /*FA*/ ("PLX", Snes65816AddrMode.Implied),
            /*FB*/ ("XCE", Snes65816AddrMode.Implied),
            /*FC*/ ("JSR", Snes65816AddrMode.AbsoluteIndirectX),
            /*FD*/ ("SBC", Snes65816AddrMode.AbsoluteX),
            /*FE*/ ("INC", Snes65816AddrMode.AbsoluteX),
            /*FF*/ ("SBC", Snes65816AddrMode.AbsoluteLongX),
        };

        // eFlag forces 8-bit M/X regardless of P (real hardware behavior);
        // mFlagSet/xFlagSet seed the starting M/X when eFlag is false, then
        // get tracked live as REP/SEP are decoded within this call. XCE
        // mid-range is a known remaining gap. See
        // EmuSen_Debugging_Tools_Reference_v5.md §3.7 for the full story.
        public static List<DisassembledInstruction> Disassemble(Func<int, byte> readByte, int address, int count, bool eFlag, bool mFlagSet, bool xFlagSet)
        {
            var result = new List<DisassembledInstruction>(count);
            int addr = address;
            bool curM = mFlagSet;
            bool curX = xFlagSet;

            for (int i = 0; i < count; i++)
            {
                byte opcode = readByte(addr);
                (string mnemonic, Snes65816AddrMode mode) = Table[opcode];

                int operandLen = OperandLength(mode, eFlag, curM, curX);
                var bytes = new byte[1 + operandLen];
                bytes[0] = opcode;
                for (int b = 0; b < operandLen; b++) bytes[1 + b] = readByte(addr + 1 + b);

                string operandText = FormatOperand(mode, bytes, addr);
                result.Add(new DisassembledInstruction(addr, bytes, mnemonic, operandText));

                // REP clears the P bits set in its operand (0 = 16-bit for
                // M/X); SEP sets them (1 = 8-bit). Bit 0x20 is M, bit 0x10
                // is X - standard 65816 status register layout. Both
                // opcodes are fixed 1-byte-immediate regardless of M/X, so
                // bytes[1] is always the mask here.
                if (!eFlag)
                {
                    if (opcode == 0xC2) // REP
                    {
                        if ((bytes[1] & 0x20) != 0) curM = false;
                        if ((bytes[1] & 0x10) != 0) curX = false;
                    }
                    else if (opcode == 0xE2) // SEP
                    {
                        if ((bytes[1] & 0x20) != 0) curM = true;
                        if ((bytes[1] & 0x10) != 0) curX = true;
                    }
                }

                addr += bytes.Length;
            }

            return result;
        }

        private static int OperandLength(Snes65816AddrMode mode, bool eFlag, bool mFlagSet, bool xFlagSet)
        {
            switch (mode)
            {
                case Snes65816AddrMode.Implied:
                case Snes65816AddrMode.Accumulator:
                    return 0;
                case Snes65816AddrMode.Immediate8:
                    return 1;
                case Snes65816AddrMode.ImmediateA:
                    return (eFlag || mFlagSet) ? 1 : 2;
                case Snes65816AddrMode.ImmediateXY:
                    return (eFlag || xFlagSet) ? 1 : 2;
                case Snes65816AddrMode.DirectPage:
                case Snes65816AddrMode.DirectPageX:
                case Snes65816AddrMode.DirectPageY:
                case Snes65816AddrMode.DirectPageIndirect:
                case Snes65816AddrMode.DirectPageIndirectX:
                case Snes65816AddrMode.DirectPageIndirectY:
                case Snes65816AddrMode.DirectPageIndirectLong:
                case Snes65816AddrMode.DirectPageIndirectLongY:
                case Snes65816AddrMode.StackRelative:
                case Snes65816AddrMode.StackRelativeIndirectY:
                case Snes65816AddrMode.Relative8:
                    return 1;
                case Snes65816AddrMode.Absolute:
                case Snes65816AddrMode.AbsoluteX:
                case Snes65816AddrMode.AbsoluteY:
                case Snes65816AddrMode.AbsoluteIndirect:
                case Snes65816AddrMode.AbsoluteIndirectLong:
                case Snes65816AddrMode.AbsoluteIndirectX:
                case Snes65816AddrMode.Relative16:
                case Snes65816AddrMode.BlockMove:
                    return 2;
                case Snes65816AddrMode.AbsoluteLong:
                case Snes65816AddrMode.AbsoluteLongX:
                    return 3;
                default:
                    return 0;
            }
        }

        private static string FormatOperand(Snes65816AddrMode mode, byte[] bytes, int instrAddr)
        {
            // bytes[0] is the opcode itself; operand bytes start at [1].
            switch (mode)
            {
                case Snes65816AddrMode.Implied:
                    return "";
                case Snes65816AddrMode.Accumulator:
                    return "A";
                case Snes65816AddrMode.Immediate8:
                    return $"#${bytes[1]:X2}";
                case Snes65816AddrMode.ImmediateA:
                case Snes65816AddrMode.ImmediateXY:
                    return bytes.Length == 2 ? $"#${bytes[1]:X2}" : $"#${bytes[1] | (bytes[2] << 8):X4}";
                case Snes65816AddrMode.DirectPage:
                    return $"${bytes[1]:X2}";
                case Snes65816AddrMode.DirectPageX:
                    return $"${bytes[1]:X2},X";
                case Snes65816AddrMode.DirectPageY:
                    return $"${bytes[1]:X2},Y";
                case Snes65816AddrMode.DirectPageIndirect:
                    return $"(${bytes[1]:X2})";
                case Snes65816AddrMode.DirectPageIndirectX:
                    return $"(${bytes[1]:X2},X)";
                case Snes65816AddrMode.DirectPageIndirectY:
                    return $"(${bytes[1]:X2}),Y";
                case Snes65816AddrMode.DirectPageIndirectLong:
                    return $"[${bytes[1]:X2}]";
                case Snes65816AddrMode.DirectPageIndirectLongY:
                    return $"[${bytes[1]:X2}],Y";
                case Snes65816AddrMode.Absolute:
                    return $"${bytes[1] | (bytes[2] << 8):X4}";
                case Snes65816AddrMode.AbsoluteX:
                    return $"${bytes[1] | (bytes[2] << 8):X4},X";
                case Snes65816AddrMode.AbsoluteY:
                    return $"${bytes[1] | (bytes[2] << 8):X4},Y";
                case Snes65816AddrMode.AbsoluteIndirect:
                    return $"(${bytes[1] | (bytes[2] << 8):X4})";
                case Snes65816AddrMode.AbsoluteIndirectLong:
                    return $"[${bytes[1] | (bytes[2] << 8):X4}]";
                case Snes65816AddrMode.AbsoluteIndirectX:
                    return $"(${bytes[1] | (bytes[2] << 8):X4},X)";
                case Snes65816AddrMode.AbsoluteLong:
                    return $"${(bytes[1] | (bytes[2] << 8) | (bytes[3] << 16)):X6}";
                case Snes65816AddrMode.AbsoluteLongX:
                    return $"${(bytes[1] | (bytes[2] << 8) | (bytes[3] << 16)):X6},X";
                case Snes65816AddrMode.StackRelative:
                    return $"${bytes[1]:X2},S";
                case Snes65816AddrMode.StackRelativeIndirectY:
                    return $"(${bytes[1]:X2},S),Y";
                case Snes65816AddrMode.Relative8:
                {
                    sbyte offset = (sbyte)bytes[1];
                    int target = (instrAddr + 2 + offset) & 0xFFFF;
                    return $"${target:X4}";
                }
                case Snes65816AddrMode.Relative16:
                {
                    short offset = (short)(bytes[1] | (bytes[2] << 8));
                    int target = (instrAddr + 3 + offset) & 0xFFFF;
                    return $"${target:X4}";
                }
                case Snes65816AddrMode.BlockMove:
                    // MVN/MVP operand bytes are (destBank, srcBank) per the
                    // 65816 convention - shown plainly as two bank bytes
                    // rather than asserting src/dest labels, since that
                    // ordering is exactly the kind of easy-to-get-backwards
                    // detail this file's honesty note is warning about.
                    return $"${bytes[1]:X2},${bytes[2]:X2}";
                default:
                    return "";
            }
        }
    }
}
