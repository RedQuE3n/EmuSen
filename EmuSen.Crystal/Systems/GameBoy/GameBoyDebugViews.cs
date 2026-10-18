using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;
using static EmuSen.Crystal.Systems.SystemDebugSpaces;

namespace EmuSen.Crystal.Systems.GameBoy
{
    // The Game Boy's and the Color's debugger views, decoded from the spaces and units any Game Boy engine publishes; emusen-systems' gb views are their twin - see Mercury_Debug.md §3, EmuSen_Backend.md §17.2.
    public sealed class GameBoyDebugViews : ISystemDebugViews
    {
        public static readonly GameBoyDebugViews Instance = new();

        public const string Vram = "VRAM", Oam = "OAM", Rom = "ROM", BgPalettes = "BGPAL", ObjPalettes = "OBJPAL";
        public const string PpuUnit = "PPU", ApuUnit = "APU", TimerUnit = "Timer", HdmaUnit = "HDMA";
        public const int SpriteCount = 40, VramBankStride = 0x2000, WramBankSize = 0x1000, ScreenHeight = 144;

        // The four greys a Game Boy picture is shown in, lightest first; the view's choice, since the console defines none.
        public static readonly byte[] Shades = { 0xFF, 0xAA, 0x55, 0x00 };

        public static readonly string[] Channels = { "Pulse 1", "Pulse 2", "Wave", "Noise" };

        private static DebugUnit? Unit(IReadOnlyList<DebugUnit> units, string name)
        {
            foreach (DebugUnit u in units) if (string.Equals(u.Name, name, StringComparison.OrdinalIgnoreCase)) return u;
            return null;
        }

        private static ulong R(DebugUnit? unit, string name) => unit is null ? 0 : Register(unit.Registers, name) ?? 0;

        private static bool Has(DebugUnit? unit, string name) => unit is not null && Register(unit.Registers, name) is not null;

        // Colour mode: the PPU publishes its colour registers only then.
        private static bool Colour(DebugUnit? ppu) => Has(ppu, "VBK");

        // A Color running a Game Boy game: two VRAM banks and no colour registers.
        private static bool Compatibility(IReadOnlyList<IDebugMemorySpace> spaces, DebugUnit? ppu) => !Colour(ppu) && Find(spaces, Vram) is { Size: 2 * VramBankStride };

        private static byte Expand(int channel) => (byte)((channel << 3) | (channel >> 2));

        private static (byte, byte, byte) Rgb(IDebugMemorySpace ram, int entry)
        {
            int lo = ram.Read(entry), hi = ram.Read(entry + 1);
            int rgb555 = lo | (hi << 8);
            return (Expand(rgb555 & 0x1F), Expand((rgb555 >> 5) & 0x1F), Expand((rgb555 >> 10) & 0x1F));
        }

        private static DebugPaletteInfo ColourPalette(int index, IDebugMemorySpace ram, int palette)
        {
            var colors = new List<(byte r, byte g, byte b)>(4);
            for (int c = 0; c < 4; c++) colors.Add(Rgb(ram, (palette * 8) + (c * 2)));
            return new DebugPaletteInfo(index, colors);
        }

        private static DebugPaletteInfo CompatibilityPalette(int index, ulong register, IDebugMemorySpace ram, int palette)
        {
            var colors = new List<(byte r, byte g, byte b)>(4);
            for (int c = 0; c < 4; c++) colors.Add(Rgb(ram, (palette * 8) + ((int)((register >> (c * 2)) & 0x03) * 2)));
            return new DebugPaletteInfo(index, colors);
        }

        private static DebugPaletteInfo ShadePalette(int index, ulong register)
        {
            var colors = new List<(byte r, byte g, byte b)>(4);
            for (int c = 0; c < 4; c++)
            {
                byte shade = Shades[(register >> (c * 2)) & 0x03];
                colors.Add((shade, shade, shade));
            }
            return new DebugPaletteInfo(index, colors);
        }

        // Four DMG palettes of greys, the Color's sixteen, or the DMG registers through palette RAM - see Mercury_Debug.md §3.1.
        public IReadOnlyList<DebugPaletteInfo> Palettes(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            if (Unit(units, PpuUnit) is not { } ppu) return Array.Empty<DebugPaletteInfo>();
            ulong bgp = R(ppu, "BGP"), obp0 = R(ppu, "OBP0"), obp1 = R(ppu, "OBP1");
            var palettes = new List<DebugPaletteInfo>();
            if (Colour(ppu) || Compatibility(spaces, ppu))
            {
                if (Find(spaces, BgPalettes) is not { Size: >= 64 } bg || Find(spaces, ObjPalettes) is not { Size: >= 64 } obj) return Array.Empty<DebugPaletteInfo>();
                if (Colour(ppu))
                {
                    for (int p = 0; p < 8; p++) palettes.Add(ColourPalette(p, bg, p));
                    for (int p = 0; p < 8; p++) palettes.Add(ColourPalette(8 + p, obj, p));
                    return palettes;
                }
                palettes.Add(CompatibilityPalette(0, bgp, bg, 0));
                palettes.Add(CompatibilityPalette(1, obp0, obj, 0));
                palettes.Add(CompatibilityPalette(2, obp1, obj, 1));
                return palettes;
            }
            palettes.Add(ShadePalette(0, bgp));
            palettes.Add(ShadePalette(1, obp0));
            palettes.Add(ShadePalette(2, obp1));
            return palettes;
        }

        // Each palette a row of four 16-pixel swatches.
        public (byte[] Rgba, int Width, int Height) PaletteSwatch(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            IReadOnlyList<DebugPaletteInfo> palettes = Palettes(spaces, units);
            if (palettes.Count == 0) return (Array.Empty<byte>(), 0, 0);
            const int swatch = 16, columns = 4;
            int width = columns * swatch, height = palettes.Count * swatch;
            var rgba = new byte[width * height * 4];
            for (int p = 0; p < palettes.Count; p++)
                for (int c = 0; c < columns; c++)
                {
                    var (r, g, b) = palettes[p].Colors[c];
                    for (int y = 0; y < swatch; y++)
                        for (int x = 0; x < swatch; x++)
                        {
                            int o = ((((p * swatch) + y) * width) + (c * swatch) + x) * 4;
                            (rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]) = (r, g, b, 0xFF);
                        }
                }
            return (rgba, width, height);
        }

        // Every tile of VRAM, sixteen a row, grey by pixel value: a tile has no palette of its own until a map entry picks one.
        public (byte[] Rgba, int Width, int Height) TileSheet(IReadOnlyList<IDebugMemorySpace> spaces)
        {
            if (Find(spaces, Vram) is not { Size: > 0 } vram) return (Array.Empty<byte>(), 0, 0);
            const int perRow = 16;
            int tiles = vram.Size / 16, rows = (tiles + perRow - 1) / perRow;
            int width = perRow * 8, height = rows * 8;
            var rgba = new byte[width * height * 4];
            for (int tile = 0; tile < tiles; tile++)
            {
                int x0 = (tile % perRow) * 8, y0 = (tile / perRow) * 8;
                for (int row = 0; row < 8; row++)
                {
                    byte low = vram.Read((tile * 16) + (row * 2)), high = vram.Read((tile * 16) + (row * 2) + 1);
                    for (int column = 0; column < 8; column++)
                    {
                        int bit = 7 - column;
                        byte shade = Shades[((low >> bit) & 1) | (((high >> bit) & 1) << 1)];
                        int o = (((y0 + row) * width) + x0 + column) * 4;
                        (rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]) = (shade, shade, shade, 0xFF);
                    }
                }
            }
            return (rgba, width, height);
        }

        // One byte per map cell; the Color's attribute byte is in the other VRAM bank, not the next address.
        public int TilemapEntryStride => 1;

        public string DecodeTilemapEntry(IDebugMemorySpace space, int address) => $"${space.Read(address):X2}";

        // The tile number, and in colour mode the attribute from bank 1.
        public string DecodeTilemapEntry(IDebugMemorySpace space, int address, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            byte tile = space.Read(address);
            if (!Colour(Unit(units, PpuUnit))) return $"${tile:X2}";
            byte attributes = space.Read(address + VramBankStride);
            string flips = $"{((attributes & 0x20) != 0 ? "X" : "-")}{((attributes & 0x40) != 0 ? "Y" : "-")}";
            return $"${tile:X2} p{attributes & 0x07} b{(attributes >> 3) & 0x01} {flips}{((attributes & 0x80) != 0 ? " pri" : "")}";
        }

        // 16 bytes a tile, interleaved by row rather than split into two planes as an NES tile is.
        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp)
        {
            if (bpp != 2) throw new ArgumentException($"A Game Boy tile is always 2bpp; got {bpp}.", nameof(bpp));
            var pixels = new byte[64];
            for (int row = 0; row < 8; row++)
            {
                byte low = space.Read(address + (row * 2)), high = space.Read(address + (row * 2) + 1);
                for (int column = 0; column < 8; column++)
                {
                    int bit = 7 - column;
                    pixels[(row * 8) + column] = (byte)(((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1));
                }
            }
            return pixels;
        }

        // Five vectors of eight bytes each, which is why a handler is nearly always a jump.
        public IReadOnlyList<InterruptVector> InterruptVectors { get; } = new[]
        {
            new InterruptVector("VBlank", 0x0040), new InterruptVector("LCD STAT", 0x0048), new InterruptVector("Timer", 0x0050),
            new InterruptVector("Serial", 0x0058), new InterruptVector("Joypad", 0x0060),
        };

        public int MaxSprites => SpriteCount;

        // OAM's forty entries with LCDC's height; one parked off the top or bottom is hidden, the standard way to hide one.
        public IReadOnlyList<DebugSpriteInfo> Sprites(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            if (Unit(units, PpuUnit) is not { } ppu || Find(spaces, Oam) is not { Size: >= 4 * SpriteCount } oam) return Array.Empty<DebugSpriteInfo>();
            int height = (R(ppu, "LCDC") & 0x04) != 0 ? 16 : 8;
            bool colour = Colour(ppu);
            var sprites = new List<DebugSpriteInfo>();
            for (int i = 0; i < SpriteCount; i++)
            {
                int y = oam.Read(i * 4) - 16, x = oam.Read((i * 4) + 1) - 8;
                byte tile = oam.Read((i * 4) + 2), attributes = oam.Read((i * 4) + 3);
                if (y <= -height || y >= ScreenHeight) continue;
                int palette = colour ? attributes & 0x07 : (attributes & 0x10) != 0 ? 1 : 0;
                sprites.Add(new DebugSpriteInfo(i, x, y, 8, height, tile, palette, (attributes & 0x80) != 0 ? 0 : 1, (attributes & 0x20) != 0, (attributes & 0x40) != 0));
            }
            return sprites;
        }

        private static int Scale(ulong value, int max) => (int)Math.Round(value / (double)max * 100.0);

        // Each channel from its unit; a level is a 0-100 rescale of its own volume, which for the wave is a shift - see Mercury_Apu.md §3.4.
        public IReadOnlyList<DebugAudioChannelInfo> AudioChannels(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units, uint mutes)
        {
            var channels = new List<DebugAudioChannelInfo>();
            for (int i = 0; i < Channels.Length; i++)
            {
                if (Unit(units, Channels[i]) is not { } c) continue;
                bool active = R(c, "Enabled") != 0 && R(c, "Dac") != 0, muted = (mutes & (1u << i)) != 0;
                ulong vol = R(c, "Vol");
                switch (i)
                {
                    case 2:
                        channels.Add(new DebugAudioChannelInfo(i, Channels[i], active, vol == 0 ? 0 : Scale(4 - vol, 3), muted,
                            $"Freq={R(c, "Freq")} Vol={vol} Len={R(c, "Len")} Dac={R(c, "Dac") != 0}"));
                        break;
                    case 3:
                        channels.Add(new DebugAudioChannelInfo(i, Channels[i], active, Scale(vol, 15), muted,
                            $"Shift={R(c, "Shift")} Div={R(c, "Div")} Short={R(c, "Short") != 0} Vol={vol} Len={R(c, "Len")}"));
                        break;
                    default:
                        string sweep = Has(c, "SweepPeriod") ? $" Sweep={R(c, "SweepPeriod")}/{R(c, "SweepShift")}{(R(c, "SweepNegate") != 0 ? "-" : "+")}" : "";
                        channels.Add(new DebugAudioChannelInfo(i, Channels[i], active, Scale(vol, 15), muted,
                            $"Duty={R(c, "Duty")} Freq={R(c, "Freq")} Vol={vol} Len={R(c, "Len")}{sweep}"));
                        break;
                }
            }
            return channels;
        }

        // HDMA is one channel with two modes, so the table has one row - see Mercury_Cgb.md §4.
        public IReadOnlyList<DebugDmaChannel> DmaChannels(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            if (Unit(units, HdmaUnit) is not { } h) return Array.Empty<DebugDmaChannel>();
            int blocks = (int)R(h, "BlocksLeft");
            bool hblank = R(h, "HBlank") != 0;
            return new[]
            {
                new DebugDmaChannel(0, blocks > 0 && !hblank, hblank, (byte)(hblank ? 0x80 : 0x00), 0x00, (int)R(h, "Source"), blocks * 16,
                    hblank && blocks > 0, 0, (byte)Math.Min(byte.MaxValue, blocks), 0x8000 | (int)R(h, "Destination")),
            };
        }

        public string? NameDmaDestination(byte register) => "VRAM";

        // Only the ranges whose destination does not depend on the cartridge's bank state.
        public PhysicalAddress? ResolvePhysical(int address, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            if (Unit(units, PpuUnit) is not { } ppu) return null;
            if (address < 0x4000) return new PhysicalAddress(Rom, address);
            if (address < 0x8000) return null;
            if (address < 0xA000) return new PhysicalAddress(Vram, ((int)R(ppu, "VBK") * VramBankStride) + (address - 0x8000));
            if (address < 0xC000) return null;
            if (address < 0xD000) return new PhysicalAddress("WRAM", address - 0xC000);
            if (address < 0xE000) return new PhysicalAddress("WRAM", (Math.Max(1, (int)R(ppu, "SVBK")) * WramBankSize) + (address - 0xD000));
            if (address < 0xFE00) return null;
            if (address < 0xFEA0) return new PhysicalAddress(Oam, address - 0xFE00);
            if (address < 0xFF00) return new PhysicalAddress("PROHIBITED", address - 0xFEA0, false);
            if (address < 0xFF80) return new PhysicalAddress("IOREG", address - 0xFF00, false);
            if (address < 0xFFFF) return new PhysicalAddress("HRAM", address - 0xFF80);
            return new PhysicalAddress("IE", 0, false);
        }

        // The cartridge from its header, then the CPU, PPU, colour and sound lines.
        public string? Summary(IReadOnlyList<DebugRegisterValue> registers, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            if (Register(registers, "PC") is null || Unit(units, PpuUnit) is not { } ppu) return null;
            ulong C(string name) => Register(registers, name) ?? 0;
            var text = new StringBuilder();
            if (Find(spaces, Rom) is { Size: >= GbCartridge.HeaderLength } rom)
            {
                var head = new byte[GbCartridge.HeaderLength];
                for (int i = 0; i < head.Length; i++) head[i] = rom.Read(i);
                GbCartridge cart = GbCartridge.ReadManaged(head, rom.Size);
                text.Append($"{cart.Board}, {cart.RomBanks}x16K ROM, {(cart.RamLength == 0 ? "no RAM" : $"{cart.RamLength / 1024}K RAM")}{(cart.HasBattery ? ", battery" : "")}{(cart.HasTimer ? ", RTC" : "")}\n");
            }
            text.Append($"CPU  A={C("A"):X2} F={C("F"):X2} BC={C("BC"):X4} DE={C("DE"):X4} HL={C("HL"):X4} SP={C("SP"):X4} PC={C("PC"):X4}\n");
            text.Append($"     IME={C("IME")} HALT={C("HALT")} IE={C("IE"):X2} IF={C("IF"):X2}\n");
            text.Append($"PPU  LCDC={R(ppu, "LCDC"):X2} STAT={R(ppu, "STAT"):X2} SCY={R(ppu, "SCY"):X2} SCX={R(ppu, "SCX"):X2} WY={R(ppu, "WY"):X2} WX={R(ppu, "WX"):X2}");
            if (Colour(ppu))
            {
                ulong blocks = R(Unit(units, HdmaUnit), "BlocksLeft");
                text.Append($"\nCGB  VBK={R(ppu, "VBK")} SVBK={R(ppu, "SVBK")} speed={((R(ppu, "KEY1") & 0x80) != 0 ? "double" : "normal")} HDMA={(blocks == 0 ? "idle" : $"{blocks} blocks")}");
            }
            if (Unit(units, ApuUnit) is { } apu)
            {
                ulong nr50 = R(apu, "$FF24"), nr51 = R(apu, "$FF25");
                text.Append($"\nAPU  power={(R(apu, "Power") != 0 ? "on" : "off")} vol={(nr50 >> 4) & 7}/{nr50 & 7} pan={nr51:X2}");
            }
            return text.ToString();
        }
    }
}
