namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // Twelve conditional traps, every one of them a full sixty-four bit comparison - see Mars_Cpu.md §15.
    public sealed partial class Cpu
    {
        private void ExecuteTrap(uint instruction)
        {
            ulong left = Read(Rs(instruction));
            ulong right = Read(Rt(instruction));

            TrapIf((instruction & 0x3F) switch
            {
                0x30 => (long)left >= (long)right,
                0x31 => left >= right,
                0x32 => (long)left < (long)right,
                0x33 => left < right,
                0x34 => left == right,
                _ => left != right,
            });
        }

        // The immediate sign-extends to sixty-four bits even where the comparison is unsigned - see §15.1.
        private void ExecuteTrapImmediate(uint instruction)
        {
            ulong left = Read(Rs(instruction));
            long right = SignedImmediate(instruction);

            TrapIf(Rt(instruction) switch
            {
                0x08 => (long)left >= right,
                0x09 => left >= (ulong)right,
                0x0A => (long)left < right,
                0x0B => left < (ulong)right,
                0x0C => left == (ulong)right,
                _ => left != (ulong)right,
            });
        }

        private void TrapIf(bool condition)
        {
            if (condition) throw Raise(ExceptionCode.Trap, CurrentPc);
        }
    }
}
