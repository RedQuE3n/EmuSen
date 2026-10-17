using System;
using EmuSen.Cores.Native;

namespace EmuSen.Cores
{
    // A device the frontend lends one core machine, the machine's from create to free: the core ABI's emusen_device_lend and the taking of the pictures on it - see EmuSen_DevicePicture.md §17.
    public interface IDeviceLend : IDisposable
    {
        // What the core is given at create; the machine sets its Size.
        CoreInterface.DeviceLend Device { get; }

        // On the thread that runs the machine: the frame's picture copied where it lies once its ready is reached, done then signalled; false for one not newer than the last.
        bool Take(in CoreInterface.DeviceFrame frame);

        // Every copy waited for, so that nothing of the core's is read; before the machine is freed.
        void Settle();
    }
}
