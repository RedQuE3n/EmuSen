using System;
using EmuSen.Cores.Native;

namespace EmuSen.Cores
{
    // A ring made on the presenter's device for one machine whose core claims DEVICE_RING, freed after the machine - see EmuSen_Backend.md §33.2.
    public interface IDeviceRing : IDisposable
    {
        // The platform library's emusen_serenity_ring, which the presenter borrows.
        nint Handle { get; }
        int Width { get; }
        int Height { get; }

        // Lent to the machine; false where its core refuses this ring.
        bool LendTo(CoreMachine machine);
    }
}
