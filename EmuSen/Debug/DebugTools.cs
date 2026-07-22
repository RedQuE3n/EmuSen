using System;
using System.Collections.Generic;
using System.Text;

namespace EmuSen.Debug
{
    // Reusable debugging helpers, pulled out of the one-off diagnostic code we've
    // hand-written repeatedly throughout this project (tile decoding, CGRAM-to-RGB
    // conversion, hex dumps, bitfield breakdowns). Deliberately dependency-free -
    // no Raylib, no SNES-specific types - just byte arrays and primitives, so
    // these stay reusable for a future NES core too, not just this one.
    //
    // These are meant to be called from behind a DebugSettings toggle or a
    // keypress-triggered trace, the same way the rest of the diagnostic code in
    // this project works - none of this runs unconditionally.
    public static class DebugTools
    {
        // Formats `count` bytes starting at `address` (wrapping within a 64KB
        // space, same convention VRAM/CGRAM addressing already uses elsewhere)
        // as a single space-separated hex line, prefixed with a label.
        public static string HexDump(string label, byte[] data, int address, int count)
        {
            var sb = new StringBuilder($"[{label}] bytes @ 0x{address:X4}: ");
            for (int i = 0; i < count; i++)
            {
                sb.Append($"{data[(address + i) % data.Length]:X2} ");
            }
            return sb.ToString();
        }

        // Decodes a single tile at `address` in `vram` as an 8x8 grid of pixel
        // index values (0 = transparent) and renders it as a multi-line ASCII
        // picture - one digit per pixel, '.' for transparent. bpp must be 2 or 4.
        // 2bpp tiles are 16 bytes; 4bpp tiles are 32 bytes, laid out as two
        // 2bpp-style planes back to back (bytes 0-15 = bitplanes 0/1 row-
        // interleaved, bytes 16-31 = bitplanes 2/3 the same way) - this is the
        // standard SNES tile format, consistent across every layer.
        public static string DecodeTileAscii(byte[] vram, int address, int bpp)
        {
            if (bpp != 2 && bpp != 4)
            {
                return $"[DecodeTileAscii] unsupported bpp={bpp} (must be 2 or 4)";
            }

            var sb = new StringBuilder();
            for (int row = 0; row < 8; row++)
            {
                byte p0 = vram[(address + row * 2) % vram.Length];
                byte p1 = vram[(address + row * 2 + 1) % vram.Length];
                byte p2 = 0, p3 = 0;
                if (bpp == 4)
                {
                    p2 = vram[(address + 16 + row * 2) % vram.Length];
                    p3 = vram[(address + 16 + row * 2 + 1) % vram.Length];
                }

                sb.Append("  ");
                for (int col = 0; col < 8; col++)
                {
                    int bit = 7 - col;
                    int val = ((p0 >> bit) & 1) | (((p1 >> bit) & 1) << 1);
                    if (bpp == 4)
                    {
                        val |= (((p2 >> bit) & 1) << 2) | (((p3 >> bit) & 1) << 3);
                    }
                    sb.Append(val == 0 ? '.' : val.ToString("X1"));
                    sb.Append(' ');
                }
                sb.Append('\n');
            }
            return sb.ToString();
        }

        // Converts a raw SNES BGR555 color (two CGRAM bytes) into 8-bit RGB.
        // Same formula as Renderer's own SnesColor, duplicated here rather than
        // shared because that one returns a Raylib Color and this file is
        // deliberately kept Raylib-free.
        public static (byte r, byte g, byte b) CgramToRgb(byte lo, byte hi, float brightness = 1f)
        {
            int c = lo | (hi << 8);
            int r = (c & 0x1F) << 3;
            int g = ((c >> 5) & 0x1F) << 3;
            int b = ((c >> 10) & 0x1F) << 3;
            return ((byte)(r * brightness), (byte)(g * brightness), (byte)(b * brightness));
        }

        // Dumps every color in one palette group as index/raw-value/RGB, one line
        // each. bpp determines colors-per-palette: 4 for 2bpp graphics, 16 for
        // 4bpp. paletteIndex is which group (0-7 for 2bpp, 0-7 for 4bpp - SNES
        // always has 8 palette groups per bit depth, just different sizes).
        public static string DumpPalette(byte[] cgram, int paletteIndex, int bpp, float brightness = 1f)
        {
            int colorsPerPalette = bpp == 2 ? 4 : 16;
            int baseIndex = paletteIndex * colorsPerPalette;

            var sb = new StringBuilder($"[DumpPalette] palette {paletteIndex} ({bpp}bpp, base CGRAM index 0x{baseIndex:X2}):\n");
            for (int i = 0; i < colorsPerPalette; i++)
            {
                int cgIndex = baseIndex + i;
                int byteOffset = (cgIndex * 2) % cgram.Length;
                byte lo = cgram[byteOffset];
                byte hi = cgram[(byteOffset + 1) % cgram.Length];
                var (r, g, b) = CgramToRgb(lo, hi, brightness);
                sb.Append($"  color {i}: CGRAM ${cgIndex:X2} = 0x{(lo | (hi << 8)):X4} -> RGB ({r},{g},{b})\n");
            }
            return sb.ToString();
        }

        // Describes which named bits are set in a byte, e.g. for quickly reading
        // a register like CGADSUB or $4200 without manually masking bits each
        // time. Pass (bit, name) pairs; unset bits are omitted from the output.
        // Example: DescribeBits("CGADSUB", ppu.Cgadsub, (7,"subtract"), (6,"half"), (5,"mathEnabled"))
        public static string DescribeBits(string label, byte value, params (int bit, string name)[] bits)
        {
            var set = new StringBuilder();
            foreach (var (bit, name) in bits)
            {
                if ((value & (1 << bit)) != 0)
                {
                    if (set.Length > 0) set.Append(", ");
                    set.Append(name);
                }
            }
            return $"[{label}] 0x{value:X2} -> [{(set.Length > 0 ? set.ToString() : "none set")}]";
        }

        // Small reusable helper for the "trigger a bounded trace on a keypress"
        // pattern used throughout this project (e.g. the BG scroll trace, the
        // IRQ instruction trace). Call Start() when the trigger fires, then
        // check ShouldLog() each frame/step - it counts down and returns false
        // once exhausted, so callers don't need to hand-roll a countdown field.
        public class BoundedTrace
        {
            private int _remaining;

            public void Start(int frames)
            {
                _remaining = frames;
            }

            public bool ShouldLog()
            {
                if (_remaining <= 0) return false;
                _remaining--;
                return true;
            }

            public bool IsActive => _remaining > 0;
        }

        // Reusable "only log when this actually changed" helper - the same
        // pattern already hand-rolled three separate times in this project
        // (CameraRamLogging in MemoryBus.cs, CgWriteLogging in Ppu.cs,
        // HvIrqChangeLogging in MemoryBus.cs each keep their own "was" field
        // and do their own equality check). One of these per value you want
        // to watch replaces that boilerplate:
        //
        //   private readonly ChangeTracker<byte> _mosaicTracker = new();
        //   ...
        //   if (_mosaicTracker.Changed(data)) Console.WriteLine($"[MOSAIC] now 0x{data:X2}");
        //
        // Changed() returns true (and updates the stored value) whenever the
        // new value differs from the last one seen - including the very
        // first call, so there's no separate "have we seen a value yet" flag
        // to manage either.
        public class ChangeTracker<T>
        {
            private T? _last;
            private bool _hasValue;

            public bool Changed(T newValue)
            {
                if (_hasValue && EqualityComparer<T>.Default.Equals(_last, newValue))
                {
                    return false;
                }
                _last = newValue;
                _hasValue = true;
                return true;
            }

            public T? Last => _last;
        }
    }
}
