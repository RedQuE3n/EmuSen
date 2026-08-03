namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // Stack pushes and pulls. Only PHP/BRK ever put the B bit on the bus - see Moon_CPU.md §4.1.
    public partial class Cpu
    {
        private void OpPHA()
        {
            ConsumeImplied();
            Push(A);
        }

        private void OpPHP()
        {
            ConsumeImplied();
            Push((byte)(P | 0x30));
        }

        private void OpPLA()
        {
            ConsumeImplied();
            A = SetZeroNegative(PullWithDummy());
        }

        // I is held back to the poll, so an unmasking PLP cannot take the interrupt it just enabled.
        private void OpPLP()
        {
            ConsumeImplied();
            byte pulled = PullWithDummy();
            bool previousI = GetFlag(CpuFlags.I);

            _delayedI = (pulled & (byte)CpuFlags.I) != 0;
            _hasDelayedI = true;

            P = (byte)((pulled | (byte)CpuFlags.U) & ~(byte)CpuFlags.B);
            SetFlag(CpuFlags.I, previousI);
        }
    }
}
