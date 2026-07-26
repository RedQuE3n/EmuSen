using System;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.DianaOS;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    // NOP/unknown-opcode fallback, software interrupts (BRK/COP/RTI), WAI/STP
    // halt states, and the two block-move opcodes (MVN/MVP) - the "not really
    // any other category" operations, grouped together the way most 65816
    // references group them.
    public partial class Cpu
    {
        private void OpUnknown(uint address) { }

        private void OpNOP(uint address) { }

        private void OpBRK(uint address)
        {
            PC++; // Skip the signature byte

            if (!E) Push8(PB);
            Push16(PC);
            Push8(P);

            SetFlag(CpuFlags.I, true);
            SetFlag(CpuFlags.D, false); // BRK automatically clears Decimal mode

            ushort vectorAddr = E ? (ushort)0xFFFE : (ushort)0xFFE6;
            byte low = _bus.Read8(vectorAddr);
            byte high = _bus.Read8((uint)(vectorAddr + 1));

            PB = 0x00;
            PC = (ushort)((high << 8) | low);
        }

        private void OpCOP(uint address)
        {
            PC++; // Skip the signature byte

            if (!E) Push8(PB);
            Push16(PC);
            Push8(P);

            SetFlag(CpuFlags.I, true);
            SetFlag(CpuFlags.D, false);

            ushort vectorAddr = E ? (ushort)0xFFF4 : (ushort)0xFFE4;
            byte low = _bus.Read8(vectorAddr);
            byte high = _bus.Read8((uint)(vectorAddr + 1));

            PB = 0x00;
            PC = (ushort)((high << 8) | low);
        }

        private void OpRTI(uint address)
        {
            P = Pop8();
            PC = Pop16();

            if (!E)
            {
                PB = Pop8();
            }

            if (E)
            {
                SetFlag(CpuFlags.M, true);
                SetFlag(CpuFlags.X, true);
            }
            if (GetFlag(CpuFlags.X))
            {
                X &= 0x00FF;
                Y &= 0x00FF;
            }
        }

        // WAI/STP - see Venus_CPU.md §4.
        private void OpWAI(uint address) { _waitingForInterrupt = true; }
        private void OpSTP(uint address) { _stopped = true; }

        private void OpMVN(uint address)
        {
            byte destBank = (byte)(address >> 8);
            byte srcBank = (byte)(address & 0xFF);

            // The Data Bank (DB) is updated to the destination bank during a block move
            DB = destBank;

            // Read from Source, Write to Destination
            byte val = _bus.Read8((uint)((srcBank << 16) | X));
            _bus.Write8((uint)((destBank << 16) | Y), val);

            // MVN moves forward: Increment X and Y
            if (IsIndex8Bit)
            {
                X = (ushort)((X + 1) & 0xFF);
                Y = (ushort)((Y + 1) & 0xFF);
            }
            else
            {
                X++;
                Y++;
            }

            // Decrement the 16-bit Accumulator counter
            A--;

            // If the counter hasn't rolled over past 0, loop the instruction
            if (A != 0xFFFF)
            {
                PC -= 3;
            }
        }

        private void OpMVP(uint address)
        {
            byte destBank = (byte)(address >> 8);
            byte srcBank = (byte)(address & 0xFF);

            DB = destBank;

            byte val = _bus.Read8((uint)((srcBank << 16) | X));
            _bus.Write8((uint)((destBank << 16) | Y), val);

            if (IsIndex8Bit)
            {
                X = (ushort)((X - 1) & 0xFF);
                Y = (ushort)((Y - 1) & 0xFF);
            }
            else
            {
                X--;
                Y--;
            }

            A--;

            if (A != 0xFFFF)
            {
                PC -= 3;
            }
        }
    }
}
