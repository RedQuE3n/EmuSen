using System;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Moon.Memory;

namespace EmuSen.Cores.Nintendo.Moon.Video
{
    // The 2C02. Scanline-granularity, not per-dot - see Moon_PPU.md §1 for what that costs.
    public sealed partial class Ppu
    {
        public const int ScreenWidth = 256;
        public const int ScreenHeight = 240;
        public const int VisibleScanlines = 240;
        public const int VBlankScanline = 241;
        public const int PreRenderScanline = 261;
        public const int TotalScanlines = 262;
        public const int OamSize = 256;
        public const int SpritesPerLine = 8;

        [SkipInState] private readonly Cartridge _cart;

        // Two nametables of real RAM; four-screen boards bring their own second pair.
        public readonly byte[] Ciram = new byte[0x1000];
        public readonly byte[] PaletteRam = new byte[0x20];
        public readonly byte[] Oam = new byte[OamSize];

        public byte Control;
        public byte Mask;
        public byte Status;
        public byte OamAddress;

        // The loopy registers: v is the live address, t the staged one - see Moon_PPU.md §2.
        public ushort V;
        public ushort T;
        public byte FineX;
        public bool WriteToggle;

        // $2007 reads below the palette lag one access behind - see Moon_PPU.md §2.3.
        public byte ReadBuffer;

        public long FrameCount;

        // Set while the PPU wants the CPU's NMI line asserted; the CPU latches the edge itself.
        public bool NmiOutput => NmiEnabled && VBlankFlag;

        [SkipInState] public byte[] FrameRgba = new byte[ScreenWidth * ScreenHeight * 4];

        // Fast-forward drops the pixel writes only; sprite 0 still has to hit or games poll forever.
        [SkipInState] public bool SkipRendering;

        // Palette index per pixel of the line being drawn, so sprite priority has bg opacity to test.
        [SkipInState] private readonly byte[] _bgLine = new byte[ScreenWidth];

        [SkipInState] private readonly int[] _spriteIndices = new int[SpritesPerLine];
        [SkipInState] private int _spriteCount;

        public Ppu(Cartridge cart) => _cart = cart;

        public bool NmiEnabled => (Control & 0x80) != 0;
        public bool SpritesAre8x16 => (Control & 0x20) != 0;
        public int BackgroundPatternBase => (Control & 0x10) != 0 ? 0x1000 : 0x0000;
        public int SpritePatternBase => (Control & 0x08) != 0 ? 0x1000 : 0x0000;
        public int AddressIncrement => (Control & 0x04) != 0 ? 32 : 1;

        public bool Grayscale => (Mask & 0x01) != 0;
        public bool ShowBackgroundLeft => (Mask & 0x02) != 0;
        public bool ShowSpritesLeft => (Mask & 0x04) != 0;
        public bool ShowBackground => (Mask & 0x08) != 0;
        public bool ShowSprites => (Mask & 0x10) != 0;
        public bool RenderingEnabled => ShowBackground || ShowSprites;

        public bool VBlankFlag
        {
            get => (Status & 0x80) != 0;
            set => Status = (byte)(value ? Status | 0x80 : Status & ~0x80);
        }

        public bool Sprite0Hit
        {
            get => (Status & 0x40) != 0;
            set => Status = (byte)(value ? Status | 0x40 : Status & ~0x40);
        }

        public bool SpriteOverflow
        {
            get => (Status & 0x20) != 0;
            set => Status = (byte)(value ? Status | 0x20 : Status & ~0x20);
        }

        public void Reset()
        {
            Control = 0;
            Mask = 0;
            Status = 0;
            OamAddress = 0;
            V = 0;
            T = 0;
            FineX = 0;
            WriteToggle = false;
            ReadBuffer = 0;
            FrameCount = 0;
            Array.Clear(Ciram);
            Array.Clear(PaletteRam);
            Array.Clear(Oam);
            Array.Clear(FrameRgba);
        }

        // $2000-$2007, already mirrored down by the bus.
        public byte ReadRegister(int register)
        {
            switch (register & 0x07)
            {
                case 2:
                {
                    byte value = Status;
                    VBlankFlag = false;
                    WriteToggle = false;
                    return value;
                }

                case 4:
                    return Oam[OamAddress];

                case 7:
                {
                    ushort address = (ushort)(V & 0x3FFF);
                    byte value;

                    if (address >= 0x3F00)
                    {
                        // Palette reads are immediate, but the buffer still takes the byte mirrored beneath.
                        value = ReadPalette(address);
                        ReadBuffer = ReadCiram(address);
                    }
                    else
                    {
                        value = ReadBuffer;
                        ReadBuffer = ReadVram(address);
                    }

                    V = (ushort)((V + AddressIncrement) & 0x7FFF);
                    return value;
                }

                default:
                    // The open-bus latch is not modelled; unreadable registers read back as zero.
                    return 0;
            }
        }

        public void WriteRegister(int register, byte value)
        {
            switch (register & 0x07)
            {
                case 0:
                    Control = value;
                    T = (ushort)((T & 0xF3FF) | ((value & 0x03) << 10));
                    break;

                case 1:
                    Mask = value;
                    break;

                case 3:
                    OamAddress = value;
                    break;

                case 4:
                    Oam[OamAddress] = value;
                    OamAddress++;
                    break;

                case 5:
                    if (!WriteToggle)
                    {
                        FineX = (byte)(value & 0x07);
                        T = (ushort)((T & 0xFFE0) | (value >> 3));
                    }
                    else
                    {
                        T = (ushort)((T & 0x8FFF) | ((value & 0x07) << 12));
                        T = (ushort)((T & 0xFC1F) | ((value & 0xF8) << 2));
                    }
                    WriteToggle = !WriteToggle;
                    break;

                case 6:
                    if (!WriteToggle)
                    {
                        T = (ushort)((T & 0x00FF) | ((value & 0x3F) << 8));
                    }
                    else
                    {
                        T = (ushort)((T & 0xFF00) | value);
                        V = T;
                    }
                    WriteToggle = !WriteToggle;
                    break;

                case 7:
                    WriteVram((ushort)(V & 0x3FFF), value);
                    V = (ushort)((V + AddressIncrement) & 0x7FFF);
                    break;
            }
        }

        public byte ReadVram(ushort address)
        {
            address &= 0x3FFF;

            if (address < 0x2000) return _cart.Mapper.ReadChr(address);
            if (address < 0x3F00) return ReadCiram(address);
            return ReadPalette(address);
        }

        public void WriteVram(ushort address, byte value)
        {
            address &= 0x3FFF;

            if (address < 0x2000)
            {
                _cart.Mapper.WriteChr(address, value);
            }
            else if (address < 0x3F00)
            {
                Ciram[NametableOffset(address)] = value;
            }
            else
            {
                PaletteRam[PaletteOffset(address)] = value;
            }
        }

        private byte ReadCiram(ushort address) => Ciram[NametableOffset(address)];

        private byte ReadPalette(ushort address)
        {
            byte value = PaletteRam[PaletteOffset(address)];
            return Grayscale ? (byte)(value & 0x30) : value;
        }

        // $3000-$3EFF mirrors $2000-$2EFF before the board's own mirroring applies.
        public int NametableOffset(ushort address)
        {
            int index = (address - 0x2000) & 0x0FFF;
            int table = index / 0x0400;
            int offset = index % 0x0400;

            int page = _cart.Mapper.Mirroring switch
            {
                Mirroring.Horizontal => table >> 1,
                Mirroring.Vertical => table & 1,
                Mirroring.SingleScreenLower => 0,
                Mirroring.SingleScreenUpper => 1,
                _ => table,
            };

            return (page * 0x0400) + offset;
        }

        // The four sprite backdrop entries are holes that mirror the background's - see Moon_PPU.md §2.4.
        public static int PaletteOffset(ushort address)
        {
            int index = address & 0x1F;
            if ((index & 0x13) == 0x10) index &= 0x0F;
            return index;
        }
    }
}
