namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // Add, subtract, compare and the memory increments. The 2A03 ignores the D flag - see Moon_CPU.md §4.2.
    public partial class Cpu
    {
        private void OpADC(byte operand)
        {
            int sum = A + operand + (GetFlag(CpuFlags.C) ? 1 : 0);
            byte result = (byte)sum;

            SetFlag(CpuFlags.C, sum > 0xFF);
            // Overflow means both inputs agreed on sign and the result disagreed with them.
            SetFlag(CpuFlags.V, ((A ^ result) & (operand ^ result) & 0x80) != 0);

            A = SetZeroNegative(result);
        }

        // Subtracting is adding the one's complement, since the borrow is carry-inverted.
        private void OpSBC(byte operand) => OpADC((byte)~operand);

        private void Compare(byte register, byte operand)
        {
            SetFlag(CpuFlags.C, register >= operand);
            SetZeroNegative((byte)(register - operand));
        }

        private void OpINC(ushort address)
        {
            byte value = ReadModifyWriteFetch(address);
            Write(address, SetZeroNegative((byte)(value + 1)));
        }

        private void OpDEC(ushort address)
        {
            byte value = ReadModifyWriteFetch(address);
            Write(address, SetZeroNegative((byte)(value - 1)));
        }
    }
}
