using System;
using EmuSen.Debug;

namespace EmuSen.Video
{
    public partial class Ppu
    {
        // --- Write Operations ---

        private void WriteUnmapped(ushort offset, byte data) { }

        private void WriteINIDISP(ushort offset, byte data) { Inidisp = data; }
        
        private void WriteOBSEL(ushort offset, byte data) { Obsel = data; }
        
        private void WriteOAMADDL(ushort offset, byte data)
        {
            _oamAddr = (ushort)(((uint)_oamAddr & 0x0200u) | ((uint)data << 1));
        }

        private void WriteOAMADDH(ushort offset, byte data)
        {
            _oamAddr = (ushort)((((uint)data & 0x01u) << 9) | ((uint)_oamAddr & 0x01FFu));

            // Bit 7: OAM priority rotation - confirmed real via the SNESdev
            // wiki's Sprites page ("OAMADD can adjust this with 'priority
            // rotation'"). When enabled, sprite 0 is no longer necessarily
            // topmost; instead the sprite at (OAMAddr & 0xFE) >> 1 becomes
            // the first/topmost one, wrapping through all 128 from there.
            // Previously not parsed at all - this bit was silently dropped.
            PriorityRotationEnabled = (data & 0x80) != 0;
        }

        private void WriteOAMDATA(ushort offset, byte data)
        {
            if (_oamAddr < Oam.Length) Oam[_oamAddr] = data;
            _oamAddr = (ushort)((_oamAddr + 1) % 544);
        }

        private byte _lastBgmode;
        private void WriteBGMODE(ushort offset, byte data)
        {
            if (DebugSettings.BgModeChangeLogging && data != _lastBgmode)
            {
                Console.WriteLine($"[BGMODE] 0x{data:X2} mode={data & 0x07} bg3HighPri={(data >> 3) & 1} " +
                    $"tile16: bg1={(data >> 4) & 1} bg2={(data >> 5) & 1} bg3={(data >> 6) & 1} bg4={(data >> 7) & 1} " +
                    $"(scanline={CurrentScanline})");
            }
            _lastBgmode = data;
            Bgmode = data;
        }

        private void WriteBGSC(ushort offset, byte data)
        {
            int bg = offset - 0x2107;
            BgSc[bg] = data;
        }

        private void WriteBGNBA(ushort offset, byte data)
        {
            if (offset == 0x210B) Bg12Nba = data;
            else Bg34Nba = data;
        }

        private void WriteBGHOFS(ushort offset, byte data)
        {
            int bg = (offset - 0x210D) / 2;

            // BGnHOFS = (new<<8) | (Prev1 & ~7) | (Prev2 & 7); Prev1 = Prev2 = new
            //
            // The REAL fix for the horizontal-only scroll jitter investigated
            // at length earlier in this project's history (see the BG2
            // scroll jitter notes) - not the "borrow VOFS's simpler formula"
            // workaround that used to be here. The actual bug: Prev2 must be
            // a SEPARATE latch shared only by the four BGnHOFS registers,
            // confirmed via the Mesen team's own reference documentation
            // (snesdev.mesen.ca/wiki, PPU_Registers page - maintained by the
            // same team as the emulator's source, SnesPpu.cpp): "there is
            // only one Prev1 shared by all eight BGnxOFS registers, and only
            // one Prev2 shared by the four BGnHOFS registers." Note: the
            // active project is now called MesenCE (github.com/nesdev-org/
            // MesenCE) - the original SourMesen/Mesen2 repo was archived June
            // 2026 as a frozen snapshot; MesenCE is the direct, actively
            // maintained continuation of the same codebase, not a rewrite.
            // The previous implementation instead read THIS REGISTER'S OWN
            // previously-committed value for the Prev2 term, which silently
            // drops bit 2 of the low byte on every write - verified by
            // simulating both formulas against the exact real write data
            // captured earlier: the register's-own-value version reversed
            // direction repeatedly; this two-latch version reconstructs every
            // value exactly, zero reversals.
            BgScrollX[bg] = (ushort)(((data << 8) | (_bgOfsLatch & ~7) | (_bgHOfsLatch & 7)) & 0x3FF);

            if (DebugSettings.AllScrollWriteLogging) Console.WriteLine($"[SCROLL] BG{bg + 1} HOFS write: data=0x{data:X2} latchWas=0x{_bgOfsLatch:X2} -> BgScrollX[{bg}] now {BgScrollX[bg]}");
            _bgOfsLatch = data;
            _bgHOfsLatch = data;
            if (bg == 2 && DebugSettings.Bg3ScrollWriteLogging) Console.WriteLine($"[BG3SCROLL] HOFS write: offset=0x{offset:X4} data=0x{data:X2} -> BgScrollX[2] now {BgScrollX[2]}");

            // $210D also feeds Mode 7's own H-scroll (M7HOFS) via a SEPARATE
            // write-twice latch (_m7OfsLatch) from the BG-scroll ones above -
            // see its declaration in Ppu.cs. Runs unconditionally (not just
            // when Mode 7 is selected) since real hardware latches the value
            // regardless of the active mode; only whether it's ever read
            // back depends on that. Unaffected by the change above.
            if (bg == 0)
            {
                int m7Raw = ((data << 8) | _m7OfsLatch) & 0x1FFF;
                M7HOfs = (short)(m7Raw >= 0x1000 ? m7Raw - 0x2000 : m7Raw);
                _m7OfsLatch = data;
            }
        }

        private void WriteBGVOFS(ushort offset, byte data)
        {
            int bg = (offset - 0x210E) / 2;
            // BGnVOFS = (new<<8) | latch; latch = new
            BgScrollY[bg] = (ushort)(((data << 8) | _bgOfsLatch) & 0x3FF);
            if (DebugSettings.AllScrollWriteLogging) Console.WriteLine($"[SCROLL] BG{bg + 1} VOFS write: data=0x{data:X2} latchWas=0x{_bgOfsLatch:X2} -> BgScrollY[{bg}] now {BgScrollY[bg]}");
            _bgOfsLatch = data;
            if (bg == 2 && DebugSettings.Bg3ScrollWriteLogging) Console.WriteLine($"[BG3SCROLL] VOFS write: offset=0x{offset:X4} data=0x{data:X2} -> BgScrollY[2] now {BgScrollY[2]}");

            // Same piggyback as WriteBGHOFS above, for M7VOFS.
            if (bg == 0)
            {
                int m7Raw = ((data << 8) | _m7OfsLatch) & 0x1FFF;
                M7VOfs = (short)(m7Raw >= 0x1000 ? m7Raw - 0x2000 : m7Raw);
                _m7OfsLatch = data;
            }
        }


        private void WriteVMAIN(ushort offset, byte data) { _vmain = data; }

        private void WriteVMADDL(ushort offset, byte data)
        {
            _vramAddr = (ushort)(((uint)_vramAddr & 0xFF00u) | (uint)data);
        }

        private void WriteVMADDH(ushort offset, byte data)
        {
            _vramAddr = (ushort)(((uint)_vramAddr & 0x00FFu) | ((uint)data << 8));
        }

        private void WriteVMDATAL(ushort offset, byte data)
        {
            Vram[(_vramAddr * 2) & 0xFFFF] = data;
            _vramTouched[(_vramAddr * 2) & 0xFFFF] = true;
            if ((_vmain & 0x80) == 0) { _vramAddr = (ushort)(_vramAddr + VramStep()); }
        }

        private void WriteVMDATAH(ushort offset, byte data)
        {
            Vram[((_vramAddr * 2) + 1) & 0xFFFF] = data;
            _vramTouched[((_vramAddr * 2) + 1) & 0xFFFF] = true;
            if ((_vmain & 0x80) != 0) { _vramAddr = (ushort)(_vramAddr + VramStep()); }
        }

        private void WriteCGADD(ushort offset, byte data)
        {
            _cgadd = data;
            _cgLowByte = true;
        }

        private void WriteCGDATA(ushort offset, byte data)
        {
            if (_cgLowByte)
            {
                _cgLatch = data;
                _cgLowByte = false;
            }
            else
            {
                int idx = _cgadd; 
                int memIdx = (idx * 2) & 0x1FF;
                Cgram[memIdx] = _cgLatch;
                Cgram[memIdx + 1] = data;
                _cgramTouched[memIdx] = true;
                _cgramTouched[memIdx + 1] = true;

                if (DebugSettings.CgWriteLogging)
                {
                    Console.WriteLine($"[CGWRITE] color idx=0x{idx:X2} (pal={idx / 16} entry={idx % 16}) bytes=({_cgLatch:X2},{data:X2})");
                }

                ushort color16 = (ushort)(((uint)data << 8) | (uint)_cgLatch);

                uint r = (uint)(((color16 >> 0) & 0x001Fu) << 3);
                uint g = (uint)(((color16 >> 5) & 0x001Fu) << 3);
                uint b = (uint)(((color16 >> 10) & 0x001Fu) << 3);

                r |= (r >> 5);
                g |= (g >> 5);
                b |= (b >> 5);

                Palette[idx] = 0xFF000000u | (r << 16) | (g << 8) | b;

                _cgadd++; 
                _cgLowByte = true;
            }
        }

        private void WriteTM(ushort offset, byte data) { Tm = data; }
        private void WriteTS(ushort offset, byte data) { Ts = data;}
        private void WriteCGWSEL(ushort offset, byte data) { Cgwsel = data; }
        private void WriteCGADSUB(ushort offset, byte data) { Cgadsub = data; }
        private void WriteW12Sel(ushort offset, byte data) { W12Sel = data; }
        private void WriteW34Sel(ushort offset, byte data) { W34Sel = data; }
        private void WriteWObjSel(ushort offset, byte data) { WObjSel = data; }
        private void WriteWh0(ushort offset, byte data) { Wh0 = data; }
        private void WriteWh1(ushort offset, byte data) { Wh1 = data; }
        private void WriteWh2(ushort offset, byte data) { Wh2 = data; }
        private void WriteWh3(ushort offset, byte data) { Wh3 = data; }
        private void WriteWBgLog(ushort offset, byte data) { WBgLog = data; }
        private void WriteWObjLog(ushort offset, byte data) { WObjLog = data; }
        private void WriteTmw(ushort offset, byte data) { Tmw = data; }
        private void WriteTsw(ushort offset, byte data) { Tsw = data; }
        private void WriteMOSAIC(ushort offset, byte data)
        {
            Mosaic = data;
            if (DebugSettings.MosaicWriteLogging)
            {
                Console.WriteLine($"[MOSAIC] wrote 0x{data:X2} size={((data >> 4) & 0x0F) + 1} " +
                    $"enable(bg1-4)={data & 0x0F:X1} at scanline={CurrentScanline}" +
                    (CurrentScanline is >= 1 and <= 223 ? " [MID-FRAME - anchor overridden]" : " [vblank/frame-start - anchor stays 0]"));
            }
            // See _mosaicStartScanline's declaration in Ppu.cs. A write
            // landing during vblank (the common case) sets this to a
            // scanline number that OnScanlineStart will immediately reset to
            // 0 the moment the next frame's scanline 0 begins, so it only
            // "sticks" for writes that happen during the active display.
            _mosaicStartScanline = CurrentScanline;
        }

        private void WriteM7SEL(ushort offset, byte data) { M7Sel = data; }

        // M7A-D/M7X/M7Y share one latch (_m7Latch), separate from both the
        // BG-scroll latch and the M7HOFS/VOFS latch - see the field comments
        // in Ppu.cs. Same "latch on this write, combine with latch on next"
        // shape as everywhere else in this file, just without BG-scroll's
        // bit-shuffling since these are plain 16-bit (or 13-bit signed for
        // X/Y) values that split cleanly across two 8-bit writes.
        private void WriteM7A(ushort offset, byte data) { M7A = (short)((data << 8) | _m7Latch); _m7Latch = data; }
        private void WriteM7B(ushort offset, byte data)
        {
            M7B = (short)((data << 8) | _m7Latch);
            _m7Latch = data;
            // MPYL/M/H ($2134-$2136) read back M7A * this raw byte - NOT
            // M7A * the combined 16-bit M7B - confirmed via ff6hacking's
            // register docs: "the product of the 16-bit value written to
            // $211B and the 8-bit value most recently written to $211C".
            // Updates on every write to $211C (both halves of the pair),
            // matching "most recently written" literally.
            _m7bLastByte = (sbyte)data;
        }
        private void WriteM7C(ushort offset, byte data) { M7C = (short)((data << 8) | _m7Latch); _m7Latch = data; }
        private void WriteM7D(ushort offset, byte data) { M7D = (short)((data << 8) | _m7Latch); _m7Latch = data; }

        private void WriteM7X(ushort offset, byte data)
        {
            int raw = ((data << 8) | _m7Latch) & 0x1FFF;
            M7X = (short)(raw >= 0x1000 ? raw - 0x2000 : raw);
            _m7Latch = data;
        }

        private void WriteM7Y(ushort offset, byte data)
        {
            int raw = ((data << 8) | _m7Latch) & 0x1FFF;
            M7Y = (short)(raw >= 0x1000 ? raw - 0x2000 : raw);
            _m7Latch = data;
        }

        // SETINI ($2133): stored so writes don't get lost, but the modes it
        // controls aren't implemented - interlace (would need a real
        // alternating-field frame count, not just a flag), pseudo-hires
        // (needs the color-math blend path to consider it), EXTBG (needs
        // Mode 7, currently skipped), and overscan (would need the renderer's
        // fixed 224-line ScreenH to become dynamic). None of these are things
        // a stored byte alone can fix - flagging honestly rather than
        // pretending this register does something it doesn't yet.
        private void WriteSETINI(ushort offset, byte data) { Setini = data; }
        // --- Read Operations ---

        private byte ReadUnmapped(ushort offset) { return 0x00; }

        private byte ReadOAMDATA(ushort offset)
        {
            byte v = _oamAddr < Oam.Length ? Oam[_oamAddr] : (byte)0;
            _oamAddr = (ushort)((_oamAddr + 1) % 544);
            return v;
        }

        private byte ReadVMDATAL(ushort offset)
        {
            byte v = Vram[(_vramAddr * 2) & 0xFFFF];
            if ((_vmain & 0x80) == 0) { _vramAddr = (ushort)(_vramAddr + VramStep()); }
            return v;
        }

        private byte ReadVMDATAH(ushort offset)
        {
            byte v = Vram[((_vramAddr * 2) + 1) & 0xFFFF];
            if ((_vmain & 0x80) != 0) { _vramAddr = (ushort)(_vramAddr + VramStep()); }
            return v;
        }

        private byte ReadCGDATA(ushort offset)
        {
            return Cgram[(_cgadd * 2) & 0x1FF];
        }

        // MPYL/M/H ($2134-$2136) - see Mpy's declaration in Ppu.cs for what
        // it actually multiplies.
        private byte ReadMPYL(ushort offset) => (byte)(Mpy & 0xFF);
        private byte ReadMPYM(ushort offset) => (byte)((Mpy >> 8) & 0xFF);
        private byte ReadMPYH(ushort offset) => (byte)((Mpy >> 16) & 0xFF);

        // SLHV ($2137): reading it latches the current H/V position into
        // OPHCT/OPVCT. Real hardware also latches this way via WRIO ($4201)
        // bit 7 transitions and the Super Scope's trigger - neither of those
        // paths are wired up here, only the direct $2137 read is. The
        // returned byte itself is genuine open bus on real hardware; 0 is as
        // good a stand-in as any.
        private byte ReadSLHV(ushort offset)
        {
            _latchedV = (ushort)CurrentScanline;
            // H is not tracked - see CurrentLineCycles's comment. Always 0
            // rather than a fabricated conversion from LineCycles, since this
            // renderer has no real per-dot H position to report.
            _latchedH = 0;
            return 0;
        }

        // OPHCT/OPVCT ($213C/$213D): each is a 9-bit value read as two
        // sequential 8-bit reads (low byte first, then high byte in bit 0 -
        // bits 1-7 of the high read are PPU2 open bus, returned as 0 here).
        // Each register tracks its own low/high toggle independently; only
        // reading STAT78 ($213F) resets both back to "low" (see ReadSTAT78).
        private byte ReadOPHCT(ushort offset)
        {
            byte result = !_ophctHigh ? (byte)(_latchedH & 0xFF) : (byte)((_latchedH >> 8) & 0x01);
            _ophctHigh = !_ophctHigh;
            return result;
        }

        private byte ReadOPVCT(ushort offset)
        {
            byte result = !_opvctHigh ? (byte)(_latchedV & 0xFF) : (byte)((_latchedV >> 8) & 0x01);
            _opvctHigh = !_opvctHigh;
            return result;
        }

        // STAT77 ($213E): trm-vvvv - Time Over, Range Over, Master/slave
        // select (always 0 here - real consoles almost universally read this
        // as 0 too), PPU1 version in the low nibble (1 is what most real
        // units report).
        private byte ReadSTAT77(ushort offset)
        {
            byte result = 0x01; // PPU1 version = 1
            if (TimeOver) result |= 0x80;
            if (RangeOver) result |= 0x40;
            return result;
        }

        // STAT78 ($213F): flupvvvv - interlace field (bit 7, now tracked via
        // FieldParity - see its declaration in Ppu.cs for the full citation
        // and what is/isn't implemented around it), NTSC/PAL region (0 =
        // NTSC, matching the 262-scanline timing already used throughout
        // this project), PPU2 version in the low nibble. Reading this also
        // resets the OPHCT/OPVCT high/low toggle back to "low", per
        // documented hardware behavior.
        private byte ReadSTAT78(ushort offset)
        {
            _ophctHigh = false;
            _opvctHigh = false;
            byte result = 0x01; // PPU2 version = 1
            if (FieldParity) result |= 0x80;
            return result;
        }
    }
}
