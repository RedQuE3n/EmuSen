using System;
using EmuSen.Serenity.Native;

namespace EmuSen.Serenity.Slang
{
    // An image a chain's picture is presented in, exported with its two semaphores for the compositor to import - see EmuSen_DevicePicture.md §4.2.
    public sealed unsafe class SlangShown : IDisposable
    {
        private readonly SerenityHandle _handle;

        public int Width { get; }
        public int Height { get; }
        public ulong MemoryBytes { get; }

        // The three descriptors, each the caller's to hand on or close; -1 once taken.
        public int MemoryFd { get; private set; }
        public int ReadyFd { get; private set; }
        public int FreeFd { get; private set; }

        internal nint Value => _handle.Value;

        private SlangShown(SerenityHandle handle, SerenityNative.ShownInfo info)
        {
            _handle = handle;
            (Width, Height, MemoryBytes, MemoryFd, ReadyFd, FreeFd) = (info.Width, info.Height, info.MemoryBytes, info.MemoryFd, info.ReadyFd, info.FreeFd);
        }

        // Null, with the words why, on a device that cannot export or the C#'s; linear lays the memory out row after row.
        public static SlangShown? TryCreate(SlangVulkan device, int width, int height, bool linear, out string? problem)
        {
            problem = null;
            if (!device.IsNative) { problem = "the C#'s device exports nothing"; return null; }
            var info = new SerenityNative.ShownInfo { Size = (uint)sizeof(SerenityNative.ShownInfo) };
            nint made = SerenityNative.ShownNew(device.NativeValue, width, height, linear ? 1u : 0u, &info);
            GC.KeepAlive(device);
            if (made == 0) { problem = SerenityNative.Words(); return null; }
            return new SlangShown(new SerenityHandle(made, SerenityNative.ShownFree), info);
        }

        // Each descriptor handed on once; the caller then owns it.
        public int TakeMemoryFd() { int fd = MemoryFd; MemoryFd = -1; return fd; }
        public int TakeReadyFd() { int fd = ReadyFd; ReadyFd = -1; return fd; }
        public int TakeFreeFd() { int fd = FreeFd; FreeFd = -1; return fd; }

        public void Dispose()
        {
            foreach (int fd in new[] { MemoryFd, ReadyFd, FreeFd }) if (fd >= 0) Close(fd);
            (MemoryFd, ReadyFd, FreeFd) = (-1, -1, -1);
            _handle.Dispose();
        }

        [System.Runtime.InteropServices.DllImport("libc", EntryPoint = "close")]
        private static extern int Close(int fd);
    }
}
