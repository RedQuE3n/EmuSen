using System;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Managed
{
    // A machine's ring on the presenter's device, made through the platform library's C interface - see EmuSen_Backend.md §33.2.
    internal sealed unsafe class PlatformRing : IDeviceRing
    {
        // emusen_serenity_ring_info and emusen_serenity_ring_image, field for field.
        private struct RingInfo
        {
            public uint Size, Count;
            public long Ready, Done;
            public fixed byte DeviceUuid[16];
            public fixed byte DriverUuid[16];
        }

        private struct RingImageInfo
        {
            public uint Size, MemoryType;
            public ulong Handle, AllocationSize;
            public uint Width, Height, Format, Usage;
        }

        private static readonly Lazy<bool> Resolved = new(Resolve);
        private static delegate* unmanaged[Cdecl]<nint, uint, uint, uint, RingInfo*, nint> _new;
        private static delegate* unmanaged[Cdecl]<nint, uint, RingImageInfo*, int> _image;
        private static delegate* unmanaged[Cdecl]<nint, int> _free;

        private nint _handle;
        private readonly RingInfo _info;
        private readonly CoreInterface.RingImage[] _images;

        private PlatformRing(nint handle, RingInfo info, CoreInterface.RingImage[] images, int width, int height) => (_handle, _info, _images, Width, Height) = (handle, info, images, width, height);

        public nint Handle => _handle;
        public int Width { get; }
        public int Height { get; }

        private static bool Resolve()
        {
            if (!PlatformLibrary.Available) return false;
            nint n = PlatformLibrary.Export("emusen_serenity_ring_new"), i = PlatformLibrary.Export("emusen_serenity_ring_image"), f = PlatformLibrary.Export("emusen_serenity_ring_free");
            if (n == 0 || i == 0 || f == 0) return false;
            _new = (delegate* unmanaged[Cdecl]<nint, uint, uint, uint, RingInfo*, nint>)n;
            _image = (delegate* unmanaged[Cdecl]<nint, uint, RingImageInfo*, int>)i;
            _free = (delegate* unmanaged[Cdecl]<nint, int>)f;
            return true;
        }

        private static string Words() => PlatformLibrary.LastError();

        // A ring of three images width by height on the device, or null with the reason given once a session.
        public static PlatformRing? Make(nint device, int width, int height, Action<string> refused)
        {
            if (!Resolved.Value) { refused($"no ring is lent to the core: {PlatformLibrary.Report}"); return null; }
            var info = new RingInfo { Size = (uint)sizeof(RingInfo) };
            nint made = _new(device, (uint)width, (uint)height, 3, &info);
            if (made == 0) { refused($"no ring is lent to the core: {Words()}"); return null; }
            var images = new CoreInterface.RingImage[info.Count];
            for (uint k = 0; k < info.Count; k++)
            {
                var image = new RingImageInfo { Size = (uint)sizeof(RingImageInfo) };
                if (_image(made, k, &image) < 0) { refused($"no ring is lent to the core: {Words()}"); _free(made); return null; }
                images[k] = new CoreInterface.RingImage { Size = (uint)sizeof(CoreInterface.RingImage), MemoryType = image.MemoryType, Handle = image.Handle, AllocationSize = image.AllocationSize, Width = image.Width, Height = image.Height, Format = image.Format, Usage = image.Usage };
            }
            return new PlatformRing(made, info, images, width, height);
        }

        public bool LendTo(CoreMachine machine)
        {
            fixed (CoreInterface.RingImage* first = _images)
            {
                var lend = new CoreInterface.Ring { Size = (uint)sizeof(CoreInterface.Ring), Count = (uint)_images.Length, Images = first, ImageSize = (nuint)sizeof(CoreInterface.RingImage), Ready = (ulong)_info.Ready, Done = (ulong)_info.Done };
                RingInfo info = _info;
                for (int i = 0; i < 16; i++) (lend.DeviceUuid[i], lend.DriverUuid[i]) = (info.DeviceUuid[i], info.DriverUuid[i]);
                return machine.RingLend(&lend);
            }
        }

        public void Dispose()
        {
            if (_handle == 0) return;
            _free(_handle);
            _handle = 0;
        }
    }
}
