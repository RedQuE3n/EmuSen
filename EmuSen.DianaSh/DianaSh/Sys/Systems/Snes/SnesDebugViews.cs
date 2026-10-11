using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Cauldron;
using EmuSen.DianaSh.DianaSh.Lib;
using static EmuSen.DianaSh.DianaSh.Sys.Systems.SystemDebugSpaces;

namespace EmuSen.DianaSh.DianaSh.Sys.Systems.Snes
{
    // The SNES's debugger views over the spaces both its engines publish, so Venus and VenusRT show one decoding - see EmuSen_CoreAPI.md §31.
    public sealed class SnesDebugViews : ISystemDebugViews
    {
        public static readonly SnesDebugViews Instance = new();

        // The space names the views read; IO is $4000-$43FF as last written, the layout VenusRT's space has - see EmuSen_CoreAPI.md §31.2.
        public const string Vram = "VRAM", Cgram = "CGRAM", Io = "IO";
        public const int IoSize = 0x400;

        public IReadOnlyList<DebugPaletteInfo> Palettes(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Find(spaces, Cgram) is { } cgram ? Palettes(cgram) : Array.Empty<DebugPaletteInfo>();

        // All 256 entries as sixteen palettes of sixteen, each BGR555 channel scaled to 0-255.
        public static IReadOnlyList<DebugPaletteInfo> Palettes(IDebugMemorySpace cgram)
        {
            var result = new List<DebugPaletteInfo>(16);
            for (int p = 0; p < 16; p++)
            {
                var colors = new List<(byte r, byte g, byte b)>(16);
                for (int c = 0; c < 16; c++)
                {
                    int raw = Word(cgram, (p * 16 + c) * 2);
                    colors.Add(((byte)((raw & 0x1F) * 255 / 31), (byte)(((raw >> 5) & 0x1F) * 255 / 31), (byte)(((raw >> 10) & 0x1F) * 255 / 31)));
                }
                result.Add(new DebugPaletteInfo(p, colors));
            }
            return result;
        }

        public (byte[] Rgba, int Width, int Height) PaletteSwatch(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Find(spaces, Cgram) is { } cgram ? PaletteSwatch(cgram) : (Array.Empty<byte>(), 0, 0);

        // A 16x16 grid of 12-pixel swatches, each channel shifted left three as the frame's colours are.
        public static (byte[] Rgba, int Width, int Height) PaletteSwatch(IDebugMemorySpace cgram)
        {
            const int cols = 16, swatch = 12, width = cols * swatch, height = 16 * swatch;
            byte[] buffer = new byte[width * height * 4];
            for (int i = 0; i < 256; i++)
            {
                int raw = Word(cgram, i * 2);
                byte r = (byte)((raw & 0x1F) << 3), g = (byte)(((raw >> 5) & 0x1F) << 3), b = (byte)(((raw >> 10) & 0x1F) << 3);
                int originX = (i % cols) * swatch, originY = (i / cols) * swatch;
                for (int y = 0; y < swatch; y++)
                {
                    for (int x = 0; x < swatch; x++)
                    {
                        int o = ((originY + y) * width + originX + x) * 4;
                        (buffer[o], buffer[o + 1], buffer[o + 2], buffer[o + 3]) = (r, g, b, 255);
                    }
                }
            }
            return (buffer, width, height);
        }

        public (byte[] Rgba, int Width, int Height) TileSheet(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Find(spaces, Vram) is { } vram ? TileSheet(vram) : (Array.Empty<byte>(), 0, 0);

        // VRAM's 2048 tiles as 4bpp, 32 a row, each index n drawn as grey 17n.
        public static (byte[] Rgba, int Width, int Height) TileSheet(IDebugMemorySpace vram)
        {
            const int width = 256, height = 512;
            byte[] bytes = new byte[0x10000];
            for (int a = 0; a < bytes.Length; a++) bytes[a] = vram.Read(a);
            byte[] buffer = new byte[width * height * 4];
            for (int tile = 0; tile < 2048; tile++)
            {
                int baseX = (tile % 32) * 8, baseY = (tile / 32) * 8, at = tile * 32;
                for (int r = 0; r < 8; r++)
                {
                    byte p0 = bytes[at + r * 2], p1 = bytes[at + r * 2 + 1], p2 = bytes[at + 16 + r * 2], p3 = bytes[at + 16 + r * 2 + 1];
                    for (int col = 0; col < 8; col++)
                    {
                        int bit = 7 - col;
                        int pixel = ((p0 >> bit) & 1) | (((p1 >> bit) & 1) << 1) | (((p2 >> bit) & 1) << 2) | (((p3 >> bit) & 1) << 3);
                        byte v = (byte)(pixel * 17);
                        int o = ((baseY + r) * width + baseX + col) * 4;
                        (buffer[o], buffer[o + 1], buffer[o + 2], buffer[o + 3]) = (v, v, v, 255);
                    }
                }
            }
            return (buffer, width, height);
        }

        // Two bytes: tile 0-9, palette 10-12, priority 13, flips 14-15.
        public int TilemapEntryStride => 2;

        string ISystemDebugViews.DecodeTilemapEntry(IDebugMemorySpace space, int address) => DecodeTilemapEntry(space, address);

        public static string DecodeTilemapEntry(IDebugMemorySpace space, int address)
        {
            int entry = space.Read(address) | (space.Read(address + 1) << 8);
            return $"{entry & 0x3FF:X3}{(entry >> 10) & 0x7}{((entry & 0x2000) != 0 ? 'P' : '.')}{((entry & 0x4000) != 0 ? 'H' : '.')}{((entry & 0x8000) != 0 ? 'V' : '.')}";
        }

        byte[] ISystemDebugViews.DecodeTilePixels(IDebugMemorySpace space, int address, int bpp) => DecodeTilePixels(space, address, bpp);

        // Planar, bpp/2 row-interleaved pairs of bit planes 16 bytes apart.
        public static byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp)
        {
            if (bpp != 2 && bpp != 4 && bpp != 8)
                throw new ArgumentException($"bpp must be 2, 4, or 8 (got {bpp}) - the SNES has no other planar tile depth.");

            var pixels = new byte[64];
            for (int row = 0; row < 8; row++)
            {
                for (int pair = 0; pair < bpp / 2; pair++)
                {
                    byte lo = space.Read(address + pair * 16 + row * 2), hi = space.Read(address + pair * 16 + row * 2 + 1);
                    for (int col = 0; col < 8; col++)
                    {
                        int bit = 7 - col;
                        pixels[row * 8 + col] |= (byte)((((lo >> bit) & 1) | (((hi >> bit) & 1) << 1)) << (pair * 2));
                    }
                }
            }
            return pixels;
        }

        public IReadOnlyList<DebugDmaChannel> DmaChannels(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Find(spaces, Io) is { Size: IoSize } io ? DmaChannels(io) : Array.Empty<DebugDmaChannel>();

        // The eight channels from $4300-$437F, MDMAEN and HDMAEN; an HDMA channel is running while enabled and its line counter is not the terminating 0 - see `man dma`.
        public static IReadOnlyList<DebugDmaChannel> DmaChannels(IDebugMemorySpace io)
        {
            byte general = io.Read(0x20B), hdma = io.Read(0x20C);
            var result = new DebugDmaChannel[8];
            for (int c = 0; c < 8; c++)
            {
                int at = 0x300 + c * 16;
                byte R(int r) => io.Read(at + r);
                int bank = R(4) << 16, count = R(5) | (R(6) << 8);
                bool hdmaEnabled = (hdma & (1 << c)) != 0;
                result[c] = new DebugDmaChannel(c, (general & (1 << c)) != 0, hdmaEnabled, R(0), R(1),
                    bank | R(2) | (R(3) << 8), count == 0 ? 0x10000 : count,
                    hdmaEnabled && R(10) != 0, bank | R(8) | (R(9) << 8), R(10), (R(7) << 16) | count);
            }
            return result;
        }

        string? ISystemDebugViews.NameDmaDestination(byte register) => NameDmaDestination(register);

        // The B bus's $2100-$21FF as fullsnes names them, the APU's four ports mirrored through $217F.
        public static string NameDmaDestination(byte register)
        {
            string? name = register switch
            {
                < 0x40 => BBusNames[register],
                <= 0x7F => $"APUI0{register & 3}",
                0x80 => "WMDATA",
                0x81 => "WMADDL",
                0x82 => "WMADDM",
                0x83 => "WMADDH",
                _ => null,
            };
            return name == null ? $"${0x2100 + register:X4}" : $"{name} ${0x2100 + register:X4}";
        }

        private static readonly string[] BBusNames =
        {
            "INIDISP", "OBSEL", "OAMADDL", "OAMADDH", "OAMDATA", "BGMODE", "MOSAIC", "BG1SC",
            "BG2SC", "BG3SC", "BG4SC", "BG12NBA", "BG34NBA", "BG1HOFS", "BG1VOFS", "BG2HOFS",
            "BG2VOFS", "BG3HOFS", "BG3VOFS", "BG4HOFS", "BG4VOFS", "VMAIN", "VMADDL", "VMADDH",
            "VMDATAL", "VMDATAH", "M7SEL", "M7A", "M7B", "M7C", "M7D", "M7X",
            "M7Y", "CGADD", "CGDATA", "W12SEL", "W34SEL", "WOBJSEL", "WH0", "WH1",
            "WH2", "WH3", "WBGLOG", "WOBJLOG", "TM", "TS", "TMW", "TSW",
            "CGWSEL", "CGADSUB", "COLDATA", "SETINI", "MPYL", "MPYM", "MPYH", "SLHV",
            "RDOAM", "RDVRAML", "RDVRAMH", "RDCGRAM", "OPHCT", "OPVCT", "STAT77", "STAT78",
        };

        // Where each 65816 vector's pointer lives in bank 0, native mode's then emulation mode's.
        public static readonly IReadOnlyList<InterruptVector> Vectors = new InterruptVector[]
        {
            new("COP", 0x00FFE4, 2, "Native (E=0)"),
            new("BRK", 0x00FFE6, 2, "Native (E=0)"),
            new("ABORT", 0x00FFE8, 2, "Native (E=0)"),
            new("NMI", 0x00FFEA, 2, "Native (E=0)"),
            new("IRQ", 0x00FFEE, 2, "Native (E=0)"),
            new("COP", 0x00FFF4, 2, "Emulation (E=1)"),
            new("ABORT", 0x00FFF8, 2, "Emulation (E=1)"),
            new("NMI", 0x00FFFA, 2, "Emulation (E=1)"),
            new("RESET", 0x00FFFC, 2, "Emulation (E=1)"),
            new("IRQ/BRK", 0x00FFFE, 2, "Emulation (E=1)"),
        };

        public IReadOnlyList<InterruptVector> InterruptVectors => Vectors;

        // P's bits by the names `eval` takes, and the registers' other names.
        private static readonly (string Name, int Bit)[] Flags =
            { ("flag.c", 0), ("flag.z", 1), ("flag.i", 2), ("flag.d", 3), ("flag.x", 4), ("flag.m", 5), ("flag.v", 6), ("flag.n", 7) };

        private static readonly (string Alias, string Register)[] Aliases = { ("sp", "S"), ("dp", "D"), ("pbr", "PB"), ("dbr", "DB") };

        private static readonly string[] Names = BuildNames();

        private static string[] BuildNames()
        {
            var names = new List<string>();
            foreach (var (alias, _) in Aliases) names.Add(alias);
            foreach (var (flag, _) in Flags) names.Add(flag);
            return names.ToArray();
        }

        public IReadOnlyList<string> SymbolNames => Names;

        public long? Symbol(string name, IReadOnlyList<DebugRegisterValue> registers)
        {
            foreach (var (flag, bit) in Flags)
                if (string.Equals(name, flag, StringComparison.OrdinalIgnoreCase)) return Register(registers, "P") is { } p ? (long)((p >> bit) & 1) : null;
            foreach (var (alias, register) in Aliases)
                if (string.Equals(name, alias, StringComparison.OrdinalIgnoreCase)) return Register(registers, register) is { } v ? (long)v : null;
            return null;
        }

        public string? Summary(IReadOnlyList<DebugRegisterValue> registers)
        {
            ulong? R(string n) => Register(registers, n);
            if (R("PB") is not { } pb || R("PC") is not { } pc || R("A") is not { } a || R("X") is not { } x || R("Y") is not { } y
                || R("S") is not { } s || R("D") is not { } d || R("DB") is not { } db || R("P") is not { } p || R("E") is not { } e) return null;
            return CpuStateLine((byte)pb, (ushort)pc, (ushort)a, (ushort)x, (ushort)y, (ushort)s, (ushort)d, (byte)db, (byte)p, e != 0);
        }

        // The 65816's state on one line, P's letters upper case when set; in emulation mode bits 5 and 4 read '-' and B.
        public static string CpuStateLine(byte pb, ushort pc, ushort a, ushort x, ushort y, ushort s, ushort d, byte db, byte p, bool emulation)
        {
            var flags = new StringBuilder(8);
            void Bit(int bit, char letter) => flags.Append((p & (1 << bit)) != 0 ? char.ToUpperInvariant(letter) : letter);
            Bit(7, 'n');
            Bit(6, 'v');
            if (emulation)
            {
                flags.Append('-');
                Bit(4, 'b');
            }
            else
            {
                Bit(5, 'm');
                Bit(4, 'x');
            }
            Bit(3, 'd');
            Bit(2, 'i');
            Bit(1, 'z');
            Bit(0, 'c');
            return $"[STATE] CPU  PC={pb:X2}:{pc:X4}  A={a:X4} X={x:X4} Y={y:X4} S={s:X4} D={d:X4} DB={db:X2}  P={p:X2} [{flags}]  E={(emulation ? 1 : 0)}";
        }

        private static int Word(IDebugMemorySpace space, int address) => space.Read(address) | (space.Read(address + 1) << 8);
    }
}
