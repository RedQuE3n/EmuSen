using EmuSen.Cores.Nintendo.Mars.Cpu.Disassembler;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Mars.Rsp
{
    // A pure decode of one RSP word; every address it computes is an offset into IMEM or DMEM - see Mars_Disassembler.md §5.
    public static class RspDisassembler
    {
        private const uint ImemMask = 0xFFC;
        private const uint DmemMask = 0xFFF;

        // The libultra names, by the four bits the interpreter reads - see Mars_Rsp.md §5.
        private static readonly string[] Cop0Names =
        {
            "SP_MEM_ADDR", "SP_DRAM_ADDR", "SP_RD_LEN", "SP_WR_LEN", "SP_STATUS", "SP_DMA_FULL", "SP_DMA_BUSY", "SP_SEMAPHORE",
            "DPC_START", "DPC_END", "DPC_CURRENT", "DPC_STATUS", "DPC_CLOCK", "DPC_BUFBUSY", "DPC_PIPEBUSY", "DPC_TMEM",
        };

        // Index 2 and 3 both reach VCE, because only the low two bits are read - see Mars_RspVector.md §3.
        private static readonly string[] ControlNames = { "vco", "vcc", "vce", "vce" };

        // Null where the references agree on no name for a function - see Mars_Disassembler.md §7.2.
        private static readonly string?[] VectorNames =
        {
            "vmulf", "vmulu", "vrndp", "vmulq", "vmudl", "vmudm", "vmudn", "vmudh",
            "vmacf", "vmacu", "vrndn", "vmacq", "vmadl", "vmadm", "vmadn", "vmadh",
            "vadd", "vsub", "vsut", "vabs", "vaddc", "vsubc", "vaddb", "vsubb",
            "vaccb", "vsucb", "vsad", "vsac", "vsum", "vsar", null, null,
            "vlt", "veq", "vne", "vge", "vcl", "vch", "vcr", "vmrg",
            "vand", "vnand", "vor", "vnor", "vxor", "vnxor", null, null,
            "vrcp", "vrcpl", "vrcph", "vmov", "vrsq", "vrsql", "vrsqh", "vnop",
            "vextt", "vextq", "vextn", null, "vinst", "vinsq", "vinsn", "vnull",
        };

        // The load format 10 does nothing and has no name; the store of that format does - see Mars_RspVector.md §4.
        private static readonly string?[] LoadNames = { "lbv", "lsv", "llv", "ldv", "lqv", "lrv", "lpv", "luv", "lhv", "lfv", null, "ltv" };
        private static readonly string[] StoreNames = { "sbv", "ssv", "slv", "sdv", "sqv", "srv", "spv", "suv", "shv", "sfv", "swv", "stv" };

        // How far each format shifts its seven-bit offset - see Mars_Disassembler.md §5.3.
        private static readonly int[] OffsetScale = { 0, 1, 2, 3, 4, 4, 3, 3, 4, 4, 4, 4 };

        public static MarsInstruction Decode(uint word, uint address)
        {
            int rs = MarsOperands.Rs(word), rt = MarsOperands.Rt(word);

            return (word >> 26) switch
            {
                0x00 => Special(word),
                0x01 => RegImm(word, address),
                0x02 => Jump("j", word),
                0x03 => Jump("jal", word),

                0x04 when rs == 0 && rt == 0 => Branch("b", "", word, address),
                0x04 when rt == 0 => Branch("beqz", $"{Gpr(rs)}, ", word, address),
                0x04 => Branch("beq", $"{Gpr(rs)}, {Gpr(rt)}, ", word, address),
                0x05 when rt == 0 => Branch("bnez", $"{Gpr(rs)}, ", word, address),
                0x05 => Branch("bne", $"{Gpr(rs)}, {Gpr(rt)}, ", word, address),
                0x06 => Branch("blez", $"{Gpr(rs)}, ", word, address),
                0x07 => Branch("bgtz", $"{Gpr(rs)}, ", word, address),

                0x08 => MarsOperands.Plain("addi", $"{Gpr(rt)}, {Gpr(rs)}, {MarsOperands.Hex((short)word)}"),
                0x09 => MarsOperands.Plain("addiu", $"{Gpr(rt)}, {Gpr(rs)}, {MarsOperands.Hex((short)word)}"),
                0x0A => MarsOperands.Plain("slti", $"{Gpr(rt)}, {Gpr(rs)}, {MarsOperands.Hex((short)word)}"),
                0x0B => MarsOperands.Plain("sltiu", $"{Gpr(rt)}, {Gpr(rs)}, {MarsOperands.Hex((short)word)}"),
                0x0C => MarsOperands.Plain("andi", $"{Gpr(rt)}, {Gpr(rs)}, {MarsOperands.Hex(word & 0xFFFF)}"),
                0x0D => MarsOperands.Plain("ori", $"{Gpr(rt)}, {Gpr(rs)}, {MarsOperands.Hex(word & 0xFFFF)}"),
                0x0E => MarsOperands.Plain("xori", $"{Gpr(rt)}, {Gpr(rs)}, {MarsOperands.Hex(word & 0xFFFF)}"),
                0x0F => MarsOperands.Plain("lui", $"{Gpr(rt)}, {MarsOperands.Hex(word & 0xFFFF)}"),

                0x10 when rs == 0x00 => MarsOperands.Plain("mfc0", $"{Gpr(rt)}, {Cop0Names[MarsOperands.Rd(word) & 0xF]}"),
                0x10 when rs == 0x04 => MarsOperands.Plain("mtc0", $"{Gpr(rt)}, {Cop0Names[MarsOperands.Rd(word) & 0xF]}"),
                0x12 => Cop2(word),

                0x20 => Scalar("lb", word, StaticReferenceKind.Read),
                0x21 => Scalar("lh", word, StaticReferenceKind.Read),
                0x23 => Scalar("lw", word, StaticReferenceKind.Read),
                0x24 => Scalar("lbu", word, StaticReferenceKind.Read),
                0x25 => Scalar("lhu", word, StaticReferenceKind.Read),
                0x27 => Scalar("lwu", word, StaticReferenceKind.Read),
                0x28 => Scalar("sb", word, StaticReferenceKind.Write),
                0x29 => Scalar("sh", word, StaticReferenceKind.Write),
                0x2B => Scalar("sw", word, StaticReferenceKind.Write),

                0x32 => Vector(LoadNames, word, StaticReferenceKind.Read),
                0x3A => Vector(StoreNames, word, StaticReferenceKind.Write),

                _ => MarsOperands.Word(word),
            };
        }

        private static MarsInstruction Special(uint word)
        {
            int rs = MarsOperands.Rs(word), rt = MarsOperands.Rt(word), rd = MarsOperands.Rd(word);

            string shift = $"{Gpr(rd)}, {Gpr(rt)}, {MarsOperands.Hex(MarsOperands.Sa(word))}";
            string variable = $"{Gpr(rd)}, {Gpr(rt)}, {Gpr(rs)}";
            string three = $"{Gpr(rd)}, {Gpr(rs)}, {Gpr(rt)}";

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
                0x0D => MarsOperands.Plain("break", MarsOperands.Code((word >> 6) & 0xFFFFF)),

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

                _ => MarsOperands.Word(word),
            };
        }

        // Four selectors are named; the interpreter branches on the others too - see Mars_Disassembler.md §3.1.
        private static MarsInstruction RegImm(uint word, uint address)
        {
            int rs = MarsOperands.Rs(word);

            return MarsOperands.Rt(word) switch
            {
                0x00 => Branch("bltz", $"{Gpr(rs)}, ", word, address),
                0x01 => Branch("bgez", $"{Gpr(rs)}, ", word, address),
                0x10 => Branch("bltzal", $"{Gpr(rs)}, ", word, address),
                0x11 when rs == 0 => Branch("bal", "", word, address),
                0x11 => Branch("bgezal", $"{Gpr(rs)}, ", word, address),
                _ => MarsOperands.Word(word),
            };
        }

        private static MarsInstruction Cop2(uint word)
        {
            int rt = MarsOperands.Rt(word), rd = MarsOperands.Rd(word);

            if ((word & (1u << 25)) != 0) return VectorOperation(word);

            // The transfers' element is a byte index into the register, not a selector - see Mars_RspVector.md §3.
            string element = $"v{rd}[{(word >> 7) & 0xF}]";

            return MarsOperands.Rs(word) switch
            {
                0x00 => MarsOperands.Plain("mfc2", $"{Gpr(rt)}, {element}"),
                0x02 => MarsOperands.Plain("cfc2", $"{Gpr(rt)}, {ControlNames[rd & 3]}"),
                0x04 => MarsOperands.Plain("mtc2", $"{Gpr(rt)}, {element}"),
                0x06 => MarsOperands.Plain("ctc2", $"{Gpr(rt)}, {ControlNames[rd & 3]}"),
                _ => MarsOperands.Word(word),
            };
        }

        private static MarsInstruction VectorOperation(uint word)
        {
            uint funct = word & 0x3F;
            string? name = VectorNames[funct];
            if (name is null) return MarsOperands.Word(word);

            int vd = MarsOperands.Sa(word), vs = MarsOperands.Rd(word), vt = MarsOperands.Rt(word);
            string selected = $"v{vt}{Selector((int)((word >> 21) & 0xF))}";

            return funct switch
            {
                0x37 or 0x3F => MarsOperands.Plain(name),

                // One lane in, one lane out, and the destination lane lives in the vs field - see Mars_Disassembler.md §5.2.
                >= 0x30 and <= 0x36 => MarsOperands.Plain(name, $"v{vd}[{vs & 7}], {selected}"),

                _ => MarsOperands.Plain(name, $"v{vd}, v{vs}, {selected}"),
            };
        }

        // The SGI spelling: nothing for the whole register, then quarters, halves and single elements - see Mars_Disassembler.md §5.1.
        private static string Selector(int e) => e switch
        {
            0 or 1 => "",
            2 or 3 => $"[{e - 2}q]",
            >= 4 and <= 7 => $"[{e - 4}h]",
            _ => $"[{e - 8}]",
        };

        private static MarsInstruction Vector(string?[] names, uint word, StaticReferenceKind kind)
        {
            int format = MarsOperands.Rd(word);
            string? name = format < names.Length ? names[format] : null;
            if (name is null) return MarsOperands.Word(word);

            int offset = ((int)(word << 25) >> 25) << OffsetScale[format];
            string operands = $"v{MarsOperands.Rt(word)}[{(word >> 7) & 0xF}], {MarsOperands.Hex(offset)}({Gpr(MarsOperands.Rs(word))})";

            return MarsOperands.Rs(word) == 0
                ? new(name, operands, kind, unchecked((uint)offset) & DmemMask)
                : new(name, operands, null, 0);
        }

        private static MarsInstruction Scalar(string mnemonic, uint word, StaticReferenceKind kind)
        {
            string operands = $"{Gpr(MarsOperands.Rt(word))}, {MarsOperands.Hex((short)word)}({Gpr(MarsOperands.Rs(word))})";

            return MarsOperands.Rs(word) == 0
                ? new(mnemonic, operands, kind, unchecked((uint)(short)word) & DmemMask)
                : new(mnemonic, operands, null, 0);
        }

        // Counted from the delay slot and wrapped inside instruction memory - see Mars_Rsp.md §2.
        private static MarsInstruction Branch(string mnemonic, string registers, uint word, uint address)
        {
            uint target = unchecked(address + 4 + (uint)((short)word << 2)) & ImemMask;
            return new(mnemonic, $"{registers}0x{target:X3}", StaticReferenceKind.Call, target);
        }

        private static MarsInstruction Jump(string mnemonic, uint word)
        {
            uint target = (word << 2) & ImemMask;
            return new(mnemonic, $"0x{target:X3}", StaticReferenceKind.Call, target);
        }

        private static string Gpr(int index) => MarsOperands.Gpr[index];
    }
}
