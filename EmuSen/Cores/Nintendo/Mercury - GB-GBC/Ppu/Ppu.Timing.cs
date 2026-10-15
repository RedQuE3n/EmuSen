using EmuSen.Cores.Nintendo.Mercury.Memory;

namespace EmuSen.Cores.Nintendo.Mercury.Video
{
    // The mode machine: 80 dots of OAM scan, a variable draw, then hblank - see Mercury_Ppu.md §2.
    public sealed partial class Ppu
    {
        // One T-cycle. A disabled LCD does not advance at all, which is why the core keeps a budget - see Mercury_Ppu.md §1.1.
        public void Tick()
        {
            if (!LcdEnabled) return;

            Dot++;

            if (Dot >= CyclesPerScanline)
            {
                BeginScanline();
                return;
            }

            if (Ly >= ScreenHeight) return;

            if (Dot == OamScanCycles)
            {
                // The WY comparison happens on every line, line 0 included, and the match latches - see Mercury_Ppu.md §4.2.
                if (WindowEnabled && Ly == Wy) WindowTriggered = true;

                DrawingEnd = OamScanCycles + BaseDrawingCycles + (Scx & 0x07);
                EnterMode(PpuMode.Drawing);
            }
            else if (Dot == DrawingEnd && Mode == PpuMode.Drawing)
            {
                RenderScanline(Ly);
                EnterMode(PpuMode.HBlank);

                // An hblank-driven HDMA moves its next block here, which is the whole point of it - see Mercury_Cgb.md §4.
                _bus.OnHBlankStarted();
            }
            else if (Dot == CyclesPerScanline - 4 && Ly == ScreenHeight - 1 && _bus.CgbHardware)
            {
                // A CGB's mode-2 source fires for line 144 one machine cycle before vblank - see Mercury_Native.md §13.11.
                UpdateStatLine(oamAtVBlank: true);
            }
        }

        private void BeginScanline()
        {
            Dot = 0;
            Ly++;

            if (Ly >= TotalScanlines)
            {
                Ly = 0;
                WindowLine = 0;
                WindowTriggered = false;
                FrameCount++;
                FrameComplete = true;
            }

            if (Ly == ScreenHeight)
            {
                // On a DMG the mode-2 source also fires as line 144 begins, with vblank - see Mercury_Native.md §13.11.
                Mode = PpuMode.VBlank;
                UpdateStatLine(oamAtVBlank: !_bus.CgbHardware);
                _bus.Request(Interrupt.VBlank);
            }
            else if (Ly < ScreenHeight)
            {
                EnterMode(PpuMode.OamScan);
            }
            else
            {
                // Lines 145-153 stay in mode 1; only LY=LYC can move the STAT line here.
                UpdateStatLine();
            }
        }

        private void EnterMode(PpuMode mode)
        {
            Mode = mode;
            UpdateStatLine();
        }

        // Four sources feed one line into the interrupt controller; a source going high while another already - see Mercury_Ppu.md §3.2.
        private void UpdateStatLine(bool oamAtVBlank = false)
        {
            // With the LCD off the line does not move, so nothing written then raises the interrupt - see Mercury_Native.md §13.11.
            if (!LcdEnabled) return;

            bool line =
                ((StatEnables & 0x40) != 0 && LycMatches) ||
                ((StatEnables & 0x20) != 0 && (Mode == PpuMode.OamScan || oamAtVBlank)) ||
                ((StatEnables & 0x10) != 0 && Mode == PpuMode.VBlank) ||
                ((StatEnables & 0x08) != 0 && Mode == PpuMode.HBlank);

            if (line && !StatLine) _bus.Request(Interrupt.LcdStat);
            StatLine = line;
        }

        // Switching the LCD off parks it at the top of the frame and blanks the panel - see Mercury_Ppu.md §2.3.
        private void DisableLcd()
        {
            LycWhileOff = Ly == Lyc;
            Ly = 0;
            Dot = 0;
            Mode = PpuMode.HBlank;
            WindowLine = 0;
            WindowTriggered = false;

            ClearScreen();
        }

        private void EnableLcd()
        {
            Ly = 0;
            Dot = 0;
            WindowLine = 0;
            WindowTriggered = false;
            DrawingEnd = OamScanCycles + BaseDrawingCycles + (Scx & 0x07);

            EnterMode(PpuMode.OamScan);
        }
    }
}
