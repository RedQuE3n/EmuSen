using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mercury.Memory;

namespace EmuSen.Cores.Nintendo.Mercury.Video
{
    // The four states the LCD controller cycles through; the value is what STAT bits 1-0 report.
    public enum PpuMode
    {
        HBlank = 0,
        VBlank = 1,
        OamScan = 2,
        Drawing = 3,
    }

    // The DMG LCD controller. Scanline-granularity renderer on a T-cycle clock - see Mercury_Ppu.md §1.
    public sealed partial class Ppu
    {
        public const int ScreenWidth = MercuryCore.ScreenWidthPixels;
        public const int ScreenHeight = MercuryCore.ScreenHeightPixels;

        public const int TotalScanlines = 154;
        public const int CyclesPerScanline = 456;

        public const int OamScanCycles = 80;

        // The shortest mode 3 there is; SCX's fine scroll lengthens it - see Mercury_Ppu.md §2.2.
        public const int BaseDrawingCycles = 172;

        public const int SpritesPerLine = 10;
        public const int SpriteCount = 40;

        // How far into VRAM bank 1 sits; the map attributes and half the tiles live there on a CGB.
        public const int VramBankStride = MemoryBus.VramBankSize;

        [SkipInState] private readonly MemoryBus _bus;

        public byte Lcdc;
        public byte Scy;
        public byte Scx;
        public byte Ly;
        public byte Lyc;
        public byte Bgp;
        public byte Obp0;
        public byte Obp1;
        public byte Wy;
        public byte Wx;

        // Only the five source-enable bits are stored; mode and LY=LYC are recomputed on read.
        public byte StatEnables;

        public PpuMode Mode;

        // T-cycles into the current scanline.
        public int Dot;

        // Latched when mode 3 begins, because SCX may change again before the line ends.
        public int DrawingEnd;

        // The window's own line counter: it advances only on lines the window drew - see Mercury_Ppu.md §4.2.
        public int WindowLine;

        // WY is compared against LY every line, and the match latches for the rest of the frame.
        public bool WindowTriggered;

        // STAT's interrupt is a level on a shared line, so only its rising edge requests - see Mercury_Ppu.md §3.2.
        public bool StatLine;

        public long FrameCount;

        // Set when LY wraps to 0; the core clears it and presents - see Mercury_Ppu.md §1.1.
        public bool FrameComplete;

        [SkipInState] public byte[] FrameRgba = new byte[ScreenWidth * ScreenHeight * 4];

        [SkipInState] public bool SkipRendering;

        // Pre-palette background colour per pixel of the current line, which is what sprite priority tests.
        [SkipInState] private readonly byte[] _bgColorIndex = new byte[ScreenWidth];

        // The CGB map attribute's own priority bit, kept per pixel because it is per tile - see Mercury_Cgb.md §3.
        [SkipInState] private readonly bool[] _bgPriority = new bool[ScreenWidth];

        // Pixels a sprite has already claimed this line; a later sprite never overwrites one - see Mercury_Ppu.md §5.2.
        [SkipInState] private readonly bool[] _spriteClaimed = new bool[ScreenWidth];

        [SkipInState] private readonly int[] _spriteIndices = new int[SpritesPerLine];
        [SkipInState] private int _spriteCount;

        public Ppu(MemoryBus bus) => _bus = bus;

        public bool LcdEnabled => (Lcdc & 0x80) != 0;
        public int WindowTileMapBase => (Lcdc & 0x40) != 0 ? 0x1C00 : 0x1800;
        public bool WindowEnabled => (Lcdc & 0x20) != 0;
        public bool TileDataIsUnsigned => (Lcdc & 0x10) != 0;
        public int BgTileMapBase => (Lcdc & 0x08) != 0 ? 0x1C00 : 0x1800;
        public int SpriteHeight => (Lcdc & 0x04) != 0 ? 16 : 8;
        public bool SpritesEnabled => (Lcdc & 0x02) != 0;
        public bool BgEnabled => (Lcdc & 0x01) != 0;

        public bool LycMatches => Ly == Lyc;

        // Bit 7 is unwired and reads back set; bits 2-0 are the hardware's own state, not stored bits.
        public byte ReadStat() =>
            (byte)(0x80 | StatEnables | (LycMatches ? 0x04 : 0x00) | (int)Mode);

        public void WriteStat(byte data)
        {
            StatEnables = (byte)(data & 0x78);
            UpdateStatLine();
        }

        public void WriteLyc(byte data)
        {
            Lyc = data;
            UpdateStatLine();
        }

        public void WriteLcdc(byte data)
        {
            bool wasEnabled = LcdEnabled;
            Lcdc = data;

            if (wasEnabled && !LcdEnabled) DisableLcd();
            else if (!wasEnabled && LcdEnabled) EnableLcd();
        }

        public void Reset()
        {
            // What the DMG boot ROM leaves behind, since Mercury starts past it - see Mercury_Cpu.md §5.
            Lcdc = 0x91;
            StatEnables = 0x00;
            Scy = 0;
            Scx = 0;
            Ly = 0;
            Lyc = 0;
            Bgp = 0xFC;
            Obp0 = 0xFF;
            Obp1 = 0xFF;
            Wy = 0;
            Wx = 0;

            Mode = PpuMode.OamScan;
            Dot = 0;
            DrawingEnd = OamScanCycles + BaseDrawingCycles;
            WindowLine = 0;
            WindowTriggered = false;
            StatLine = false;
            FrameCount = 0;
            FrameComplete = false;

            ResetCgbPalettes();
            ClearScreen();
        }
    }
}
