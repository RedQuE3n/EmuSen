using System;

namespace EmuSen.Cores
{
    // A core whose picture may lie on the device the frontend lends it, read there by the chain instead of copied out - see EmuSen_DevicePicture.md §17.
    public interface IDevicePicture
    {
        // Set before the game is loaded: asked once for each machine made, null for none.
        Func<IDeviceLend?>? DeviceLender { get; set; }

        // The running machine's lend, and whether the last frame's picture was taken on it.
        IDeviceLend? Lend { get; }
        bool PictureOnDevice { get; }
    }
}
