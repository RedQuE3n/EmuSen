namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // The eight conditional branches, which cost one cycle to take and another to leave the page.
    public partial class Cpu
    {
        private void Branch(bool taken)
        {
            sbyte offset = (sbyte)Read(PC++);
            if (!taken) return;

            Read(PC);
            ushort target = (ushort)(PC + offset);

            // The chip fixes the high byte a cycle late, reading the half-computed address first.
            if ((target & 0xFF00) != (PC & 0xFF00))
            {
                Read((ushort)((PC & 0xFF00) | (target & 0x00FF)));
            }

            PC = target;
        }
    }
}
