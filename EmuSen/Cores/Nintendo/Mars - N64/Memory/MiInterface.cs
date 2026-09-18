using System;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // Which device is asking for attention - see Mars_Memory.md §8.
    [Flags]
    public enum MiInterrupt
    {
        None = 0,
        SignalProcessor = 1 << 0,
        SerialInterface = 1 << 1,
        AudioInterface = 1 << 2,
        VideoInterface = 1 << 3,
        PeripheralInterface = 1 << 4,
        DisplayProcessor = 1 << 5,
    }

    // The one wire between six devices and the CPU's single interrupt line - see Mars_Memory.md §8.
    public sealed class MiInterface
    {
        public const uint Mode = 0x00;
        public const uint Version = 0x04;
        public const uint Interrupt = 0x08;
        public const uint InterruptMask = 0x0C;

        // The mode register's write side: a length, then a clear and a set bit for each flag - see Mars_Memory.md §8.3.
        public const uint ModeRepeatLength = 0x7F;
        public const uint ModeClearRepeat = 0x080;
        public const uint ModeSetRepeat = 0x100;
        public const uint ModeClearEbus = 0x200;
        public const uint ModeSetEbus = 0x400;
        public const uint ModeClearUpper = 0x1000;
        public const uint ModeSetUpper = 0x2000;

        // The display processor's interrupt has no clear of its own - see Mars_Rdp.md §6.
        public const uint ModeClearDisplayProcessor = 0x800;

        // The flags as the mode register reads them back, beside the length.
        public const uint ModeRepeating = 0x080;
        public const uint ModeEbus = 0x100;
        public const uint ModeUpper = 0x200;

        // The RCP's revision, which both references and the FPGA core agree on - see Mars_Memory.md §8.3.
        public const uint RcpVersion = 0x0202_0102;

        public MiInterrupt Pending;
        public MiInterrupt Mask;

        // The next store to RDRAM is repeated across RepeatLength bytes, then this clears - see Mars_Memory.md §8.4.
        public bool Repeating;
        public uint RepeatCount;
        public bool Ebus;
        public bool Upper;

        public int RepeatLength => (int)RepeatCount + 1;

        // Level-triggered: the line stays asserted for as long as a device is both raised and unmasked.
        public bool Asserted => (Pending & Mask) != MiInterrupt.None;

        public void Raise(MiInterrupt source) => Pending |= source;

        public void Clear(MiInterrupt source) => Pending &= ~source;

        public uint Read32(uint offset)
        {
            return (offset & 0x0C) switch
            {
                Mode => RepeatCount | (Repeating ? ModeRepeating : 0) | (Ebus ? ModeEbus : 0) | (Upper ? ModeUpper : 0),
                Version => RcpVersion,
                Interrupt => (uint)Pending,
                InterruptMask => (uint)Mask,
                _ => 0,
            };
        }

        public void Write32(uint offset, uint value)
        {
            if ((offset & 0x0C) == Mode)
            {
                WriteMode(value);
                return;
            }

            if ((offset & 0x0C) != InterruptMask) return;

            // Two bits per device, one to clear the mask and one to set it - see Mars_Memory.md §8.1.
            for (int source = 0; source < 6; source++)
            {
                var flag = (MiInterrupt)(1 << source);

                if ((value & (1u << (source * 2))) != 0) Mask &= ~flag;
                if ((value & (1u << (source * 2 + 1))) != 0) Mask |= flag;
            }
        }

        // A set bit wins over its clear when a write carries both, as in the FPGA core - see Mars_Memory.md §8.3.
        private void WriteMode(uint value)
        {
            RepeatCount = value & ModeRepeatLength;

            if ((value & ModeClearRepeat) != 0) Repeating = false;
            if ((value & ModeSetRepeat) != 0) Repeating = true;
            if ((value & ModeClearEbus) != 0) Ebus = false;
            if ((value & ModeSetEbus) != 0) Ebus = true;
            if ((value & ModeClearUpper) != 0) Upper = false;
            if ((value & ModeSetUpper) != 0) Upper = true;

            if ((value & ModeClearDisplayProcessor) != 0) Clear(MiInterrupt.DisplayProcessor);
        }
    }
}
