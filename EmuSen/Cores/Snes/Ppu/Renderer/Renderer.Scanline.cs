using System;
using System.Numerics;
using Raylib_cs;
using EmuSen.Memory;
using EmuSen.Debug;
using EmuSen.Graphics;

namespace EmuSen.Video
{
    public partial class Renderer
    {
        public void RenderScanline(MemoryBus bus, int py)
        {
            if (py < 0 || py >= ScreenH) return;

            Ppu ppu = bus.Ppu;
            int mode = ppu.Bgmode & 0x07;
            bool forceBlank = (ppu.Inidisp & 0x80) != 0;

            if (py == 0)
            {
                ppu.RangeOver = false;
                ppu.TimeOver = false;
                bool hiRes = mode == 5 || mode == 6 || (ppu.Setini & 0x08) != 0;
                _frameWidth = hiRes ? MaxOutputW : ScreenW;
            }

            if (forceBlank)
            {
                for (int px = 0; px < _frameWidth; px++) _screenPixels[py * MaxOutputW + px] = new Color(0, 0, 0, 255);
                return;
            }

            float brightness = (ppu.Inidisp & 0x0F) / 15f;

            EvaluateSpritesForScanline(ppu, py, brightness);

            // Main screen backdrop: plain CGRAM color 0, as always.
            Color mainBackdrop = SnesColor(ppu.Cgram[0], ppu.Cgram[1], brightness);

            // Sub screen backdrop is different: wherever nothing is drawn on the sub
            // screen, real hardware uses the FIXED COLOR register ($2132) as the fallback.
            Color subBackdrop = new Color(
                (byte)(((ppu.FixedColorR & 0x1F) << 3) * brightness),
                (byte)(((ppu.FixedColorG & 0x1F) << 3) * brightness),
                (byte)(((ppu.FixedColorB & 0x1F) << 3) * brightness),
                (byte)255
            );

            for (int px = 0; px < ScreenW; px++)
            {
                _mainLineBuf[px] = mainBackdrop;
                _mainLineLayer[px] = LayerBackdrop;
                _subLineBuf[px] = subBackdrop;
                _subLineLayer[px] = LayerBackdrop;
            }

            bool bg3ForcedTop = (ppu.Bgmode & 0x08) != 0;
            bool isMode7 = (ppu.Bgmode & 0x07) == 7;

            if (isMode7)
            {
                bool extbgEnabled = (ppu.Setini & 0x40) != 0;
                if (extbgEnabled && (ppu.Tm & 0x02) != 0) RenderMode7Bg2Extbg(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerBg2, true, false);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 0, true);
                if ((ppu.Tm & 0x01) != 0) RenderMode7(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerBg1, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 1, true);
                if (extbgEnabled && (ppu.Tm & 0x02) != 0) RenderMode7Bg2Extbg(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerBg2, true, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 2, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 3, true);
            }
            else if (mode == 6)
            {
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 0, true);
                if ((ppu.Tm & 0x01) != 0) RenderBg1(ppu, py, false, brightness, _mainLineBuf, _mainLineLayer, LayerBg1, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 1, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 2, true);
                if ((ppu.Tm & 0x01) != 0) RenderBg1(ppu, py, true, brightness, _mainLineBuf, _mainLineLayer, LayerBg1, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 3, true);
            }
            else if (mode >= 2 && mode <= 5)
            {
                if ((ppu.Tm & 0x02) != 0) RenderBg2(ppu, py, false, brightness, _mainLineBuf, _mainLineLayer, LayerBg2, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 0, true);
                if ((ppu.Tm & 0x01) != 0) RenderBg1(ppu, py, false, brightness, _mainLineBuf, _mainLineLayer, LayerBg1, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 1, true);
                if ((ppu.Tm & 0x02) != 0) RenderBg2(ppu, py, true, brightness, _mainLineBuf, _mainLineLayer, LayerBg2, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 2, true);
                if ((ppu.Tm & 0x01) != 0) RenderBg1(ppu, py, true, brightness, _mainLineBuf, _mainLineLayer, LayerBg1, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 3, true);
            }
            else
            {
                if ((ppu.Tm & 0x08) != 0) RenderBg4(ppu, py, false, brightness, _mainLineBuf, _mainLineLayer, LayerBg4, true);
                if ((ppu.Tm & 0x04) != 0) RenderBg3(ppu, py, false, brightness, _mainLineBuf, _mainLineLayer, LayerBg3, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 0, true);
                if ((ppu.Tm & 0x08) != 0) RenderBg4(ppu, py, true, brightness, _mainLineBuf, _mainLineLayer, LayerBg4, true);
                if (!bg3ForcedTop && (ppu.Tm & 0x04) != 0) RenderBg3(ppu, py, true, brightness, _mainLineBuf, _mainLineLayer, LayerBg3, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 1, true);
                if ((ppu.Tm & 0x02) != 0) RenderBg2(ppu, py, false, brightness, _mainLineBuf, _mainLineLayer, LayerBg2, true);
                if ((ppu.Tm & 0x01) != 0) RenderBg1(ppu, py, false, brightness, _mainLineBuf, _mainLineLayer, LayerBg1, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 2, true);
                if ((ppu.Tm & 0x02) != 0) RenderBg2(ppu, py, true, brightness, _mainLineBuf, _mainLineLayer, LayerBg2, true);
                if ((ppu.Tm & 0x01) != 0) RenderBg1(ppu, py, true, brightness, _mainLineBuf, _mainLineLayer, LayerBg1, true);
                if ((ppu.Tm & 0x10) != 0) RenderObj(ppu, py, brightness, _mainLineBuf, _mainLineLayer, LayerObj, 3, true);
            }

            if (bg3ForcedTop && (ppu.Tm & 0x04) != 0)
            {
                RenderBg3(ppu, py, true, brightness, _mainLineBuf, _mainLineLayer, LayerBg3, true);
            }

            if (isMode7)
            {
                bool extbgEnabled = (ppu.Setini & 0x40) != 0;
                if (extbgEnabled && (ppu.Ts & 0x02) != 0)
                {
                    RenderMode7Bg2Extbg(ppu, py, brightness, _subLineBuf, _subLineLayer, LayerBg2, false, false);
                    RenderMode7Bg2Extbg(ppu, py, brightness, _subLineBuf, _subLineLayer, LayerBg2, false, true);
                }
                if ((ppu.Ts & 0x01) != 0) RenderMode7(ppu, py, brightness, _subLineBuf, _subLineLayer, LayerBg1, false);
            }
            else
            {
                if ((ppu.Ts & 0x08) != 0) RenderBg4(ppu, py, false, brightness, _subLineBuf, _subLineLayer, LayerBg4, false);
                if ((ppu.Ts & 0x08) != 0) RenderBg4(ppu, py, true, brightness, _subLineBuf, _subLineLayer, LayerBg4, false);
                if ((ppu.Ts & 0x04) != 0) RenderBg3(ppu, py, false, brightness, _subLineBuf, _subLineLayer, LayerBg3, false);
                if ((ppu.Ts & 0x02) != 0) RenderBg2(ppu, py, false, brightness, _subLineBuf, _subLineLayer, LayerBg2, false);
                if ((ppu.Ts & 0x01) != 0) RenderBg1(ppu, py, false, brightness, _subLineBuf, _subLineLayer, LayerBg1, false);
            }
            if ((ppu.Ts & 0x10) != 0)
            {
                for (int p = 0; p <= 3; p++) RenderObj(ppu, py, brightness, _subLineBuf, _subLineLayer, LayerObj, p, false);
            }

            // --- FINAL BLEND ---
            bool subtractMode = (ppu.Cgadsub & 0x80) != 0;
            bool halfMode = (ppu.Cgadsub & 0x40) != 0;

            // Extract Color Math Enable from CGWSEL bits 4-5
            // 0 = Always, 1 = Math Window, 2 = Main Window, 3 = Never
            int colorMathEnable = (ppu.Cgwsel >> 4) & 0x03;
            // Check CGWSEL Bit 1: Are we using the Sub Screen or forced Fixed Color?
            bool useSubScreen = (ppu.Cgwsel & 0x02) != 0;

            if (_frameWidth == MaxOutputW)
            {
                // Hi-res pass-through (skips window/math complexity for now)
                for (int outX = 0; outX < MaxOutputW; outX++)
                {
                    int srcX = outX >> 1;
                    _screenPixels[py * MaxOutputW + outX] = (outX & 1) == 0 ? _mainLineBuf[srcX] : _subLineBuf[srcX];
                }
            }
            else
            {
                for (int px = 0; px < ScreenW; px++)
                {
                    int winningLayer = _mainLineLayer[px];
                    bool participates = winningLayer == LayerObj
                        ? LayerParticipatesInColorMath(ppu.Cgadsub, winningLayer, _objPalette[px])
                        : LayerParticipatesInColorMath(ppu.Cgadsub, winningLayer);

                    bool mathAllowed = false;
                    if (colorMathEnable == 0) mathAllowed = true;
                    else if (colorMathEnable == 3) mathAllowed = false;
                    else
                    {
                        // colorMathEnable 1 = Inside Window, 2 = Outside Window
                        bool inWindow = IsColorMathWindowMasked(ppu, px, true);
                        mathAllowed = (colorMathEnable == 1) ? inWindow : !inWindow;
                    }

                    if (participates && mathAllowed)
                    {
                        // SNES Hardware quirk: Half color math is disabled if blending against the fixed color
                        // (either explicitly or via an empty subscreen pixel) UNLESS the backdrop is enabled in CGADSUB.
                        bool isFixedColor = !useSubScreen || _subLineLayer[px] == LayerBackdrop;
                        bool actualHalfMode = halfMode;
                        if (isFixedColor && (ppu.Cgadsub & 0x20) == 0)
                        {
                            actualHalfMode = false;
                        }

                        Color mathOperand = useSubScreen ? _subLineBuf[px] : subBackdrop;
                        _screenPixels[py * MaxOutputW + px] = BlendColors(_mainLineBuf[px], mathOperand, subtractMode, actualHalfMode);
                    }
                    else
                    {
                        _screenPixels[py * MaxOutputW + px] = _mainLineBuf[px];
                    }
                }
            }
        }

        private static bool IsWindowMasked(Ppu ppu, int layerId, bool isMainScreen, int px)
        {
            if (!DebugSettings.WindowingEnabled) return false;
            byte selByte;
            int nibbleShift;
            int logBits;

            switch (layerId)
            {
                case LayerBg1: selByte = ppu.W12Sel; nibbleShift = 0; logBits = ppu.WBgLog & 0x03; break;
                case LayerBg2: selByte = ppu.W12Sel; nibbleShift = 4; logBits = (ppu.WBgLog >> 2) & 0x03; break;
                case LayerBg3: selByte = ppu.W34Sel; nibbleShift = 0; logBits = (ppu.WBgLog >> 4) & 0x03; break;
                case LayerBg4: selByte = ppu.W34Sel; nibbleShift = 4; logBits = (ppu.WBgLog >> 6) & 0x03; break;
                case LayerObj: selByte = ppu.WObjSel; nibbleShift = 0; logBits = ppu.WObjLog & 0x03; break;
                default: return false;
            }

            byte enableReg = isMainScreen ? ppu.Tmw : ppu.Tsw;
            int enableBit = layerId switch { LayerBg1 => 0x01, LayerBg2 => 0x02, LayerBg3 => 0x04, LayerBg4 => 0x08, LayerObj => 0x10, _ => 0 };
            if ((enableReg & enableBit) == 0) return false;

            int nibble = (selByte >> nibbleShift) & 0x0F;
            bool w1Invert = (nibble & 0x01) != 0;
            bool w1Enable = (nibble & 0x02) != 0;
            bool w2Invert = (nibble & 0x04) != 0;
            bool w2Enable = (nibble & 0x08) != 0;

            if (!w1Enable && !w2Enable) return false;

            bool w1 = px >= ppu.Wh0 && px <= ppu.Wh1;
            bool w2 = px >= ppu.Wh2 && px <= ppu.Wh3;
            if (w1Invert) w1 = !w1;
            if (w2Invert) w2 = !w2;

            if (w1Enable && w2Enable)
            {
                switch (logBits)
                {
                    case 0: return w1 || w2;   
                    case 1: return w1 && w2;   
                    case 2: return w1 ^ w2;    
                    default: return !(w1 ^ w2); 
                }
            }
            return w1Enable ? w1 : w2;
        }

        private static bool IsColorMathWindowMasked(Ppu ppu, int px, bool isMainScreen)
        {
            if (!DebugSettings.WindowingEnabled) return false;

            // Color Math Window uses WOBJSEL ($2125) bits 4-7
            int nibble = (ppu.WObjSel >> 4) & 0x0F;
            bool w1Invert = (nibble & 0x01) != 0;
            bool w1Enable = (nibble & 0x02) != 0;
            bool w2Invert = (nibble & 0x04) != 0;
            bool w2Enable = (nibble & 0x08) != 0;

            if (!w1Enable && !w2Enable) return false;

            bool w1 = px >= ppu.Wh0 && px <= ppu.Wh1;
            bool w2 = px >= ppu.Wh2 && px <= ppu.Wh3;
            if (w1Invert) w1 = !w1;
            if (w2Invert) w2 = !w2;

            bool masked = false;
            if (w1Enable && w2Enable)
            {
                // Logic is defined in WOBJLOG ($212B) bits 2-3
                int logBits = (ppu.WObjLog >> 2) & 0x03;
                switch (logBits)
                {
                    case 0: masked = w1 || w2; break;     // OR
                    case 1: masked = w1 && w2; break;     // AND
                    case 2: masked = w1 ^ w2; break;      // XOR
                    default: masked = !(w1 ^ w2); break;  // XNOR
                }
            }
            else
            {
                masked = w1Enable ? w1 : w2;
            }

            // CGWSEL bits 6-7 invert the final window output for Main/Sub screens
            bool invertFinal = isMainScreen ? (ppu.Cgwsel & 0x40) != 0 : (ppu.Cgwsel & 0x80) != 0;
            if (invertFinal) masked = !masked;

            return masked;
        }
    }
}