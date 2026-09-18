namespace EmuSen.Cores.Nintendo.Mars.Vi
{
    // Scanning a frame buffer into the raster: the geometry the registers imply, the borders, and the walk - see Mars_Video.md §2.
    public sealed partial class Vi
    {
        private const int Replicate = 3;

        // Where the picture sits in the raster, how far each step moves through the frame buffer, and which columns carry signal.
        private readonly record struct Picture(
            int Left, int Top, int Columns, int Rows, int Stride, int ActiveLines,
            uint StartX, uint StepX, uint StartY, uint StepY, int FirstColumn, int LastColumn, bool Lower);

        // True when a frame reached the raster; a frame buffer at zero, a signal too short, or a second blank in a row produce none - see §2.2.
        public bool Scan()
        {
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

            Walk(picture, origin);
            return true;
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
        private void Walk(Picture picture, uint origin)
        {
            int width = (int)Register(Width) & 0xFFF;
            bool resample = AntiAlias != Replicate;
            bool wide = (Type & 1) != 0;
            bool divot = DivotEnabled;
            int bug = 0;

            for (int row = 0; row < picture.Rows; row++)
            {
                uint down = picture.StartY + (uint)row * picture.StepY;
                int source = width * (int)(down >> 10);
                int below = source + width;
                int fractionY = (int)(down >> 5) & 0x1F;
                int line = picture.Top * picture.Stride + picture.Left + (picture.Lower ? RasterWidth : 0) + picture.Stride * row;

                // A row the next one reads again leaves the row after that fetching its own line twice - see Mars_VideoFilter.md §3.
                bug = (down >> 10) == ((picture.StartY + (uint)(row + 1) * picture.StepY) >> 10) ? 2 : bug >> 1;

                uint across = picture.StartX;
                for (int column = 0; column < picture.Columns; column++, across += picture.StepX)
                {
                    int step = (int)(across >> 10);
                    int fractionX = (int)(across >> 5) & 0x1F;

                    Pixel color = Across(origin, source + step, wide, width, 0, divot);

                    if (resample)
                    {
                        Pixel next = Across(origin, source + step + 1, wide, width, 0, divot);
                        Pixel under = Across(origin, below + step, wide, width, bug, divot);
                        Pixel underNext = Across(origin, below + step + 1, wide, width, bug, divot);

                        color = Mix(color, under, fractionY);
                        next = Mix(next, underNext, fractionY);
                        color = Mix(color, next, fractionX);
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
            Pixel pixel = Fetch(origin, at, wide);

            if (AntiAlias > Covered) pixel = pixel with { Coverage = 7 };

            // A whole pixel takes the dither filter instead, which is what that test has decided since Mars_VideoPasses.md §1.
            if (pixel.Coverage == 7) return DitherFilterEnabled ? Dither(origin, at, wide, width, bug, pixel) : pixel;

            return Filter(origin, at, wide, width, bug, pixel);
        }

        // Five bits a channel become eight by moving up, not by filling in; the three bits below are the anti-aliasing's to fill - see §2.6.
        private Pixel Fetch(uint origin, int at, bool wide)
        {
            byte[] rdram = _bus.Rdram;

            if (wide)
            {
                uint address = (origin & 0xFF_FFFC) + (uint)at * 4;
                if (address + 3 >= rdram.Length) return default;

                return new Pixel(rdram[address], rdram[address + 1], rdram[address + 2], (rdram[address + 3] >> 5) & 7);
            }

            uint word = (origin & 0xFF_FFFE) + (uint)at * 2;
            if (word + 1 >= rdram.Length) return default;

            int pixel = (rdram[word] << 8) | rdram[word + 1];

            // The high bit of a coverage is the word's own, the two below it are the hidden bits beside it - see Mars_VideoFilter.md §1.
            int coverage = ((pixel & 1) << 2) | _bus.RdramHidden[word >> 1];

            return new Pixel((pixel >> 8) & 0xF8, (pixel & 0x7C0) >> 3, (pixel & 0x3E) << 2, coverage);
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
