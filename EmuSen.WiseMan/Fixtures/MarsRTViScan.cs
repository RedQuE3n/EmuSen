using System;
using EmuSen.Cores.Nintendo.MarsRT;

namespace EmuSen.WiseMan.Fixtures
{
    // MarsRT's scan-out behind its test export, fed the VI's registers and memory - see Mars_Native.md §5.4.
    public sealed unsafe class MarsRTViScan : IDisposable
    {
        // The raster's lines, a PAL field's - see Mars_Video.md §2.
        public const int RasterHeight = 625;

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
            if (registers.Length != 14 || held is { Length: not RasterHeight }) throw new ArgumentException("14 registers and 625 held lines.");
            fixed (uint* r = registers)
            fixed (int* h = held) SetExport(_handle, r, h, wasBlank ? 1u : 0u);
        }

        // True when a walk ran; the frame is composed either way.
        public bool Scan(byte[] rdram, byte[] hidden)
        {
            int result;
            fixed (byte* r = rdram)
            fixed (byte* h = hidden) result = RunExport(_handle, r, (nuint)rdram.Length, h, (nuint)hidden.Length);
            return result >= 0 ? result != 0 : throw new InvalidOperationException($"MarsRT refused the scan: status {result}.");
        }

        public (int[] Held, bool WasBlank) Held()
        {
            var held = new int[RasterHeight];
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
    }
}
