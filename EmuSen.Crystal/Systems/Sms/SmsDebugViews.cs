using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;
using static EmuSen.Crystal.Systems.SystemDebugSpaces;

namespace EmuSen.Crystal.Systems.Sms
{
    // The Master System's and the SG-1000's debugger views, decoded from the spaces and units any engine of the two publishes; emusen-systems' sms views are their twin - see Endou_Native.md §15.5.
    public sealed class SmsDebugViews : ISystemDebugViews
    {
        public static readonly SmsDebugViews MasterSystem = new(sg1000: false), Sg1000 = new(sg1000: true);

        public const string Vram = "VRAM", Cram = "CRAM", Ram = "RAM", Rom = "ROM";
        public const string VdpUnit = "VDP", PsgUnit = "PSG", BoardUnit = "Board", IoUnit = "IO";

        // The VDP unit's Variant for the 315-5124, whose first blue level differs - see Endou_Disputes.md D-1.
        public const ulong Variant315_5124 = 1;

        public static readonly string[] Channels = { "Tone 1", "Tone 2", "Tone 3", "Noise" };

        // The boards by the Board unit's Mapper - see Endou_Native.md §15.4.
        public static readonly string[] Boards = { "Plain", "Sega", "Codemasters", "SG-1000 RAM" };

        // A component's four levels on the 315-5246, and the 315-5124's blue - beryl-sms-colours is the Rust owner, see Endou_Disputes.md D-1.
        private static readonly byte[] Levels = { 0, 85, 170, 255 }, Blue5124 = { 0, 0x68, 170, 255 };

        // The TMS9918A's sixteen colours - beryl-sms-colours is the Rust owner, see Endou_Disputes.md D-11.
        public static readonly (byte R, byte G, byte B)[] TmsColours =
        {
            (0x00, 0x00, 0x00), (0x00, 0x00, 0x00), (0x21, 0xC8, 0x42), (0x5E, 0xDC, 0x78), (0x54, 0x55, 0xED), (0x7D, 0x76, 0xFC), (0xD4, 0x52, 0x4D), (0x42, 0xEC, 0xF5),
            (0xFC, 0x55, 0x54), (0xFF, 0x79, 0x78), (0xD4, 0xC1, 0x54), (0xE6, 0xCE, 0x80), (0x21, 0xB0, 0x3B), (0xC9, 0x5B, 0xBA), (0xCC, 0xCC, 0xCC), (0xFF, 0xFF, 0xFF),
        };

        private readonly bool _sg1000;

        private SmsDebugViews(bool sg1000) => _sg1000 = sg1000;

        private static DebugUnit? Unit(IReadOnlyList<DebugUnit> units, string name)
        {
            foreach (DebugUnit u in units) if (string.Equals(u.Name, name, StringComparison.OrdinalIgnoreCase)) return u;
            return null;
        }

        private static ulong R(DebugUnit? unit, string name) => unit is null ? 0 : Register(unit.Registers, name) ?? 0;

        private static (byte, byte, byte) Colour(byte entry, bool sms1)
        {
            int e = entry & 0x3F;
            return (Levels[e & 3], Levels[(e >> 2) & 3], (sms1 ? Blue5124 : Levels)[(e >> 4) & 3]);
        }

        public IReadOnlyList<DebugPaletteInfo> Palettes(IReadOnlyList<IDebugMemorySpace> spaces) => Palettes(spaces, Array.Empty<DebugUnit>());

        // CRAM's background and sprite palettes in the VDP unit's chip's colours; with VRAM and no CRAM, the TMS9918A's sixteen.
        public IReadOnlyList<DebugPaletteInfo> Palettes(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            if (Find(spaces, Cram) is not { Size: >= 32 } cram)
                return Find(spaces, Vram) is null ? Array.Empty<DebugPaletteInfo>() : new[] { new DebugPaletteInfo(0, TmsColours) };
            bool sms1 = R(Unit(units, VdpUnit), "Variant") == Variant315_5124;
            var entries = new byte[32];
            for (int i = 0; i < 32; i++) entries[i] = cram.Read(i);
            var palettes = new List<DebugPaletteInfo>(2);
            for (int p = 0; p < 2; p++)
            {
                var colors = new List<(byte r, byte g, byte b)>(16);
                for (int c = 0; c < 16; c++) colors.Add(Colour(entries[p * 16 + c], sms1));
                palettes.Add(new DebugPaletteInfo(p, colors));
            }
            return palettes;
        }

        public (byte[] Rgba, int Width, int Height) PaletteSwatch(IReadOnlyList<IDebugMemorySpace> spaces) => PaletteSwatch(spaces, Array.Empty<DebugUnit>());

        // Each palette a row of sixteen 16-pixel swatches.
        public (byte[] Rgba, int Width, int Height) PaletteSwatch(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            IReadOnlyList<DebugPaletteInfo> palettes = Palettes(spaces, units);
            if (palettes.Count == 0) return (Array.Empty<byte>(), 0, 0);
            const int swatch = 16, columns = 16;
            int width = columns * swatch, height = palettes.Count * swatch;
            var rgba = new byte[width * height * 4];
            for (int p = 0; p < palettes.Count; p++)
                for (int c = 0; c < palettes[p].Colors.Count; c++)
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

        // A mode 4 pixel from its row's four plane bytes, plane 0 the lowest bit.
        private static byte Planar(byte[] bytes, int at, int column)
        {
            int v = 0;
            for (int p = 0; p < 4; p++) v |= ((bytes[at + p] >> (7 - column)) & 1) << p;
            return (byte)v;
        }

        // Every tile of VRAM, grey by pixel value: mode 4's 32-byte tiles sixteen a row, or the SG-1000's 8-byte patterns thirty-two a row.
        public (byte[] Rgba, int Width, int Height) TileSheet(IReadOnlyList<IDebugMemorySpace> spaces)
        {
            if (Find(spaces, Vram) is not { Size: >= 1 } vram) return (Array.Empty<byte>(), 0, 0);
            int bytesPer = _sg1000 ? 8 : 32, perRow = _sg1000 ? 32 : 16;
            int tiles = vram.Size / bytesPer;
            int width = perRow * 8, height = (tiles + perRow - 1) / perRow * 8;
            var bytes = new byte[tiles * bytesPer];
            for (int i = 0; i < bytes.Length; i++) bytes[i] = vram.Read(i);
            var rgba = new byte[width * height * 4];
            for (int tile = 0; tile < tiles; tile++)
            {
                int x0 = tile % perRow * 8, y0 = tile / perRow * 8;
                for (int row = 0; row < 8; row++)
                    for (int column = 0; column < 8; column++)
                    {
                        byte shade = _sg1000 ? (byte)(((bytes[tile * 8 + row] << column) & 0x80) != 0 ? 0xFF : 0) : (byte)(Planar(bytes, tile * 32 + row * 4, column) * 17);
                        int o = (((y0 + row) * width) + x0 + column) * 4;
                        (rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]) = (shade, shade, shade, 0xFF);
                    }
            }
            return (rgba, width, height);
        }

        public int TilemapEntryStride => _sg1000 ? 1 : 2;

        // Mode 4's word entry as its tile, palette, flips and priority, or the TMS9918's pattern name.
        public string DecodeTilemapEntry(IDebugMemorySpace space, int address)
        {
            if (_sg1000) return $"${space.Read(address):X2}";
            int lo = space.Read(address), hi = space.Read(unchecked(address + 1));
            int w = lo | (hi << 8);
            string flips = ((w & 0x200) != 0 ? "X" : "-") + ((w & 0x400) != 0 ? "Y" : "-");
            return $"${w & 0x1FF:X3} p{(w >> 11) & 1} {flips}{((w & 0x1000) != 0 ? " pri" : "")}";
        }

        // A tile's 64 pixels: mode 4's four planes or the TMS9918's one; another depth refused.
        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp)
        {
            int depth = _sg1000 ? 1 : 4, length = _sg1000 ? 8 : 32;
            if (bpp != depth)
                throw new ArgumentException(_sg1000 ? $"An SG-1000 pattern is always 1bpp; got {bpp}." : $"A Master System tile is always 4bpp; got {bpp}.", nameof(bpp));
            var bytes = new byte[length];
            for (int i = 0; i < length; i++) bytes[i] = space.Read(unchecked(address + i));
            var pixels = new byte[64];
            for (int row = 0; row < 8; row++)
                for (int column = 0; column < 8; column++)
                    pixels[row * 8 + column] = _sg1000 ? (byte)((bytes[row] >> (7 - column)) & 1) : Planar(bytes, row * 4, column);
            return pixels;
        }

        public int MaxSprites => _sg1000 ? 32 : 64;

        // The first line a sprite shows on, a sprite past line 255 above the first; null for one wholly below the active lines.
        private static int? ShownY(byte y, int height, int lines)
        {
            int top = y + 1;
            if (top + height > 256) return top - 256;
            return top < lines ? top : null;
        }

        // The sprite table the VDP unit's registers point at, to the $D0 that ends it where the chip honours one - see Endou_Native.md §15.5.
        public IReadOnlyList<DebugSpriteInfo> Sprites(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            if (Unit(units, VdpUnit) is not { } vdp) return Array.Empty<DebugSpriteInfo>();
            if (Find(spaces, Vram) is not { Size: >= 0x4000 } vram) return Array.Empty<DebugSpriteInfo>();
            ulong r0 = R(vdp, "R0"), r1 = R(vdp, "R1"), r5 = R(vdp, "R5"), r6 = R(vdp, "R6");
            int lines = unchecked((int)Math.Max(R(vdp, "Lines"), 1));
            int size = (r1 & 2) != 0 ? 16 : 8, zoom = (r1 & 1) != 0 ? 2 : 1;
            var sprites = new List<DebugSpriteInfo>();
            if (_sg1000)
            {
                int tmsBase = (int)((r5 & 0x7F) << 7);
                var tms = new byte[128];
                for (int i = 0; i < tms.Length; i++) tms[i] = vram.Read(tmsBase + i);
                for (int i = 0; i < 32; i++)
                {
                    byte y = tms[i * 4], x = tms[i * 4 + 1], name = tms[i * 4 + 2], attr = tms[i * 4 + 3];
                    if (y == 0xD0) break;
                    if (ShownY(y, size * zoom, lines) is not { } top) continue;
                    int tile = size == 16 ? name & 0xFC : name;
                    sprites.Add(new DebugSpriteInfo(i, x - ((attr & 0x80) != 0 ? 32 : 0), top, size * zoom, size * zoom, tile, attr & 0x0F, 1, false, false));
                }
                return sprites;
            }
            int satBase = (int)((r5 & 0x7E) << 7);
            var table = new byte[256];
            for (int i = 0; i < table.Length; i++) table[i] = vram.Read(satBase + i);
            int height = size * zoom;
            for (int i = 0; i < 64; i++)
            {
                byte y = table[i];
                if (lines == 192 && y == 0xD0) break;
                if (ShownY(y, height, lines) is not { } top) continue;
                int x = table[0x80 + 2 * i] - ((r0 & 8) != 0 ? 8 : 0);
                int tile = table[0x81 + 2 * i] | (int)((r6 & 4) << 6);
                sprites.Add(new DebugSpriteInfo(i, x, top, 8 * zoom, height, tile, 1, 1, false, false));
            }
            return sprites;
        }

        // The PSG's three tones and its noise: a channel sounds while its attenuation is under 15, its level the complement out of 100 by whole steps.
        public IReadOnlyList<DebugAudioChannelInfo> AudioChannels(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units, uint mutes)
        {
            if (Unit(units, PsgUnit) is not { } psg) return Array.Empty<DebugAudioChannelInfo>();
            var channels = new List<DebugAudioChannelInfo>(4);
            for (int i = 0; i < 4; i++)
            {
                ulong att = R(psg, $"Volume{i}");
                int level = (int)((15 - Math.Min(att, 15)) * 100 / 15);
                ulong noise = R(psg, "Noise");
                string line = i < 3 ? $"Period={R(psg, $"Tone{i}")} Att={att}" : $"Rate={noise & 3} White={((noise & 4) != 0 ? "True" : "False")} Att={att}";
                channels.Add(new DebugAudioChannelInfo(i, Channels[i], att < 15, level, (mutes & (1u << i)) != 0, line));
            }
            return channels;
        }

        // A cartridge's board by its Mapper; "?" for a number no board is named for.
        public string? UnitLabel(DebugUnit unit)
        {
            if (unit.Kind != "cartridge" || Register(unit.Registers, "Mapper") is not { } mapper) return null;
            return mapper < (ulong)Boards.Length ? Boards[mapper] : "?";
        }

        // $C000 and up as work RAM or Sega's mapper's cartridge RAM, nothing while port $3E disables work RAM; below, null.
        public PhysicalAddress? ResolvePhysical(int address, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            if (address < 0xC000 || address > 0xFFFF) return null;
            DebugUnit? board = Unit(units, BoardUnit);
            int cartRam = unchecked((int)R(board, "Ram"));
            if (R(board, "Mapper") == 1 && (R(board, "Control") & 0x10) != 0 && cartRam > 0) return new PhysicalAddress("CARTRAM", (address & 0x3FFF) % Math.Min(cartRam, 0x4000), true);
            if ((R(Unit(units, IoUnit), "MemoryControl") & 0x10) != 0) return null;
            int size = Find(spaces, Ram)?.Size ?? 0;
            return size > 0 ? new PhysicalAddress("RAM", (address - 0xC000) % size, true) : null;
        }

        // The board, then the Z80, the VDP, the PSG and the I/O chip's ports.
        public string? Summary(IReadOnlyList<DebugRegisterValue> registers, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            if (Register(registers, "PC") is null || Unit(units, VdpUnit) is not { } vdp) return null;
            ulong C(string n) => Register(registers, n) ?? 0;
            var text = new StringBuilder();
            if (Unit(units, BoardUnit) is { } board)
            {
                int rom = (Find(spaces, Rom)?.Size ?? 0) / 1024;
                ulong ram = R(board, "Ram") / 1024;
                text.Append($"Board {UnitLabel(board) ?? ""}, {rom} KiB ROM{(ram > 0 ? $", {ram} KiB RAM" : "")}{(R(board, "Battery") != 0 ? ", battery" : "")}\n");
            }
            text.Append($"Z80  AF={C("AF"):X4} BC={C("BC"):X4} DE={C("DE"):X4} HL={C("HL"):X4} IX={C("IX"):X4} IY={C("IY"):X4} SP={C("SP"):X4} PC={C("PC"):X4}\n");
            text.Append($"     I={C("I"):X2} R={C("R"):X2} IM={C("IM")} IFF1={C("IFF1")} IFF2={C("IFF2")}\n");
            text.Append($"VDP  R0={R(vdp, "R0"):X2} R1={R(vdp, "R1"):X2} mode {(R(vdp, "Mode4") != 0 ? "4" : "TMS")}, display {((R(vdp, "R1") & 0x40) != 0 ? "on" : "off")}, line {R(vdp, "Line")} of {R(vdp, "Lines")}, status={R(vdp, "Status"):X2}");
            if (Unit(units, PsgUnit) is { } psg)
                text.Append($"\nPSG  tones {R(psg, "Tone0"):X3} {R(psg, "Tone1"):X3} {R(psg, "Tone2"):X3} noise {R(psg, "Noise"):X} att {R(psg, "Volume0"):X}{R(psg, "Volume1"):X}{R(psg, "Volume2"):X}{R(psg, "Volume3"):X}");
            if (Unit(units, IoUnit) is { } io)
                text.Append($"\nIO   $3E={R(io, "MemoryControl"):X2} $3F={R(io, "IoControl"):X2}");
            return text.ToString();
        }
    }
}
