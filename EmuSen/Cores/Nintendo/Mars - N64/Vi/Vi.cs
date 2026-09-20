using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.Cores.Nintendo.Mars.Vi
{
    // The video interface: fourteen registers and the raster they scan a frame buffer into - see Mars_Video.md §1.
    public sealed partial class Vi
    {
        public const uint Control = 0x00;
        public const uint Origin = 0x04;
        public const uint Width = 0x08;
        public const uint Interrupt = 0x0C;
        public const uint CurrentLine = 0x10;
        public const uint Burst = 0x14;
        public const uint VerticalSync = 0x18;
        public const uint HorizontalSync = 0x1C;
        public const uint Leap = 0x20;
        public const uint HorizontalStart = 0x24;
        public const uint VerticalStart = 0x28;
        public const uint VerticalBurst = 0x2C;
        public const uint ScaleX = 0x30;
        public const uint ScaleY = 0x34;

        public const int Registers = 14;

        // The widest and tallest signal the interface can raise, which is the raster every frame is placed in - see §2.1.
        public const int RasterWidth = 640;
        public const int RasterHeight = 625;

        private const int NtscHeight = 480;
        private const int PalHeight = 576;
        private const int NtscSyncLines = 525;

        [EmuSen.Common.SkipInState] private readonly MemoryBus _bus;
        private readonly uint[] _registers = new uint[Registers];

        // Four bytes a pixel, red first; kept between frames, because a line nothing rewrites stays on the screen - see §2.4.
        // Rebuilt by every scan, so a state need not carry its 1.4MB - see Mars_SaveStates.md §2.
        [EmuSen.Common.SkipInState] private readonly byte[] _raster = new byte[RasterWidth * RasterHeight * 4];
        private readonly int[] _held = new int[RasterHeight];

        // The raster at the multiple, when the last walk read the scaled memory; the multiple is what the picture's width is divided by - see Mars_Video.md §2.9.
        [EmuSen.Common.SkipInState] private byte[] _rasterScaled = System.Array.Empty<byte>();
        [EmuSen.Common.SkipInState] private int _rasterScale = 1;
        [EmuSen.Common.SkipInState] private int _outputScale = 1;

        public int OutputScale => _outputScale;

        public int OutputWidth => RasterWidth * _outputScale;

        private bool _wasBlank;

        public Vi(MemoryBus bus) => _bus = bus;

        public uint Read32(uint offset) => offset < Registers * 4 ? _registers[offset >> 2] : 0;

        public void Write32(uint offset, uint value)
        {
            // The sync registers set the clock both this and the audio interface run at, so both settle first - see Mars_Performance.md §9.
            _bus.Settle();

            // Writing the current line clears the interrupt; the counter it names is the interface's own - see Mars_VideoTiming.md §2.
            if ((offset & ~3u) == CurrentLine) _bus.Mi.Clear(MiInterrupt.VideoInterface);

            if (offset < Registers * 4) _registers[offset >> 2] = value;

            _bus.Reschedule();
        }

        public int FrameHeight => (IsPal ? PalHeight : NtscHeight) >> (Serrate ? 0 : 1);

        public ReadOnlySpan<byte> Frame => Raster(FrameHeight);

        // The raster's first lines by count, for a composition that took the count before the registers could move - see §2.7.
        public ReadOnlySpan<byte> Raster(int rows) =>
            _outputScale == 1 ? _raster.AsSpan(0, RasterWidth * rows * 4) : _rasterScaled.AsSpan(0, OutputWidth * rows * _outputScale * 4);

        private uint Register(uint offset) => _registers[offset >> 2];

        private int Type => (int)Register(Control) & 3;

        public bool Serrate => (Register(Control) & (1 << 6)) != 0;

        private int AntiAlias => (int)(Register(Control) >> 8) & 3;

        private bool GammaEnabled => (Register(Control) & (1 << 3)) != 0;

        private bool DivotEnabled => (Register(Control) & (1 << 4)) != 0;

        private bool DitherFilterEnabled => (Register(Control) & (1 << 16)) != 0;

        private bool IsPal => (int)(Register(VerticalSync) & 0x3FF) > NtscSyncLines + 25;
    }
}
