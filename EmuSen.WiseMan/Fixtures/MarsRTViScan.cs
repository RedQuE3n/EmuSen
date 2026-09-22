using System;
using System.Reflection;
using EmuSen.Cores.Nintendo.Mars.Native;
using VideoInterface = EmuSen.Cores.Nintendo.Mars.Vi.Vi;

namespace EmuSen.WiseMan.Fixtures
{
    // MarsRT's scan-out behind its test export, fed a C# VI's registers and memory - see Mars_Native.md §5.4.
    public sealed unsafe class MarsRTViScan : IDisposable
    {
        private static readonly delegate* unmanaged<nint> NewExport = (delegate* unmanaged<nint>)MarsNative.Export("mars_vi_scan_new");
        private static readonly delegate* unmanaged<nint, void> FreeExport = (delegate* unmanaged<nint, void>)MarsNative.Export("mars_vi_scan_free");
        private static readonly delegate* unmanaged<nint, uint*, int*, uint, int> SetExport = (delegate* unmanaged<nint, uint*, int*, uint, int>)MarsNative.Export("mars_vi_scan_set");
        private static readonly delegate* unmanaged<nint, int*, int> HeldExport = (delegate* unmanaged<nint, int*, int>)MarsNative.Export("mars_vi_scan_held");
        private static readonly delegate* unmanaged<nint, uint, void> RepeatExport = (delegate* unmanaged<nint, uint, void>)MarsNative.Export("mars_vi_scan_repeat_rows");
        private static readonly delegate* unmanaged<nint, byte*, nuint, byte*, nuint, int> RunExport = (delegate* unmanaged<nint, byte*, nuint, byte*, nuint, int>)MarsNative.Export("mars_vi_scan_run");
        private static readonly delegate* unmanaged<nint, uint*, byte*, nuint, long> FrameExport = (delegate* unmanaged<nint, uint*, byte*, nuint, long>)MarsNative.Export("mars_vi_scan_frame");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> RasterExport = (delegate* unmanaged<nint, byte*, nuint, long>)MarsNative.Export("mars_vi_scan_raster");

        private nint _handle;

        public static bool Available => NewExport != null;

        public MarsRTViScan()
        {
            if (!Available) throw new InvalidOperationException($"MarsRT is not in use: {MarsNative.Report}");
            _handle = NewExport();
        }

        public bool RepeatRows
        {
            set => RepeatExport(_handle, value ? 1u : 0u);
        }

        // The registers alone, or with the held lines and blank flag a scan reads and writes.
        public void Set(uint[] registers, int[]? held = null, bool wasBlank = false)
        {
            if (registers.Length != 14 || held is { Length: not VideoInterface.RasterHeight }) throw new ArgumentException("14 registers and 625 held lines.");
            fixed (uint* r = registers)
            fixed (int* h = held) SetExport(_handle, r, h, wasBlank ? 1u : 0u);
        }

        // True when a walk ran, as the C# Scan() returns; the frame is composed either way.
        public bool Scan(byte[] rdram, byte[] hidden)
        {
            int result;
            fixed (byte* r = rdram)
            fixed (byte* h = hidden) result = RunExport(_handle, r, (nuint)rdram.Length, h, (nuint)hidden.Length);
            return result >= 0 ? result != 0 : throw new InvalidOperationException($"MarsRT refused the scan: status {result}.");
        }

        public (int[] Held, bool WasBlank) Held()
        {
            var held = new int[VideoInterface.RasterHeight];
            int blank;
            fixed (int* h = held) blank = HeldExport(_handle, h);
            return (held, blank != 0);
        }

        public byte[] Frame(out int width, out int height, out int rowRepeat)
        {
            uint* shape = stackalloc uint[3];
            long length = FrameExport(_handle, shape, null, 0);
            var frame = new byte[length];
            fixed (byte* f = frame) FrameExport(_handle, shape, f, (nuint)frame.Length);
            (width, height, rowRepeat) = ((int)shape[0], (int)shape[1], (int)shape[2]);
            return frame;
        }

        // The whole raster with its coverage bytes, 640 by 625.
        public byte[] Raster()
        {
            long length = RasterExport(_handle, null, 0);
            var raster = new byte[length];
            fixed (byte* r = raster) RasterExport(_handle, r, (nuint)raster.Length);
            return raster;
        }

        public void Dispose()
        {
            if (_handle != 0) FreeExport(_handle);
            _handle = 0;
            GC.SuppressFinalize(this);
        }

        ~MarsRTViScan()
        {
            if (_handle != 0) FreeExport(_handle);
        }

        // The C# VI's private scan state, read and written the way the serializer reaches it.
        public static uint[] RegistersOf(VideoInterface vi) => (uint[])Field("_registers").GetValue(vi)!;

        public static int[] HeldLinesOf(VideoInterface vi) => (int[])Field("_held").GetValue(vi)!;

        public static bool WasBlankOf(VideoInterface vi) => (bool)Field("_wasBlank").GetValue(vi)!;

        public static void SetWasBlank(VideoInterface vi, bool value) => Field("_wasBlank").SetValue(vi, value);

        public static byte[] RasterOf(VideoInterface vi) => (byte[])Field("_raster").GetValue(vi)!;

        private static FieldInfo Field(string name) =>
            typeof(VideoInterface).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic) ?? throw new MissingFieldException(nameof(VideoInterface), name);
    }
}
