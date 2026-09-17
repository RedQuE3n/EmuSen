using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.Cores.Nintendo.Mars.Rsp
{
    // The signal processor's scalar half: MIPS without exceptions, alignment or a second word - see Mars_Rsp.md.
    public sealed class Rsp
    {
        // Every program counter is an offset into instruction memory, and wraps inside it - see §2.
        public const uint PcMask = 0xFFC;

        // Every data address is an offset into data memory, and wraps inside it - see §4.
        public const uint DataMask = 0xFFF;

        public readonly uint[] Gpr = new uint[32];

        public uint Pc;
        public uint NextPc;

        public bool Halted = true;
        public bool Broke;

        private readonly MarsBus _bus;

        public Rsp(MarsBus bus) => _bus = bus;

        public void Start(uint pc)
        {
            Pc = pc & PcMask;
            NextPc = (Pc + 4) & PcMask;
            Halted = false;
        }

        public void Step()
        {
            if (Halted) return;

            uint instruction = ReadInstruction(Pc);

            Pc = NextPc;
            NextPc = (Pc + 4) & PcMask;

            Execute(instruction);
        }

        private void Execute(uint instruction)
        {
            uint op = instruction >> 26;

            switch (op)
            {
                case 0x00: ExecuteSpecial(instruction); return;
                case 0x01: ExecuteRegImm(instruction); return;

                case 0x02: Jump(instruction); return;
                case 0x03: Link(); Jump(instruction); return;

                case 0x04: BranchIf(Read(Rs(instruction)) == Read(Rt(instruction)), instruction); return;
                case 0x05: BranchIf(Read(Rs(instruction)) != Read(Rt(instruction)), instruction); return;
                case 0x06: BranchIf((int)Read(Rs(instruction)) <= 0, instruction); return;
                case 0x07: BranchIf((int)Read(Rs(instruction)) > 0, instruction); return;

                // Overflow is not a fault here, so the two additions and the two subtractions agree - see §3.
                case 0x08:
                case 0x09: Write(Rt(instruction), Read(Rs(instruction)) + Immediate(instruction)); return;

                case 0x0A: Write(Rt(instruction), (int)Read(Rs(instruction)) < (int)Immediate(instruction) ? 1u : 0u); return;
                case 0x0B: Write(Rt(instruction), Read(Rs(instruction)) < Immediate(instruction) ? 1u : 0u); return;
                case 0x0C: Write(Rt(instruction), Read(Rs(instruction)) & (instruction & 0xFFFF)); return;
                case 0x0D: Write(Rt(instruction), Read(Rs(instruction)) | (instruction & 0xFFFF)); return;
                case 0x0E: Write(Rt(instruction), Read(Rs(instruction)) ^ (instruction & 0xFFFF)); return;
                case 0x0F: Write(Rt(instruction), instruction << 16); return;

                case 0x10: ExecuteCop0(instruction); return;

                case 0x20: Write(Rt(instruction), (uint)(int)(sbyte)ReadData(Address(instruction), 1)); return;
                case 0x21: Write(Rt(instruction), (uint)(int)(short)ReadData(Address(instruction), 2)); return;
                case 0x23:
                case 0x27: Write(Rt(instruction), ReadData(Address(instruction), 4)); return;
                case 0x24: Write(Rt(instruction), ReadData(Address(instruction), 1)); return;
                case 0x25: Write(Rt(instruction), ReadData(Address(instruction), 2)); return;

                case 0x28: WriteData(Address(instruction), Read(Rt(instruction)), 1); return;
                case 0x29: WriteData(Address(instruction), Read(Rt(instruction)), 2); return;
                case 0x2B: WriteData(Address(instruction), Read(Rt(instruction)), 4); return;

                // Nothing here faults, so an encoding this half does not own simply does nothing - see §6.
                default: return;
            }
        }

        private void ExecuteSpecial(uint instruction)
        {
            int shift = (int)((instruction >> 6) & 0x1F);

            switch (instruction & 0x3F)
            {
                case 0x00: Write(Rd(instruction), Read(Rt(instruction)) << shift); return;
                case 0x02: Write(Rd(instruction), Read(Rt(instruction)) >> shift); return;
                case 0x03: Write(Rd(instruction), (uint)((int)Read(Rt(instruction)) >> shift)); return;

                case 0x04: Write(Rd(instruction), Read(Rt(instruction)) << (int)(Read(Rs(instruction)) & 0x1F)); return;
                case 0x06: Write(Rd(instruction), Read(Rt(instruction)) >> (int)(Read(Rs(instruction)) & 0x1F)); return;
                case 0x07: Write(Rd(instruction), (uint)((int)Read(Rt(instruction)) >> (int)(Read(Rs(instruction)) & 0x1F))); return;

                case 0x08: NextPc = Read(Rs(instruction)) & PcMask; return;
                case 0x09: LinkAndJump(Rd(instruction), Read(Rs(instruction))); return;

                case 0x0D: Break(); return;

                case 0x20:
                case 0x21: Write(Rd(instruction), Read(Rs(instruction)) + Read(Rt(instruction))); return;
                case 0x22:
                case 0x23: Write(Rd(instruction), Read(Rs(instruction)) - Read(Rt(instruction))); return;

                case 0x24: Write(Rd(instruction), Read(Rs(instruction)) & Read(Rt(instruction))); return;
                case 0x25: Write(Rd(instruction), Read(Rs(instruction)) | Read(Rt(instruction))); return;
                case 0x26: Write(Rd(instruction), Read(Rs(instruction)) ^ Read(Rt(instruction))); return;
                case 0x27: Write(Rd(instruction), ~(Read(Rs(instruction)) | Read(Rt(instruction)))); return;

                case 0x2A: Write(Rd(instruction), (int)Read(Rs(instruction)) < (int)Read(Rt(instruction)) ? 1u : 0u); return;
                case 0x2B: Write(Rd(instruction), Read(Rs(instruction)) < Read(Rt(instruction)) ? 1u : 0u); return;

                default: return;
            }
        }

        private void ExecuteRegImm(uint instruction)
        {
            int value = (int)Read(Rs(instruction));
            int selector = Rt(instruction);

            if ((selector & 0x10) != 0) Link();

            BranchIf((selector & 1) != 0 ? value >= 0 : value < 0, instruction);
        }

        // Coprocessor zero is the interface registers, reached from this side by index - see §5.
        private void ExecuteCop0(uint instruction)
        {
            int register = Rd(instruction) & 0x0F;

            switch (Rs(instruction))
            {
                case 0x00: Write(Rt(instruction), _bus.ReadRspControl(register)); return;
                case 0x04: _bus.WriteRspControl(register, Read(Rt(instruction))); return;
                default: return;
            }
        }

        private void Break()
        {
            Halted = true;
            Broke = true;
        }

        private void Link() => Write(31, NextPc);

        // The target is read before the link is written, which is the only way they can be one register - see §2.1.
        private void LinkAndJump(int register, uint target)
        {
            uint jump = target & PcMask;

            Write(register, NextPc);
            NextPc = jump;
        }

        private void Jump(uint instruction) => NextPc = (instruction << 2) & PcMask;

        private void BranchIf(bool taken, uint instruction)
        {
            if (taken) NextPc = (Pc + (Immediate(instruction) << 2)) & PcMask;
        }

        // An unaligned access is ordinary here: the bytes are taken in order and wrap - see §4.
        private uint ReadData(uint address, int size)
        {
            uint value = 0;
            for (int i = 0; i < size; i++) value = (value << 8) | _bus.SpDmem[(address + (uint)i) & DataMask];

            return value;
        }

        private void WriteData(uint address, uint value, int size)
        {
            for (int i = 0; i < size; i++)
            {
                _bus.SpDmem[(address + (uint)i) & DataMask] = (byte)(value >> ((size - 1 - i) * 8));
            }
        }

        private uint ReadInstruction(uint pc)
        {
            uint offset = pc & PcMask;
            return ((uint)_bus.SpImem[offset] << 24) | ((uint)_bus.SpImem[offset + 1] << 16)
                | ((uint)_bus.SpImem[offset + 2] << 8) | _bus.SpImem[offset + 3];
        }

        private uint Address(uint instruction) => (Read(Rs(instruction)) + Immediate(instruction)) & DataMask;

        private uint Read(int register) => Gpr[register];

        // Register zero is wired to zero here too, so a write to it is dropped rather than stored.
        private void Write(int register, uint value)
        {
            if (register != 0) Gpr[register] = value;
        }

        private static uint Immediate(uint instruction) => (uint)(int)(short)instruction;

        private static int Rs(uint instruction) => (int)((instruction >> 21) & 0x1F);

        private static int Rt(uint instruction) => (int)((instruction >> 16) & 0x1F);

        private static int Rd(uint instruction) => (int)((instruction >> 11) & 0x1F);
    }
}
