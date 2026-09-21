namespace EmuSen.Cores.Nintendo.Mars.Rdp.Gpu
{
    // The multiple's shading on a compute device: the host walks and bins rows, the device shades them - see Mars_Gpu.md §5.
    public sealed class GpuRasteriser : IDisposable
    {
        public const int TileSize = 8;
        public const int PrimitiveWords = 8;
        public const int RowWords = 4;

        private struct Push { public uint ImageWord, Width, PixelWords, MemoryWords, TileX0, TileY0, TilesWide, TilesHigh; }

        private readonly GpuDevice _device;
        private readonly GpuProgram _shade;
        private readonly GpuBuffer _memory;
        private readonly uint _memoryWords;

        private uint[] _primitives = new uint[PrimitiveWords * 256];
        private uint[] _rows = new uint[RowWords * 4096];
        private uint[] _tileOffsets = new uint[1];
        private uint[] _tileRows = new uint[4096];
        private int _primitiveCount, _rowCount;

        private uint _imageWord, _width, _pixelWords;
        private int _minX, _minY, _maxX, _maxY;

        // Host and device twins of each input, grown together; and the host side of a readback.
        private readonly Staged[] _inputs = { new(), new(), new(), new() };
        private GpuBuffer? _readback;

        private sealed class Staged { public GpuBuffer? Host, Device; }

        // What the batch asked that this phase does not shade, and columns past the image's width, which the CPU path wraps into the next row - see §5.
        public long Flushes, RowsShaded, PrimitivesNotShaded, ColumnsPastTheWidth, RowsOfUnsupportedImages;

        public string DeviceName => _device.Name;

        private GpuRasteriser(GpuDevice device, uint memoryWords)
        {
            _device = device;
            _memoryWords = memoryWords;
            _memory = device.CreateBuffer((ulong)memoryWords * 4, GpuMemory.Device);
            _shade = device.CreateProgram(GpuShaders.Load("shade"), buffers: 5, pushBytes: 32);
            Clear();
        }

        // Null, with the reason, when the memory at this multiple is more than the device will bind: the CPU path's case.
        public static GpuRasteriser? TryCreate(GpuDevice device, long scaledRdramBytes, out string report)
        {
            ulong bytes = (ulong)scaledRdramBytes / 2 * 4;
            if (bytes > device.MaxBufferBytes)
            {
                report = $"{device.Name} binds {device.MaxBufferBytes >> 20} MB at most and the memory at this multiple is {bytes >> 20} MB";
                return null;
            }

            try
            {
                report = device.Name;
                return new GpuRasteriser(device, (uint)(scaledRdramBytes / 2));
            }
            catch (Exception e)
            {
                report = $"{device.Name}: {e.Message}";
                return null;
            }
        }

        // The memory emptied, as the shadow is when a state is read.
        public void Clear()
        {
            ResetBatch();
            _device.Submit(commands => commands.Fill(_memory, 0));
        }

        // The image the rows that follow are drawn into; a change of image ends the batch - see §5.
        public void Image(uint address, int width, int pixelBytes)
        {
            uint word = address >> 1, words = (uint)(pixelBytes / 2);
            if (word == _imageWord && width == _width && words == _pixelWords) return;

            Flush();
            _imageWord = word;
            _width = (uint)width;
            _pixelWords = words;
        }

        public int FillPrimitive(uint color)
        {
            if ((_primitiveCount + 1) * PrimitiveWords > _primitives.Length) Array.Resize(ref _primitives, _primitives.Length * 2);

            int at = _primitiveCount * PrimitiveWords;
            Array.Clear(_primitives, at, PrimitiveWords);
            _primitives[at] = 3;
            _primitives[at + 1] = color;
            return _primitiveCount++;
        }

        public void Row(int primitive, int y, int left, int right)
        {
            // An eight-bit image shares a word between two pixels, which two invocations cannot both write - see §5.
            if (_pixelWords == 0) { RowsOfUnsupportedImages++; return; }

            if (right >= _width) { ColumnsPastTheWidth += right - Math.Max(left, (int)_width) + 1; right = (int)_width - 1; }
            if (left < 0) left = 0;
            if (right < left || y < 0) return;

            if ((_rowCount + 1) * RowWords > _rows.Length) Array.Resize(ref _rows, _rows.Length * 2);

            int at = _rowCount++ * RowWords;
            _rows[at] = (uint)primitive;
            _rows[at + 1] = (uint)y;
            _rows[at + 2] = (uint)left;
            _rows[at + 3] = (uint)right;

            _minX = Math.Min(_minX, left);
            _maxX = Math.Max(_maxX, right);
            _minY = Math.Min(_minY, y);
            _maxY = Math.Max(_maxY, y);
        }

        public void NotShaded() => PrimitivesNotShaded++;

        private void ResetBatch()
        {
            _primitiveCount = 0;
            _rowCount = 0;
            _minX = _minY = int.MaxValue;
            _maxX = _maxY = int.MinValue;
        }

        // Rows into the tiles they touch, in the order they were drawn: counted, summed, then placed - see §5.
        private int Bin(int tileX0, int tileY0, int tilesWide, int tilesHigh)
        {
            int tiles = tilesWide * tilesHigh;
            if (_tileOffsets.Length < tiles + 1) _tileOffsets = new uint[tiles + 1];
            Array.Clear(_tileOffsets, 0, tiles + 1);

            for (int r = 0; r < _rowCount; r++)
            {
                int at = r * RowWords;
                int slot = ((int)_rows[at + 1] / TileSize - tileY0) * tilesWide - tileX0;
                for (int t = (int)_rows[at + 2] / TileSize; t <= (int)_rows[at + 3] / TileSize; t++) _tileOffsets[slot + t + 1]++;
            }

            for (int t = 0; t < tiles; t++) _tileOffsets[t + 1] += _tileOffsets[t];

            int total = (int)_tileOffsets[tiles];
            if (_tileRows.Length < total) _tileRows = new uint[Math.Max(total, _tileRows.Length * 2)];

            // Placed from each tile's start with a cursor, which the offsets are restored from afterwards.
            var cursor = new uint[tiles];
            Array.Copy(_tileOffsets, cursor, tiles);
            for (int r = 0; r < _rowCount; r++)
            {
                int at = r * RowWords;
                int slot = ((int)_rows[at + 1] / TileSize - tileY0) * tilesWide - tileX0;
                for (int t = (int)_rows[at + 2] / TileSize; t <= (int)_rows[at + 3] / TileSize; t++) _tileRows[cursor[slot + t]++] = (uint)r;
            }

            return total;
        }

        private GpuBuffer Upload(int which, uint[] words, int count)
        {
            Staged staged = _inputs[which];
            ulong bytes = (ulong)Math.Max(count, 1) * 4;

            if (staged.Host is null || staged.Host.Bytes < bytes)
            {
                staged.Host?.Dispose();
                staged.Device?.Dispose();
                ulong capacity = System.Numerics.BitOperations.RoundUpToPowerOf2(bytes);
                staged.Host = _device.CreateBuffer(capacity);
                staged.Device = _device.CreateBuffer(capacity, GpuMemory.Device);
            }

            words.AsSpan(0, count).CopyTo(staged.Host.Span<uint>());
            return staged.Host;
        }

        // Everything recorded is shaded, and nothing is in flight when this returns.
        public void Flush()
        {
            if (_rowCount == 0) { ResetBatch(); return; }

            int tileX0 = _minX / TileSize, tileY0 = _minY / TileSize;
            int tilesWide = _maxX / TileSize - tileX0 + 1, tilesHigh = _maxY / TileSize - tileY0 + 1;
            int listed = Bin(tileX0, tileY0, tilesWide, tilesHigh);

            int[] counts = { _primitiveCount * PrimitiveWords, _rowCount * RowWords, tilesWide * tilesHigh + 1, listed };
            uint[][] sources = { _primitives, _rows, _tileOffsets, _tileRows };
            for (int i = 0; i < 4; i++) Upload(i, sources[i], counts[i]);

            var push = new Push
            {
                ImageWord = _imageWord, Width = _width, PixelWords = _pixelWords, MemoryWords = _memoryWords,
                TileX0 = (uint)tileX0, TileY0 = (uint)tileY0, TilesWide = (uint)tilesWide, TilesHigh = (uint)tilesHigh,
            };

            _device.Submit(commands =>
            {
                for (int i = 0; i < 4; i++) commands.Copy(_inputs[i].Host!, _inputs[i].Device!, (ulong)Math.Max(counts[i], 1) * 4);
                commands.Dispatch(_shade, new[] { _memory, _inputs[0].Device!, _inputs[1].Device!, _inputs[2].Device!, _inputs[3].Device! }, push, (uint)tilesWide, (uint)tilesHigh);
            });

            Flushes++;
            RowsShaded += _rowCount;
            ResetBatch();
        }

        // The device's words from one byte address for so many bytes, laid into the shadow's two arrays as the CPU path would have left them.
        public void Read(long address, long bytes, byte[] rdram, byte[] hidden)
        {
            Flush();

            long firstWord = address >> 1, words = Math.Min((bytes + 1) >> 1, _memoryWords - firstWord);
            if (words <= 0) return;

            if (_readback is null || _readback.Bytes < (ulong)words * 4)
            {
                _readback?.Dispose();
                _readback = _device.CreateBuffer(System.Numerics.BitOperations.RoundUpToPowerOf2((ulong)words * 4));
            }

            _device.Submit(commands => commands.Copy(_memory, _readback, (ulong)words * 4, (ulong)firstWord * 4));

            ReadOnlySpan<uint> read = _readback.Span<uint>()[..(int)words];
            for (int i = 0; i < read.Length; i++)
            {
                long word = firstWord + i;
                rdram[word * 2] = (byte)(read[i] >> 8);
                rdram[word * 2 + 1] = (byte)read[i];
                hidden[word] = (byte)(read[i] >> 16);
            }
        }

        public void Dispose()
        {
            _readback?.Dispose();
            foreach (Staged staged in _inputs) { staged.Host?.Dispose(); staged.Device?.Dispose(); }
            _shade.Dispose();
            _memory.Dispose();
        }
    }
}
