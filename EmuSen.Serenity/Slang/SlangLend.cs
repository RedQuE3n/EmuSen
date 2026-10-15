using System;
using EmuSen.Serenity.Native;

namespace EmuSen.Serenity.Slang
{
    // The slang device lent to one core machine, and the host's taking of the pictures the core hands on it - see EmuSen_DevicePicture.md §17.
    public sealed unsafe class SlangLend : IDisposable
    {
        private readonly SerenityHandle _handle;

        // What the core is given at create, as the core ABI's emusen_device_lend holds it.
        public uint ApiVersion { get; }
        public ulong GetInstanceProcAddr { get; }
        public ulong Instance { get; }
        public ulong PhysicalDevice { get; }
        public ulong Device { get; }
        public uint QueueFamily { get; }
        public uint QueueIndex { get; }
        public uint HostQueueFamily { get; }
        public ulong Done { get; }

        // The device lent, which a chain must be on to take the pictures.
        public SlangVulkan Owner { get; }

        internal nint Value => _handle.Value;

        private SlangLend(SlangVulkan owner, SerenityHandle handle, SerenityNative.LendInfo info)
        {
            (Owner, _handle) = (owner, handle);
            (ApiVersion, GetInstanceProcAddr, Instance, PhysicalDevice, Device) = (info.ApiVersion, info.GetInstanceProcAddr, info.Instance, info.PhysicalDevice, info.Device);
            (QueueFamily, QueueIndex, HostQueueFamily, Done) = (info.QueueFamily, info.QueueIndex, info.HostQueueFamily, info.Done);
        }

        // Null, with the words why, on the C#'s device or one with no timeline semaphores or too few queues.
        public static SlangLend? TryCreate(SlangVulkan device, out string? problem)
        {
            problem = null;
            if (!device.IsNative) { problem = "the C#'s device lends nothing"; return null; }
            var info = new SerenityNative.LendInfo { Size = (uint)sizeof(SerenityNative.LendInfo) };
            nint made = SerenityNative.LendNew(device.NativeValue, &info);
            GC.KeepAlive(device);
            if (made == 0) { problem = SerenityNative.Words(); return null; }
            return new SlangLend(device, new SerenityHandle(made, SerenityNative.LendFree), info);
        }

        // On the thread that runs the machine: the core's picture copied on the device once its ready reaches value, done then signalled; false for one not newer than the last.
        public bool Take(ulong buffer, ulong offset, int width, int height, int stride, ulong ready, ulong value)
        {
            int status = SerenityNative.LendTake(_handle.Value, buffer, offset, width, height, stride, ready, value);
            GC.KeepAlive(this);
            if (status < 0) throw new InvalidOperationException(SerenityNative.Words());
            return status == 1;
        }

        // Every copy waited for, so that nothing of the core's is read; before the machine is freed.
        public void Settle()
        {
            int status = SerenityNative.LendSettle(_handle.Value);
            GC.KeepAlive(this);
            if (status < 0) throw new InvalidOperationException(SerenityNative.Words());
        }

        public void Dispose() => _handle.Dispose();
    }
}
