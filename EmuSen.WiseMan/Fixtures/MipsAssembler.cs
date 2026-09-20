using System.Collections.Generic;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.WiseMan.Fixtures
{
    // Emits MIPS III words so instruction tests read as assembly rather than hex - see Mars_Cpu.md §10.
    public sealed class MipsAssembler
    {
        // Where a test program is assembled, and the cached window the CPU fetches it through.
        public const uint LoadAddress = 0x0000_0000;
        public const ulong EntryPoint = 0xFFFF_FFFF_8000_0000;

        private readonly List<uint> _words = new();

        public MipsAssembler Word(uint word)
        {
            _words.Add(word);
            return this;
        }

        public MipsAssembler Nop() => Word(0);

        public MipsAssembler Addi(int rt, int rs, short immediate) => I(0x08, rs, rt, immediate);
        public MipsAssembler Addiu(int rt, int rs, short immediate) => I(0x09, rs, rt, immediate);
        public MipsAssembler Slti(int rt, int rs, short immediate) => I(0x0A, rs, rt, immediate);
        public MipsAssembler Sltiu(int rt, int rs, short immediate) => I(0x0B, rs, rt, immediate);
        public MipsAssembler Andi(int rt, int rs, ushort immediate) => I(0x0C, rs, rt, (short)immediate);
        public MipsAssembler Ori(int rt, int rs, ushort immediate) => I(0x0D, rs, rt, (short)immediate);
        public MipsAssembler Lui(int rt, ushort immediate) => I(0x0F, 0, rt, (short)immediate);
        public MipsAssembler Daddi(int rt, int rs, short immediate) => I(0x18, rs, rt, immediate);
        public MipsAssembler Daddiu(int rt, int rs, short immediate) => I(0x19, rs, rt, immediate);

        public MipsAssembler Add(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x20);
        public MipsAssembler Addu(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x21);
        public MipsAssembler Sub(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x22);
        public MipsAssembler Subu(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x23);
        public MipsAssembler And(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x24);
        public MipsAssembler Or(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x25);
        public MipsAssembler Xor(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x26);
        public MipsAssembler Nor(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x27);
        public MipsAssembler Slt(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x2A);
        public MipsAssembler Sltu(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x2B);
        public MipsAssembler Dadd(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x2C);
        public MipsAssembler Daddu(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x2D);

        public MipsAssembler Dsub(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x2E);
        public MipsAssembler Dsubu(int rd, int rs, int rt) => R(rs, rt, rd, 0, 0x2F);

        public MipsAssembler Sll(int rd, int rt, int sa) => R(0, rt, rd, sa, 0x00);
        public MipsAssembler Srl(int rd, int rt, int sa) => R(0, rt, rd, sa, 0x02);
        public MipsAssembler Sra(int rd, int rt, int sa) => R(0, rt, rd, sa, 0x03);
        public MipsAssembler Srav(int rd, int rt, int rs) => R(rs, rt, rd, 0, 0x07);
        public MipsAssembler Dsll(int rd, int rt, int sa) => R(0, rt, rd, sa, 0x38);
        public MipsAssembler Dsll32(int rd, int rt, int sa) => R(0, rt, rd, sa, 0x3C);
        public MipsAssembler Dsrl32(int rd, int rt, int sa) => R(0, rt, rd, sa, 0x3E);

        public MipsAssembler Dsrl(int rd, int rt, int sa) => R(0, rt, rd, sa, 0x3A);
        public MipsAssembler Dsra(int rd, int rt, int sa) => R(0, rt, rd, sa, 0x3B);
        public MipsAssembler Dsra32(int rd, int rt, int sa) => R(0, rt, rd, sa, 0x3F);
        public MipsAssembler Dsllv(int rd, int rt, int rs) => R(rs, rt, rd, 0, 0x14);
        public MipsAssembler Dsrlv(int rd, int rt, int rs) => R(rs, rt, rd, 0, 0x16);
        public MipsAssembler Dsrav(int rd, int rt, int rs) => R(rs, rt, rd, 0, 0x17);

        public MipsAssembler Mult(int rs, int rt) => R(rs, rt, 0, 0, 0x18);
        public MipsAssembler Multu(int rs, int rt) => R(rs, rt, 0, 0, 0x19);
        public MipsAssembler Dmult(int rs, int rt) => R(rs, rt, 0, 0, 0x1C);
        public MipsAssembler Dmultu(int rs, int rt) => R(rs, rt, 0, 0, 0x1D);
        public MipsAssembler Ddiv(int rs, int rt) => R(rs, rt, 0, 0, 0x1E);
        public MipsAssembler Ddivu(int rs, int rt) => R(rs, rt, 0, 0, 0x1F);
        public MipsAssembler Div(int rs, int rt) => R(rs, rt, 0, 0, 0x1A);
        public MipsAssembler Divu(int rs, int rt) => R(rs, rt, 0, 0, 0x1B);
        public MipsAssembler Mfhi(int rd) => R(0, 0, rd, 0, 0x10);
        public MipsAssembler Mflo(int rd) => R(0, 0, rd, 0, 0x12);

        public MipsAssembler Beq(int rs, int rt, short offset) => I(0x04, rs, rt, offset);
        public MipsAssembler Bne(int rs, int rt, short offset) => I(0x05, rs, rt, offset);
        public MipsAssembler Beql(int rs, int rt, short offset) => I(0x14, rs, rt, offset);
        public MipsAssembler Bnel(int rs, int rt, short offset) => I(0x15, rs, rt, offset);
        public MipsAssembler Bgez(int rs, short offset) => I(0x01, rs, 0x01, offset);
        public MipsAssembler Bltz(int rs, short offset) => I(0x01, rs, 0x00, offset);
        public MipsAssembler Blez(int rs, short offset) => I(0x06, rs, 0, offset);
        public MipsAssembler Bgtz(int rs, short offset) => I(0x07, rs, 0, offset);
        public MipsAssembler Bgezal(int rs, short offset) => I(0x01, rs, 0x11, offset);
        public MipsAssembler Bltzal(int rs, short offset) => I(0x01, rs, 0x10, offset);

        public MipsAssembler J(uint target) => Word((0x02u << 26) | ((target >> 2) & 0x03FF_FFFF));
        public MipsAssembler Jal(uint target) => Word((0x03u << 26) | ((target >> 2) & 0x03FF_FFFF));
        public MipsAssembler Jr(int rs) => R(rs, 0, 0, 0, 0x08);
        public MipsAssembler Jalr(int rd, int rs) => R(rs, 0, rd, 0, 0x09);

        public MipsAssembler Lb(int rt, int rs, short offset) => I(0x20, rs, rt, offset);
        public MipsAssembler Lbu(int rt, int rs, short offset) => I(0x24, rs, rt, offset);
        public MipsAssembler Lh(int rt, int rs, short offset) => I(0x21, rs, rt, offset);
        public MipsAssembler Lhu(int rt, int rs, short offset) => I(0x25, rs, rt, offset);
        public MipsAssembler Lw(int rt, int rs, short offset) => I(0x23, rs, rt, offset);
        public MipsAssembler Lwu(int rt, int rs, short offset) => I(0x27, rs, rt, offset);
        public MipsAssembler Ld(int rt, int rs, short offset) => I(0x37, rs, rt, offset);
        public MipsAssembler Lwl(int rt, int rs, short offset) => I(0x22, rs, rt, offset);
        public MipsAssembler Lwr(int rt, int rs, short offset) => I(0x26, rs, rt, offset);
        public MipsAssembler Ldl(int rt, int rs, short offset) => I(0x1A, rs, rt, offset);
        public MipsAssembler Ldr(int rt, int rs, short offset) => I(0x1B, rs, rt, offset);
        public MipsAssembler Swl(int rt, int rs, short offset) => I(0x2A, rs, rt, offset);
        public MipsAssembler Swr(int rt, int rs, short offset) => I(0x2E, rs, rt, offset);
        public MipsAssembler Sdl(int rt, int rs, short offset) => I(0x2C, rs, rt, offset);
        public MipsAssembler Sdr(int rt, int rs, short offset) => I(0x2D, rs, rt, offset);

        public MipsAssembler Sb(int rt, int rs, short offset) => I(0x28, rs, rt, offset);
        public MipsAssembler Sh(int rt, int rs, short offset) => I(0x29, rs, rt, offset);
        public MipsAssembler Sw(int rt, int rs, short offset) => I(0x2B, rs, rt, offset);
        public MipsAssembler Sd(int rt, int rs, short offset) => I(0x3F, rs, rt, offset);

        public MipsAssembler Mfc0(int rt, int rd) => Word((0x10u << 26) | (0u << 21) | ((uint)rt << 16) | ((uint)rd << 11));
        public MipsAssembler Mtc0(int rt, int rd) => Word((0x10u << 26) | (4u << 21) | ((uint)rt << 16) | ((uint)rd << 11));

        public MipsAssembler Mfc1(int rt, int fs) => Cop1(0x00, rt, fs);
        public MipsAssembler Dmfc1(int rt, int fs) => Cop1(0x01, rt, fs);
        public MipsAssembler Cfc1(int rt, int fs) => Cop1(0x02, rt, fs);
        public MipsAssembler Mtc1(int rt, int fs) => Cop1(0x04, rt, fs);
        public MipsAssembler Dmtc1(int rt, int fs) => Cop1(0x05, rt, fs);
        public MipsAssembler Ctc1(int rt, int fs) => Cop1(0x06, rt, fs);

        public MipsAssembler Lwc1(int ft, int rs, short offset) => I(0x31, rs, ft, offset);
        public MipsAssembler Ldc1(int ft, int rs, short offset) => I(0x35, rs, ft, offset);
        public MipsAssembler Swc1(int ft, int rs, short offset) => I(0x39, rs, ft, offset);
        public MipsAssembler Sdc1(int ft, int rs, short offset) => I(0x3D, rs, ft, offset);

        // A coprocessor-1 arithmetic word: format, function, and the three register fields.
        public MipsAssembler Cop1Format(int format, int funct, int fd, int fs, int ft) =>
            Word((0x11u << 26) | ((uint)format << 21) | ((uint)ft << 16) | ((uint)fs << 11)
                 | ((uint)fd << 6) | (uint)funct);

        // The sub-opcode field decides the whole of a coprocessor-1 move, reserved forms included.
        public MipsAssembler Cop1(int rs, int rt, int fs) =>
            Word((0x11u << 26) | ((uint)rs << 21) | ((uint)rt << 16) | ((uint)fs << 11));

        public MipsAssembler Dmfc0(int rt, int rd) => Word((0x10u << 26) | (1u << 21) | ((uint)rt << 16) | ((uint)rd << 11));
        public MipsAssembler Dmtc0(int rt, int rd) => Word((0x10u << 26) | (5u << 21) | ((uint)rt << 16) | ((uint)rd << 11));

        public MipsAssembler Cache(int op, int rs, short offset) => I(0x2F, rs, op, offset);
        public MipsAssembler Ll(int rt, int rs, short offset) => I(0x30, rs, rt, offset);
        public MipsAssembler Sc(int rt, int rs, short offset) => I(0x38, rs, rt, offset);
        public MipsAssembler Lld(int rt, int rs, short offset) => I(0x34, rs, rt, offset);
        public MipsAssembler Scd(int rt, int rs, short offset) => I(0x3C, rs, rt, offset);


        public MipsAssembler Tlbr() => Word((0x10u << 26) | (0x10u << 21) | 0x01);
        public MipsAssembler Tlbwi() => Word((0x10u << 26) | (0x10u << 21) | 0x02);
        public MipsAssembler Tlbwr() => Word((0x10u << 26) | (0x10u << 21) | 0x06);
        public MipsAssembler Tlbp() => Word((0x10u << 26) | (0x10u << 21) | 0x08);
        public MipsAssembler Eret() => Word((0x10u << 26) | (0x10u << 21) | 0x18);

        public MipsAssembler Tge(int rs, int rt) => R(rs, rt, 0, 0, 0x30);
        public MipsAssembler Tgeu(int rs, int rt) => R(rs, rt, 0, 0, 0x31);
        public MipsAssembler Tlt(int rs, int rt) => R(rs, rt, 0, 0, 0x32);
        public MipsAssembler Tltu(int rs, int rt) => R(rs, rt, 0, 0, 0x33);
        public MipsAssembler Teq(int rs, int rt) => R(rs, rt, 0, 0, 0x34);
        public MipsAssembler Tne(int rs, int rt) => R(rs, rt, 0, 0, 0x36);

        public MipsAssembler Tgei(int rs, short immediate) => I(0x01, rs, 0x08, immediate);
        public MipsAssembler Tgeiu(int rs, short immediate) => I(0x01, rs, 0x09, immediate);
        public MipsAssembler Tlti(int rs, short immediate) => I(0x01, rs, 0x0A, immediate);
        public MipsAssembler Tltiu(int rs, short immediate) => I(0x01, rs, 0x0B, immediate);
        public MipsAssembler Teqi(int rs, short immediate) => I(0x01, rs, 0x0C, immediate);
        public MipsAssembler Tnei(int rs, short immediate) => I(0x01, rs, 0x0E, immediate);

        public MipsAssembler Syscall() => R(0, 0, 0, 0, 0x0C);
        public MipsAssembler Break() => R(0, 0, 0, 0, 0x0D);

        // The CO=1 space, whose operand bits hardware ignores entirely - see Mars_Cop0.md §10.
        public MipsAssembler Cop0Function(uint funct, uint operands = 0) =>
            Word((0x10u << 26) | (0x10u << 21) | (operands << 6) | funct);

        public MipsAssembler Cop0SubOpcode(int rs) => Word((0x10u << 26) | ((uint)rs << 21));

        public uint[] ToArray() => _words.ToArray();

        public MemoryBus LoadInto(MemoryBus bus)
        {
            for (int i = 0; i < _words.Count; i++) bus.Write32(LoadAddress + (uint)(i * 4), _words[i]);
            return bus;
        }

        // A machine with this program in memory and the program counter already on it.
        public Cpu Build(MemoryBus? existing = null) =>
            new(LoadInto(existing ?? new MemoryBus())) { Pc = EntryPoint, NextPc = EntryPoint + 4 };

        public Cpu Run(int steps, MemoryBus? existing = null)
        {
            var cpu = Build(existing);

            cpu.Run(steps);
            return cpu;
        }

        private MipsAssembler I(uint op, int rs, int rt, short immediate) =>
            Word((op << 26) | ((uint)rs << 21) | ((uint)rt << 16) | (ushort)immediate);

        private MipsAssembler R(int rs, int rt, int rd, int sa, uint funct) =>
            Word(((uint)rs << 21) | ((uint)rt << 16) | ((uint)rd << 11) | ((uint)sa << 6) | funct);
    }
}
