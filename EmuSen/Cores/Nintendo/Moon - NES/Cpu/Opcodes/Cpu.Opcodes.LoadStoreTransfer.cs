namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // Loads, stores and register-to-register transfers.
    public partial class Cpu
    {
        private void OpLDA(byte operand) => A = SetZeroNegative(operand);

        private void OpLDX(byte operand) => X = SetZeroNegative(operand);

        private void OpLDY(byte operand) => Y = SetZeroNegative(operand);

        // TXS is the one transfer that sets no flags, because S is not a data register.
        private void OpTXS()
        {
            ConsumeImplied();
            S = X;
        }
    }
}
