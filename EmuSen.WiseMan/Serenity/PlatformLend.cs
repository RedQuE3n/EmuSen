using System;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;

namespace EmuSen.WiseMan.Serenity
{
    // A C# machine's lend of the presenter's device through the platform library's C interface, as the C# implementation made it until 2026-11-04 - see EmuSen_Backend.md §14.12 and §40.8.
    internal sealed unsafe class PlatformLend : IDeviceLend
    {
        // emusen_serenity_lend_info, field for field.
        private struct LendInfo
        {
            public uint Size, ApiVersion;
            public ulong GetInstanceProcAddr, Instance, PhysicalDevice, Device;
            public uint QueueFamily, QueueIndex, HostQueueFamily, Reserved;
            public ulong Done;
        }

        private static readonly Lazy<bool> Resolved = new(Resolve);
        private static delegate* unmanaged[Cdecl]<nint, LendInfo*, nint> _new;
        private static delegate* unmanaged[Cdecl]<nint, ulong, ulong, int, int, int, ulong, ulong, int> _take;
        private static delegate* unmanaged[Cdecl]<nint, int> _settle;
        private static delegate* unmanaged[Cdecl]<nint, int> _free;

        private nint _handle;

        private PlatformLend(nint handle, CoreInterface.DeviceLend device) => (_handle, Device) = (handle, device);

        public CoreInterface.DeviceLend Device { get; }

        // The lend's emusen_serenity_lend, which the presenter borrows to read the pictures.
        public nint Handle => _handle;

        private static bool Resolve()
        {
            if (!PlatformLibrary.Available) return false;
            nint n = PlatformLibrary.Export("emusen_serenity_lend_new"), t = PlatformLibrary.Export("emusen_serenity_lend_take");
            nint s = PlatformLibrary.Export("emusen_serenity_lend_settle"), f = PlatformLibrary.Export("emusen_serenity_lend_free");
            if (n == 0 || t == 0 || s == 0 || f == 0) return false;
            _new = (delegate* unmanaged[Cdecl]<nint, LendInfo*, nint>)n;
            _take = (delegate* unmanaged[Cdecl]<nint, ulong, ulong, int, int, int, ulong, ulong, int>)t;
            _settle = (delegate* unmanaged[Cdecl]<nint, int>)s;
            _free = (delegate* unmanaged[Cdecl]<nint, int>)f;
            return true;
        }

        private static string Words() => PlatformLibrary.LastError();

        // A new lend of the device for one machine, or null with the reason logged once a session: no library, or a device that lends nothing.
        public static PlatformLend? Make(nint device, Action<string> refused)
        {
            if (device == 0) return null;
            if (!Resolved.Value)
            {
                refused($"no device is lent to the core: {PlatformLibrary.Report}");
                return null;
            }
            var info = new LendInfo { Size = (uint)sizeof(LendInfo) };
            nint made = _new(device, &info);
            if (made == 0)
            {
                refused($"no device is lent to the core: {Words()}");
                return null;
            }
            return new PlatformLend(made, new CoreInterface.DeviceLend
            {
                ApiVersion = info.ApiVersion, GetInstanceProcAddr = (nint)info.GetInstanceProcAddr, Instance = info.Instance, PhysicalDevice = info.PhysicalDevice,
                Device = info.Device, QueueFamily = info.QueueFamily, QueueIndex = info.QueueIndex, HostQueueFamily = info.HostQueueFamily, Done = info.Done,
            });
        }

        public bool Take(in CoreInterface.DeviceFrame f)
        {
            int status = _take(_handle, f.Buffer, f.Offset, f.Width, f.Height, f.Stride, f.Ready, f.Value);
            if (status < 0) throw new InvalidOperationException(Words());
            return status == 1;
        }

        public void Settle()
        {
            if (_settle(_handle) < 0) ErrorLog.Warning("picture", $"a lent device's copies could not be settled: {Words()}");
        }

        public void Dispose()
        {
            if (_handle == 0) return;
            _free(_handle);
            _handle = 0;
        }
    }
}
