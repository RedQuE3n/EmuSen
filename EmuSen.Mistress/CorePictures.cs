using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Serenity;
using EmuSen.Serenity.Slang;

namespace EmuSen.Mistress
{
    // Serenity's lent device in the form the cores take: the core ABI's emusen_device_lend and the taking of the pictures on it - see EmuSen_DevicePicture.md §17.
    public sealed class SlangDeviceLend(SlangLend slang) : IDeviceLend
    {
        public SlangLend Slang { get; } = slang;

        public CoreInterface.DeviceLend Device => new()
        {
            ApiVersion = Slang.ApiVersion, GetInstanceProcAddr = (nint)Slang.GetInstanceProcAddr, Instance = Slang.Instance, PhysicalDevice = Slang.PhysicalDevice,
            Device = Slang.Device, QueueFamily = Slang.QueueFamily, QueueIndex = Slang.QueueIndex, HostQueueFamily = Slang.HostQueueFamily, Done = Slang.Done,
        };

        public bool Take(in CoreInterface.DeviceFrame f) => Slang.Take(f.Buffer, f.Offset, f.Width, f.Height, f.Stride, f.Ready, f.Value);

        public void Settle() => Slang.Settle();

        public void Dispose() => Slang.Dispose();

        // Null for nothing lent.
        public static SlangDeviceLend? Of(SlangLend? slang) => slang is null ? null : new(slang);
    }

    // A core's picture before its palette in the form the presenter takes - see EmuSen_CoreAPI.md §6.30.
    public static class CorePictures
    {
        public static RawPicture? ForPresenter(CoreRawPicture? raw) =>
            raw is { } r ? new RawPicture(r.Encoding, r.Width, r.Height, r.Element, r.Stride, r.Values) : null;
    }
}
