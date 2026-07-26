using System;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Shell;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    // Bitwise operations against the accumulator (ORA/AND/EOR), the two
    // "test/set bits without disturbing the operand's other bits" opcodes
    // (TRB/TSB), and BIT (test bits, immediate vs. non-immediate is a real
    // behavioral split - see Venus_CPU.md §6).
    public partial class Cpu
    {
        private void OpTRB(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                SetFlag(CpuFlags.Z, ((A & 0xFF) & val) == 0);

                // Reset the bits specified by the Accumulator
                val = (byte)(val & ~(A & 0xFF));
                _bus.Write8(address, val);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.Z, (A & val) == 0);

                val = (ushort)(val & ~A);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
            }
        }

        private void OpTSB(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                SetFlag(CpuFlags.Z, ((A & 0xFF) & val) == 0);

                val = (byte)(val | (A & 0xFF));
                _bus.Write8(address, val);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.Z, (A & val) == 0);

                val = (ushort)(val | A);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
            }
        }

        private void OpBIT(uint address)
        {
            // BIT (non-immediate): Z from A & mem; N and V copied from the top two
            // bits of the memory operand (bits 7/6 in 8-bit mode, 15/14 in 16-bit).
            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                SetFlag(CpuFlags.Z, ((A & 0xFF) & operand) == 0);
                SetFlag(CpuFlags.N, (operand & 0x80) != 0);
                SetFlag(CpuFlags.V, (operand & 0x40) != 0);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.Z, (A & operand) == 0);
                SetFlag(CpuFlags.N, (operand & 0x8000) != 0);
                SetFlag(CpuFlags.V, (operand & 0x4000) != 0);
            }
        }

        private void OpBITImm(uint address)
        {
            // BIT immediate is the special case: it ONLY affects Z, leaving N and V alone.
            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                SetFlag(CpuFlags.Z, ((A & 0xFF) & operand) == 0);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.Z, (A & operand) == 0);
            }
        }

        private void OpORA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)((A & 0xFF) | operand);
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                A = (ushort)(A | operand);
                UpdateZN(A, false);
            }
        }

        private void OpAND(uint address)
        {
            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)((A & 0xFF) & operand);
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                A = (ushort)(A & operand);
                UpdateZN(A, false);
            }
        }

        private void OpEOR(uint address)
        {
            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)((A & 0xFF) ^ operand);
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                A = (ushort)(A ^ operand);
                UpdateZN(A, false);
            }
        }
    }
}
