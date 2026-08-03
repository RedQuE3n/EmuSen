namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // The opcodes NMOS decoding produces by accident. Real games use them - see Moon_CPU.md §6.
    public partial class Cpu
    {
        private void OpSLO(ushort address)
        {
            byte value = ShiftLeft(ReadModifyWriteFetch(address));
            Write(address, value);
            OpORA(value);
        }

        private void OpRLA(ushort address)
        {
            byte value = RotateLeft(ReadModifyWriteFetch(address));
            Write(address, value);
            OpAND(value);
        }

        private void OpSRE(ushort address)
        {
            byte value = ShiftRight(ReadModifyWriteFetch(address));
            Write(address, value);
            OpEOR(value);
        }

        private void OpRRA(ushort address)
        {
            byte value = RotateRight(ReadModifyWriteFetch(address));
            Write(address, value);
            OpADC(value);
        }

        // The decrement half sets no flags of its own; the compare that follows owns them all.
        private void OpDCP(ushort address)
        {
            byte value = (byte)(ReadModifyWriteFetch(address) - 1);
            Write(address, value);
            Compare(A, value);
        }

        private void OpISC(ushort address)
        {
            byte value = (byte)(ReadModifyWriteFetch(address) + 1);
            Write(address, value);
            OpSBC(value);
        }

        private void OpSAX(ushort address) => Write(address, (byte)(A & X));

        private void OpLAX(byte operand)
        {
            A = SetZeroNegative(operand);
            X = A;
        }

        // The AND's own N bit is what lands in C, which is what makes ANC useful for sign extension.
        private void OpANC(byte operand)
        {
            OpAND(operand);
            SetFlag(CpuFlags.C, GetFlag(CpuFlags.N));
        }

        private void OpALR(byte operand)
        {
            OpAND(operand);
            A = ShiftRight(A);
        }

        // C and V come off the rotated result's top bits rather than the adder - see Moon_CPU.md §6.2.
        private void OpARR(byte operand)
        {
            byte masked = (byte)(A & operand);
            byte result = (byte)((masked >> 1) | (GetFlag(CpuFlags.C) ? 0x80 : 0x00));

            SetZeroNegative(result);
            SetFlag(CpuFlags.C, (result & 0x40) != 0);
            SetFlag(CpuFlags.V, (((result >> 6) ^ (result >> 5)) & 0x01) != 0);

            A = result;
        }

        // Subtracts without borrow from A AND X, landing in X - the one undocumented compare.
        private void OpSBX(byte operand)
        {
            byte masked = (byte)(A & X);
            SetFlag(CpuFlags.C, masked >= operand);
            X = SetZeroNegative((byte)(masked - operand));
        }

        private void OpANE(byte operand) => A = SetZeroNegative((byte)((A | UnstableMagic) & X & operand));

        private void OpLXA(byte operand)
        {
            A = SetZeroNegative((byte)((A | UnstableMagic) & operand));
            X = A;
        }

        private void OpLAS(byte operand)
        {
            S = SetZeroNegative((byte)(operand & S));
            A = S;
            X = S;
        }

        // Crossing a page makes the value itself become the address's high byte - see Moon_CPU.md §6.3.
        private void UnstableStore(byte value, ushort effective, byte baseHigh, bool crossed)
        {
            byte stored = (byte)(value & (byte)(baseHigh + 1));
            ushort address = crossed ? (ushort)((stored << 8) | (effective & 0x00FF)) : effective;
            Write(address, stored);
        }

        private void OpSHA_AbsoluteY()
        {
            ushort effective = AddrAbsoluteIndexedUnstable(Y, out byte baseHigh, out bool crossed);
            UnstableStore((byte)(A & X), effective, baseHigh, crossed);
        }

        private void OpSHA_IndirectY()
        {
            ushort effective = AddrIndirectIndexedUnstable(out byte baseHigh, out bool crossed);
            UnstableStore((byte)(A & X), effective, baseHigh, crossed);
        }

        private void OpSHX()
        {
            ushort effective = AddrAbsoluteIndexedUnstable(Y, out byte baseHigh, out bool crossed);
            UnstableStore(X, effective, baseHigh, crossed);
        }

        private void OpSHY()
        {
            ushort effective = AddrAbsoluteIndexedUnstable(X, out byte baseHigh, out bool crossed);
            UnstableStore(Y, effective, baseHigh, crossed);
        }

        // TAS is SHA with a side effect: the stack pointer is clobbered before the store.
        private void OpTAS()
        {
            ushort effective = AddrAbsoluteIndexedUnstable(Y, out byte baseHigh, out bool crossed);
            S = (byte)(A & X);
            UnstableStore(S, effective, baseHigh, crossed);
        }
    }
}
