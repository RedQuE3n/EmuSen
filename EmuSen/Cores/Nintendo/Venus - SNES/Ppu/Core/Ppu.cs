using System;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus.Video
{
    public struct PpuRegister
    {
        public string Name;
        public Action<ushort, byte> Write;
        public Func<ushort, byte> Read;
    }

    public partial class Ppu
    {
        // --- Core Memory ---
        public byte[] Vram = new byte[64 * 1024];   // 32K words
        public byte[] Cgram = new byte[512];        // 256 palette entries (15-bit color)
        public byte[] Oam = new byte[544];          // 512-byte main table + 32-byte high table

        // --- Decoded 32-bit ARGB Palette ---
        public uint[] Palette = new uint[256];

        // --- Register Handlers ---
        [EmuSen.Common.SkipInState] private PpuRegister[] _registers = null!;

        // --- Register State ---
        public byte Inidisp;                 // $2100 - brightness + force blank
        public byte Obsel;                   // $2101 - sprite size / tile base
        public byte Bgmode;                  // $2105
        public byte FixedColorR;
        public byte FixedColorG;
        public byte FixedColorB;
        public byte Cgwsel;                  // $2130 - Color math control A (window/screen selection)
        public byte Cgadsub;                 // $2131 - Color math control B (per-layer enable, add/sub, half)
        public byte W12Sel;                  // $2123 - Window mask settings, BG1/BG2
        public byte W34Sel;                  // $2124 - Window mask settings, BG3/BG4
        public byte WObjSel;                 // $2125 - Window mask settings, OBJ/Color
        public byte Wh0;                     // $2126 - Window 1 left
        public byte Wh1;                     // $2127 - Window 1 right
        public byte Wh2;                     // $2128 - Window 2 left
        public byte Wh3;                     // $2129 - Window 2 right
        public byte WBgLog;                  // $212A - Window BG mask logic (OR/AND/XOR/XNOR per BG)
        public byte WObjLog;                 // $212B - Window OBJ/Color mask logic
        public byte Tmw;                     // $212E - Window mask enable, main screen
        public byte Tsw;                     // $212F - Window mask enable, sub screen
        public byte[] BgSc = new byte[4];    // $2107-$210A - BG tilemap base + size
        public byte Bg12Nba;                 // $210B - BG1/BG2 character data base
        public byte Bg34Nba;                 // $210C - BG3/BG4 character data base
        // Scroll values are genuinely 10-bit (0-1023) on real hardware, not 8-bit -
        // needed once tilemaps are wider/taller than one 32x32 screen.
        public ushort[] BgScrollX = new ushort[4];
        public ushort[] BgScrollY = new ushort[4];
        public byte Tm;                      // $212C - main screen designation
        public byte Ts;                     // $212D - Sub Screen Designation
        public byte Mosaic;                  // $2106 - mosaic size + per-BG enable
        public byte Setini;                  // $2133 - screen mode select. See Venus_PPU.md §10.

        // --- Mode 7 ($211A-$2120) ---
        public byte M7Sel;                   // $211A - screen-over mode, H/V flip
        public short M7A, M7B, M7C, M7D;     // $211B-$211E - signed 8.8 fixed-point matrix
        public short M7X, M7Y;               // $211F/$2120 - signed 13-bit center/pivot point
        public short M7HOfs, M7VOfs;         // piggyback $210D/$210E - see Venus_PPU.md §2

        private byte _m7Latch;
        private byte _m7OfsLatch;
        private sbyte _m7bLastByte; // most recent raw byte written to $211C (M7B) - see WriteM7B

        // MPYL/M/H ($2134-$2136): 24-bit (well, up to 24 significant bits)

        public int Mpy => M7A * _m7bLastByte;

        public int CurrentScanline;
        public int CurrentLineCycles;

        private ushort _latchedH;
        private ushort _latchedV;
        private bool _ophctHigh;
        private bool _opvctHigh;

        public bool TimeOver;
        public bool RangeOver;

        private int _mosaicStartScanline;
        public int MosaicStartScanline => _mosaicStartScanline;


        public bool FieldParity;

        public void OnScanlineStart(int scanline)
        {
            if (scanline == 0)
            {
                _mosaicStartScanline = 0;
                FieldParity = !FieldParity;
            }
        }

        private byte _vmain;                 // $2115 - VRAM increment mode
        private ushort _vramAddr;            // $2116/$2117 - word address
        public ushort CurrentVramAddr => _vramAddr;

        private byte _cgadd;                 // $2121
        public byte CurrentCgAddr => _cgadd;
        private bool _cgLowByte = true;
        private byte _cgLatch;

        private ushort _oamAddr;             // $2102/$2103 (byte address into Oam)

        public bool PriorityRotationEnabled;
        public int FirstSpriteIndex => PriorityRotationEnabled ? (_oamAddr & 0xFE) >> 1 : 0;

        private byte _bgOfsLatch;

        private byte _bgHOfsLatch;

        private bool[] _vramTouched = new bool[65536];
        private bool[] _cgramTouched = new bool[512];
        public bool WasVramTouched(int byteAddr) => _vramTouched[byteAddr & 0xFFFF];
        public bool WasCgramTouched(int byteAddr) => _cgramTouched[byteAddr & 0x1FF];
        public bool FixedColorEverWritten { get; private set; }

        public Ppu()
        {
            BuildRegisterTable();
        }

        public void WriteRegister(uint offset, byte data)
        {
            uint index = offset - 0x2100;
            if (index < _registers.Length)
            {
                _registers[index].Write((ushort)offset, data);
            }
            // --- COLDATA: Fixed Color for Color Math ($2132) ---
            if (offset == 0x2132)
            {
                FixedColorEverWritten = true;
                // BGRxxxxx format: Top 3 bits dictate which color channel to update
                if ((data & 0x20) != 0) FixedColorR = (byte)(data & 0x1F);
                if ((data & 0x40) != 0) FixedColorG = (byte)(data & 0x1F);
                if ((data & 0x80) != 0) FixedColorB = (byte)(data & 0x1F);
                return;
            }
        }

        public byte ReadRegister(uint offset)
        {
            uint index = offset - 0x2100;
            if (index < _registers.Length)
            {
                return _registers[index].Read((ushort)offset);
            }
            return 0;
        }

        // --- Tile Decoding Utilities ---

        public uint[] DecodeTile4Bpp(ushort vramWordAddress, int paletteNumber)
        {
            uint[] pixels = new uint[64]; // An 8x8 tile has 64 pixels
            
            // VRAM address is in words, so multiply by 2 for the byte index
            int baseByteAddr = vramWordAddress * 2;
            
            // A 4bpp palette has 16 colors. paletteNumber shifts us to the correct block in CGRAM.
            int paletteBaseIndex = paletteNumber * 16; 

            for (int y = 0; y < 8; y++)
            {
                // Fetch the 4 bytes that make up this specific row of 8 pixels
                byte bp1 = Vram[(baseByteAddr + (y * 2)) & 0xFFFF];
                byte bp2 = Vram[(baseByteAddr + (y * 2) + 1) & 0xFFFF];
                
                byte bp3 = Vram[(baseByteAddr + 16 + (y * 2)) & 0xFFFF];
                byte bp4 = Vram[(baseByteAddr + 16 + (y * 2) + 1) & 0xFFFF];

                for (int x = 0; x < 8; x++)
                {
                    // Pixels are stored left-to-right, meaning bit 7 is the leftmost pixel (x=0)
                    int bit = 7 - x;

                    // Extract the specific bit from each plane and shift them into a 4-bit number
                    int colorIndex = (((bp1 >> bit) & 1) << 0) |
                                     (((bp2 >> bit) & 1) << 1) |
                                     (((bp3 >> bit) & 1) << 2) |
                                     (((bp4 >> bit) & 1) << 3);

                    // Index 0 in any SNES palette is transparent (we represent this with Alpha 0x00)
                    if (colorIndex == 0)
                    {
                        pixels[y * 8 + x] = 0x00000000;
                    }
                    else
                    {
                        // Map the 4-bit index to our decoded 32-bit CGRAM palette
                        pixels[y * 8 + x] = Palette[(paletteBaseIndex + colorIndex) & 0xFF];
                    }
                }
            }

            return pixels;
        }

        private int VramStep()
        {
            switch (_vmain & 0x03)
            {
                case 0: return 1;
                case 1: return 32;
                default: return 128;
            }
        }

    }
}
