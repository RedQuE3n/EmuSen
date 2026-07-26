using System;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.DianaOS;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    // Every shift/rotate opcode (ASL/LSR/ROL/ROR), each in its accumulator
    // and memory-operand forms.
    public partial class Cpu
    {
        private void OpASL(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                SetFlag(CpuFlags.C, (val & 0x80) != 0); // Bit 7 goes to Carry
                val = (byte)(val << 1);
                _bus.Write8(address, val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.C, (val & 0x8000) != 0); // Bit 15 goes to Carry
                val = (ushort)(val << 1);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
                UpdateZN(val, false);
            }
        }

        private void OpASLA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte a8 = (byte)(A & 0xFF);
                SetFlag(CpuFlags.C, (a8 & 0x80) != 0);
                a8 = (byte)(a8 << 1);
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                SetFlag(CpuFlags.C, (A & 0x8000) != 0);
                A = (ushort)(A << 1);
                UpdateZN(A, false);
            }
        }

        private void OpROLMem(uint address)
        {
            int carryIn = GetFlag(CpuFlags.C) ? 1 : 0;

            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                SetFlag(CpuFlags.C, (val & 0x80) != 0);
                val = (byte)((val << 1) | carryIn);
                _bus.Write8(address, val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.C, (val & 0x8000) != 0);
                val = (ushort)((val << 1) | carryIn);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
                UpdateZN(val, false);
            }
        }

        private void OpROLA(uint address)
        {
            int carryIn = GetFlag(CpuFlags.C) ? 1 : 0;

            if (IsMemory8Bit)
            {
                byte a8 = (byte)(A & 0xFF);
                SetFlag(CpuFlags.C, (a8 & 0x80) != 0);
                a8 = (byte)((a8 << 1) | carryIn);
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                SetFlag(CpuFlags.C, (A & 0x8000) != 0);
                A = (ushort)((A << 1) | carryIn);
                UpdateZN(A, false);
            }
        }

        private void OpRORA(uint address)
        {
            int carryIn = GetFlag(CpuFlags.C) ? 1 : 0;

            if (IsMemory8Bit)
            {
                byte a8 = (byte)(A & 0xFF);
                SetFlag(CpuFlags.C, (a8 & 0x01) != 0);
                a8 = (byte)((a8 >> 1) | (carryIn << 7));
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                SetFlag(CpuFlags.C, (A & 0x0001) != 0);
                A = (ushort)((A >> 1) | (carryIn << 15));
                UpdateZN(A, false);
            }
        }

        private void OpRORMem(uint address)
        {
            int carryIn = GetFlag(CpuFlags.C) ? 1 : 0;

            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                SetFlag(CpuFlags.C, (val & 0x01) != 0); // Bit 0 falls into Carry
                val = (byte)((val >> 1) | (carryIn << 7)); // Old Carry shifts into Bit 7
                _bus.Write8(address, val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.C, (val & 0x0001) != 0); // Bit 0 falls into Carry
                val = (ushort)((val >> 1) | (carryIn << 15)); // Old Carry shifts into Bit 15
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
                UpdateZN(val, false);
            }
        }

        private void OpLSRA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte a8 = (byte)(A & 0xFF);
                SetFlag(CpuFlags.C, (a8 & 0x01) != 0);
                a8 = (byte)(a8 >> 1);
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                SetFlag(CpuFlags.C, (A & 0x0001) != 0);
                A = (ushort)(A >> 1);
                UpdateZN(A, false);
            }
        }

        private void OpLSR(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                SetFlag(CpuFlags.C, (val & 0x01) != 0); // Bit 0 goes to Carry
                val = (byte)(val >> 1);
                _bus.Write8(address, val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.C, (val & 0x0001) != 0); // Bit 0 goes to Carry
                val = (ushort)(val >> 1);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
                UpdateZN(val, false);
            }
        }
    }
}
