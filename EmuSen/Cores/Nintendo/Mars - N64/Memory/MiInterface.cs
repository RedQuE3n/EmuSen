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

        public MiInterrupt Pending;
        public MiInterrupt Mask;

        // Level-triggered: the line stays asserted for as long as a device is both raised and unmasked.
        public bool Asserted => (Pending & Mask) != MiInterrupt.None;

        public void Raise(MiInterrupt source) => Pending |= source;

        public void Clear(MiInterrupt source) => Pending &= ~source;

        public uint Read32(uint offset)
        {
            return (offset & 0x0C) switch
            {
                Interrupt => (uint)Pending,
                InterruptMask => (uint)Mask,
                _ => 0,
            };
        }

        public void Write32(uint offset, uint value)
        {
            if ((offset & 0x0C) != InterruptMask) return;

            // Two bits per device, one to clear the mask and one to set it - see Mars_Memory.md §8.1.
            for (int source = 0; source < 6; source++)
            {
                var flag = (MiInterrupt)(1 << source);

                if ((value & (1u << (source * 2))) != 0) Mask &= ~flag;
                if ((value & (1u << (source * 2 + 1))) != 0) Mask |= flag;
            }
        }
    }
}
