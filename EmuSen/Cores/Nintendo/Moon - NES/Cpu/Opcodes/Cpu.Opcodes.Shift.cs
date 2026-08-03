namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // Shifts and rotates. The value helpers are shared with the read-modify-write undocumenteds.
    public partial class Cpu
    {
        private byte ShiftLeft(byte value)
        {
            SetFlag(CpuFlags.C, (value & 0x80) != 0);
            return SetZeroNegative((byte)(value << 1));
        }

        private byte ShiftRight(byte value)
        {
            SetFlag(CpuFlags.C, (value & 0x01) != 0);
            return SetZeroNegative((byte)(value >> 1));
        }

        private byte RotateLeft(byte value)
        {
            byte carryIn = (byte)(GetFlag(CpuFlags.C) ? 0x01 : 0x00);
            SetFlag(CpuFlags.C, (value & 0x80) != 0);
            return SetZeroNegative((byte)((value << 1) | carryIn));
        }

        private byte RotateRight(byte value)
        {
            byte carryIn = (byte)(GetFlag(CpuFlags.C) ? 0x80 : 0x00);
            SetFlag(CpuFlags.C, (value & 0x01) != 0);
            return SetZeroNegative((byte)((value >> 1) | carryIn));
        }

        private void OpASL(ushort address) => Write(address, ShiftLeft(ReadModifyWriteFetch(address)));

        private void OpLSR(ushort address) => Write(address, ShiftRight(ReadModifyWriteFetch(address)));

        private void OpROL(ushort address) => Write(address, RotateLeft(ReadModifyWriteFetch(address)));

        private void OpROR(ushort address) => Write(address, RotateRight(ReadModifyWriteFetch(address)));
    }
}
