using System;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

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
            bool decimalMode = GetFlag(CpuFlags.D);

            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)(A & 0xFF);

                if (decimalMode)
                {
                    AdcDecimalByte(a8, operand, carry, out int result, out bool c, out bool v);
                    SetFlag(CpuFlags.C, c);
                    SetFlag(CpuFlags.V, v);
                    A = (ushort)((A & 0xFF00) | result);
                    UpdateZN((ushort)result, true);
                }
                else
                {
                    int result = a8 + operand + carry;
                    SetFlag(CpuFlags.C, result > 0xFF);
                    SetFlag(CpuFlags.V, ((~(a8 ^ operand)) & (a8 ^ result) & 0x80) != 0);
                    A = (ushort)((A & 0xFF00) | (result & 0xFF));
                    UpdateZN((ushort)(result & 0xFF), true);
                }
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));

                if (decimalMode)
                {
                    AdcDecimalByte((byte)(A & 0xFF), (byte)(operand & 0xFF), carry, out int lowResult, out bool lowCarry, out _);
                    // The high byte's own v (from AdcDecimalByte, using its
                    // pre-high-nibble-correction intermediate - see that
                    // method's comment) IS the 16-bit result's V flag - a
                    // separate first attempt at computing V here from the
                    // FINAL corrected high byte instead gave the wrong
                    // answer on every test where that byte's high-nibble
                    // correction actually fired, caught via CpuValidation.
                    AdcDecimalByte((byte)(A >> 8), (byte)(operand >> 8), lowCarry ? 1 : 0, out int highResult, out bool c, out bool v);
                    ushort result16 = (ushort)((highResult << 8) | lowResult);

                    SetFlag(CpuFlags.C, c);
                    SetFlag(CpuFlags.V, v);
                    A = result16;
                    UpdateZN(A, false);
                }
                else
                {
                    int result = A + operand + carry;
                    SetFlag(CpuFlags.C, result > 0xFFFF);
                    SetFlag(CpuFlags.V, ((~(A ^ operand)) & (A ^ result) & 0x8000) != 0);
                    A = (ushort)(result & 0xFFFF);
                    UpdateZN(A, false);
                }
            }
        }

        // One decimal-mode ADC byte-add: adds two BCD-encoded bytes plus a
        // binary carry-in, correcting each nibble that exceeds 9 by adding
        // 6 (the standard BCD adjustment), propagating a carry out of the
        // low nibble into the high nibble the same way a real BCD adder
        // does. result is the corrected byte (0-99 in BCD, i.e. 0x00-0x99);
        // carryOut is true if the corrected sum exceeds 99 (0x99); v is the
        // usual sign-overflow check but against this same fully-corrected
        // result (an earlier attempt using the pre-high-nibble-correction
        // intermediate matched N/Z/C but got V wrong on every test where
        // the high-nibble correction actually fired - caught via the
        // CpuValidation harness, not assumed from memory).
        private static void AdcDecimalByte(byte a, byte b, int carryIn, out int result, out bool carryOut, out bool v)
        {
            int lo = (a & 0x0F) + (b & 0x0F) + carryIn;
            if (lo > 9) lo += 6;

            int hi = (a >> 4) + (b >> 4) + (lo > 0x0F ? 1 : 0);
            lo &= 0x0F;

            int preCorrection = ((hi & 0x0F) << 4) | lo;
            v = ((~(a ^ b)) & (a ^ preCorrection) & 0x80) != 0;

            if (hi > 9) hi += 6;
            carryOut = hi > 0x0F;
            result = ((hi & 0x0F) << 4) | lo;
        }

        private void OpSBC(uint address)
        {
            int borrow = GetFlag(CpuFlags.C) ? 0 : 1;
            bool decimalMode = GetFlag(CpuFlags.D);

            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)(A & 0xFF);
                int binaryResult = a8 - operand - borrow;

                // SBC's C/V always reflect the BINARY subtraction, even in
                // decimal mode - a documented, real 65816 quirk (distinct
                // from ADC, whose flags DO reflect the decimal result).
                // N and Z are the exception: both reflect the decimal-
                // corrected result, not the binary one - caught via the
                // CpuValidation harness (a byte can be zero once decimal-
                // corrected while the raw binary subtraction result isn't,
                // e.g. 0x34-0xD3-1 = 0x60 binary but 0x00 once BCD-
                // corrected), not assumed from memory.
                SetFlag(CpuFlags.C, binaryResult >= 0);
                SetFlag(CpuFlags.V, ((a8 ^ operand) & (a8 ^ binaryResult) & 0x80) != 0);

                int result = decimalMode ? SbcDecimalByte(a8, operand, borrow) : binaryResult & 0xFF;
                UpdateZN((ushort)result, true);
                A = (ushort)((A & 0xFF00) | (result & 0xFF));
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                int binaryResult = A - operand - borrow;

                SetFlag(CpuFlags.C, binaryResult >= 0);
                SetFlag(CpuFlags.V, ((A ^ operand) & (A ^ binaryResult) & 0x8000) != 0);

                int result16;
                if (decimalMode)
                {
                    int lowResult = SbcDecimalByte((byte)(A & 0xFF), (byte)(operand & 0xFF), borrow, out bool lowBorrowOut);
                    int highResult = SbcDecimalByte((byte)(A >> 8), (byte)(operand >> 8), lowBorrowOut ? 1 : 0, out _);
                    result16 = (highResult << 8) | lowResult;
                }
                else
                {
                    result16 = binaryResult & 0xFFFF;
                }
                UpdateZN((ushort)result16, false);
                A = (ushort)result16;
            }
        }

        // One decimal-mode SBC byte-subtract: subtracts BCD-encoded byte b
        // (plus a binary borrow-in) from a, correcting each nibble that
        // goes negative by subtracting 6, propagating the borrow out of
        // the low nibble into the high nibble. Only the numeric result
        // uses this - see OpSBC's own comment on why the flags never do.
        private static int SbcDecimalByte(byte a, byte b, int borrowIn) => SbcDecimalByte(a, b, borrowIn, out _);

        private static int SbcDecimalByte(byte a, byte b, int borrowIn, out bool borrowOut)
        {
            int lo = (a & 0x0F) - (b & 0x0F) - borrowIn;
            int hi = (a >> 4) - (b >> 4);
            if (lo < 0) { lo += 10; hi--; }

            borrowOut = hi < 0;
            if (hi < 0) hi += 10;

            return ((hi & 0x0F) << 4) | (lo & 0x0F);
        }
    }
}
