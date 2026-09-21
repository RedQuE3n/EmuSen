namespace EmuSen.Cores.Nintendo.Mars.Vi
{
    public sealed partial class Vi
    {
        // One band of the walk's rows with caches of its own; presentation scratch, never state - see Mars_Video.md §2.12.
        [EmuSen.Common.SkipInState] private Walker[] _walkers = System.Array.Empty<Walker>();

        // As many bands as a quarter of the processors, at most four; the walk shares the machine with the emulation and the RDP's workers.
        private static readonly int Bands = System.Math.Clamp(System.Environment.ProcessorCount / 4, 1, 4);

        private const int MinimumBandRows = 32;

        private sealed partial class Walker
        {
            // Each source line's samples, made once a scan rather than once a use; presentation scratch, never state - see Mars_Performance.md §3 and §7.
            private readonly Pixel[] _samples = new Pixel[2 * RowSpan];
            private readonly int[] _sampledRow = new int[2 * RowSpan];
            private int _row;

            // Each slot's samples before divot, which the three a divot needs would otherwise make three times - see Mars_Performance.md §21.
            private readonly Pixel[] _plain = new Pixel[2 * RowSpan];
            private readonly int[] _plainRow = new int[2 * RowSpan];

            // A window of whole source lines fetched once a scan, contiguous in the index the walk and the filters use - see Mars_Performance.md §21.
            private Pixel[] _window = System.Array.Empty<Pixel>();
            private int _windowWidth;
            private int _windowFrom;
            private int _windowCount;

            // Which source line each of the two slots holds, and whether it was sampled with the fetch bug folding it - see Mars_Performance.md §7.
            private readonly int[] _slotLine = new int[2];
            private readonly bool[] _slotFolded = new bool[2];
            private readonly bool[] _slotLive = new bool[2];
            private readonly int[] _slotStamp = new int[2];

            // What the walk reads and how it filters, set from the job before each walk so a deferred walk reads a snapshot - see Mars_Video.md §2.7.
            private byte[] _sourceRdram = System.Array.Empty<byte>();
            private byte[] _sourceHidden = System.Array.Empty<byte>();
            private uint _sourceBase;
            private int _sourceCount;
            private int _sourceLength;
            private int _scanAntiAlias;
            private bool _scanDither;
            private bool _scanGamma;

            // What this walk reads and how it filters, and caches emptied, since memory has changed since the last walk - see Mars_Performance.md §7.
            internal void Begin(byte[] rdram, byte[] hidden, uint sourceBase, int sourceCount, int sourceLength, int antiAlias, bool dither, bool gamma, int width)
            {
                _sourceRdram = rdram;
                _sourceHidden = hidden;
                _sourceBase = sourceBase;
                _sourceCount = sourceCount;
                _sourceLength = sourceLength;
                _scanAntiAlias = antiAlias;
                _scanDither = dither;
                _scanGamma = gamma;
                _slotLive[0] = _slotLive[1] = false;
                OpenWindow(width);
            }

            // Vi.Walk's rows from one to another, into the raster, which no other band's rows touch.
            internal void Rows(Picture picture, uint origin, int width, bool resample, bool wide, bool divot, int rasterWidth, byte[] raster, int from, int to)
            {
                int first = (int)(picture.StartX >> 10);

                // The counter's closed form at the band's first row: two there, one after, from the row before it - see Mars_Gpu.md §13.1.
                int bug = from > 0 && (picture.StartY + (uint)(from - 1) * picture.StepY) >> 10 == (picture.StartY + (uint)from * picture.StepY) >> 10 ? 2 : 0;

                for (int row = from; row < to; row++)
                {
                    uint down = picture.StartY + (uint)row * picture.StepY;
                    int source = width * (int)(down >> 10);
                    int below = source + width;
                    int fractionY = (int)(down >> 5) & 0x1F;
                    int line = picture.Top * picture.Stride + picture.Left + (picture.Lower ? rasterWidth : 0) + picture.Stride * row;

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
                        if (pixel < 0 || pixel + 3 >= raster.Length) continue;

                        bool shown = column >= picture.FirstColumn && column < picture.LastColumn;

                        // Gamma is the last thing a pixel meets; a dark column is written as zero whether or not it passes through - see Mars_VideoPasses.md §4.2.
                        color = Gamma(color);

                        raster[pixel] = (byte)(shown ? color.Red : 0);
                        raster[pixel + 1] = (byte)(shown ? color.Green : 0);
                        raster[pixel + 2] = (byte)(shown ? color.Blue : 0);

                        // A darkened column keeps the coverage the raster already held, because only the colour is cleared - see §2.5.
                        if (shown) raster[pixel + 3] = (byte)color.Coverage;
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
                    uint address = origin + (uint)at * 4;
                    if (address + 3 >= _sourceLength) return default;

                    int index = Within(address, 4);
                    return new Pixel(rdram[index], rdram[index + 1], rdram[index + 2], (rdram[index + 3] >> 5) & 7);
                }

                uint word = origin + (uint)at * 2;
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
                if (index + (uint)size > (uint)_sourceCount)
                    throw new System.InvalidOperationException($"The scan reached frame buffer address {address:X} outside the lines captured for it, {_sourceBase:X} for {_sourceCount:X} bytes.");

                return (int)index;
            }
        }
    }
}
