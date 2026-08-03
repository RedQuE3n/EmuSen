namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // Bitwise operations against the accumulator, plus BIT's split of N/V off the operand itself.
    public partial class Cpu
    {
        private void OpAND(byte operand) => A = SetZeroNegative((byte)(A & operand));

        private void OpORA(byte operand) => A = SetZeroNegative((byte)(A | operand));

        private void OpEOR(byte operand) => A = SetZeroNegative((byte)(A ^ operand));

        // Z comes from the masked accumulator, but N and V are copied straight off the operand's top bits.
        private void OpBIT(byte operand)
        {
            SetFlag(CpuFlags.Z, (A & operand) == 0);
            SetFlag(CpuFlags.N, (operand & 0x80) != 0);
            SetFlag(CpuFlags.V, (operand & 0x40) != 0);
        }
    }
}
