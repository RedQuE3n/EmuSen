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
                EnterMode(PpuMode.VBlank);
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

        // Four sources feed one line into the interrupt controller; a source going high while
        // another already holds the line high requests nothing - see Mercury_Ppu.md §3.2.
        private void UpdateStatLine()
        {
            bool line =
                ((StatEnables & 0x40) != 0 && LycMatches) ||
                ((StatEnables & 0x20) != 0 && Mode == PpuMode.OamScan) ||
                ((StatEnables & 0x10) != 0 && Mode == PpuMode.VBlank) ||
                ((StatEnables & 0x08) != 0 && Mode == PpuMode.HBlank);

            if (line && !StatLine) _bus.Request(Interrupt.LcdStat);
            StatLine = line;
        }

        // Switching the LCD off parks it at the top of the frame and blanks the panel - see Mercury_Ppu.md §2.3.
        private void DisableLcd()
        {
            Ly = 0;
            Dot = 0;
            Mode = PpuMode.HBlank;
            WindowLine = 0;
            WindowTriggered = false;
            StatLine = false;

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
