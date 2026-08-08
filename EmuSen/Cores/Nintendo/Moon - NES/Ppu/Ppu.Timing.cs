namespace EmuSen.Cores.Nintendo.Moon.Video
{
    // The dot-by-dot clock. Modelled on Mesen's NesPpu::Exec - see Moon_PPU.md §1.
    public sealed partial class Ppu
    {
        public const int DotsPerScanline = 341;
        public const int LastDot = DotsPerScanline - 1;

        public int Cycle;
        public int Scanline;

        // Free-running dot count, which is the clock A12 edge detection is measured against.
        public long PpuClock;

        // Set for the one Step that crosses into a new frame, and cleared by the core that reads it.
        public bool FrameComplete;

        public void Step(int dots)
        {
            for (int i = 0; i < dots; i++) Tick();
        }

        private void Tick()
        {
            PpuClock++;

            if (Cycle < LastDot)
            {
                Cycle++;
                RunDot();
            }
            else
            {
                BeginScanline();
            }
        }

        private void BeginScanline()
        {
            Cycle = 0;
            Scanline++;

            if (Scanline > PreRenderScanline)
            {
                Scanline = 0;
                FrameCount++;
                FrameComplete = true;
                DecayOpenBus();
            }
        }

        // Cycle 1 onwards; cycle 0 is idle on every line - see Moon_PPU.md §1.1.
        private void RunDot()
        {
            if (Scanline < VisibleScanlines) RunVisibleDot();
            else if (Scanline == VBlankScanline && Cycle == 1) EnterVBlank();
            else if (Scanline == PreRenderScanline) RunPreRenderDot();
        }

        private void EnterVBlank()
        {
            // A $2002 read in the two dots before this suppresses the flag entirely - see Moon_PPU.md §1.2.
            if (!SuppressVBlank) VBlankFlag = true;
            SuppressVBlank = false;
        }

        private void RunVisibleDot()
        {
            FetchForDot();

            if (Cycle == 256)
            {
                RenderScanline(Scanline);
                if (RenderingEnabled) IncrementY();
            }
            else if (Cycle == 257)
            {
                if (RenderingEnabled) CopyHorizontal();

                // V walks the line as hardware fetches; the renderer needs where the line starts.
                RenderV = V;
            }
        }

        private void RunPreRenderDot()
        {
            if (Cycle == 1)
            {
                VBlankFlag = false;
                Sprite0Hit = false;
                SpriteOverflow = false;
            }

            FetchForDot();

            if (Cycle == 256 && RenderingEnabled) IncrementY();
            else if (Cycle == 257)
            {
                if (RenderingEnabled) CopyHorizontal();
                RenderV = V;
            }
            else if (Cycle >= 280 && Cycle <= 304 && RenderingEnabled)
            {
                CopyVertical();
                RenderV = V;
            }

            // With rendering on, an odd frame drops the last dot of the pre-render line.
            else if (Cycle == 339 && RenderingEnabled && (FrameCount & 1) != 0) Cycle = LastDot;
        }

        // The addresses hardware would put on the bus, which is what a board watching A12 sees.
        private void FetchForDot()
        {
            if (!RenderingEnabled) return;

            if ((Cycle >= 1 && Cycle <= 256) || (Cycle >= 321 && Cycle <= 336))
            {
                BackgroundFetch();
            }
            else if (Cycle >= 257 && Cycle <= 320)
            {
                SpriteFetch();
            }
            else if (Cycle == 337 || Cycle == 339)
            {
                SetBusAddress((ushort)(0x2000 | (V & 0x0FFF)));
            }
        }

        // Four fetches per tile: nametable, attribute, then the two pattern planes.
        private void BackgroundFetch()
        {
            switch (Cycle & 0x07)
            {
                case 1:
                    SetBusAddress((ushort)(0x2000 | (V & 0x0FFF)));
                    break;
                case 3:
                    SetBusAddress((ushort)(0x23C0 | (V & 0x0C00) | ((V >> 4) & 0x38) | ((V >> 2) & 0x07)));
                    break;
                case 5:
                case 7:
                    SetBusAddress((ushort)(BackgroundPatternBase | ((V >> 12) & 0x07)));
                    break;
            }

            if ((Cycle & 0x07) == 0 && Cycle != 0) IncrementCoarseXLive();
        }

        // The garbage nametable reads matter here only because they hold A12 low between sprite fetches.
        private void SpriteFetch()
        {
            switch ((Cycle - 257) & 0x07)
            {
                case 0:
                    SetBusAddress((ushort)(0x2000 | (V & 0x0FFF)));
                    break;
                case 2:
                    SetBusAddress((ushort)(0x23C0 | (V & 0x0C00) | ((V >> 4) & 0x38) | ((V >> 2) & 0x07)));
                    break;
                case 4:
                case 6:
                    SetBusAddress((ushort)(SpritePatternBaseForLine() | 0x08));
                    break;
            }
        }

        // 8x16 sprites take their table from the tile's low bit, so the line's sprites decide A12.
        private int SpritePatternBaseForLine()
        {
            if (!SpritesAre8x16) return SpritePatternBase;

            // The slots past the end of the line's sprites are not idle: hardware
            // fetches tile $FF through them, and $FF's low bit selects the high
            // table. That is what keeps the MMC3 counter clocking on a line with
            // nothing on it, and without it SMB3's title screen loses its floor
            // entirely - see Moon_Memory.md §4.6b.
            if (_spriteCount < 8) return 0x1000;

            for (int s = 0; s < _spriteCount; s++)
            {
                if ((Oam[(_spriteIndices[s] * 4) + 1] & 0x01) != 0) return 0x1000;
            }

            return 0x0000;
        }

        private void SetBusAddress(ushort address)
        {
            BusAddress = address;
            _cart.Mapper.OnPpuAddress(address, PpuClock);
        }

        private void IncrementCoarseXLive()
        {
            ushort v = V;
            IncrementCoarseX(ref v);
            V = v;
        }
    }
}
