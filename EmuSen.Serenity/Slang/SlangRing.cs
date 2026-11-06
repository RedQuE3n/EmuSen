using System;
using System.Collections.Generic;
using EmuSen.Serenity.Native;

namespace EmuSen.Serenity.Slang
{
    // A ring made on the slang device and exported, lent to a hosted core whose device writes each picture into a slot, read by a chain - see EmuSen_LibretroHost.md §10f.
    // One frame's picture in a ring: its slot, the value the ring's ready reaches once the slot holds it, and its field structure - see EmuSen_Backend.md §33.
    public readonly record struct RingPicture(SlangRing Ring, int Slot, ulong Value, int Fields, int Field);

    public sealed unsafe class SlangRing : IDisposable
    {
        private readonly SerenityHandle? _handle;
        private readonly nint _borrowed;

        // One exported image, as the core ABI's emusen_ring_image holds it; the descriptor stays the ring's.
        public readonly record struct Image(uint MemoryType, ulong Handle, ulong AllocationSize, uint Width, uint Height, uint Format, uint Usage);

        public IReadOnlyList<Image> Images { get; }
        public long Ready { get; }
        public long Done { get; }
        public byte[] DeviceUuid { get; }
        public byte[] DriverUuid { get; }
        public SlangVulkan Owner { get; }

        internal nint Value => _handle?.Value ?? _borrowed;

        // A ring another component made and frees, here only read: the frontend API's runtime lends each machine one and the presenter borrows it - see EmuSen_Backend.md §33.
        public static SlangRing Borrow(SlangVulkan owner, nint ring) =>
            ring == 0 ? throw new System.ArgumentException("no ring to borrow", nameof(ring)) : new SlangRing(owner, ring);

        private SlangRing(SlangVulkan owner, nint borrowed)
        {
            (Owner, _borrowed, Images) = (owner, borrowed, System.Array.Empty<Image>());
            (DeviceUuid, DriverUuid) = (System.Array.Empty<byte>(), System.Array.Empty<byte>());
        }

        private SlangRing(SlangVulkan owner, SerenityHandle handle, SerenityNative.RingInfo info, Image[] images)
        {
            (Owner, _handle, Ready, Done, Images) = (owner, handle, info.Ready, info.Done, images);
            DeviceUuid = new ReadOnlySpan<byte>(info.DeviceUuid, 16).ToArray();
            DriverUuid = new ReadOnlySpan<byte>(info.DriverUuid, 16).ToArray();
        }

        // Null, with the words why, on a device that cannot export images and timeline semaphores.
        public static SlangRing? TryCreate(SlangVulkan device, int width, int height, int count, out string? problem)
        {
            problem = null;
            var info = new SerenityNative.RingInfo { Size = (uint)sizeof(SerenityNative.RingInfo) };
            nint made = SerenityNative.RingNew(device.NativeValue, (uint)width, (uint)height, (uint)count, &info);
            GC.KeepAlive(device);
            if (made == 0) { problem = SerenityNative.Words(); return null; }
            var handle = new SerenityHandle(made, SerenityNative.RingFree);
            var images = new Image[info.Count];
            for (uint i = 0; i < info.Count; i++)
            {
                var image = new SerenityNative.RingImageInfo { Size = (uint)sizeof(SerenityNative.RingImageInfo) };
                if (SerenityNative.RingImage(made, i, &image) < 0) { problem = SerenityNative.Words(); handle.Dispose(); return null; }
                images[i] = new Image(image.MemoryType, image.Handle, image.AllocationSize, image.Width, image.Height, image.Format, image.Usage);
            }
            return new SlangRing(device, handle, info, images);
        }

        // After the machine it was lent to, and after every chain that advanced from it has rendered.
        public void Dispose() => _handle?.Dispose();
    }
}
