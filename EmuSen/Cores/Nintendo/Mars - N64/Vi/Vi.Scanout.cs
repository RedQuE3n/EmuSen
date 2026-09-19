namespace EmuSen.Cores.Nintendo.Mars.Vi
{
    // Scanning a frame buffer into the raster: the geometry the registers imply, the borders, and the walk - see Mars_Video.md §2.
    public sealed partial class Vi
    {
        private const int Replicate = 3;

        // Room for every source pixel one row can reach: 640 columns at the largest step, and the neighbour beyond - see Mars_Performance.md §3.
        private const int RowSpan = RasterWidth * 4 + 2;

        // Each source line's samples, made once a scan rather than once a use; presentation scratch, never state - see Mars_Performance.md §3 and §7.
        [EmuSen.Common.SkipInState] private readonly Pixel[] _samples = new Pixel[2 * RowSpan];
        [EmuSen.Common.SkipInState] private readonly int[] _sampledRow = new int[2 * RowSpan];
        [EmuSen.Common.SkipInState] private int _row;

        // Each slot's samples before divot, which the three a divot needs would otherwise make three times - see Mars_Performance.md §21.
        [EmuSen.Common.SkipInState] private readonly Pixel[] _plain = new Pixel[2 * RowSpan];
        [EmuSen.Common.SkipInState] private readonly int[] _plainRow = new int[2 * RowSpan];

        // A window of whole source lines fetched once a scan, contiguous in the index the walk and the filters use - see Mars_Performance.md §21.
        private const int WindowLines = 32;
        [EmuSen.Common.SkipInState] private Pixel[] _window = System.Array.Empty<Pixel>();
        [EmuSen.Common.SkipInState] private int _windowWidth;
        [EmuSen.Common.SkipInState] private int _windowFrom;
        [EmuSen.Common.SkipInState] private int _windowCount;

        // Which source line each of the two slots holds, and whether it was sampled with the fetch bug folding it - see Mars_Performance.md §7.
        [EmuSen.Common.SkipInState] private readonly int[] _slotLine = new int[2];
        [EmuSen.Common.SkipInState] private readonly bool[] _slotFolded = new bool[2];
        [EmuSen.Common.SkipInState] private readonly bool[] _slotLive = new bool[2];
        [EmuSen.Common.SkipInState] private readonly int[] _slotStamp = new int[2];

        // What the walk reads and how it filters, set from the job before each walk so a deferred walk reads a snapshot - see Mars_Video.md §2.7.
        [EmuSen.Common.SkipInState] private byte[] _sourceRdram = System.Array.Empty<byte>();
        [EmuSen.Common.SkipInState] private byte[] _sourceHidden = System.Array.Empty<byte>();
        [EmuSen.Common.SkipInState] private uint _sourceBase;
        [EmuSen.Common.SkipInState] private int _sourceCount;
        [EmuSen.Common.SkipInState] private int _sourceLength;
        [EmuSen.Common.SkipInState] private int _scanAntiAlias;
        [EmuSen.Common.SkipInState] private bool _scanDither;
        [EmuSen.Common.SkipInState] private bool _scanGamma;

        // A scan's own job, walked at once over live memory - see Mars_Video.md §2.7.
        [EmuSen.Common.SkipInState] private readonly ScanJob _immediate = new();

        // Where the picture sits in the raster, how far each step moves through the frame buffer, and which columns carry signal.
        internal readonly record struct Picture(
            int Left, int Top, int Columns, int Rows, int Stride, int ActiveLines,
            uint StartX, uint StepX, uint StartY, uint StepY, int FirstColumn, int LastColumn, bool Lower);

        // True when a frame reached the raster; a frame buffer at zero, a signal too short, or a second blank in a row produce none - see §2.2.
        public bool Scan()
        {
            if (!Prepare(_immediate)) return false;

            _bus.Dp.WaitForRange(_immediate.From, _immediate.Count);
            Walk(_immediate);
            return true;
        }

        // Everything a scan does before its walk, which alone reads the frame buffer; true when the walk is due - see §2.7.
        public bool Prepare(ScanJob job)
        {
            job.Captured = false;

            uint origin = Register(Origin) & 0xFF_FFFF;
            if (origin == 0) return false;

            if (!Measure(out Picture picture)) return false;

            bool blank = (Type & 2) == 0;
            if (blank && _wasBlank) return false;
            _wasBlank = blank;

            bool signal = picture.Columns > 0 && picture.Left < RasterWidth;

            if (blank)
            {
                System.Array.Clear(_held);
                System.Array.Clear(_raster);
            }
            else
            {
                Borders(picture, signal);
            }

            if (!signal) return false;

            job.Picture = picture;
            job.Origin = origin;
            job.Width = (int)Register(Width) & 0xFFF;
            job.Wide = (Type & 1) != 0;
            job.Resample = AntiAlias != Replicate;
            job.Divot = DivotEnabled;
            job.AntiAlias = AntiAlias;
            job.Dither = DitherFilterEnabled;
            job.Gamma = GammaEnabled;
            Reach(job);
            return true;
        }

        // The bytes of RDRAM a walk can reach, which a capture copies and any walk waits for the display processor over - see §2.7.
        private void Reach(ScanJob job)
        {
            Picture picture = job.Picture;
            int width = job.Width, bytesPerPixel = job.Wide ? 4 : 2;
            uint origin = job.Wide ? job.Origin & 0xFF_FFFC : job.Origin & 0xFF_FFFE;

            // The window fetches two lines above the first row's and three below the last's; a sample reaches a line and two pixels further, a step a row's span - see §2.7.
            long firstLine = (long)(picture.StartY >> 10) - 3;
            long lastLine = (((long)picture.StartY + (long)System.Math.Max(picture.Rows - 1, 0) * picture.StepY) >> 10) + 4;
            long from = origin + ((firstLine - 1) * width - RowSpan - 4) * bytesPerPixel;
            long to = origin + ((lastLine + 2) * width + 2 * RowSpan + 4) * bytesPerPixel;

            int length = _bus.Rdram.Length;
            from = System.Math.Clamp(from, 0, length) & ~1L;
            to = System.Math.Clamp(to, from, length);

            job.From = (uint)from;
            job.Count = (int)(to - from);
        }

        // The frame buffer's lines a walk can reach, copied out so the walk can run while the machine moves on - see §2.7.
        public void Capture(ScanJob job)
        {
            _bus.Dp.WaitForRange(job.From, job.Count);

            int count = job.Count;
            if (job.Rdram.Length < count) job.Rdram = new byte[count];
            if (job.Hidden.Length < count / 2) job.Hidden = new byte[count / 2];

            System.Buffer.BlockCopy(_bus.Rdram, (int)job.From, job.Rdram, 0, count);
            System.Buffer.BlockCopy(_bus.RdramHidden, (int)(job.From >> 1), job.Hidden, 0, count / 2);

            job.Base = job.From;
            job.Length = _bus.Rdram.Length;
            job.Captured = true;
        }

        // The registers as lengths and steps, with the picture pulled back inside the raster where it starts before it - see §2.1.
        private bool Measure(out Picture picture)
        {
            picture = default;

            int top = (int)(Register(VerticalStart) >> 16) & 0x3FF;
            int left = (int)(Register(HorizontalStart) >> 16) & 0x3FF;
            int columns = ((int)Register(HorizontalStart) & 0x3FF) - left;

            // Vertical is counted in half lines, so a picture is half as tall as the register says - see §1.2.
            int rows = (((int)Register(VerticalStart) & 0x3FF) - top) >> 1;

            uint stepX = Register(ScaleX) & 0xFFF;
            uint startX = (Register(ScaleX) >> 16) & 0xFFF;
            uint stepY = Register(ScaleY) & 0xFFF;
            uint startY = (Register(ScaleY) >> 16) & 0xFFF;

            bool pal = IsPal;
            left -= pal ? 128 : 108;

            bool leftClamped = left < 0;
            if (leftClamped)
            {
                startX += stepX * (uint)(-left);
                columns += left;
                left = 0;
            }

            top = (top - (pal ? 44 : 34)) / 2;
            if (top < 0)
            {
                startY += stepY * (uint)(-top);
                top = 0;
            }

            bool columnsClamped = columns + left > RasterWidth;
            if (columnsClamped) columns = RasterWidth - left;

            int activeLines = ((int)Register(VerticalSync) & 0x3FF) - (pal ? 44 : 34);
            if (activeLines < 0) return false;

            bool serrate = Serrate;
            activeLines >>= serrate ? 0 : 1;

            // The last visible line of a field is odd or even by turns, and only the interlaced signal alternates - see §2.3.
            bool lower = serrate && (Register(CurrentLine) & 1) == 0;

            picture = new Picture(
                left, top, columns, rows, RasterWidth << (serrate ? 1 : 0), activeLines,
                startX, stepX, startY, stepY,
                leftClamped ? 0 : 8, columnsClamped ? columns : columns - 7, lower);

            return true;
        }

        // Everything the picture does not cover goes dark, but only once its two frames of grace have run out - see §2.4.
        private void Borders(Picture picture, bool signal)
        {
            int right = picture.Columns + picture.Left;

            if (picture.Left is > 0 and < RasterWidth)
            {
                for (int line = 0; line < picture.ActiveLines; line++) Darken(line, 0, picture.Left);
            }

            if (right >= 0 && right < RasterWidth)
            {
                for (int line = 0; line < picture.ActiveLines; line++) Darken(line, right, RasterWidth - right);
            }

            int at = 0;
            for (; at < (picture.Top << (Serrate ? 1 : 0)) + (picture.Lower ? 1 : 0); at++) Fade(picture, signal, at);

            // An interlaced field holds its own lines and fades the other field's, which is how a still picture keeps both - see §2.4.
            for (int row = 0; row < picture.Rows; row++)
            {
                Hold(picture, signal, at);
                if (Serrate) Fade(picture, signal, at + 1);
                at += Serrate ? 2 : 1;
            }

            // Past the picture the count is spent rather than watched, so a line already out is darkened again every frame - see §2.4.
            for (; at < picture.ActiveLines; at++)
            {
                if (_held[at] != 0) _held[at]--;
                if (_held[at] == 0) Expire(picture, signal, at);
            }
        }

        private void Hold(Picture picture, bool signal, int line)
        {
            if (line >= RasterHeight) return;

            if (signal) _held[line] = 2;
            else if (_held[line] != 0 && --_held[line] == 0) Darken(line, 0, RasterWidth);
        }

        private void Fade(Picture picture, bool signal, int line)
        {
            if (line < RasterHeight && _held[line] != 0 && --_held[line] == 0) Expire(picture, signal, line);
        }

        // A spent line keeps whatever lies outside the picture, which the borders have already darkened - see §2.4.
        private void Expire(Picture picture, bool signal, int line)
        {
            if (signal) Darken(line, picture.Left, picture.Columns);
            else Darken(line, 0, RasterWidth);
        }

        private void Darken(int line, int from, int count)
        {
            if (line >= RasterHeight || count <= 0) return;

            System.Array.Clear(_raster, (line * RasterWidth + from) * 4, count * 4);
        }

        // One pass over the picture: each row starts a new line of the frame buffer, each step a new pixel of it - see §2.5.
        public void Walk(ScanJob job)
        {
            Picture picture = job.Picture;
            uint origin = job.Origin;
            int width = job.Width;
            bool resample = job.Resample;
            bool wide = job.Wide;
            bool divot = job.Divot;
            int bug = 0;

            _scanAntiAlias = job.AntiAlias;
            _scanDither = job.Dither;
            _scanGamma = job.Gamma;

            if (job.Captured)
            {
                _sourceRdram = job.Rdram;
                _sourceHidden = job.Hidden;
                _sourceBase = job.Base;
                _sourceCount = job.Count;
                _sourceLength = job.Length;
            }
            else
            {
                _sourceRdram = _bus.Rdram;
                _sourceHidden = _bus.RdramHidden;
                _sourceBase = 0;
                _sourceCount = _bus.Rdram.Length;
                _sourceLength = _bus.Rdram.Length;
            }

            int first = (int)(picture.StartX >> 10);

            // RDRAM has changed since the last scan, so nothing a slot holds from it stands - see Mars_Performance.md §7.
            _slotLive[0] = _slotLive[1] = false;
            OpenWindow(width);

            for (int row = 0; row < picture.Rows; row++)
            {
                uint down = picture.StartY + (uint)row * picture.StepY;
                int source = width * (int)(down >> 10);
                int below = source + width;
                int fractionY = (int)(down >> 5) & 0x1F;
                int line = picture.Top * picture.Stride + picture.Left + (picture.Lower ? RasterWidth : 0) + picture.Stride * row;

                // A row the next one reads again leaves the row after that fetching its own line twice - see Mars_VideoFilter.md §3.
                bug = (down >> 10) == ((picture.StartY + (uint)(row + 1) * picture.StepY) >> 10) ? 2 : bug >> 1;

                // The filters reach two lines up and two down from the pair, and a wrap at a row's end one further - see Mars_Performance.md §21.
                Lines(origin, wide, (int)(down >> 10) - 2, (int)(down >> 10) + 3);

                int here = Slot(source, folded: false, avoid: -1);
                int under = Slot(below, folded: bug == 1, avoid: here);

                uint across = picture.StartX;
                for (int column = 0; column < picture.Columns; column++, across += picture.StepX)
                {
                    int step = (int)(across >> 10);
                    int fractionX = (int)(across >> 5) & 0x1F;

                    Pixel color = Remembered(here, step - first, origin, source + step, wide, width, 0, divot);

                    // A mix by a zero fraction is the near pixel, so the far one is never asked for - see Mars_Performance.md §5.
                    if (resample && fractionX != 0)
                    {
                        Pixel next = Remembered(here, step + 1 - first, origin, source + step + 1, wide, width, 0, divot);

                        if (fractionY != 0)
                        {
                            Pixel lower = Remembered(under, step - first, origin, below + step, wide, width, bug, divot);
                            Pixel lowerNext = Remembered(under, step + 1 - first, origin, below + step + 1, wide, width, bug, divot);

                            color = Mix(color, lower, fractionY);
                            next = Mix(next, lowerNext, fractionY);
                        }

                        color = Mix(color, next, fractionX);
                    }
                    else if (resample && fractionY != 0)
                    {
                        color = Mix(color, Remembered(under, step - first, origin, below + step, wide, width, bug, divot), fractionY);
                    }

                    int pixel = (line + column) * 4;
                    if (pixel < 0 || pixel + 3 >= _raster.Length) continue;

                    bool shown = column >= picture.FirstColumn && column < picture.LastColumn;

                    // Gamma is the last thing a pixel meets; a dark column is written as zero whether or not it passes through - see Mars_VideoPasses.md §4.2.
                    color = Gamma(color);

                    _raster[pixel] = (byte)(shown ? color.Red : 0);
                    _raster[pixel + 1] = (byte)(shown ? color.Green : 0);
                    _raster[pixel + 2] = (byte)(shown ? color.Blue : 0);

                    // A darkened column keeps the coverage the raster already held, because only the colour is cleared - see §2.5.
                    if (shown) _raster[pixel + 3] = (byte)color.Coverage;
                }
            }
        }

        // The slot already holding this line as this fold samples it, or the one the row's other line does not need - see Mars_Performance.md §7.
        private int Slot(int line, bool folded, int avoid)
        {
            for (int slot = 0; slot < 2; slot++)
            {
                if (_slotLive[slot] && _slotLine[slot] == line && _slotFolded[slot] == folded) return slot;
            }

            int taken = avoid == 0 ? 1 : 0;
            _slotLine[taken] = line;
            _slotFolded[taken] = folded;
            _slotLive[taken] = true;
            _slotStamp[taken] = NextStamp();
            return taken;
        }

        // A new stamp for a slot's new line, so what it held before is stale without being cleared - see Mars_Performance.md §3.
        private int NextStamp()
        {
            if (++_row != int.MaxValue) return _row;

            System.Array.Clear(_sampledRow);
            _row = 1;
            return _row;
        }

        // Sized for the frame buffer's width and emptied, since RDRAM has changed - see Mars_Performance.md §21.
        private void OpenWindow(int width)
        {
            if (width != _windowWidth)
            {
                _window = new Pixel[WindowLines * width];
                _windowWidth = width;
            }

            _windowCount = 0;
        }

        // The lines from first to last fetched into the window, in order; the window slides down when they no longer fit - see Mars_Performance.md §21.
        private void Lines(uint origin, bool wide, int first, int last)
        {
            if (_windowWidth == 0) return;

            if (_windowCount == 0 || first < _windowFrom)
            {
                _windowFrom = first;
                _windowCount = 0;
            }
            else if (last - _windowFrom >= WindowLines)
            {
                int dropped = first - _windowFrom;
                int kept = System.Math.Max(_windowCount - dropped, 0);
                if (kept > 0) System.Array.Copy(_window, dropped * _windowWidth, _window, 0, kept * _windowWidth);
                _windowFrom = first;
                _windowCount = kept;
            }

            while (_windowFrom + _windowCount <= last)
            {
                int line = _windowFrom + _windowCount;
                int at = line * _windowWidth;
                int slot = _windowCount * _windowWidth;
                for (int x = 0; x < _windowWidth; x++) _window[slot + x] = Fetch(origin, at + x, wide);
                _windowCount++;
            }
        }

        // A sample is a function of RDRAM and the registers alone, neither of which a scan changes, so the first answer stands - see Mars_Performance.md §3.
        private Pixel Remembered(int slot, int offset, uint origin, int at, bool wide, int width, int bug, bool divot)
        {
            if ((uint)offset >= RowSpan) return Across(origin, at, wide, width, bug, divot);

            int index = slot * RowSpan + offset;
            if (_sampledRow[index] == _slotStamp[slot]) return _samples[index];

            // The divot's neighbours are this row's own samples one step either side, remembered the same way - see Mars_Performance.md §21.
            Pixel pixel = divot
                ? Divot(
                    Sampled(slot, offset, origin, at, wide, width, bug),
                    Sampled(slot, offset - 1, origin, at - 1, wide, width, bug),
                    Sampled(slot, offset + 1, origin, at + 1, wide, width, bug))
                : Sampled(slot, offset, origin, at, wide, width, bug);

            _samples[index] = pixel;
            _sampledRow[index] = _slotStamp[slot];
            return pixel;
        }

        private Pixel Sampled(int slot, int offset, uint origin, int at, bool wide, int width, int bug)
        {
            if ((uint)offset >= RowSpan) return Sample(origin, at, wide, width, bug);

            int index = slot * RowSpan + offset;
            if (_plainRow[index] == _slotStamp[slot]) return _plain[index];

            Pixel pixel = Sample(origin, at, wide, width, bug);
            _plain[index] = pixel;
            _plainRow[index] = _slotStamp[slot];
            return pixel;
        }

        // The pixel at this index from the window, or fetched when the index falls outside it - see Mars_Performance.md §21.
        private Pixel Fetched(uint origin, int at, bool wide)
        {
            uint index = (uint)(at - _windowFrom * _windowWidth);
            return index < (uint)(_windowCount * _windowWidth) ? _window[index] : Fetch(origin, at, wide);
        }

        // The pixel a step lands on with its two neighbours across the row, which divot needs and nothing else does - see Mars_VideoPasses.md §2.
        private Pixel Across(uint origin, int at, bool wide, int width, int bug, bool divot)
        {
            Pixel pixel = Sample(origin, at, wide, width, bug);

            if (!divot) return pixel;

            return Divot(pixel, Sample(origin, at - 1, wide, width, bug), Sample(origin, at + 1, wide, width, bug));
        }

        // The pixel a step lands on, filtered against its neighbours where the mode reads coverage and the pixel is not whole - see Mars_VideoFilter.md §1.
        private Pixel Sample(uint origin, int at, bool wide, int width, int bug)
        {
            Pixel pixel = Fetched(origin, at, wide);

            if (_scanAntiAlias > Covered) pixel = pixel with { Coverage = 7 };

            // A whole pixel takes the dither filter instead, which is what that test has decided since Mars_VideoPasses.md §1.
            if (pixel.Coverage == 7) return _scanDither ? Dither(origin, at, wide, width, bug, pixel) : pixel;

            return Filter(origin, at, wide, width, bug, pixel);
        }

        // Five bits a channel become eight by moving up, not by filling in; the three bits below are the anti-aliasing's to fill - see §2.6.
        private Pixel Fetch(uint origin, int at, bool wide)
        {
            byte[] rdram = _sourceRdram;

            if (wide)
            {
                uint address = (origin & 0xFF_FFFC) + (uint)at * 4;
                if (address + 3 >= _sourceLength) return default;

                int index = Within(address, 4);
                return new Pixel(rdram[index], rdram[index + 1], rdram[index + 2], (rdram[index + 3] >> 5) & 7);
            }

            uint word = (origin & 0xFF_FFFE) + (uint)at * 2;
            if (word + 1 >= _sourceLength) return default;

            int half = Within(word, 2);
            int pixel = (rdram[half] << 8) | rdram[half + 1];

            // The high bit of a coverage is the word's own, the two below it are the hidden bits beside it - see Mars_VideoFilter.md §1.
            int coverage = ((pixel & 1) << 2) | _sourceHidden[half >> 1];

            return new Pixel((pixel >> 8) & 0xF8, (pixel & 0x7C0) >> 3, (pixel & 0x3E) << 2, coverage);
        }

        // An address inside memory that a snapshot does not hold is a defect in Capture's reach, and is said so rather than read stale - see §2.7.
        private int Within(uint address, int size)
        {
            uint index = address - _sourceBase;
            if (index + (uint)size > (uint)_sourceCount) throw new System.InvalidOperationException($"The scan reached frame buffer address {address:X} outside the lines captured for it.");

            return (int)index;
        }

        // Mixing moves the colour and leaves the coverage, so a mixed pixel still carries the one the step landed on - see Mars_VideoFilter.md §1.
        private static Pixel Mix(Pixel near, Pixel far, int fraction)
        {
            if (fraction == 0) return near;

            return near with
            {
                Red = Between(near.Red, far.Red, fraction),
                Green = Between(near.Green, far.Green, fraction),
                Blue = Between(near.Blue, far.Blue, fraction),
            };
        }

        private static int Between(int near, int far, int fraction) => ((((far - near) * fraction + 16) >> 5) + near) & 0xFF;
    }
}
