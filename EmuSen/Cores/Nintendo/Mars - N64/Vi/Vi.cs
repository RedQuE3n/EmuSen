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

        private readonly MarsBus _bus;
        private readonly uint[] _registers = new uint[Registers];

        // Four bytes a pixel, red first; kept between frames, because a line nothing rewrites stays on the screen - see §2.4.
        private readonly byte[] _raster = new byte[RasterWidth * RasterHeight * 4];
        private readonly int[] _held = new int[RasterHeight];

        private bool _wasBlank;

        public Vi(MarsBus bus) => _bus = bus;

        public uint Read32(uint offset) => offset < Registers * 4 ? _registers[offset >> 2] : 0;

        public void Write32(uint offset, uint value)
        {
            if (offset < Registers * 4) _registers[offset >> 2] = value;
        }

        public int FrameHeight => (IsPal ? PalHeight : NtscHeight) >> (Serrate ? 0 : 1);

        public ReadOnlySpan<byte> Frame => _raster.AsSpan(0, RasterWidth * FrameHeight * 4);

        private uint Register(uint offset) => _registers[offset >> 2];

        private int Type => (int)Register(Control) & 3;

        private bool Serrate => (Register(Control) & (1 << 6)) != 0;

        private int AntiAlias => (int)(Register(Control) >> 8) & 3;

        private bool IsPal => (int)(Register(VerticalSync) & 0x3FF) > NtscSyncLines + 25;
    }
}
