using System;
using System.Collections.Generic;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;
using static EmuSen.Crystal.Systems.SystemDebugSpaces;

namespace EmuSen.Crystal.Systems.Nes
{
    // The NES's debugger views, decoded from the spaces any NES engine publishes - see Moon_Debug.md §4, EmuSen_CoreAPI.md §31.
    public sealed class NesDebugViews : ISystemDebugViews
    {
        public static readonly NesDebugViews Instance = new();

        public const int NmiVector = 0xFFFA, ResetVector = 0xFFFC, IrqVector = 0xFFFE;

        // The 64 NTSC colours as RGB triples, emphasis not applied; the debugger's table, not the picture's - see Moon_PPU.md §3.5.
        public static readonly byte[] MasterPalette =
        {
            84, 84, 84,     0, 30, 116,     8, 16, 144,    48, 0, 136,    68, 0, 100,    92, 0, 48,     84, 4, 0,      60, 24, 0,
            32, 42, 0,      8, 58, 0,       0, 64, 0,      0, 60, 0,      0, 50, 60,     0, 0, 0,       0, 0, 0,       0, 0, 0,
            152, 150, 152,  8, 76, 196,     48, 50, 236,   92, 30, 228,   136, 20, 176,  160, 20, 100,  152, 34, 32,   120, 60, 0,
            84, 90, 0,      40, 114, 0,     8, 124, 0,     0, 118, 40,    0, 102, 120,   0, 0, 0,       0, 0, 0,       0, 0, 0,
            236, 238, 236,  76, 154, 236,   120, 124, 236, 176, 98, 236,  228, 84, 236,  236, 88, 180,  236, 106, 100, 212, 136, 32,
            160, 170, 0,    116, 196, 0,    76, 208, 32,   56, 204, 108,  56, 180, 204,  60, 60, 60,    0, 0, 0,       0, 0, 0,
            236, 238, 236,  168, 204, 236,  188, 188, 236, 212, 178, 236, 236, 174, 236, 236, 174, 212, 236, 180, 176, 228, 196, 144,
            204, 210, 120,  180, 222, 120,  168, 226, 144, 152, 226, 180, 160, 214, 228, 160, 162, 160, 0, 0, 0,       0, 0, 0,
        };

        // $3F10, $3F14, $3F18 and $3F1C are the backdrop entries below them.
        public static int PaletteOffset(int entry)
        {
            int index = entry & 0x1F;
            if ((index & 0x13) == 0x10) index &= 0x0F;
            return index;
        }

        private static (byte R, byte G, byte B) Colour(IDebugMemorySpace palette, int entry)
        {
            int offset = (palette.Read(PaletteOffset(entry)) & 0x3F) * 3;
            return (MasterPalette[offset], MasterPalette[offset + 1], MasterPalette[offset + 2]);
        }

        public IReadOnlyList<DebugPaletteInfo> Palettes(IReadOnlyList<IDebugMemorySpace> spaces)
        {
            if (Find(spaces, NesSystem.Palette) is not { Size: > 0 } palette) return Array.Empty<DebugPaletteInfo>();
            var palettes = new List<DebugPaletteInfo>();
            for (int p = 0; p < 8; p++)
            {
                var colors = new List<(byte r, byte g, byte b)>();
                for (int c = 0; c < 4; c++) colors.Add(Colour(palette, p * 4 + c));
                palettes.Add(new DebugPaletteInfo(p, colors));
            }
            return palettes;
        }

        public (byte[] Rgba, int Width, int Height) PaletteSwatch(IReadOnlyList<IDebugMemorySpace> spaces)
        {
            if (Find(spaces, NesSystem.Palette) is not { Size: > 0 } palette) return (Array.Empty<byte>(), 0, 0);
            const int swatch = 16, columns = 16, rows = 2;
            int width = columns * swatch, height = rows * swatch;
            var rgba = new byte[width * height * 4];
            for (int entry = 0; entry < 32; entry++)
            {
                var (r, g, b) = Colour(palette, entry);
                int originX = entry % columns * swatch, originY = entry / columns * swatch;
                for (int y = 0; y < swatch; y++)
                    for (int x = 0; x < swatch; x++)
                    {
                        int offset = ((originY + y) * width + originX + x) * 4;
                        (rgba[offset], rgba[offset + 1], rgba[offset + 2], rgba[offset + 3]) = (r, g, b, 0xFF);
                    }
            }
            return (rgba, width, height);
        }

        // Grayscale by pixel value, since a tile has no palette of its own until a nametable picks one.
        public (byte[] Rgba, int Width, int Height) TileSheet(IReadOnlyList<IDebugMemorySpace> spaces)
        {
            if (Find(spaces, NesSystem.Chr) is not { Size: > 0 } chr) return (Array.Empty<byte>(), 0, 0);
            const int tilesPerRow = 16;
            int tileCount = chr.Size / 16;
            int rows = (tileCount + tilesPerRow - 1) / tilesPerRow;
            int width = tilesPerRow * 8, height = rows * 8;
            var rgba = new byte[width * height * 4];
            byte[] shades = { 0, 85, 170, 255 };
            for (int tile = 0; tile < tileCount; tile++)
            {
                int originX = tile % tilesPerRow * 8, originY = tile / tilesPerRow * 8;
                for (int row = 0; row < 8; row++)
                {
                    byte low = chr.Read(tile * 16 + row), high = chr.Read(tile * 16 + row + 8);
                    for (int column = 0; column < 8; column++)
                    {
                        int bit = 7 - column;
                        byte shade = shades[((low >> bit) & 1) | (((high >> bit) & 1) << 1)];
                        int offset = ((originY + row) * width + originX + column) * 4;
                        (rgba[offset], rgba[offset + 1], rgba[offset + 2], rgba[offset + 3]) = (shade, shade, shade, 0xFF);
                    }
                }
            }
            return (rgba, width, height);
        }

        // One byte per nametable cell; the attribute byte is separate and coarser - see Moon_Debug.md §4.
        public int TilemapEntryStride => 1;

        // Offset 0x3C0 and up in a nametable page is the attribute table, not tiles.
        public string DecodeTilemapEntry(IDebugMemorySpace space, int address)
        {
            int page = (address / 0x400) & 0x03, offset = address % 0x400;
            if (offset >= 0x3C0) return $"attr ${space.Read(address):X2}";
            byte tile = space.Read(address);
            int tileX = offset % 32, tileY = offset / 32;
            byte attribute = space.Read(page * 0x400 + 0x3C0 + tileY / 4 * 8 + tileX / 4);
            int quadrant = ((tileY & 0x02) << 1) | (tileX & 0x02);
            return $"${tile:X2} p{(attribute >> quadrant) & 0x03}";
        }

        // 16 bytes per tile: eight rows of the low bitplane, then eight of the high one.
        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp)
        {
            if (bpp != 2) throw new ArgumentException($"An NES tile is always 2bpp; got {bpp}.", nameof(bpp));
            var pixels = new byte[64];
            for (int row = 0; row < 8; row++)
            {
                byte low = space.Read(address + row), high = space.Read(address + row + 8);
                for (int column = 0; column < 8; column++)
                {
                    int bit = 7 - column;
                    pixels[row * 8 + column] = (byte)(((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1));
                }
            }
            return pixels;
        }

        // A CPU address's destination where the console fixes it, null for $8000 and up, which the board banks; no view reads it until the views carry it - see EmuSen_Backend.md §21.
        public PhysicalAddress? ResolvePhysical(int cpuAddress) => cpuAddress switch
        {
            < 0x2000 => new PhysicalAddress(NesSystem.Ram, cpuAddress & 0x07FF),
            < 0x4000 => new PhysicalAddress(NesSystem.PpuRegisters, cpuAddress & 0x07, false),
            < 0x4020 => new PhysicalAddress(NesSystem.ApuRegisters, cpuAddress - 0x4000, false),
            < 0x8000 => new PhysicalAddress(NesSystem.PrgRam, cpuAddress & 0x1FFF),
            _ => null,
        };

        public PhysicalAddress? ResolvePhysical(int address, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) => ResolvePhysical(address);

        // The units an NES engine lists, by the names the console's views read them under - see EmuSen_Backend.md §23.
        public const string Ppu = "PPU", Board = "Board", Cartridge = "Cartridge";

        private static DebugUnit? Unit(IReadOnlyList<DebugUnit> units, string name)
        {
            foreach (var u in units) if (string.Equals(u.Name, name, StringComparison.Ordinal)) return u;
            return null;
        }

        private static long Value(DebugUnit? unit, string register) => unit is not null && Register(unit.Registers, register) is { } v ? (long)v : 0;

        private static bool Flag(DebugUnit? unit, string register) => Value(unit, register) != 0;

        public int MaxSprites => 64;

        // The 64 OAM entries, 8x8 or 8x16 by PPUCTRL bit 5; one parked below the visible lines is the usual way to hide it.
        public IReadOnlyList<DebugSpriteInfo> Sprites(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            if (Find(spaces, NesSystem.Oam) is not { Size: >= 256 } oam || Unit(units, Ppu) is not { } ppu) return Array.Empty<DebugSpriteInfo>();
            int height = (Value(ppu, "PPUCTRL") & 0x20) != 0 ? 16 : 8;
            var sprites = new List<DebugSpriteInfo>();
            for (int i = 0; i < 64; i++)
            {
                int y = oam.Read(i * 4), x = oam.Read(i * 4 + 3);
                byte tile = oam.Read(i * 4 + 1), attributes = oam.Read(i * 4 + 2);
                if (y >= 240) continue;
                sprites.Add(new DebugSpriteInfo(i, x, y + 1, 8, height, tile, attributes & 0x03, (attributes & 0x20) != 0 ? 0 : 1, (attributes & 0x40) != 0, (attributes & 0x80) != 0));
            }
            return sprites;
        }

        // The 2A03's five channels from their units, the three display rules of Moon_Native.md §9.4: a pulse silenced below period 8 or past a sweep target of $7FF, an envelope's output its volume when constant and else its decay, the DMC active while bytes remain.
        public IReadOnlyList<DebugAudioChannelInfo> AudioChannels(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units, uint mutes)
        {
            if (Unit(units, "Pulse 1") is null) return Array.Empty<DebugAudioChannelInfo>();
            bool Muted(int index) => (mutes & (1u << index)) != 0;
            long Envelope(DebugUnit? u) => Flag(u, "ConstantVolume") ? Value(u, "Volume") : Value(u, "Decay");
            DebugAudioChannelInfo Pulse(int index, string name)
            {
                DebugUnit? u = Unit(units, name);
                long period = Value(u, "TimerPeriod"), change = period >> (int)Value(u, "SweepShift");
                long target = Flag(u, "SweepNegate") ? period - change - (Flag(u, "OnesComplement") ? 1 : 0) : period + change;
                bool silenced = period < 8 || target > 0x7FF;
                long length = Value(u, "LengthCounter"), volume = Envelope(u);
                return new(index, name, length > 0 && !silenced, Scale(volume, 15), Muted(index), $"Duty={Value(u, "Duty")} Period={period} Vol={volume} Len={length} Sweep={Flag(u, "SweepEnabled")}");
            }
            DebugUnit? triangle = Unit(units, "Triangle"), noise = Unit(units, "Noise"), dmc = Unit(units, "DMC");
            long triangleLength = Value(triangle, "LengthCounter"), trianglePeriod = Value(triangle, "TimerPeriod"), noiseLength = Value(noise, "LengthCounter");
            long noiseVolume = Envelope(noise), dmcOut = Value(dmc, "OutputLevel");
            return new DebugAudioChannelInfo[]
            {
                Pulse(0, "Pulse 1"),
                Pulse(1, "Pulse 2"),
                new(2, "Triangle", triangleLength > 0, triangleLength > 0 && trianglePeriod >= 2 ? 100 : 0, Muted(2), $"Period={trianglePeriod} Len={triangleLength} Halt={Flag(triangle, "ControlFlag")}"),
                new(3, "Noise", noiseLength > 0, Scale(noiseVolume, 15), Muted(3), $"PeriodIndex={Value(noise, "PeriodIndex")} Vol={noiseVolume} Len={noiseLength} Short={Flag(noise, "ShortMode")}"),
                new(4, "DMC", Value(dmc, "BytesRemaining") > 0, Scale(dmcOut, 127), Muted(4),
                    $"Rate={Value(dmc, "RateIndex")} Out={dmcOut} Addr=0x{Value(dmc, "SampleAddress"):X4} Len={Value(dmc, "SampleLength")} Loop={Flag(dmc, "Loop")}"),
            };
        }

        private static int Scale(long value, int max) => (int)Math.Round(value / (double)max * 100.0);

        // A cartridge's board by its mapper number's name, the pack's own; "?" for a number it names no board for.
        public string? UnitLabel(DebugUnit unit) => unit.Kind == "cartridge" && Register(unit.Registers, "Mapper") is { } mapper ? NesSystem.BoardName((int)mapper) ?? "?" : null;

        // The cartridge, the processor and the PPU, from the spaces' sizes and the units, after the host's own head line.
        public string? Summary(IReadOnlyList<DebugRegisterValue> registers, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            if (Unit(units, Ppu) is not { } ppu) return null;
            DebugUnit? board = Unit(units, Board), cartridge = Unit(units, Cartridge);
            long R(string name) => Register(registers, name) is { } v ? (long)v : 0;
            long P(string name) => Value(ppu, name);
            int mapper = (int)Value(board, "Mapper");
            int prg = (Find(spaces, "PRGROM")?.Size ?? 0) / 0x4000, chr = (Find(spaces, NesSystem.Chr)?.Size ?? 0) / 0x2000;
            string chrText = Flag(cartridge, "ChrRam") ? "8K CHR RAM" : $"{chr}x8K CHR ROM";
            var text = new System.Text.StringBuilder();
            text.AppendLine($"Mapper {mapper} ({NesSystem.BoardName(mapper) ?? "?"}), {prg}x16K PRG, {chrText}, mirroring {(NesMirroring)Value(cartridge, "Mirroring")}{(Flag(cartridge, "Battery") ? ", battery" : "")}");
            text.AppendLine($"CPU  A={R("A"):X2} X={R("X"):X2} Y={R("Y"):X2} S={R("S"):X2} PC={R("PC"):X4} P={R("P"):X2}");
            long mask = P("PPUMASK"), status = P("PPUSTATUS");
            text.AppendLine($"PPU  CTRL={P("PPUCTRL"):X2} MASK={mask:X2} STATUS={status:X2} v={P("v"):X4} t={P("t"):X4} x={P("x")}");
            text.Append($"     rendering={((mask & 0x18) != 0 ? "on" : "off")} vblank={(status >> 7) & 1} sprite0={(status >> 6) & 1} scanline={P("Scanline")}");
            return text.ToString();
        }

        public IReadOnlyList<InterruptVector> InterruptVectors { get; } = new[]
        {
            new InterruptVector("NMI", NmiVector),
            new InterruptVector("RESET", ResetVector),
            new InterruptVector("IRQ/BRK", IrqVector),
        };
    }
}
