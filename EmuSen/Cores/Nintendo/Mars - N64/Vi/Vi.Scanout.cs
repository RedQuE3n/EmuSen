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

            for (int row = 0; row < picture.Rows; row++)
            {
                uint down = picture.StartY + (uint)row * picture.StepY;
                int source = width * (int)(down >> 10);
                int below = source + width;
                int fractionY = (int)(down >> 5) & 0x1F;
                int line = picture.Top * picture.Stride + picture.Left + (picture.Lower ? RasterWidth : 0) + picture.Stride * row;

                uint across = picture.StartX;
                for (int column = 0; column < picture.Columns; column++, across += picture.StepX)
                {
                    int step = (int)(across >> 10);
                    int fractionX = (int)(across >> 5) & 0x1F;

                    var color = Fetch(origin, source + step, wide);

                    if (resample)
                    {
                        var next = Fetch(origin, source + step + 1, wide);

                        color = Mix(color, Fetch(origin, below + step, wide), fractionY);
                        next = Mix(next, Fetch(origin, below + step + 1, wide), fractionY);
                        color = Mix(color, next, fractionX);
                    }

                    int pixel = (line + column) * 4;
                    if (pixel < 0 || pixel + 3 >= _raster.Length) continue;

                    bool shown = column >= picture.FirstColumn && column < picture.LastColumn;
                    _raster[pixel] = (byte)(shown ? color.Red : 0);
                    _raster[pixel + 1] = (byte)(shown ? color.Green : 0);
                    _raster[pixel + 2] = (byte)(shown ? color.Blue : 0);

                    // A darkened column keeps the coverage the raster already held, because only the colour is cleared - see §2.5.
                    if (shown) _raster[pixel + 3] = 7;
                }
            }
        }

        // Five bits a channel become eight by moving up, not by filling in; the three bits below are the anti-aliasing's to fill - see §2.6.
        private (int Red, int Green, int Blue) Fetch(uint origin, int at, bool wide)
        {
            byte[] rdram = _bus.Rdram;

            if (wide)
            {
                uint address = (origin & 0xFF_FFFC) + (uint)at * 4;
                if (address + 3 >= rdram.Length) return (0, 0, 0);

                return (rdram[address], rdram[address + 1], rdram[address + 2]);
            }

            uint word = (origin & 0xFF_FFFE) + (uint)at * 2;
            if (word + 1 >= rdram.Length) return (0, 0, 0);

            int pixel = (rdram[word] << 8) | rdram[word + 1];
            return ((pixel >> 8) & 0xF8, (pixel & 0x7C0) >> 3, (pixel & 0x3E) << 2);
        }

        private static (int Red, int Green, int Blue) Mix((int Red, int Green, int Blue) near, (int Red, int Green, int Blue) far, int fraction)
        {
            if (fraction == 0) return near;

            return (Between(near.Red, far.Red, fraction), Between(near.Green, far.Green, fraction), Between(near.Blue, far.Blue, fraction));
        }

        private static int Between(int near, int far, int fraction) => ((((far - near) * fraction + 16) >> 5) + near) & 0xFF;
    }
}
