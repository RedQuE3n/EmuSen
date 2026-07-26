using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Debug;

namespace EmuSen.Shell
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

        // Collapses a short repeating cycle of trace entries into one
        // summary instead of writing every repeat separately. Built for
        // CpuVerboseLogging/Spc700VerboseLogging: a real session produced a
        // 1GB cpu.log almost entirely from polling/delay loops (VBlank
        // wait, DMA busy-wait, the APU handshake) executing the same
        // handful of instructions thousands of times in a row - typically
        // 2+ instructions (load/branch, decrement/branch), not a single
        // repeated line, which is why this detects cycle lengths 1..8, not
        // just immediate duplicates.
        //
        // Compares on a caller-supplied value key (e.g. PB/PC/opcode/target
        // address as a small struct), not the rendered string - a locked-in
        // loop costs one struct comparison per call, no allocation. Render
        // only happens for entries that actually get emitted, so the
        // string-formatting cost that used to run every single instruction
        // now only runs for instructions that are actually written out.
        //
        // Nothing is ever dropped: every key that comes in either gets
        // rendered individually or is exactly reconstructable from the
        // confirmed cycle template plus its repeat count - see Flush()'s
        // own comment for why callers still need to call it.
        public sealed class RepeatCollapsingTrace<TKey> where TKey : IEquatable<TKey>
        {
            // Long enough for the short polling/delay-loop bodies real
            // 65816/SPC700 code actually uses, short enough that scanning
            // for a not-yet-confirmed cycle (the only part that isn't O(1))
            // stays cheap.
            private const int MaxCycleLength = 8;

            // Below this many repeats, collapsing would replace a handful
            // of duplicate lines with a marker line that's often longer
            // than what it saved - not worth the noise, so small incidental
            // matches (two instructions that just happen to repeat once,
            // not an actual loop) get written out in full instead.
            private const int MinRepeatsToCollapse = 4;

            private readonly Action<string> _sink;
            private readonly Func<TKey, string> _render;
            private readonly Func<int, int, string> _renderMarker;
            private readonly List<TKey> _pending = new();

            private TKey[]? _confirmedCycle;
            private int _posInCycle;
            private int _cycleRepeats;

            // renderMarker(cycleLength, repeats) builds the "collapsed"
            // summary line - callers must give it the same [TAG] prefix
            // render() uses. A tagless marker (the original version of
            // this just hardcoded "    ^ ...") can't be routed by
            // CategorizedLogWriter's prefix table, so it falls through to
            // the "general" category - which isn't in the cpu/apu
            // console-echo suppression list, so every collapsed loop
            // ended up blasting the live console again despite that fix.
            public RepeatCollapsingTrace(Action<string> sink, Func<TKey, string> render, Func<int, int, string> renderMarker)
            {
                _sink = sink;
                _render = render;
                _renderMarker = renderMarker;
            }

            public void Log(TKey key)
            {
                if (_confirmedCycle != null)
                {
                    if (key.Equals(_confirmedCycle[_posInCycle]))
                    {
                        _posInCycle++;
                        if (_posInCycle == _confirmedCycle.Length)
                        {
                            _posInCycle = 0;
                            _cycleRepeats++;
                        }
                        return;
                    }
                    FlushConfirmedCycle();
                }

                _pending.Add(key);
                TryConfirmCycle();

                // No cycle has formed here and isn't going to - a cycle of
                // length <= MaxCycleLength must already show up within the
                // most recent 2*MaxCycleLength entries, so anything older
                // than that can be emitted now instead of held forever.
                int cap = MaxCycleLength * 2;
                while (_pending.Count > cap)
                {
                    _sink(_render(_pending[0]));
                    _pending.RemoveAt(0);
                }
            }

            private void TryConfirmCycle()
            {
                int count = _pending.Count;
                for (int c = 1; c <= MaxCycleLength && count >= c * 2; c++)
                {
                    bool match = true;
                    for (int i = 0; i < c; i++)
                    {
                        if (!_pending[count - 2 * c + i].Equals(_pending[count - c + i]))
                        {
                            match = false;
                            break;
                        }
                    }
                    if (!match) continue;

                    // Everything before the matched pair is unrelated,
                    // never-repeated content - it can't retroactively join
                    // this cycle, so it goes out individually now.
                    for (int i = 0; i < count - 2 * c; i++) _sink(_render(_pending[i]));

                    _confirmedCycle = new TKey[c];
                    for (int i = 0; i < c; i++) _confirmedCycle[i] = _pending[count - c + i];
                    _cycleRepeats = 2;
                    _posInCycle = 0;
                    _pending.Clear();
                    return;
                }
            }

            private void FlushConfirmedCycle()
            {
                if (_confirmedCycle == null) return;

                if (_cycleRepeats < MinRepeatsToCollapse)
                {
                    for (int r = 0; r < _cycleRepeats; r++)
                    {
                        foreach (TKey key in _confirmedCycle) _sink(_render(key));
                    }
                }
                else
                {
                    foreach (TKey key in _confirmedCycle) _sink(_render(key));
                    _sink(_renderMarker(_confirmedCycle.Length, _cycleRepeats));
                }

                // Partial trailing repeat - lines that matched again but the
                // cycle broke before completing another full pass. They
                // genuinely executed and are byte-identical to these
                // template positions, so replay them rather than drop them.
                for (int i = 0; i < _posInCycle; i++) _sink(_render(_confirmedCycle[i]));

                _confirmedCycle = null;
                _cycleRepeats = 0;
                _posInCycle = 0;
            }

            // Call this whenever a still-in-progress cycle or unconfirmed
            // tail would otherwise never reach the sink at all - when
            // verbose logging is toggled off mid-session and on process
            // exit. Without it, a loop still running (or a short tail that
            // never got the chance to prove itself as a cycle) at either of
            // those moments would be silently lost.
            public void Flush()
            {
                FlushConfirmedCycle();
                foreach (TKey key in _pending) _sink(_render(key));
                _pending.Clear();
            }
        }
    }
}
