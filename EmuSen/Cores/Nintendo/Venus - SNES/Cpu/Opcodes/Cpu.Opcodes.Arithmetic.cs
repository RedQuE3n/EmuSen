using System;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    // Increment/decrement (INC/DEC on A, memory, X, Y), comparisons (CMP/
    // CPX/CPY), and addition/subtraction with carry (ADC/SBC) - every
    // opcode that actually does arithmetic, as opposed to just moving or
    // combining bits.
    public partial class Cpu
    {
        private void OpINCA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = (byte)((A + 1) & 0xFF);
                A = (ushort)((A & 0xFF00) | val);
                UpdateZN(val, true);
            }
            else
            {
                A = (ushort)((A + 1) & 0xFFFF);
                UpdateZN(A, false);
            }
        }

        private void OpDECA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = (byte)((A - 1) & 0xFF);
                A = (ushort)((A & 0xFF00) | val);
                UpdateZN(val, true);
            }
            else
            {
                A = (ushort)((A - 1) & 0xFFFF);
                UpdateZN(A, false);
            }
        }

        private void OpINCMem(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = (byte)((_bus.Read8(address) + 1) & 0xFF);
                _bus.Write8(address, val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                val = (ushort)((val + 1) & 0xFFFF);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
                UpdateZN(val, false);
            }
        }

        private void OpDECMem(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = (byte)((_bus.Read8(address) - 1) & 0xFF);
                _bus.Write8(address, val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                val = (ushort)((val - 1) & 0xFFFF);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
                UpdateZN(val, false);
            }
        }

        private void OpINX(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = (byte)((X + 1) & 0xFF);
                X = val;
                UpdateZN(val, true);
            }
            else
            {
                X = (ushort)((X + 1) & 0xFFFF);
                UpdateZN(X, false);
            }
        }

        private void OpINY(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = (byte)((Y + 1) & 0xFF);
                Y = val;
                UpdateZN(val, true);
            }
            else
            {
                Y = (ushort)((Y + 1) & 0xFFFF);
                UpdateZN(Y, false);
            }
        }

        private void OpDEX(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = (byte)((X - 1) & 0xFF);
                X = val;
                UpdateZN(val, true);
            }
            else
            {
                X = (ushort)((X - 1) & 0xFFFF);
                UpdateZN(X, false);
            }
        }

        private void OpDEY(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = (byte)((Y - 1) & 0xFF);
                Y = val;
                UpdateZN(val, true);
            }
            else
            {
                Y = (ushort)((Y - 1) & 0xFFFF);
                UpdateZN(Y, false);
            }
        }

        private void OpCMP(uint address)
        {
            if (DebugSettings.CpuVerboseLogging && address == 0x002140)
            {
                ushort memVal = IsMemory8Bit ? _bus.Read8(address) : (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                Console.WriteLine($"[DEBUG APU] CMP against $2140 | Accumulator (A) = 0x{A:X4} | Memory = 0x{memVal:X4} | 8-Bit M-Flag = {IsMemory8Bit}");
            }

            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)(A & 0xFF);
                int result = a8 - operand;

                SetFlag(CpuFlags.C, a8 >= operand);
                UpdateZN((ushort)(result & 0xFF), true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                int result = A - operand;

                SetFlag(CpuFlags.C, A >= operand);
                UpdateZN((ushort)(result & 0xFFFF), false);
            }
        }

        private void OpCPX(uint address)
        {
            if (IsIndex8Bit)
            {
                byte operand = _bus.Read8(address);
                byte x8 = (byte)(X & 0xFF);
                int result = x8 - operand;

                SetFlag(CpuFlags.C, x8 >= operand);
                UpdateZN((ushort)(result & 0xFF), true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                int result = X - operand;

                SetFlag(CpuFlags.C, X >= operand);
                UpdateZN((ushort)(result & 0xFFFF), false);
            }
        }

        private void OpCPY(uint address)
        {
            if (IsIndex8Bit)
            {
                byte operand = _bus.Read8(address);
                byte y8 = (byte)(Y & 0xFF);
                int result = y8 - operand;

                SetFlag(CpuFlags.C, y8 >= operand);
                UpdateZN((ushort)(result & 0xFF), true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                int result = Y - operand;

                SetFlag(CpuFlags.C, Y >= operand);
                UpdateZN((ushort)(result & 0xFFFF), false);
            }
        }

        private void OpADC(uint address)
        {
            int carry = GetFlag(CpuFlags.C) ? 1 : 0;

            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)(A & 0xFF);
                int result = a8 + operand + carry;

                SetFlag(CpuFlags.C, result > 0xFF);
                SetFlag(CpuFlags.V, ((~(a8 ^ operand)) & (a8 ^ result) & 0x80) != 0);

                A = (ushort)((A & 0xFF00) | (result & 0xFF));
                UpdateZN((ushort)(result & 0xFF), true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                int result = A + operand + carry;

                SetFlag(CpuFlags.C, result > 0xFFFF);
                SetFlag(CpuFlags.V, ((~(A ^ operand)) & (A ^ result) & 0x8000) != 0);

                A = (ushort)(result & 0xFFFF);
                UpdateZN(A, false);
            }
        }

        private void OpSBC(uint address)
        {
            int borrow = GetFlag(CpuFlags.C) ? 0 : 1;

            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)(A & 0xFF);
                int result = a8 - operand - borrow;

                SetFlag(CpuFlags.C, result >= 0);
                SetFlag(CpuFlags.V, ((a8 ^ operand) & (a8 ^ result) & 0x80) != 0);

                A = (ushort)((A & 0xFF00) | (result & 0xFF));
                UpdateZN((ushort)(result & 0xFF), true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                int result = A - operand - borrow;

                SetFlag(CpuFlags.C, result >= 0);
                SetFlag(CpuFlags.V, ((A ^ operand) & (A ^ result) & 0x8000) != 0);

                A = (ushort)(result & 0xFFFF);
                UpdateZN(A, false);
            }
        }
    }
}
