using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Disassembler
{
    // One decoded word, and the address it names when that is knowable from the word alone - see Mars_Disassembler.md §1.
    public readonly record struct MarsInstruction(string Mnemonic, string Operands, StaticReferenceKind? Kind, uint Target);

    // The spellings both of Mars's disassemblers share - see Mars_Disassembler.md §2.
    internal static class MarsOperands
    {
        internal static readonly string[] Gpr =
        {
            "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3",
            "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7",
            "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7",
            "t8", "t9", "k0", "k1", "gp", "sp", "s8", "ra",
        };

        internal static string Hex(long value) => value < 0 ? $"-0x{-value:X}" : $"0x{value:X}";

        internal static MarsInstruction Word(uint word) => new(".word", $"0x{word:X8}", null, 0);

        internal static MarsInstruction Plain(string mnemonic, string operands = "") => new(mnemonic, operands, null, 0);

        // A software code field is shown only when something was put in it - see Mars_Disassembler.md §2.
        internal static string Code(uint code) => code == 0 ? "" : Hex(code);

        internal static int Rs(uint word) => (int)((word >> 21) & 0x1F);

        internal static int Rt(uint word) => (int)((word >> 16) & 0x1F);

        internal static int Rd(uint word) => (int)((word >> 11) & 0x1F);

        internal static int Sa(uint word) => (int)((word >> 6) & 0x1F);
    }

    // A pure decode of one VR4300 word, naming what Mars's interpreter executes and no more - see Mars_Disassembler.md §3.
    public static class Vr4300Disassembler
    {
        private static readonly string[] Cop0Names =
        {
            "Index", "Random", "EntryLo0", "EntryLo1", "Context", "PageMask", "Wired", "$7",
            "BadVAddr", "Count", "EntryHi", "Compare", "Status", "Cause", "EPC", "PRId",
            "Config", "LLAddr", "WatchLo", "WatchHi", "XContext", "$21", "$22", "$23",
            "$24", "$25", "PErr", "CacheErr", "TagLo", "TagHi", "ErrorEPC", "$31",
        };

        private static readonly string[] Conditions =
        {
            "f", "un", "eq", "ueq", "olt", "ult", "ole", "ule",
            "sf", "ngle", "seq", "ngl", "lt", "nge", "le", "ngt",
        };

        public static MarsInstruction Decode(uint word, uint address)
        {
            int rs = MarsOperands.Rs(word), rt = MarsOperands.Rt(word);

            return (word >> 26) switch
            {
                0x00 => Special(word),
                0x01 => RegImm(word, address),
                0x02 => Jump("j", word, address),
                0x03 => Jump("jal", word, address),

                // Two register forms fold to one operand when the other is zero - see Mars_Disassembler.md §2.1.
                0x04 when rs == 0 && rt == 0 => Branch("b", "", word, address),
                0x04 when rt == 0 => Branch("beqz", $"{Gpr(rs)}, ", word, address),
                0x04 => Branch("beq", $"{Gpr(rs)}, {Gpr(rt)}, ", word, address),
                0x05 when rt == 0 => Branch("bnez", $"{Gpr(rs)}, ", word, address),
                0x05 => Branch("bne", $"{Gpr(rs)}, {Gpr(rt)}, ", word, address),
                0x06 => Branch("blez", $"{Gpr(rs)}, ", word, address),
                0x07 => Branch("bgtz", $"{Gpr(rs)}, ", word, address),

                0x08 => SignedImmediate("addi", word),
                0x09 => SignedImmediate("addiu", word),
                0x0A => SignedImmediate("slti", word),
                0x0B => SignedImmediate("sltiu", word),
                0x0C => UnsignedImmediate("andi", word),
                0x0D => UnsignedImmediate("ori", word),
                0x0E => UnsignedImmediate("xori", word),
                0x0F => MarsOperands.Plain("lui", $"{Gpr(rt)}, {MarsOperands.Hex(word & 0xFFFF)}"),

                0x10 => Cop0(word, address),
                0x11 => Cop1(word, address),
                0x12 => Cop2(word),

                0x14 => Branch("beql", $"{Gpr(rs)}, {Gpr(rt)}, ", word, address),
                0x15 => Branch("bnel", $"{Gpr(rs)}, {Gpr(rt)}, ", word, address),
                0x16 => Branch("blezl", $"{Gpr(rs)}, ", word, address),
                0x17 => Branch("bgtzl", $"{Gpr(rs)}, ", word, address),

                0x18 => SignedImmediate("daddi", word),
                0x19 => SignedImmediate("daddiu", word),
                0x1A => Load("ldl", Gpr(rt), word),
                0x1B => Load("ldr", Gpr(rt), word),

                0x20 => Load("lb", Gpr(rt), word),
                0x21 => Load("lh", Gpr(rt), word),
                0x22 => Load("lwl", Gpr(rt), word),
                0x23 => Load("lw", Gpr(rt), word),
                0x24 => Load("lbu", Gpr(rt), word),
                0x25 => Load("lhu", Gpr(rt), word),
                0x26 => Load("lwr", Gpr(rt), word),
                0x27 => Load("lwu", Gpr(rt), word),
                0x28 => Store("sb", Gpr(rt), word),
                0x29 => Store("sh", Gpr(rt), word),
                0x2A => Store("swl", Gpr(rt), word),
                0x2B => Store("sw", Gpr(rt), word),
                0x2C => Store("sdl", Gpr(rt), word),
                0x2D => Store("sdr", Gpr(rt), word),
                0x2E => Store("swr", Gpr(rt), word),

                // The operation code is the rt field, and nothing is read or written - see Mars_Disassembler.md §4.
                0x2F => MarsOperands.Plain("cache", $"{MarsOperands.Hex(rt)}, {Offset(word)}"),

                0x30 => Load("ll", Gpr(rt), word),
                0x31 => Load("lwc1", Fpr(rt), word),
                0x34 => Load("lld", Gpr(rt), word),
                0x35 => Load("ldc1", Fpr(rt), word),
                0x37 => Load("ld", Gpr(rt), word),
                0x38 => Store("sc", Gpr(rt), word),
                0x39 => Store("swc1", Fpr(rt), word),
                0x3C => Store("scd", Gpr(rt), word),
                0x3D => Store("sdc1", Fpr(rt), word),
                0x3F => Store("sd", Gpr(rt), word),

                _ => MarsOperands.Word(word),
            };
        }

        private static MarsInstruction Special(uint word)
        {
            int rs = MarsOperands.Rs(word), rt = MarsOperands.Rt(word), rd = MarsOperands.Rd(word), sa = MarsOperands.Sa(word);

            string shift = $"{Gpr(rd)}, {Gpr(rt)}, {MarsOperands.Hex(sa)}";
            string variable = $"{Gpr(rd)}, {Gpr(rt)}, {Gpr(rs)}";
            string three = $"{Gpr(rd)}, {Gpr(rs)}, {Gpr(rt)}";
            string pair = $"{Gpr(rs)}, {Gpr(rt)}";

            return (word & 0x3F) switch
            {
                0x00 when word == 0 => MarsOperands.Plain("nop"),
                0x00 => MarsOperands.Plain("sll", shift),
                0x02 => MarsOperands.Plain("srl", shift),
                0x03 => MarsOperands.Plain("sra", shift),
                0x04 => MarsOperands.Plain("sllv", variable),
                0x06 => MarsOperands.Plain("srlv", variable),
                0x07 => MarsOperands.Plain("srav", variable),

                0x08 => MarsOperands.Plain("jr", Gpr(rs)),
                0x09 => MarsOperands.Plain("jalr", rd == 31 ? Gpr(rs) : $"{Gpr(rd)}, {Gpr(rs)}"),

                0x0C => MarsOperands.Plain("syscall", MarsOperands.Code((word >> 6) & 0xFFFFF)),
                0x0D => MarsOperands.Plain("break", MarsOperands.Code((word >> 6) & 0xFFFFF)),
                0x0F => MarsOperands.Plain("sync"),

                0x10 => MarsOperands.Plain("mfhi", Gpr(rd)),
                0x11 => MarsOperands.Plain("mthi", Gpr(rs)),
                0x12 => MarsOperands.Plain("mflo", Gpr(rd)),
                0x13 => MarsOperands.Plain("mtlo", Gpr(rs)),

                0x14 => MarsOperands.Plain("dsllv", variable),
                0x16 => MarsOperands.Plain("dsrlv", variable),
                0x17 => MarsOperands.Plain("dsrav", variable),

                0x18 => MarsOperands.Plain("mult", pair),
                0x19 => MarsOperands.Plain("multu", pair),
                0x1A => MarsOperands.Plain("div", pair),
                0x1B => MarsOperands.Plain("divu", pair),
                0x1C => MarsOperands.Plain("dmult", pair),
                0x1D => MarsOperands.Plain("dmultu", pair),
                0x1E => MarsOperands.Plain("ddiv", pair),
                0x1F => MarsOperands.Plain("ddivu", pair),

                // The aliases drop an operand that is the zero register - see Mars_Disassembler.md §2.1.
                0x20 => MarsOperands.Plain("add", three),
                0x21 when rt == 0 => MarsOperands.Plain("move", $"{Gpr(rd)}, {Gpr(rs)}"),
                0x21 => MarsOperands.Plain("addu", three),
                0x22 when rs == 0 => MarsOperands.Plain("neg", $"{Gpr(rd)}, {Gpr(rt)}"),
                0x22 => MarsOperands.Plain("sub", three),
                0x23 when rs == 0 => MarsOperands.Plain("negu", $"{Gpr(rd)}, {Gpr(rt)}"),
                0x23 => MarsOperands.Plain("subu", three),
                0x24 => MarsOperands.Plain("and", three),
                0x25 when rt == 0 => MarsOperands.Plain("move", $"{Gpr(rd)}, {Gpr(rs)}"),
                0x25 => MarsOperands.Plain("or", three),
                0x26 => MarsOperands.Plain("xor", three),
                0x27 when rt == 0 => MarsOperands.Plain("not", $"{Gpr(rd)}, {Gpr(rs)}"),
                0x27 => MarsOperands.Plain("nor", three),
                0x2A => MarsOperands.Plain("slt", three),
                0x2B => MarsOperands.Plain("sltu", three),
                0x2C => MarsOperands.Plain("dadd", three),
                0x2D when rt == 0 => MarsOperands.Plain("move", $"{Gpr(rd)}, {Gpr(rs)}"),
                0x2D => MarsOperands.Plain("daddu", three),
                0x2E => MarsOperands.Plain("dsub", three),
                0x2F => MarsOperands.Plain("dsubu", three),

                0x30 => Trap("tge", pair, word),
                0x31 => Trap("tgeu", pair, word),
                0x32 => Trap("tlt", pair, word),
                0x33 => Trap("tltu", pair, word),
                0x34 => Trap("teq", pair, word),
                0x36 => Trap("tne", pair, word),

                0x38 => MarsOperands.Plain("dsll", shift),
                0x3A => MarsOperands.Plain("dsrl", shift),
                0x3B => MarsOperands.Plain("dsra", shift),
                0x3C => MarsOperands.Plain("dsll32", shift),
                0x3E => MarsOperands.Plain("dsrl32", shift),
                0x3F => MarsOperands.Plain("dsra32", shift),

                _ => MarsOperands.Word(word),
            };
        }

        private static MarsInstruction RegImm(uint word, uint address)
        {
            string rs = Gpr(MarsOperands.Rs(word));

            // The trap immediate is sign-extended even for the unsigned comparisons - see Mars_Cpu.md §15.1.
            string trap = $"{rs}, {MarsOperands.Hex((short)word)}";

            return MarsOperands.Rt(word) switch
            {
                0x00 => Branch("bltz", $"{rs}, ", word, address),
                0x01 => Branch("bgez", $"{rs}, ", word, address),
                0x02 => Branch("bltzl", $"{rs}, ", word, address),
                0x03 => Branch("bgezl", $"{rs}, ", word, address),

                0x08 => MarsOperands.Plain("tgei", trap),
                0x09 => MarsOperands.Plain("tgeiu", trap),
                0x0A => MarsOperands.Plain("tlti", trap),
                0x0B => MarsOperands.Plain("tltiu", trap),
                0x0C => MarsOperands.Plain("teqi", trap),
                0x0E => MarsOperands.Plain("tnei", trap),

                0x10 => Branch("bltzal", $"{rs}, ", word, address),
                0x11 when MarsOperands.Rs(word) == 0 => Branch("bal", "", word, address),
                0x11 => Branch("bgezal", $"{rs}, ", word, address),
                0x12 => Branch("bltzall", $"{rs}, ", word, address),
                0x13 => Branch("bgezall", $"{rs}, ", word, address),

                _ => MarsOperands.Word(word),
            };
        }

        // The CO half ignores every bit between the sub-opcode and the function - see Mars_Cop0.md §10.1.
        private static MarsInstruction Cop0(uint word, uint address)
        {
            int rs = MarsOperands.Rs(word), rt = MarsOperands.Rt(word), rd = MarsOperands.Rd(word);

            if ((rs & 0x10) != 0)
            {
                return (word & 0x3F) switch
                {
                    0x01 => MarsOperands.Plain("tlbr"),
                    0x02 => MarsOperands.Plain("tlbwi"),
                    0x06 => MarsOperands.Plain("tlbwr"),
                    0x08 => MarsOperands.Plain("tlbp"),
                    0x18 => MarsOperands.Plain("eret"),
                    _ => MarsOperands.Word(word),
                };
            }

            return rs switch
            {
                0x00 => MarsOperands.Plain("mfc0", $"{Gpr(rt)}, {Cop0Names[rd]}"),
                0x01 => MarsOperands.Plain("dmfc0", $"{Gpr(rt)}, {Cop0Names[rd]}"),
                0x04 => MarsOperands.Plain("mtc0", $"{Gpr(rt)}, {Cop0Names[rd]}"),
                0x05 => MarsOperands.Plain("dmtc0", $"{Gpr(rt)}, {Cop0Names[rd]}"),

                // Decoded and inert on this part, so named rather than refused - see Mars_Cop0.md §10.2.
                0x02 => MarsOperands.Plain("cfc0", $"{Gpr(rt)}, ${rd}"),
                0x06 => MarsOperands.Plain("ctc0", $"{Gpr(rt)}, ${rd}"),
                0x08 when rt <= 3 => Branch(ConditionBranch(0, rt), "", word, address),

                _ => MarsOperands.Word(word),
            };
        }

        private static MarsInstruction Cop1(uint word, uint address)
        {
            int rs = MarsOperands.Rs(word), rt = MarsOperands.Rt(word), rd = MarsOperands.Rd(word);

            return rs switch
            {
                0x00 => MarsOperands.Plain("mfc1", $"{Gpr(rt)}, {Fpr(rd)}"),
                0x01 => MarsOperands.Plain("dmfc1", $"{Gpr(rt)}, {Fpr(rd)}"),
                0x02 => MarsOperands.Plain("cfc1", $"{Gpr(rt)}, fcr{rd}"),
                0x04 => MarsOperands.Plain("mtc1", $"{Gpr(rt)}, {Fpr(rd)}"),
                0x05 => MarsOperands.Plain("dmtc1", $"{Gpr(rt)}, {Fpr(rd)}"),
                0x06 => MarsOperands.Plain("ctc1", $"{Gpr(rt)}, fcr{rd}"),

                // Only four selectors exist on this part; the rest are refused by the unit - see Mars_FpuMath.md §7.1.
                0x08 when rt <= 3 => Branch(ConditionBranch(1, rt), "", word, address),

                0x10 => FloatFormat(word, 's'),
                0x11 => FloatFormat(word, 'd'),
                0x14 => IntegerFormat(word, 'w'),
                0x15 => IntegerFormat(word, 'l'),

                _ => MarsOperands.Word(word),
            };
        }

        private static string ConditionBranch(int coprocessor, int selector) =>
            $"bc{coprocessor}{((selector & 1) != 0 ? 't' : 'f')}{((selector & 2) != 0 ? "l" : "")}";

        // Converting a format to itself is refused, so it has no mnemonic here - see Mars_FpuMath.md §6.
        private static MarsInstruction FloatFormat(uint word, char format)
        {
            string fd = Fpr(MarsOperands.Sa(word)), fs = Fpr(MarsOperands.Rd(word)), ft = Fpr(MarsOperands.Rt(word));
            uint funct = word & 0x3F;

            string? name = funct switch
            {
                0x00 => "add", 0x01 => "sub", 0x02 => "mul", 0x03 => "div",
                0x04 => "sqrt", 0x05 => "abs", 0x06 => "mov", 0x07 => "neg",
                0x08 => "round.l", 0x09 => "trunc.l", 0x0A => "ceil.l", 0x0B => "floor.l",
                0x0C => "round.w", 0x0D => "trunc.w", 0x0E => "ceil.w", 0x0F => "floor.w",
                0x20 when format == 'd' => "cvt.s",
                0x21 when format == 's' => "cvt.d",
                0x24 => "cvt.w",
                0x25 => "cvt.l",
                >= 0x30 => $"c.{Conditions[funct & 0xF]}",
                _ => null,
            };

            if (name is null) return MarsOperands.Word(word);

            string operands = funct switch
            {
                <= 0x03 => $"{fd}, {fs}, {ft}",
                >= 0x30 => $"{fs}, {ft}",
                _ => $"{fd}, {fs}",
            };

            return MarsOperands.Plain($"{name}.{format}", operands);
        }

        // The integer formats convert to a float and do nothing else - see Mars_FpuMath.md §6.
        private static MarsInstruction IntegerFormat(uint word, char format) => (word & 0x3F) switch
        {
            0x20 => MarsOperands.Plain($"cvt.s.{format}", $"{Fpr(MarsOperands.Sa(word))}, {Fpr(MarsOperands.Rd(word))}"),
            0x21 => MarsOperands.Plain($"cvt.d.{format}", $"{Fpr(MarsOperands.Sa(word))}, {Fpr(MarsOperands.Rd(word))}"),
            _ => MarsOperands.Word(word),
        };

        // One latch behind every index, but the index is still what the word names - see Mars_Fpu.md §8.
        private static MarsInstruction Cop2(uint word)
        {
            string operands = $"{Gpr(MarsOperands.Rt(word))}, ${MarsOperands.Rd(word)}";

            return MarsOperands.Rs(word) switch
            {
                0x00 => MarsOperands.Plain("mfc2", operands),
                0x01 => MarsOperands.Plain("dmfc2", operands),
                0x02 => MarsOperands.Plain("cfc2", operands),
                0x04 => MarsOperands.Plain("mtc2", operands),
                0x05 => MarsOperands.Plain("dmtc2", operands),
                0x06 => MarsOperands.Plain("ctc2", operands),
                _ => MarsOperands.Word(word),
            };
        }

        private static MarsInstruction Trap(string mnemonic, string pair, uint word)
        {
            string code = MarsOperands.Code((word >> 6) & 0x3FF);
            return MarsOperands.Plain(mnemonic, code.Length == 0 ? pair : $"{pair}, {code}");
        }

        private static MarsInstruction SignedImmediate(string mnemonic, uint word) =>
            MarsOperands.Plain(mnemonic, $"{Gpr(MarsOperands.Rt(word))}, {Gpr(MarsOperands.Rs(word))}, {MarsOperands.Hex((short)word)}");

        private static MarsInstruction UnsignedImmediate(string mnemonic, uint word) =>
            MarsOperands.Plain(mnemonic, $"{Gpr(MarsOperands.Rt(word))}, {Gpr(MarsOperands.Rs(word))}, {MarsOperands.Hex(word & 0xFFFF)}");

        // The offset counts from the delay slot, and the sum wraps in thirty-two bits - see Mars_Disassembler.md §4.
        private static MarsInstruction Branch(string mnemonic, string registers, uint word, uint address)
        {
            uint target = unchecked(address + 4 + (uint)((short)word << 2));
            return new(mnemonic, $"{registers}0x{target:X8}", StaticReferenceKind.Call, target);
        }

        // The region comes from the delay slot's address, not the jump's - see Mars_Disassembler.md §4.
        private static MarsInstruction Jump(string mnemonic, uint word, uint address)
        {
            uint target = unchecked(((address + 4) & 0xF000_0000) | ((word & 0x03FF_FFFF) << 2));
            return new(mnemonic, $"0x{target:X8}", StaticReferenceKind.Call, target);
        }

        private static MarsInstruction Load(string mnemonic, string register, uint word) =>
            Access(mnemonic, register, word, StaticReferenceKind.Read);

        private static MarsInstruction Store(string mnemonic, string register, uint word) =>
            Access(mnemonic, register, word, StaticReferenceKind.Write);

        // Only a base of zero makes the address a constant of the word - see Mars_Disassembler.md §4.
        private static MarsInstruction Access(string mnemonic, string register, uint word, StaticReferenceKind kind)
        {
            string operands = $"{register}, {Offset(word)}";
            return MarsOperands.Rs(word) == 0
                ? new(mnemonic, operands, kind, unchecked((uint)(short)word))
                : new(mnemonic, operands, null, 0);
        }

        private static string Offset(uint word) => $"{MarsOperands.Hex((short)word)}({Gpr(MarsOperands.Rs(word))})";

        private static string Gpr(int index) => MarsOperands.Gpr[index];

        private static string Fpr(int index) => $"f{index}";
    }
}
