using System;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.DianaOS;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    // Memory load/store (LDA/LDX/LDY/STA/STX/STY/STZ) and every register-to-
    // register transfer (TXA/TAX/TYA/TAY/TXY/TYX/TCS/TSC/TCD/TDC/TXS/TSX),
    // plus XBA (swap A's own two bytes) - the "move a value somewhere else,
    // unchanged" family.
    public partial class Cpu
    {
        private void OpSTZ(uint address)
        {
            if (IsMemory8Bit) _bus.Write8(address, 0x00);
            else
            {
                _bus.Write8(address, 0x00);
                _bus.Write8(address + 1, 0x00);
            }
        }

        private void OpLDA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                A = (ushort)((A & 0xFF00) | val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                A = val;
                UpdateZN(val, false);
            }
        }

        private void OpLDX(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = _bus.Read8(address);
                X = val;
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                X = val;
                UpdateZN(val, false);
            }
        }

        private void OpLDY(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = _bus.Read8(address);
                Y = val;
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                Y = val;
                UpdateZN(val, false);
            }
        }

        private void OpSTA(uint address)
        {
            if (IsMemory8Bit)
            {
                _bus.Write8(address, (byte)(A & 0xFF));
            }
            else
            {
                _bus.Write8(address, (byte)(A & 0xFF));
                _bus.Write8(address + 1, (byte)(A >> 8));
            }
        }

        private void OpSTX(uint address)
        {
            if (IsIndex8Bit)
            {
                _bus.Write8(address, (byte)(X & 0xFF));
            }
            else
            {
                _bus.Write8(address, (byte)(X & 0xFF));
                _bus.Write8(address + 1, (byte)(X >> 8));
            }
        }

        private void OpSTY(uint address)
        {
            if (IsIndex8Bit)
            {
                _bus.Write8(address, (byte)(Y & 0xFF));
            }
            else
            {
                _bus.Write8(address, (byte)(Y & 0xFF));
                _bus.Write8(address + 1, (byte)(Y >> 8));
            }
        }

        private void OpTXA(uint address)
        {
            if (IsMemory8Bit)
            {
                A = (ushort)((A & 0xFF00) | (X & 0xFF));
                UpdateZN((ushort)(X & 0xFF), true);
            }
            else
            {
                A = X;
                UpdateZN(A, false);
            }
        }

        private void OpTXY(uint address)
        {
            if (IsIndex8Bit)
            {
                Y = (ushort)(X & 0xFF);
                UpdateZN((ushort)(X & 0xFF), true);
            }
            else
            {
                Y = X;
                UpdateZN(Y, false);
            }
        }

        private void OpTYX(uint address)
        {
            if (IsIndex8Bit)
            {
                X = (ushort)(Y & 0xFF);
                UpdateZN((ushort)(Y & 0xFF), true);
            }
            else
            {
                X = Y;
                UpdateZN(X, false);
            }
        }

        private void OpTDC(uint address)
        {
            // Always full 16-bit regardless of M flag.
            A = D;
            UpdateZN(A, false);
        }

        private void OpTSC(uint address)
        {
            // Always full 16-bit regardless of M flag.
            A = S;
            UpdateZN(A, false);
        }

        private void OpTXS(uint address)
        {
            // No flags affected. In emulation mode the stack stays in page 1.
            if (E) S = (ushort)(0x0100 | (X & 0xFF));
            else S = X;
        }

        private void OpTSX(uint address)
        {
            if (IsIndex8Bit)
            {
                X = (ushort)(S & 0xFF);
                UpdateZN((ushort)(S & 0xFF), true);
            }
            else
            {
                X = S;
                UpdateZN(X, false);
            }
        }

        private void OpTAX(uint address)
        {
            if (IsIndex8Bit)
            {
                X = (ushort)(A & 0xFF);
                UpdateZN((ushort)(X & 0xFF), true);
            }
            else
            {
                X = A;
                UpdateZN(X, false);
            }
        }

        private void OpTYA(uint address)
        {
            if (IsMemory8Bit)
            {
                A = (ushort)((A & 0xFF00) | (Y & 0xFF));
                UpdateZN((ushort)(Y & 0xFF), true);
            }
            else
            {
                A = Y;
                UpdateZN(A, false);
            }
        }

        private void OpTAY(uint address)
        {
            if (IsIndex8Bit)
            {
                Y = (ushort)(A & 0xFF);
                UpdateZN((ushort)(Y & 0xFF), true);
            }
            else
            {
                Y = A;
                UpdateZN(Y, false);
            }
        }

        private void OpTCS(uint address)
        {
            if (E) S = (ushort)(0x0100 | (A & 0xFF));
            else S = A;
        }

        private void OpTCD(uint address)
        {
            D = A;
            UpdateZN(D, false);
        }

        private void OpXBA(uint address)
        {
            byte low = (byte)(A & 0xFF);
            byte high = (byte)(A >> 8);
            A = (ushort)((low << 8) | high);

            UpdateZN((ushort)(A & 0xFF), true);
        }
    }
}
