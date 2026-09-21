namespace EmuSen.Cores.Nintendo.Mars.Vi
{
    // Scanning a frame buffer into the raster: the geometry the registers imply, the borders, and the walk - see Mars_Video.md §2.
    public sealed partial class Vi
    {
        private const int Replicate = 3;

        // Room for every source pixel one row can reach: 640 columns at the largest step, and the neighbour beyond - see Mars_Performance.md §3.
        private const int RowSpan = RasterWidth * 4 + 2;

        private const int WindowLines = 32;

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

            _bus.Dp.WaitForReadRange(_immediate.From, _immediate.Count, 8);

            // The immediate walk reads the live shadow without a capture, so the device either walks it or reads it back here - see Mars_Gpu.md §11.2 and §13.
            if (_immediate.Scale > 1 && _bus.Dp.CanScanOut) DeviceScan(_immediate);
            else if (_immediate.Scale > 1) _bus.Dp.ReadBackScaled(_immediate.ScaledFrom, _immediate.ScaledCount);

            Walk(_immediate);
            return true;
        }

        // Everything a scan does before its walk, which alone reads the frame buffer; true when the walk is due - see §2.7.
        public bool Prepare(ScanJob job)
        {
            job.Captured = false;
            job.DeviceScanned = false;
            FollowTheDevice();

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
                if (_deviceRaster) { _deviceClear = true; _deviceSpans.Clear(); }
                else System.Array.Clear(_rasterScaled);
            }
            else
            {
                Borders(picture, signal);
            }

            // Nothing is walked, but what was darkened must still reach the raster the device averages.
            if (!signal)
            {
                if (_deviceRaster && RasterEditsPending) PublishRaster(null);
                return false;
            }

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

            // The scaled memory is read in place of the machine's once something has drawn into it - see Mars_Video.md §2.9.
            job.Scale = _bus.Dp.ScaledDrawn ? _bus.Dp.Scale : 1;
            if (job.Scale > 1)
            {
                int n = job.Scale;
                Picture p = picture;
                job.ScaledPicture = new Picture(p.Left * n, p.Top * n, p.Columns * n, p.Rows * n, p.Stride * n, p.ActiveLines * n,
                    p.StartX * (uint)n, p.StepX, p.StartY * (uint)n, p.StepY, p.FirstColumn * n, p.LastColumn * n, p.Lower);
                job.LiveScaledRdram = _bus.Dp.ScaledRdram;
                job.LiveScaledHidden = _bus.Dp.ScaledHidden;
                ReachScaled(job);
            }

            return true;
        }

        // The scaled memory's bytes a walk at the multiple can reach, by the same rule over the scaled origin, width and lines - see §2.9.
        private void ReachScaled(ScanJob job)
        {
            Picture picture = job.ScaledPicture;
            int n = job.Scale;
            long width = (long)job.Width * n, bytesPerPixel = job.Wide ? 4 : 2;
            long origin = (long)(job.Wide ? job.Origin & 0xFF_FFFC : job.Origin & 0xFF_FFFE) * n * n;

            long firstLine = (long)(picture.StartY >> 10) - 3;
            long lastLine = (((long)picture.StartY + (long)System.Math.Max(picture.Rows - 1, 0) * picture.StepY) >> 10) + 4;
            long from = origin + ((firstLine - 1) * width - RowSpan * n - 4) * bytesPerPixel;
            long to = origin + ((lastLine + 2) * width + 2 * RowSpan * n + 4) * bytesPerPixel;

            long length = job.LiveScaledRdram.Length;
            from = System.Math.Clamp(from, 0, length) & ~1L;
            to = System.Math.Clamp(to, from, length);

            job.ScaledFrom = (uint)from;
            job.ScaledCount = (int)(to - from);
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
        public void Capture(ScanJob job, bool walkRepeats = true)
        {
            _bus.Dp.WaitForReadRange(job.From, job.Count, 8);

            // The bytes of the last walk are still the capture, so a scan that would repeat it is recognised without copying anything - see §2.8.
            job.Repeats = Repeated(job);

            // The capture already holds this scan's bytes, so it is this scan's capture and a walk over it is still right - see §2.8.
            if (job.Repeats)
            {
                // The device captures no scaled bytes, so its repeat is its last picture, walked again only if another scan has replaced it - see Mars_Gpu.md §14.
                if (walkRepeats && job.Scale > 1 && _bus.Dp.CanScanOut)
                {
                    // Into the raster, clears since the last walk would be undone by the walk the processor makes again - see Mars_Gpu.md §15.
                    if (job.DeviceScanAt == _bus.Dp.ScanOuts && !(_deviceRaster && RasterEditsPending)) job.DeviceScanned = true;
                    else DeviceScan(job);
                }

                job.Base = job.From;
                job.Length = _bus.Rdram.Length;
                job.Captured = true;
                return;
            }

            int count = job.Count;
            if (job.Rdram.Length < count) job.Rdram = new byte[count];
            if (job.Hidden.Length < count / 2) job.Hidden = new byte[count / 2];

            System.Buffer.BlockCopy(_bus.Rdram, (int)job.From, job.Rdram, 0, count);
            System.Buffer.BlockCopy(_bus.RdramHidden, (int)(job.From >> 1), job.Hidden, 0, count / 2);

            // The scaled memory changes only when the machine's does, so its capture is due exactly when this one is - see §2.9.
            if (job.Scale > 1)
            {
                int scaled = job.ScaledCount;

                // The device walks its own memory here, where it is idle, so nothing of that memory needs copying - see Mars_Gpu.md §13.
                if (_bus.Dp.CanScanOut)
                {
                    DeviceScan(job);
                    job.Base = job.From;
                    job.Length = _bus.Rdram.Length;
                    job.Captured = true;
                    return;
                }

                // What the device holds becomes the shadow's bytes here, which is the one place the walk reads them - see Mars_Gpu.md §11.2.
                _bus.Dp.ReadBackScaled(job.ScaledFrom, scaled);

                if (job.ScaledRdram.Length < scaled) job.ScaledRdram = new byte[scaled];
                if (job.ScaledHidden.Length < scaled / 2) job.ScaledHidden = new byte[scaled / 2];
                System.Buffer.BlockCopy(job.LiveScaledRdram, (int)job.ScaledFrom, job.ScaledRdram, 0, scaled);
                System.Buffer.BlockCopy(job.LiveScaledHidden, (int)(job.ScaledFrom >> 1), job.ScaledHidden, 0, scaled / 2);
                job.ScaledBase = job.ScaledFrom;
                job.ScaledLength = job.LiveScaledRdram.Length;
            }

            job.Base = job.From;
            job.Length = _bus.Rdram.Length;
            job.Captured = true;
        }

        // The walk reads the registers this job carries and the bytes it captured and nothing else, so the same two give the same raster - see §2.8.
        private bool Repeated(ScanJob job)
        {
            var shape = (job.Picture, job.Origin, job.Width, job.Wide, job.Resample, job.Divot, job.AntiAlias, job.Dither, job.Gamma, job.From, job.Count, job.Scale, _bus.Dp.CanScanOut);
            bool same = job.LastCount == job.Count && job.LastShape.Equals(shape) && job.Rdram.Length >= job.Count;

            job.LastShape = shape;
            job.LastCount = job.Count;

            return same
                && _bus.Rdram.AsSpan((int)job.From, job.Count).SequenceEqual(job.Rdram.AsSpan(0, job.Count))
                && _bus.RdramHidden.AsSpan((int)(job.From >> 1), job.Count / 2).SequenceEqual(job.Hidden.AsSpan(0, job.Count / 2));
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

            // The raster at the multiple darkens the same lines and columns, each at the multiple - see Mars_Video.md §2.9.
            if (_deviceRaster)
            {
                DarkenOnTheDevice(line, from, count);
                return;
            }

            int n = _rasterScale;
            if (n > 1 && _rasterScaled.Length > 0)
            {
                for (int i = 0; i < n; i++) System.Array.Clear(_rasterScaled, ((line * n + i) * RasterWidth * n + from * n) * 4, count * n * 4);
            }
        }

        // One pass over the picture: each row starts a new line of the frame buffer, each step a new pixel of it - see §2.5.
        public void Walk(ScanJob job)
        {
            int scale = job.Scale;
            Picture picture = scale > 1 ? job.ScaledPicture : job.Picture;
            // Aligned as the console's twenty-four-bit register is, then scaled: the product may pass twenty-four bits - see Mars_Video.md §2.11.
            uint aligned = job.Wide ? job.Origin & 0xFF_FFFC : job.Origin & 0xFF_FFFE;
            uint origin = scale > 1 ? aligned * (uint)(scale * scale) : aligned;
            int width = job.Width * scale;
            bool resample = job.Resample;
            bool wide = job.Wide;
            bool divot = job.Divot;
            int rasterWidth = RasterWidth * scale;

            byte[] rdram, hidden;
            uint sourceBase;
            int sourceCount, sourceLength;
            if (scale > 1 && job.Captured) (rdram, hidden, sourceBase, sourceCount, sourceLength) = (job.ScaledRdram, job.ScaledHidden, job.ScaledBase, job.ScaledCount, job.ScaledLength);
            else if (scale > 1) (rdram, hidden, sourceBase, sourceCount, sourceLength) = (job.LiveScaledRdram, job.LiveScaledHidden, 0u, job.LiveScaledRdram.Length, job.LiveScaledRdram.Length);
            else if (job.Captured) (rdram, hidden, sourceBase, sourceCount, sourceLength) = (job.Rdram, job.Hidden, job.Base, job.Count, job.Length);
            else (rdram, hidden, sourceBase, sourceCount, sourceLength) = (_bus.Rdram, _bus.RdramHidden, 0u, _bus.Rdram.Length, _bus.Rdram.Length);

            // A raster at the multiple, made when the multiple changes; the walk writes one raster and names which - see Mars_Video.md §2.9.
            byte[] raster = _raster;
            if (scale > 1)
            {
                if (_rasterScale != scale) { _rasterScaled = new byte[rasterWidth * RasterHeight * scale * 4]; _rasterScale = scale; }
                raster = _rasterScaled;
            }
            _outputScale = scale;

            _shownFromDevice = job.DeviceScanned && _deviceRaster;
            if (_shownFromDevice) return;

            if (job.DeviceScanned)
            {
                WriteDevicePicture(picture, raster, rasterWidth);
                return;
            }

            // Bands of rows walked side by side, each with caches of its own, which are pure functions of memory and the registers - see Mars_Video.md §2.12.
            int bands = System.Math.Clamp(picture.Rows / MinimumBandRows, 1, Bands);
            if (_walkers.Length < bands)
            {
                var grown = new Walker[bands];
                for (int b = 0; b < bands; b++) grown[b] = b < _walkers.Length ? _walkers[b] : new Walker();
                _walkers = grown;
            }

            for (int b = 0; b < bands; b++) _walkers[b].Begin(rdram, hidden, sourceBase, sourceCount, sourceLength, job.AntiAlias, job.Dither, job.Gamma, width);

            if (bands == 1)
            {
                _walkers[0].Rows(picture, origin, width, resample, wide, divot, rasterWidth, raster, 0, picture.Rows);
                return;
            }

            try
            {
                System.Threading.Tasks.Parallel.For(0, bands, b => _walkers[b].Rows(picture, origin, width, resample, wide, divot, rasterWidth, raster, picture.Rows * b / bands, picture.Rows * (b + 1) / bands));
            }
            catch (System.AggregateException fault)
            {
                System.Runtime.ExceptionServices.ExceptionDispatchInfo.Capture(fault.InnerExceptions[0]).Throw();
            }
        }

        // The walk's parameters for the device, which walks exactly what Walk would over the same memory - see Mars_Gpu.md §13.
        private void DeviceScan(ScanJob job)
        {
            if (_deviceRaster)
            {
                PublishRaster(job);
                return;
            }

            _bus.Dp.ScanOut(ScanParametersOf(job));
            job.DeviceScanned = true;
            job.DeviceScanAt = _bus.Dp.ScanOuts;
        }

        private static Rdp.Gpu.GpuRasteriser.ScanParameters ScanParametersOf(ScanJob job)
        {
            Picture p = job.ScaledPicture;
            int n = job.Scale;
            uint aligned = job.Wide ? job.Origin & 0xFF_FFFC : job.Origin & 0xFF_FFFE;

            var scan = new Rdp.Gpu.GpuRasteriser.ScanParameters
            {
                Origin = aligned * (uint)(n * n),
                Width = (uint)(job.Width * n),
                Flags = (job.Wide ? Rdp.Gpu.GpuRasteriser.ScanWide : 0) | (job.Resample ? Rdp.Gpu.GpuRasteriser.ScanResample : 0)
                    | (job.Divot ? Rdp.Gpu.GpuRasteriser.ScanDivot : 0) | (job.Dither ? Rdp.Gpu.GpuRasteriser.ScanDither : 0)
                    | (job.Gamma ? Rdp.Gpu.GpuRasteriser.ScanGamma : 0),
                StartX = p.StartX, StepX = p.StepX, StartY = p.StartY, StepY = p.StepY,
                Rows = (uint)System.Math.Max(p.Rows, 0), Columns = (uint)System.Math.Max(p.Columns, 0),
                AntiAlias = (uint)job.AntiAlias,
            };

            return scan;
        }

        private bool RasterEditsPending => _deviceClear || _deviceSpans.Count > 0;

        // Entering, the host's raster seeds the device's; leaving, the host's is stale and is cleared, as a blank would - see Mars_Gpu.md §15.
        private void FollowTheDevice()
        {
            bool device = AveragesOnTheDevice;
            if (_deviceRaster && !device)
            {
                System.Array.Clear(_rasterScaled);
                _deviceSpans.Clear();
                _deviceClear = false;
                _shownFromDevice = false;
                _seedValid = true;
            }

            _deviceRaster = device;
        }

        // Vi.Darken's clear at the multiple, as spans of the device's raster; sent early when there are more than one submission takes.
        private void DarkenOnTheDevice(int line, int from, int count)
        {
            int n = _bus.Dp.Scale, width = RasterWidth * n;
            for (int i = 0; i < n; i++)
            {
                _deviceSpans.Add((uint)((line * n + i) * width + from * n));
                _deviceSpans.Add((uint)(count * n));
            }

            if (_deviceSpans.Count / 2 >= Rdp.Gpu.GpuRasteriser.MaxSpans - n) PublishRaster(null);
        }

        // The clears, the walk if there is one, and the average, as one submission the deferred thread waits for - see Mars_Gpu.md §15.
        private void PublishRaster(ScanJob? job)
        {
            int n = _bus.Dp.Scale, width = RasterWidth * n, height = RasterHeight * n;
            bool seeding = !_bus.Dp.HoldsRaster(width, height) && _seedValid && _rasterScale == n && _rasterScaled.Length == width * height * 4;
            System.ReadOnlySpan<byte> seed = seeding ? _rasterScaled : default;
            _seedValid = false;

            var scan = default(Rdp.Gpu.GpuRasteriser.ScanParameters);
            if (job is not null)
            {
                Picture p = job.ScaledPicture;
                scan = ScanParametersOf(job);
                scan.LineBase = (uint)(p.Top * p.Stride + p.Left + (p.Lower ? width : 0));
                scan.Stride = (uint)p.Stride;
                scan.FirstColumn = p.FirstColumn;
                scan.LastColumn = p.LastColumn;
            }

            _bus.Dp.ScanIntoRaster(width, height, _average, seed, _deviceClear, System.Runtime.InteropServices.CollectionsMarshal.AsSpan(_deviceSpans), job is not null, scan);
            _deviceSpans.Clear();
            _deviceClear = false;

            if (job is null) return;
            job.DeviceScanned = true;
            job.DeviceScanAt = _bus.Dp.ScanOuts;
        }

        // The device's words into the raster as the walk writes them: a shown pixel whole, a dark one's colour cleared and its coverage kept.
        private void WriteDevicePicture(Picture picture, byte[] raster, int rasterWidth)
        {
            System.ReadOnlySpan<byte> words = System.Runtime.InteropServices.MemoryMarshal.AsBytes(_bus.Dp.ScannedPicture);
            int limit = raster.Length / 4;

            for (int row = 0; row < picture.Rows; row++)
            {
                int line = picture.Top * picture.Stride + picture.Left + (picture.Lower ? rasterWidth : 0) + picture.Stride * row;
                int from = row * picture.Columns;

                // The shown columns are one run of whole pixels, so they go across in one copy.
                int shownFrom = System.Math.Max(System.Math.Max(picture.FirstColumn, 0), -line);
                int shownTo = System.Math.Min(System.Math.Min(picture.LastColumn, picture.Columns), limit - line);
                if (shownTo > shownFrom)
                    words.Slice((from + shownFrom) * 4, (shownTo - shownFrom) * 4).CopyTo(raster.AsSpan((line + shownFrom) * 4));

                for (int column = 0; column < picture.Columns; column++)
                {
                    if (column >= picture.FirstColumn && column < picture.LastColumn) continue;

                    int pixel = (line + column) * 4;
                    if (pixel < 0 || pixel + 3 >= raster.Length) continue;
                    raster[pixel] = raster[pixel + 1] = raster[pixel + 2] = 0;
                }
            }
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
