using System;
using System.Diagnostics;
using System.Numerics;
using Raylib_cs;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Debug;
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
        // Sub-phase breakdown within RenderScanline, for the same
        // gameplay-slowdown investigation as VenusCore's LastFramePpuMs -
        // that number stopped dropping as much as expected once BG1-4's
        // redundant double-decode was fixed (RenderBg1-4's BgLineCache),
        // meaning something else in here - sprite evaluation, or the
        // final color-math/window blend loop, neither of which that fix
        // touched - is the actual dominant cost. Accumulated across every
        // scanline of one frame, reset when a new frame starts (py==0,
        // same convention RangeOver/TimeOver already use above).
        private long _objEvalTicksAccum;
        private long _blendTicksAccum;
        private long _mainCompositeTicksAccum;
        private long _subCompositeTicksAccum;
        public double LastFrameObjEvalMs { get; private set; }
        public double LastFrameBlendMs { get; private set; }
        public double LastFrameMainCompositeMs { get; private set; }
        public double LastFrameSubCompositeMs { get; private set; }

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

                _objEvalTicksAccum = 0;
                _blendTicksAccum = 0;
                _mainCompositeTicksAccum = 0;
                _subCompositeTicksAccum = 0;
            }

            if (forceBlank)
            {
                for (int px = 0; px < _frameWidth; px++) _screenPixels[py * MaxOutputW + px] = new Color(0, 0, 0, 255);
                return;
            }

            float brightness = (ppu.Inidisp & 0x0F) / 15f;

            long objEvalStart = Stopwatch.GetTimestamp();
            EvaluateSpritesForScanline(ppu, py, brightness);
            _objEvalTicksAccum += Stopwatch.GetTimestamp() - objEvalStart;

            // Main screen backdrop: plain CGRAM color 0, as always.
            Color mainBackdrop = SnesColor(ppu.Cgram[0], ppu.Cgram[1], brightness);

            // Sub-screen backdrop fallback - see Venus_PPU.md §5.
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

            long mainCompositeStart = Stopwatch.GetTimestamp();

            CompositeScreen(ppu, py, mode, brightness, ppu.Tm, _mainLineBuf, _mainLineLayer, true);

            long subCompositeStart = Stopwatch.GetTimestamp();
            _mainCompositeTicksAccum += subCompositeStart - mainCompositeStart;

            // Same layer/priority order as the main screen, driven by TS - see Venus_PPU.md §4.1.
            CompositeScreen(ppu, py, mode, brightness, ppu.Ts, _subLineBuf, _subLineLayer, false);

            long blendStart = Stopwatch.GetTimestamp();
            _subCompositeTicksAccum += blendStart - subCompositeStart;

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

                    Color resultColor;
                    if (participates && mathAllowed)
                    {
                        // Half-color-math-disabled-for-fixed-color quirk - see Venus_PPU.md §5.
                        bool isFixedColor = !useSubScreen || _subLineLayer[px] == LayerBackdrop;
                        bool actualHalfMode = halfMode;
                        if (isFixedColor && (ppu.Cgadsub & 0x20) == 0)
                        {
                            actualHalfMode = false;
                        }

                        Color mathOperand = useSubScreen ? _subLineBuf[px] : subBackdrop;
                        resultColor = BlendColors(_mainLineBuf[px], mathOperand, subtractMode, actualHalfMode);
                    }
                    else
                    {
                        resultColor = _mainLineBuf[px];
                    }
                    _screenPixels[py * MaxOutputW + px] = resultColor;

                    if (DebugSettings.ColorMathBlendLogging && py == DebugSettings.ColorMathBlendScanline)
                    {
                        Color mc = _mainLineBuf[px];
                        Color sc = _subLineBuf[px];
                        Console.WriteLine($"[COLORMATH] px={px} mainLayer={winningLayer} main=({mc.R},{mc.G},{mc.B}) subLayer={_subLineLayer[px]} sub=({sc.R},{sc.G},{sc.B}) participates={participates} mathAllowed={mathAllowed} -> ({resultColor.R},{resultColor.G},{resultColor.B})");
                    }
                }
            }

            _blendTicksAccum += Stopwatch.GetTimestamp() - blendStart;
            double ticksToMs = 1000.0 / Stopwatch.Frequency;
            LastFrameObjEvalMs = _objEvalTicksAccum * ticksToMs;
            LastFrameBlendMs = _blendTicksAccum * ticksToMs;
            LastFrameMainCompositeMs = _mainCompositeTicksAccum * ticksToMs;
            LastFrameSubCompositeMs = _subCompositeTicksAccum * ticksToMs;
        }

        // One screen's layer stack, main or sub - layerEnable is TM or TS. See Venus_PPU.md §4.1.
        private void CompositeScreen(Ppu ppu, int py, int mode, float brightness, byte layerEnable, Color[] lineBuf, int[] lineLayer, bool isMainScreen)
        {
            bool bg3ForcedTop = (ppu.Bgmode & 0x08) != 0;

            if (mode == 7)
            {
                bool extbgEnabled = (ppu.Setini & 0x40) != 0;
                if (extbgEnabled && (layerEnable & 0x02) != 0) RenderMode7Bg2Extbg(ppu, py, brightness, lineBuf, lineLayer, LayerBg2, isMainScreen, false);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 0, isMainScreen);
                if ((layerEnable & 0x01) != 0) RenderMode7(ppu, py, brightness, lineBuf, lineLayer, LayerBg1, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 1, isMainScreen);
                if (extbgEnabled && (layerEnable & 0x02) != 0) RenderMode7Bg2Extbg(ppu, py, brightness, lineBuf, lineLayer, LayerBg2, isMainScreen, true);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 2, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 3, isMainScreen);
            }
            else if (mode == 6)
            {
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 0, isMainScreen);
                if ((layerEnable & 0x01) != 0) RenderBg1(ppu, py, false, brightness, lineBuf, lineLayer, LayerBg1, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 1, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 2, isMainScreen);
                if ((layerEnable & 0x01) != 0) RenderBg1(ppu, py, true, brightness, lineBuf, lineLayer, LayerBg1, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 3, isMainScreen);
            }
            else if (mode >= 2 && mode <= 5)
            {
                if ((layerEnable & 0x02) != 0) RenderBg2(ppu, py, false, brightness, lineBuf, lineLayer, LayerBg2, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 0, isMainScreen);
                if ((layerEnable & 0x01) != 0) RenderBg1(ppu, py, false, brightness, lineBuf, lineLayer, LayerBg1, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 1, isMainScreen);
                if ((layerEnable & 0x02) != 0) RenderBg2(ppu, py, true, brightness, lineBuf, lineLayer, LayerBg2, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 2, isMainScreen);
                if ((layerEnable & 0x01) != 0) RenderBg1(ppu, py, true, brightness, lineBuf, lineLayer, LayerBg1, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 3, isMainScreen);
            }
            else
            {
                if ((layerEnable & 0x08) != 0) RenderBg4(ppu, py, false, brightness, lineBuf, lineLayer, LayerBg4, isMainScreen);
                if ((layerEnable & 0x04) != 0) RenderBg3(ppu, py, false, brightness, lineBuf, lineLayer, LayerBg3, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 0, isMainScreen);
                if ((layerEnable & 0x08) != 0) RenderBg4(ppu, py, true, brightness, lineBuf, lineLayer, LayerBg4, isMainScreen);
                if (!bg3ForcedTop && (layerEnable & 0x04) != 0) RenderBg3(ppu, py, true, brightness, lineBuf, lineLayer, LayerBg3, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 1, isMainScreen);
                if ((layerEnable & 0x02) != 0) RenderBg2(ppu, py, false, brightness, lineBuf, lineLayer, LayerBg2, isMainScreen);
                if ((layerEnable & 0x01) != 0) RenderBg1(ppu, py, false, brightness, lineBuf, lineLayer, LayerBg1, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 2, isMainScreen);
                if ((layerEnable & 0x02) != 0) RenderBg2(ppu, py, true, brightness, lineBuf, lineLayer, LayerBg2, isMainScreen);
                if ((layerEnable & 0x01) != 0) RenderBg1(ppu, py, true, brightness, lineBuf, lineLayer, LayerBg1, isMainScreen);
                if ((layerEnable & 0x10) != 0) RenderObj(ppu, py, brightness, lineBuf, lineLayer, LayerObj, 3, isMainScreen);
            }

            if (bg3ForcedTop && (layerEnable & 0x04) != 0)
            {
                RenderBg3(ppu, py, true, brightness, lineBuf, lineLayer, LayerBg3, isMainScreen);
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