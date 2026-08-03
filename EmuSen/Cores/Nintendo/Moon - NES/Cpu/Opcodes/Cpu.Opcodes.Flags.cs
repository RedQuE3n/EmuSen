namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // The flag setters. D still latches even though the 2A03's adder ignores it - see Moon_CPU.md §4.2.
    public partial class Cpu
    {
        private void SetFlagOpcode(CpuFlags flag, bool value)
        {
            ConsumeImplied();
            SetFlag(flag, value);
        }

        // CLI/SEI write I after the interrupt poll, so either takes effect one instruction late.
        private void SetInterruptDisable(bool value)
        {
            ConsumeImplied();
            _delayedI = value;
            _hasDelayedI = true;
        }
    }
}
