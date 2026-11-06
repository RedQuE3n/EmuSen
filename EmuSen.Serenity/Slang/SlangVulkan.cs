using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using EmuSen.Serenity.Native;
using Silk.NET.Core.Native;
using Silk.NET.Vulkan;
using VkBuffer = Silk.NET.Vulkan.Buffer;
using VkImage = Silk.NET.Vulkan.Image;

namespace EmuSen.Serenity.Slang
{
    // One Vulkan device of the platform library's with a graphics queue, for the slang chains drawn on it - see EmuSen_Serenity.md §7.4.
    public sealed unsafe class SlangVulkan : IDisposable
    {
        public const string DeviceVariable = "EMUSEN_SLANG_GPU_DEVICE";

        // The library's device, the sink its layer's findings are handed to, and Silk's own entry for a test's call on it - see EmuSen_RustPlatform.md §17.2.
        private readonly SerenityHandle _native;
        private readonly Action<string>? _heard;
        private Vk? _vk;
        private long _probeEpoch;
        private bool _probing, _disposed;

        public string Name { get; }

        internal nint NativeValue => _native.Value;

        // The library's emusen_serenity_device, for the frontend API's open to lend each machine.
        public nint DeviceHandle => _native.Value;

        internal Vk Vk => _vk ??= Vk.GetApi();
        internal Device Device => new((nint)Raw(SerenityNative.RawDevice));
        internal Instance Instance => new((nint)Raw(SerenityNative.RawInstance));

        private SlangVulkan(SerenityHandle native, Action<string>? heard)
        {
            _native = native;
            _heard = heard;
            Name = SerenityNative.Text((buffer, capacity) => SerenityNative.DeviceName(native.Value, buffer, capacity)) ?? "unnamed";
        }

        // Null, with the reason, when no device with a graphics queue is there; the frontend then shows the picture unfiltered.
        public static SlangVulkan? TryCreate(out string report, string? nameContains = null) => TryCreate(out report, nameContains, null);

        // With a sink, the Khronos layer is loaded with synchronisation validation and every finding is handed to it, for a test - see EmuSen_Serenity.md §10.4.
        internal static SlangVulkan? TryCreate(out string report, string? nameContains, Action<string>? validation)
        {
            // The library attached before its first call; without it there is no device, said in words.
            if (!SerenityNative.Ready) { report = SerenityNative.Report; return null; }
            byte[]? name = SerenityNative.Utf8(nameContains ?? Environment.GetEnvironmentVariable(DeviceVariable));
            nint made;
            fixed (byte* n = name) made = SerenityNative.DeviceNew(name is null ? null : SerenityNative.Pin(n), (nuint)(name?.Length ?? 0), validation is null ? 0 : 1);
            if (made == 0)
            {
                report = SerenityNative.Words();
                return null;
            }
            var device = new SlangVulkan(new SerenityHandle(made, SerenityNative.DeviceFree), validation);
            report = device.Name;
            return device;
        }

        internal static void Check(Result result, string call)
        {
            if (result != Result.Success) throw new InvalidOperationException($"{call} returned {result}");
        }

        // The physical device's UUID, and whether its images and semaphores can be exported - see EmuSen_DevicePicture.md §4.4.
        public (byte[] Uuid, bool Exports) Identity()
        {
            var uuid = new byte[16];
            int exports;
            fixed (byte* u = uuid) exports = SerenityNative.DeviceIdentity(_native.Value, u);
            GC.KeepAlive(_native);
            return (uuid, exports == 1);
        }

        private ulong Raw(uint which)
        {
            ulong raw = SerenityNative.DeviceRaw(_native.Value, which);
            GC.KeepAlive(_native);
            return raw;
        }

        // What the library's device has waiting is handed on: the layer's findings to the sink, a probe's marks to the bench.
        internal void Heard()
        {
            if (_heard is not null) HearFindings();
            if (_probing && SlangProbe.Current is { } probe) HearMarks(probe);
        }

        private void HearFindings()
        {
            byte[] buffer = new byte[4096];
            long length;
            fixed (byte* b = buffer) length = SerenityNative.DeviceFindings(_native.Value, b, (nuint)buffer.Length);
            while (length > buffer.Length)
            {
                buffer = new byte[length];
                fixed (byte* b = buffer) length = SerenityNative.DeviceFindings(_native.Value, b, (nuint)buffer.Length);
            }
            GC.KeepAlive(_native);
            for (int at = 0; at < length;)
            {
                int end = Array.IndexOf(buffer, (byte)0, at);
                _heard!(System.Text.Encoding.UTF8.GetString(buffer, at, end - at));
                at = end + 1;
            }
        }

        // Turns the library's device's probe on or off; its phases are told on the stopwatch's clock from this moment - see EmuSen_Serenity.md §8.1.
        internal void SetProbe(bool on)
        {
            int status = SerenityNative.DeviceProbe(_native.Value, on ? 1 : 0);
            _probeEpoch = System.Diagnostics.Stopwatch.GetTimestamp();
            GC.KeepAlive(_native);
            if (status != 0) throw new InvalidOperationException(SerenityNative.Words());
            _probing = on;
        }

        private void HearMarks(ISlangProbe probe)
        {
            const int Room = 256;
            SerenityNative.Mark* marks = stackalloc SerenityNative.Mark[Room];
            long count = SerenityNative.DeviceMarks(_native.Value, marks, Room);
            SerenityNative.Mark[]? more = null;
            while (count > (more?.Length ?? Room))
            {
                more = new SerenityNative.Mark[count];
                fixed (SerenityNative.Mark* m = more) count = SerenityNative.DeviceMarks(_native.Value, m, (nuint)more.Length);
            }
            GC.KeepAlive(_native);
            for (int i = 0; i < count; i++)
            {
                SerenityNative.Mark mark = more is null ? marks[i] : more[i];
                if (mark.Kind == SerenityNative.MarkGpu) probe.Gpu(mark.Point, mark.Value / 1e6);
                else probe.Phase(mark.Point, _probeEpoch + (long)(mark.Value * System.Diagnostics.Stopwatch.Frequency / 1e9));
            }
        }

        public void Dispose()
        {
            if (_disposed) return;
            _disposed = true;
            SerenityNative.DeviceClose(_native.Value);
            if (_heard is not null) HearFindings();
            _native.Dispose();
            _vk?.Dispose();
        }
    }
}
