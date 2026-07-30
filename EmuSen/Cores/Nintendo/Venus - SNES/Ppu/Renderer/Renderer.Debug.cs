using System;
using System.Numerics;
using Raylib_cs;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.Graphics;

namespace EmuSen.Cores.Nintendo.Venus.Video
{
    public partial class Renderer
    {
        private void RenderVramSheet(Ppu ppu)
        {
            for (int tile = 0; tile < 2048; tile++)
            {
                int baseX = (tile % 32) * 8;
                int baseY = (tile / 32) * 8;
                int tileAddr = tile * 32;

                for (int r = 0; r < 8; r++)
                {
                    byte p0 = ppu.Vram[(tileAddr + r * 2) & 0xFFFF];
                    byte p1 = ppu.Vram[(tileAddr + r * 2 + 1) & 0xFFFF];
                    byte p2 = ppu.Vram[(tileAddr + 16 + r * 2) & 0xFFFF];
                    byte p3 = ppu.Vram[(tileAddr + 16 + r * 2 + 1) & 0xFFFF];

                    for (int cbit = 7; cbit >= 0; cbit--)
                    {
                        int pixel = ((p0 >> cbit) & 1) | (((p1 >> cbit) & 1) << 1) | (((p2 >> cbit) & 1) << 2) | (((p3 >> cbit) & 1) << 3);
                        byte v = (byte)(pixel * 17);
                        _sheetPixels[(baseY + r) * SheetW + baseX + (7 - cbit)] = new Color(v, v, v, (byte)255);
                    }
                }
            }
        }

        public System.Collections.Generic.List<(string label, int x, int y, int w, int h)> DumpActiveOam(Ppu ppu)
        {
            int sizeSelect = (ppu.Obsel >> 5) & 0x07;
            (int w, int h) small = ObjSmallSize[sizeSelect];
            (int w, int h) large = ObjLargeSize[sizeSelect];
            int objBase = (ppu.Obsel & 0x07) << 14;

            Console.WriteLine($"[OAM DUMP] OBSEL=0x{ppu.Obsel:X2} objBase=0x{objBase:X4} sizeSelect={sizeSelect} small={small.w}x{small.h} large={large.w}x{large.h} TM=0x{ppu.Tm:X2} TS=0x{ppu.Ts:X2} BGMODE=0x{ppu.Bgmode:X2} CGWSEL=0x{ppu.Cgwsel:X2} CGADSUB=0x{ppu.Cgadsub:X2}");

            var rects = new System.Collections.Generic.List<(string label, int x, int y, int w, int h)>();
            int activeCount = 0;
            for (int i = 0; i < 128; i++)
            {
                int oamIdx = i * 4;
                int x = ppu.Oam[oamIdx];
                int y = ppu.Oam[oamIdx + 1];
                int tileLow = ppu.Oam[oamIdx + 2];
                int attr = ppu.Oam[oamIdx + 3];

                int highTableIdx = 512 + (i / 4);
                int highBits = (ppu.Oam[highTableIdx] >> ((i % 4) * 2)) & 0x03;

                bool useLarge = (highBits & 0x02) != 0;
                int tile = tileLow | ((attr & 0x01) << 8);

                if (y == 0xE0 || y == 0xF0) continue;

                activeCount++;
                int signedX = (highBits & 0x01) != 0 ? x - 256 : x;
                int pal = 8 + ((attr & 0x0E) >> 1);
                Console.WriteLine($"[OAM DUMP]  #{i}: x={signedX} y={y} tile=0x{tile:X3} attr=0x{attr:X2} large={useLarge} pal={pal} flipX={(attr & 0x40) != 0} flipY={(attr & 0x80) != 0}");

                // Prints this sprite's actual 16-color CGRAM palette (32
                // bytes: 16 colors x 2 bytes each, BGR555). Added to
                // directly test whether a sprite with real, non-zero tile
                // pixel data (confirmed via the tile dump below) is still
                // invisible because its palette was never written with
                // real color data, as opposed to a rendering-side bug -
                // if every byte here is 0x00, that's the answer; if there
                // are non-zero, non-identical values, the palette itself
                // is fine and the bug is elsewhere.
                int palByteBase = (128 + (pal - 8) * 16) * 2;
                var palSb = new System.Text.StringBuilder($"[OAM DUMP]    pal{pal} CGRAM @ byte 0x{palByteBase:X3}: ");
                for (int c = 0; c < 16; c++)
                {
                    int off = (palByteBase + c * 2) & 0x1FF;
                    palSb.Append($"{ppu.Cgram[off]:X2}{ppu.Cgram[off + 1]:X2} ");
                }
                Console.WriteLine(palSb.ToString());

                if (useLarge)
                {
                    int tileRow = (tile >> 4) & 0x1F;
                    int tileCol = tile & 0x0F;
                    int tileAddr = (objBase + ((tileRow << 4) | tileCol) * 32) & 0xFFFF;
                    var sb = new System.Text.StringBuilder($"[OAM DUMP]    tile 0x{tile:X3} @ VRAM 0x{tileAddr:X4}: ");
                    for (int b = 0; b < 32; b++)
                    {
                        sb.Append($"{ppu.Vram[(tileAddr + b) & 0xFFFF]:X2} ");
                    }
                    Console.WriteLine(sb.ToString());
                }

                (int w, int h) size = useLarge ? large : small;
                rects.Add(($"#{i}", signedX, y, size.w, size.h));
            }

            Console.WriteLine($"[OAM DUMP] {activeCount} active sprite(s) out of 128");
            return rects;
        }

        // Moved out of DrawFrame (now gone - see FramePresenter/DrawDebugPanels)
        // since this is pure Console logging with no render-target dependency;
        // it just needs to run once per O keypress like every other hotkey.
        public void DumpBackdropAndWindowDebugInfo(Ppu ppu, long frame)
        {
            Console.WriteLine($"[OAM DUMP] --- Frame {frame} ---");
            DumpActiveOam(ppu);
            DumpBlackBg1Tiles(ppu);

            // Log the exact backdrop compositing math, since we now suspect the
            // "black squares" are actually transparent BG1 pixels correctly
            // revealing a WRONG backdrop, not bad tile/palette data (both of
            // which just checked out fine).
            float brightness = (ppu.Inidisp & 0x0F) / 15f;
            Color mainBackdrop = SnesColor(ppu.Cgram[0], ppu.Cgram[1], brightness);
            Color subBackdrop = new Color(
                (byte)(((ppu.FixedColorR & 0x1F) << 3) * brightness),
                (byte)(((ppu.FixedColorG & 0x1F) << 3) * brightness),
                (byte)(((ppu.FixedColorB & 0x1F) << 3) * brightness),
                (byte)255
            );
            bool subtractMode = (ppu.Cgadsub & 0x80) != 0;
            bool halfMode = (ppu.Cgadsub & 0x40) != 0;
            bool backdropMathEnabled = (ppu.Cgadsub & 0x20) != 0;
            Color blended = backdropMathEnabled ? BlendColors(mainBackdrop, subBackdrop, subtractMode, halfMode) : mainBackdrop;

            Console.WriteLine($"[BACKDROP] CGRAM[0]=0x{ppu.Cgram[0]:X2}{ppu.Cgram[1]:X2} (ever written: {ppu.WasCgramTouched(0) || ppu.WasCgramTouched(1)}) -> mainBackdrop=({mainBackdrop.R},{mainBackdrop.G},{mainBackdrop.B})");
            Console.WriteLine($"[BACKDROP] FixedColor R={ppu.FixedColorR} G={ppu.FixedColorG} B={ppu.FixedColorB} (2132 ever written: {ppu.FixedColorEverWritten}) -> subBackdrop=({subBackdrop.R},{subBackdrop.G},{subBackdrop.B})");
            Console.WriteLine($"[BACKDROP] CGADSUB=0x{ppu.Cgadsub:X2} backdropMathEnabled={backdropMathEnabled} subtract={subtractMode} half={halfMode} -> FINAL BACKDROP=({blended.R},{blended.G},{blended.B})");

            Console.WriteLine($"[WINDOW] W12SEL=0x{ppu.W12Sel:X2} W34SEL=0x{ppu.W34Sel:X2} WOBJSEL=0x{ppu.WObjSel:X2}");
            Console.WriteLine($"[WINDOW] WH0(w1left)={ppu.Wh0} WH1(w1right)={ppu.Wh1} WH2(w2left)={ppu.Wh2} WH3(w2right)={ppu.Wh3}");
            Console.WriteLine($"[WINDOW] WBGLOG=0x{ppu.WBgLog:X2} WOBJLOG=0x{ppu.WObjLog:X2} TMW=0x{ppu.Tmw:X2} TSW=0x{ppu.Tsw:X2}");
        }

        private void DumpBlackBg1Tiles(Ppu ppu)
        {
            // Scan the just-rendered frame for pure black pixels and, for each
            // distinct one, recompute exactly which BG1 tile/palette produced it -
            // faster and more precise than inferring it from a screenshot. Sampled
            // on a grid (every 4th pixel) to keep the output short rather than
            // reporting the same handful of tiles hundreds of times.
            int mapBase = (ppu.BgSc[0] & 0xFC) << 9;
            int sizeBits = ppu.BgSc[0] & 0x03;

            var seen = new System.Collections.Generic.HashSet<(int tile, int pal)>();
            int blackPixelCount = 0;

            for (int py = 0; py < ScreenH; py += 4)
            {
                for (int px = 0; px < ScreenW; px += 4)
                {
                    Color c = _screenPixels[py * ScreenW + px];
                    if (c.R != 0 || c.G != 0 || c.B != 0) continue;
                    blackPixelCount++;

                    int wy = (py + ppu.BgScrollY[0]) % ((sizeBits & 0x02) != 0 ? 512 : 256);
                    int wx = (px + ppu.BgScrollX[0]) % ((sizeBits & 0x01) != 0 ? 512 : 256);
                    int ty = wy >> 3;
                    int tx = wx >> 3;

                    int entryAddr = BgTilemapEntryAddress(mapBase, sizeBits, tx, ty);
                    int entry = ppu.Vram[entryAddr] | (ppu.Vram[(entryAddr + 1) & 0xFFFF] << 8);
                    int tile = entry & 0x3FF;
                    int pal = (entry >> 10) & 0x07;

                    if (seen.Add((tile, pal)))
                    {
                        Console.WriteLine($"[BLACK TILE] BG1 tile=0x{tile:X3} pal={pal} at approx screen ({px},{py}) entryAddr=0x{entryAddr:X4} priority={(entry & 0x2000) != 0}");

                        if (seen.Count == 1)
                        {
                            // BG3 scroll sanity check: if BG3 is meant to move
                            // together with BG1's cave walls, their scroll values
                            // should be close/related. Wildly different values would
                            // suggest we're reading BG3 from the wrong world position
                            // entirely, not that its data is actually wrong.
                            Console.WriteLine($"[BG3 CHECK] BG1 scroll X={ppu.BgScrollX[0]} Y={ppu.BgScrollY[0]}  |  BG3 scroll X={ppu.BgScrollX[2]} Y={ppu.BgScrollY[2]}");
                            int bg3ChrBase = (ppu.Bg34Nba & 0x0F) << 13;
                            Console.WriteLine($"[BG3 CHECK] BG3Sc(tilemap base+size)=0x{ppu.BgSc[2]:X2}  BG34NBA(chr base)=0x{ppu.Bg34Nba:X2}  ->  BG3 chrBase byte addr = 0x{bg3ChrBase:X4} (Mesen reference: 0x8000-0xBFF0)");

                            // Dump the raw 16 bytes of BG3 tile 0x031 (the specific
                            // one the diagnostic keeps flagging) so we can compare
                            // against what Mesen shows for the same tile. 2bpp tile
                            // = 16 bytes.
                            int tile031Addr = (bg3ChrBase + 0x031 * 16) & 0xFFFF;
                            var sb31 = new System.Text.StringBuilder($"[BG3 CHECK] tile 0x031 @ 0x{tile031Addr:X4}: ");
                            for (int b = 0; b < 16; b++) sb31.Append($"{ppu.Vram[(tile031Addr + b) & 0xFFFF]:X2} ");
                            Console.WriteLine(sb31.ToString());

                            // Decode those 16 bytes as a 2bpp 8x8 tile and print it as
                            // an ASCII picture. 2bpp SNES tile layout: 8 rows, each
                            // row = 2 bytes (low bitplane, high bitplane), giving a
                            // per-pixel value of 0-3. If our shape doesn't match what
                            // Mesen displays for this tile, we've got a decode bug;
                            // if the shape matches but colors don't, it's a palette
                            // question, not a tile-data question.
                            Console.WriteLine("[BG3 CHECK] decoded tile 0x031 (0=transparent):");
                            for (int row = 0; row < 8; row++)
                            {
                                byte lo = ppu.Vram[(tile031Addr + row * 2) & 0xFFFF];
                                byte hi = ppu.Vram[(tile031Addr + row * 2 + 1) & 0xFFFF];
                                var line = new System.Text.StringBuilder("  ");
                                for (int col = 0; col < 8; col++)
                                {
                                    int bit = 7 - col;
                                    int pxVal = ((lo >> bit) & 1) | (((hi >> bit) & 1) << 1);
                                    line.Append(pxVal == 0 ? '.' : (char)('0' + pxVal));
                                    line.Append(' ');
                                }
                                Console.WriteLine(line.ToString());
                            }

                            // Dump the CGRAM entries that pixel values 1, 2, 3 of BG3
                            // palette 6 map to, so we can compare against Mesen's
                            // reference values directly. In 2bpp, BG3 palettes are
                            // 4 colors each. Palette 6 base = 6 * 4 = 24 = 0x18 in
                            // CGRAM index space (byte offset 0x30).
                            Console.WriteLine("[BG3 CHECK] Our CGRAM at BG3 pal 6 (Mesen ref: $19=?, $1A=0x7AAB bright blue, $1B=?):");
                            for (int cIdx = 0x18; cIdx <= 0x1B; cIdx++)
                            {
                                int byteOffset = cIdx * 2;
                                int val = ppu.Cgram[byteOffset] | (ppu.Cgram[byteOffset + 1] << 8);
                                int r = (val & 0x1F) << 3;
                                int g = ((val >> 5) & 0x1F) << 3;
                                int b = ((val >> 10) & 0x1F) << 3;
                                Console.WriteLine($"[BG3 CHECK]   CGRAM ${cIdx:X2} = 0x{val:X4} -> RGB ({r},{g},{b})  (ever written: {ppu.WasCgramTouched(byteOffset) || ppu.WasCgramTouched(byteOffset + 1)})");
                            }

                            // Is BG3's tilemap entry itself fresh, intentional data
                            // for this level, or stale leftover content (e.g. from
                            // the title screen) that never got overwritten when the
                            // level loaded? Since BG3 doesn't scroll, this is the
                            // SAME entry address every time - if it was never
                            // written, that's the real story, not a rendering bug.
                            int mapBase3check = (ppu.BgSc[2] & 0xFC) << 9;
                            int sizeBits3check = ppu.BgSc[2] & 0x03;
                            int entryAddr3check = BgTilemapEntryAddress(mapBase3check, sizeBits3check, (px + ppu.BgScrollX[2]) >> 3, (py + ppu.BgScrollY[2]) >> 3);
                            bool tilemapEverWritten = ppu.WasVramTouched(entryAddr3check) || ppu.WasVramTouched(entryAddr3check + 1);
                            Console.WriteLine($"[BG3 CHECK] BG3 tilemap entry @ 0x{entryAddr3check:X4} ever written since power-on: {tilemapEverWritten}");

                            // Sample BG3 at the tile immediately left/right/up/down of
                            // this position - a coherent decorative pattern in its
                            // neighbors vs random/garbled tile numbers tells us a lot
                            // about whether we're reading from a sane VRAM region.
                            int mapBase3c = (ppu.BgSc[2] & 0xFC) << 9;
                            int sizeBits3c = ppu.BgSc[2] & 0x03;
                            int wy3c = (py + ppu.BgScrollY[2]) % ((sizeBits3c & 0x02) != 0 ? 512 : 256);
                            int wx3c = (px + ppu.BgScrollX[2]) % ((sizeBits3c & 0x01) != 0 ? 512 : 256);
                            int tx3c = wx3c >> 3;
                            int ty3c = wy3c >> 3;

                            foreach (var (dx, dy, label) in new (int, int, string)[] { (-1, 0, "left"), (1, 0, "right"), (0, -1, "up"), (0, 1, "down") })
                            {
                                int eAddr = BgTilemapEntryAddress(mapBase3c, sizeBits3c, tx3c + dx, ty3c + dy);
                                int e = ppu.Vram[eAddr] | (ppu.Vram[(eAddr + 1) & 0xFFFF] << 8);
                                Console.WriteLine($"[BG3 CHECK] neighbor {label}: tile=0x{e & 0x3FF:X3} pal={(e >> 10) & 0x07} priority={(e & 0x2000) != 0}");
                            }
                        }
                        Console.WriteLine($"[BLACK TILE]    ACTUAL displayed pixel: ({c.R},{c.G},{c.B})");

                        if (seen.Count <= 6)
                        {
                            float brightness = (ppu.Inidisp & 0x0F) / 15f;

                            // Faithfully replay BG1's exact per-pixel math (scroll,
                            // flip, transparency) for this exact (px,py) - not just a
                            // tile/palette lookup, the actual pixel index.
                            int sizeBits1 = ppu.BgSc[0] & 0x03;
                            int wy1 = (py + ppu.BgScrollY[0]) % ((sizeBits1 & 0x02) != 0 ? 512 : 256);
                            int wx1 = (px + ppu.BgScrollX[0]) % ((sizeBits1 & 0x01) != 0 ? 512 : 256);
                            bool flipX1 = (entry & 0x4000) != 0;
                            bool flipY1 = (entry & 0x8000) != 0;
                            int r1 = flipY1 ? 7 - (wy1 & 7) : (wy1 & 7);
                            int cbit1 = flipX1 ? (wx1 & 7) : 7 - (wx1 & 7);
                            int chrBase1 = (ppu.Bg12Nba & 0x0F) << 13;
                            int tileAddr1 = (chrBase1 + tile * 32) & 0xFFFF;
                            byte b0 = ppu.Vram[(tileAddr1 + r1 * 2) & 0xFFFF];
                            byte b1 = ppu.Vram[(tileAddr1 + r1 * 2 + 1) & 0xFFFF];
                            byte b2 = ppu.Vram[(tileAddr1 + 16 + r1 * 2) & 0xFFFF];
                            byte b3 = ppu.Vram[(tileAddr1 + 16 + r1 * 2 + 1) & 0xFFFF];
                            int bg1Pixel = ((b0 >> cbit1) & 1) | (((b1 >> cbit1) & 1) << 1) | (((b2 >> cbit1) & 1) << 2) | (((b3 >> cbit1) & 1) << 3);
                            Console.WriteLine($"[BLACK TILE]    BG1 actual pixel index at this exact spot: {bg1Pixel} (0=transparent)");

                            if (bg1Pixel != 0)
                            {
                                int cgIdx1 = (pal * 16 + bg1Pixel) * 2;
                                Color bg1Color = SnesColor(ppu.Cgram[cgIdx1 & 0x1FF], ppu.Cgram[(cgIdx1 + 1) & 0x1FF], brightness);
                                Console.WriteLine($"[BLACK TILE]    BG1 is OPAQUE here - its own color should be ({bg1Color.R},{bg1Color.G},{bg1Color.B}). If the actual pixel is black instead, the bug is in BG1's own palette lookup, not the backdrop/BG2 path at all.");
                            }
                            else
                            {
                                // BG1 is genuinely transparent here - replay BG2's exact
                                // per-pixel math the same way, since BG2 (sub-screen)
                                // is what should be blended in for this pixel.
                                int mapBase2 = (ppu.BgSc[1] & 0xFC) << 9;
                                int sizeBits2 = ppu.BgSc[1] & 0x03;
                                int wy2 = (py + ppu.BgScrollY[1]) % ((sizeBits2 & 0x02) != 0 ? 512 : 256);
                                int wx2 = (px + ppu.BgScrollX[1]) % ((sizeBits2 & 0x01) != 0 ? 512 : 256);
                                int entryAddr2 = BgTilemapEntryAddress(mapBase2, sizeBits2, wx2 >> 3, wy2 >> 3);
                                int entry2 = ppu.Vram[entryAddr2] | (ppu.Vram[(entryAddr2 + 1) & 0xFFFF] << 8);
                                int tile2 = entry2 & 0x3FF;
                                int pal2 = (entry2 >> 10) & 0x07;
                                bool flipX2 = (entry2 & 0x4000) != 0;
                                bool flipY2 = (entry2 & 0x8000) != 0;
                                int r2 = flipY2 ? 7 - (wy2 & 7) : (wy2 & 7);
                                int cbit2 = flipX2 ? (wx2 & 7) : 7 - (wx2 & 7);
                                int chrBase2 = ((ppu.Bg12Nba >> 4) & 0x0F) << 13;
                                int tileAddr2 = (chrBase2 + tile2 * 32) & 0xFFFF;
                                byte c0 = ppu.Vram[(tileAddr2 + r2 * 2) & 0xFFFF];
                                byte c1 = ppu.Vram[(tileAddr2 + r2 * 2 + 1) & 0xFFFF];
                                byte c2 = ppu.Vram[(tileAddr2 + 16 + r2 * 2) & 0xFFFF];
                                byte c3 = ppu.Vram[(tileAddr2 + 16 + r2 * 2 + 1) & 0xFFFF];
                                int bg2Pixel = ((c0 >> cbit2) & 1) | (((c1 >> cbit2) & 1) << 1) | (((c2 >> cbit2) & 1) << 2) | (((c3 >> cbit2) & 1) << 3);
                                Console.WriteLine($"[BLACK TILE]    BG1 transparent. BG2 actual pixel index here: {bg2Pixel} (0=transparent)");

                                bool backdropMathEnabled = (ppu.Cgadsub & 0x20) != 0;
                                Color mainBackdrop = SnesColor(ppu.Cgram[0], ppu.Cgram[1], brightness);
                                Color subContribution;
                                if (bg2Pixel != 0)
                                {
                                    int cgIdx2 = (pal2 * 16 + bg2Pixel) * 2;
                                    subContribution = SnesColor(ppu.Cgram[cgIdx2 & 0x1FF], ppu.Cgram[(cgIdx2 + 1) & 0x1FF], brightness);
                                    Console.WriteLine($"[BLACK TILE]    BG2 is opaque here, its color = ({subContribution.R},{subContribution.G},{subContribution.B})");
                                }
                                else
                                {
                                    subContribution = new Color((byte)(((ppu.FixedColorR & 0x1F) << 3) * brightness), (byte)(((ppu.FixedColorG & 0x1F) << 3) * brightness), (byte)(((ppu.FixedColorB & 0x1F) << 3) * brightness), (byte)255);
                                    Console.WriteLine($"[BLACK TILE]    BG2 also transparent here, sub-screen falls back to fixed color = ({subContribution.R},{subContribution.G},{subContribution.B})");
                                }

                                bool subtractMode = (ppu.Cgadsub & 0x80) != 0;
                                bool halfMode = (ppu.Cgadsub & 0x40) != 0;
                                Color expected = backdropMathEnabled ? BlendColors(mainBackdrop, subContribution, subtractMode, halfMode) : mainBackdrop;
                                Console.WriteLine($"[BLACK TILE]    EXPECTED final color (backdropMath={backdropMathEnabled}): ({expected.R},{expected.G},{expected.B}) vs ACTUAL displayed ({c.R},{c.G},{c.B})");
                            }

                            // BG3's Mode 1 forced-topmost pass runs LAST and draws
                            // unconditionally over everything, including a correctly-
                            // computed BG1 pixel - check whether BG3 has opaque,
                            // priority-marked content sitting at this exact spot.
                            if ((ppu.Bgmode & 0x08) != 0 && (ppu.Tm & 0x04) != 0)
                            {
                                int mapBase3 = (ppu.BgSc[2] & 0xFC) << 9;
                                int sizeBits3 = ppu.BgSc[2] & 0x03;
                                int wy3 = (py + ppu.BgScrollY[2]) % ((sizeBits3 & 0x02) != 0 ? 512 : 256);
                                int wx3 = (px + ppu.BgScrollX[2]) % ((sizeBits3 & 0x01) != 0 ? 512 : 256);
                                int entryAddr3 = BgTilemapEntryAddress(mapBase3, sizeBits3, wx3 >> 3, wy3 >> 3);
                                int entry3 = ppu.Vram[entryAddr3] | (ppu.Vram[(entryAddr3 + 1) & 0xFFFF] << 8);
                                bool bg3Priority = (entry3 & 0x2000) != 0;
                                int tile3 = entry3 & 0x3FF;
                                int pal3 = (entry3 >> 10) & 0x07;
                                bool flipX3 = (entry3 & 0x4000) != 0;
                                bool flipY3 = (entry3 & 0x8000) != 0;
                                int r3 = flipY3 ? 7 - (wy3 & 7) : (wy3 & 7);
                                int cbit3 = flipX3 ? (wx3 & 7) : 7 - (wx3 & 7);
                                int chrBase3 = (ppu.Bg34Nba & 0x0F) << 13;
                                int tileAddr3 = (chrBase3 + tile3 * 16) & 0xFFFF;
                                byte d0 = ppu.Vram[(tileAddr3 + r3 * 2) & 0xFFFF];
                                byte d1 = ppu.Vram[(tileAddr3 + r3 * 2 + 1) & 0xFFFF];
                                int bg3Pixel = ((d0 >> cbit3) & 1) | (((d1 >> cbit3) & 1) << 1);
                                Console.WriteLine($"[BLACK TILE]    BG3 forced-top check: tile=0x{tile3:X3} pal={pal3} priorityBit={bg3Priority} pixel={bg3Pixel} (0=transparent) - if priorityBit=True and pixel!=0, BG3 is unconditionally overwriting BG1 here.");
                                if (bg3Priority && bg3Pixel != 0)
                                {
                                    int cgIdx3 = (pal3 * 4 + bg3Pixel) * 2;
                                    Color bg3Color = SnesColor(ppu.Cgram[cgIdx3 & 0x1FF], ppu.Cgram[(cgIdx3 + 1) & 0x1FF], brightness);
                                    Console.WriteLine($"[BLACK TILE]    BG3's color here = ({bg3Color.R},{bg3Color.G},{bg3Color.B}) - THIS is what's actually winning the pixel, not BG1.");
                                }
                            }
                        }
                    }
                }
            }

            Console.WriteLine($"[BLACK TILE] {blackPixelCount} sampled black pixels, {seen.Count} distinct tile/palette combos");
        }

    }
}
