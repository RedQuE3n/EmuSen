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

        // How many pixels each way of the multiple's raster are averaged into one of the picture's - see Mars_Video.md §2.10.
        [EmuSen.Common.SkipInState] private int _average = 1;
        [EmuSen.Common.SkipInState] private byte[] _rasterAveraged = System.Array.Empty<byte>();

        public int Average
        {
            get => _average;
            set => _average = System.Math.Clamp(value, 1, 4);
        }

        // Averaging waits for a raster it divides, so a change of either setting never shows a torn picture - see §2.10.
        private int Averaging => _outputScale > 1 && _average > 1 && _outputScale % _average == 0 ? _average : 1;

        public int OutputScale => _outputScale / Averaging;

        public int OutputWidth => RasterWidth * OutputScale;

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
        public ReadOnlySpan<byte> Raster(int rows)
        {
            if (_outputScale == 1) return _raster.AsSpan(0, RasterWidth * rows * 4);

            int wide = RasterWidth * _outputScale, average = Averaging;
            if (average == 1) return _rasterScaled.AsSpan(0, wide * rows * _outputScale * 4);

            int width = wide / average, lines = rows * _outputScale / average;
            if (_rasterAveraged.Length < width * lines * 4) _rasterAveraged = new byte[width * RasterHeight * (_outputScale / average) * 4];
            BoxAverage(_rasterScaled, wide, average, _rasterAveraged.AsSpan(0, width * lines * 4));
            return _rasterAveraged.AsSpan(0, width * lines * 4);
        }

        // Each pixel of the result is the rounded mean of a square of the source, channel by channel - see Mars_Video.md §2.10.
        public static void BoxAverage(ReadOnlySpan<byte> source, int sourceWidth, int side, Span<byte> into)
        {
            int width = sourceWidth / side, lines = into.Length / (width * 4), area = side * side, half = area / 2;
            Span<int> sums = width * 4 <= 16384 ? stackalloc int[width * 4] : new int[width * 4];

            for (int y = 0; y < lines; y++)
            {
                sums.Clear();
                for (int j = 0; j < side; j++)
                {
                    ReadOnlySpan<byte> row = source.Slice((y * side + j) * sourceWidth * 4, sourceWidth * 4);
                    for (int x = 0, at = 0; x < width; x++)
                    {
                        for (int i = 0; i < side; i++, at += 4)
                        {
                            sums[x * 4] += row[at];
                            sums[x * 4 + 1] += row[at + 1];
                            sums[x * 4 + 2] += row[at + 2];
                            sums[x * 4 + 3] += row[at + 3];
                        }
                    }
                }

                Span<byte> line = into.Slice(y * width * 4, width * 4);
                for (int c = 0; c < width * 4; c++) line[c] = (byte)((sums[c] + half) / area);
            }
        }

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
