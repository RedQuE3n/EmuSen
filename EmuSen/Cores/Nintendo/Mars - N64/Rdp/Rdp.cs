using System;
using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // The display processor: a command is gathered a word at a time and runs once it is whole - see Mars_Rdp.md §3.
    public sealed partial class Rdp
    {
        public const uint TextureRectangle = 0x24;
        public const uint TextureRectangleFlipped = 0x25;
        public const uint SyncFull = 0x29;
        public const uint SetScissor = 0x2D;
        public const uint SetOtherModes = 0x2F;
        public const uint FillRectangle = 0x36;
        public const uint SetFillColor = 0x37;
        public const uint SetColorImage = 0x3F;
        public const uint LoadPalette = 0x30;
        public const uint SetTileSize = 0x32;
        public const uint LoadBlock = 0x33;
        public const uint LoadTile = 0x34;
        public const uint SetTile = 0x35;
        public const uint SetTextureImage = 0x3D;

        // A triangle carrying shade, texture and depth, which is the longest thing the stream holds.
        private const int LongestCommand = 22;

        [EmuSen.Common.SkipInState] private readonly MemoryBus _bus;
        private readonly ulong[] _command = new ulong[LongestCommand];
        private int _taken;

        // Drawing at a multiple: every coordinate times the scale, colour and depth into a memory of the scale squared, textures from the machine's own - see Mars_Rdp.md §11.
        [EmuSen.Common.SkipInState] private int _scale = 1;
        [EmuSen.Common.SkipInState] private bool _scaled;
        [EmuSen.Common.SkipInState] private byte[] _frame;
        [EmuSen.Common.SkipInState] private byte[] _frameHidden;
        [EmuSen.Common.SkipInState] public bool Drew;

        // Which rows this processor shades when several share a list: those whose row modulo the count is its index, or every row when it draws alone - see Mars_Rdp.md §2.8.
        [EmuSen.Common.SkipInState] private int _worker;
        [EmuSen.Common.SkipInState] private int _workers = 1;
        [EmuSen.Common.SkipInState] private bool _alone;

        // The word being run, for the interface's verifier - see Mars_Rdp.md §2.6.1.
        [EmuSen.Common.SkipInState] public long RunningWord;

        // Every primitive counted, and each shaded row stamped by it, so a join can tell which processor wrote a scratch field last in raster order - see §2.8.
        [EmuSen.Common.SkipInState] private long _primitiveSequence;
        [EmuSen.Common.SkipInState] private long _rowStamp;
        [EmuSen.Common.SkipInState] private long _lastShadedStamp, _memoryStamp, _pastStoredStamp, _texel0Stamp, _texel1Stamp, _lodStamp, _blendedStamp, _shiftStamp, _pastShiftStamp;

        // The furthest pixel a draw may have reached in each image since it was set, so a load from those bytes can be told - see §2.8.
        [EmuSen.Common.SkipInState] private long _colorDrawnTo, _depthDrawnTo;

        // What the list asked of several processors: primitives, those one drew alone, and loads every one waited for - see Mars_Performance.md §35.
        [EmuSen.Common.SkipInState] public long Primitives, SerialisedPrimitives, HazardLoads, AliasedReads;
        [EmuSen.Common.SkipInState] private long[] _coverageStamp = new long[SpanRows];

        // What the drain does with a gathered command: more words, run it, run it on the leading processor alone, or run it on every processor together - see §2.8.
        public enum Step { More, Ready, Leader, All, AllJoined }

        private ulong _otherModes;

        // The display processor's own four kilobytes of texture memory, in console byte order - see Mars_RdpTextures.md §2.
        public byte[] TextureMemory { get; } = new byte[0x1000];

        public Rdp(MemoryBus bus)
        {
            _bus = bus;
            _frame = bus.Rdram;
            _frameHidden = bus.RdramHidden;
            Refresh();
        }

        public int Scale => _scale;

        public bool Scaled => _scaled;

        // This processor draws at the multiple into the memory given, which nothing in the machine reads - see Mars_Rdp.md §11.
        public void DrawAt(int scale, byte[] frame, byte[] hidden)
        {
            _scale = scale;
            _scaled = scale > 1;
            _frame = frame;
            _frameHidden = hidden;
            Widen(scale);
        }

        // The native processor decides what draws alone; the scaled one follows it - see §11.
        // A processor shading on the device always draws alone, so it does not take the native one's split - see Mars_Gpu.md §11.
        public void Follow(Rdp native) { if (_gpu is null) _alone = native._alone; }

        // After the state copied from the native processor, its images and scissor are the machine's and are taken to the multiple - see §11.
        public void Rescale()
        {
            if (!_scaled) return;
            uint area = (uint)(_scale * _scale);
            _colorImage *= area;
            _colorImageWidth *= _scale;
            _depthImage *= area;
            _scissorLeft *= _scale;
            _scissorTop *= _scale;
            _scissorRight *= _scale;
            _scissorBottom *= _scale;
        }

        // True when the command this word completed was a full sync, which only the interface can answer - see §6.
        public bool Accept(ulong word) => Gather(word) != Step.More && Execute();

        // A word taken; when it completes a command, what the drain must do before running it - see §2.8.
        public Step Gather(ulong word)
        {
            _command[_taken++] = word;

            uint id = Id(_command[0]);
            if (_taken < Length(id)) return Step.More;

            _taken = 0;
            return Classify(id);
        }

        // Runs the command the last word completed; true for a full sync.
        public bool Execute() => Execute(Id(_command[0]), _command[0]);

        // A primitive is counted here so every processor counts it, drawn or not; the images re-partition the rows' bytes, so each is run by all together - see §2.8.
        private Step Classify(uint id)
        {
            switch (id)
            {
                case >= 0x08 and <= 0x0F:
                case TextureRectangle or TextureRectangleFlipped:
                case FillRectangle:
                    _primitiveSequence++;
                    Primitives++;
                    if (_workers == 1) return Step.Ready;
                    Reached();
                    _alone = Serialised(id);
                    if (_alone) SerialisedPrimitives++;
                    return _alone ? Step.Leader : Step.Ready;

                case SetColorImage or SetMaskImage:
                    return _workers > 1 ? Step.All : Step.Ready;

                case LoadTile or LoadBlock or LoadPalette:
                    if (_workers == 1 || !LoadReachesDrawn(_command[0], id == LoadBlock)) return Step.Ready;
                    HazardLoads++;
                    return Step.AllJoined;

                default:
                    return Step.Ready;
            }
        }

        // A primitive whose rows cannot be shared: a carry across its rows is live, a span can write past its row, or its rows' bytes overlap the other image's - see Mars_Rdp.md §10.2.
        private bool Serialised(uint id)
        {
            int cycle = CycleType;
            if (cycle == TwoCycle && (BlendSecondAlpha == 1 || SelectsCombined(FirstCombineCycle))) return true;
            if (cycle == OneCycle && SelectsCombined(SecondCombineCycle)) return true;

            // In quarter pixels, the rightmost edge a span can hold: the scissor's, or a rectangle's own when that is less - see §2.8.
            int right = _scissorRight;
            if (id is FillRectangle or TextureRectangle or TextureRectangleFlipped) right = Math.Min(right, Quarters(_command[0] >> 44));
            int width = _colorImageWidth << 2;
            if (cycle >= CopyCycle ? right >= width : right > width) return true;

            if (cycle >= CopyCycle || !(DepthCompare || DepthUpdate)) return false;
            long reach = Reach();
            long colorTo = _colorImage + reach * Math.Max(_colorImageBytes, 1), depthTo = _depthImage + reach * 2;
            return _colorImage < depthTo && _depthImage < colorTo;
        }

        // The greatest pixel index a primitive under this scissor can address, which is past the scissor's corner - see Mars_Rdp.md §2.6.1.
        private long Reach()
        {
            long rows = (_scissorBottom + 3) >> 2, width = _colorImageWidth;
            return Math.Max(rows * width, (rows - 1) * width + (_scissorRight >> 2) + 3);
        }

        private void Reached()
        {
            long reach = Reach();
            _colorDrawnTo = Math.Max(_colorDrawnTo, reach);
            if (CycleType <= TwoCycle && DepthUpdate) _depthDrawnTo = Math.Max(_depthDrawnTo, reach);
        }

        // Whether a load reads bytes a draw since the images were set may have written, by the interface's own extent for a load - see §2.8.
        private bool LoadReachesDrawn(ulong word, bool block)
        {
            int sl = (int)(word >> 44) & 0xFFF, tl = (int)(word >> 32) & 0xFFF, sh = (int)(word >> 12) & 0xFFF, th = (int)word & 0xFFF;
            long bits = 4 << _textureImageSize;
            long rowBytes = (_textureImageWidth * bits + 7) / 8;

            long from, to;
            if (block)
            {
                long left = (sl << 20) >> 20;
                from = _textureImage + (tl & 0x3FF) * rowBytes + left * bits / 8 - 16;
                to = _textureImage + (tl & 0x3FF) * rowBytes + ((long)sh + 1) * bits / 8 + 32;
            }
            else
            {
                from = _textureImage + (tl >> 2) * rowBytes + (sl >> 2) * bits / 8 - 16;
                to = _textureImage + (th >> 2) * rowBytes + (((long)sh >> 2) + 1) * bits / 8 + 32;
            }

            long bytes = Math.Max(_colorImageBytes, 1);
            bool color = _colorDrawnTo > 0 && from < _colorImage + _colorDrawnTo * bytes && to > _colorImage - 2 * bytes;
            bool depth = _depthDrawnTo > 0 && from < _depthImage + _depthDrawnTo * 2 && to > _depthImage;
            return color || depth;
        }

        // A row's stamp: the primitive's count over the row, so a later primitive's row is later than any row of an earlier one - see §2.8.
        private long Stamp(int row) => (_primitiveSequence << 11) | (uint)row;

        // The last row's last pixel past the image's width reads the next row's first bytes, which the owner of that row reads here, at its end of the primitive, which is the raster order's time - see §2.8.
        private void RecordAliasedRead((int First, int Last) rows, bool majorOnLeft)
        {
            if (_scaled) return;
            int y = rows.Last;
            while (y >= rows.First && (!_spanDrawn[y] || _spanRight[y] < _spanLeft[y])) y--;
            if (y < rows.First) return;

            int x = majorOnLeft ? _spanRight[y] : _spanLeft[y];
            if (x < _colorImageWidth || Owns(y) || !Owns(y + 1)) return;

            AliasedReads++;
            long stamp = Stamp(y + 1);
            int pixel = y * _colorImageWidth + x;
            ReadMemory(pixel);
            _memoryStamp = stamp;

            int deltaZEncoded = DeltaZEncoding(PrimitiveDepth ? _primitiveDeltaZ : _depthSlope);
            bool shifts = (CycleType == TwoCycle ? SecondBlendCycle.SecondAlpha : BlendSecondAlpha) == 1;
            if (!DepthCompare)
            {
                _pastStoredEncoded = 0xF;
                if (shifts) (_blendShiftA, _blendShiftB, _shiftStamp) = (0, deltaZEncoded < 0xB ? 4 : 0xF - deltaZEncoded, stamp);
            }
            else
            {
                (int stored, int hidden) = ReadDepthWord((_depthImage >> 1) + (uint)pixel);
                int storedEncoded = ((stored & 3) << 2) | hidden;
                _pastStoredEncoded = storedEncoded;
                if (shifts) (_blendShiftA, _blendShiftB, _shiftStamp) = (Math.Clamp(deltaZEncoded - storedEncoded, 0, 4), Math.Clamp(storedEncoded - deltaZEncoded, 0, 4), stamp);
            }

            _pastStoredStamp = stamp;
        }

        private static bool SelectsCombined(CombinerSelectors c) =>
            c.ColorA == 0 || c.ColorB == 0 || c.ColorD == 0 || c.ColorC == 0 || c.ColorC == 7 || c.AlphaA == 0 || c.AlphaB == 0 || c.AlphaD == 0;

        private bool Owns(int y) => _alone || y % _workers == _worker;

        public void Configure(int worker, int workers)
        {
            _worker = worker;
            _workers = workers;
            _alone = false;
        }

        // This processor's scratch replaced by the other's wherever the other wrote it later in raster order - see §2.8.
        public void TakeScratchFrom(Rdp other)
        {
            if (other._lastShadedStamp > _lastShadedStamp)
            {
                _lastShadedStamp = other._lastShadedStamp;
                (_combined, _pixel, _shade, _blenderShadeAlpha) = (other._combined, other._pixel, other._shade, other._blenderShadeAlpha);
            }

            if (other._memoryStamp > _memoryStamp) (_memoryStamp, _memory) = (other._memoryStamp, other._memory);
            if (other._pastStoredStamp > _pastStoredStamp) (_pastStoredStamp, _pastStoredEncoded) = (other._pastStoredStamp, other._pastStoredEncoded);
            if (other._texel0Stamp > _texel0Stamp) (_texel0Stamp, _texel0) = (other._texel0Stamp, other._texel0);
            if (other._texel1Stamp > _texel1Stamp) (_texel1Stamp, _texel1) = (other._texel1Stamp, other._texel1);
            if (other._lodStamp > _lodStamp) (_lodStamp, _lodFraction) = (other._lodStamp, other._lodFraction);
            if (other._blendedStamp > _blendedStamp) (_blendedStamp, _blended) = (other._blendedStamp, other._blended);
            if (other._shiftStamp > _shiftStamp) (_shiftStamp, _blendShiftA, _blendShiftB) = (other._shiftStamp, other._blendShiftA, other._blendShiftB);
            if (other._pastShiftStamp > _pastShiftStamp) (_pastShiftStamp, _pastShiftA, _pastShiftB) = (other._pastShiftStamp, other._pastShiftA, other._pastShiftB);

            for (int x = 0; x < _coverage.Length; x++)
            {
                if (other._coverageStamp[x] > _coverageStamp[x]) (_coverageStamp[x], _coverage[x]) = (other._coverageStamp[x], other._coverage[x]);
            }
        }

        // Everything the state holds, the decoded modes, and the stamps, so a processor joining a list stands where the leader stands - see §2.8.
        public void CopyStateFrom(Rdp other)
        {
            using var stream = new System.IO.MemoryStream();
            using (var w = new System.IO.BinaryWriter(stream, System.Text.Encoding.UTF8, leaveOpen: true)) EmuSen.Common.StateSerializer.Write(w, other);
            stream.Position = 0;

            // The walker's scratch is read at the console's width, and widened again after for a processor at a multiple - see §11.
            if (_scaled) Widen(1);
            using (var r = new System.IO.BinaryReader(stream, System.Text.Encoding.UTF8, leaveOpen: true)) EmuSen.Common.StateSerializer.Read(r, this);
            if (_scaled) Widen(_scale);
            Refresh();

            _primitiveSequence = other._primitiveSequence;
            (_colorDrawnTo, _depthDrawnTo) = (other._colorDrawnTo, other._depthDrawnTo);
            (_lastShadedStamp, _memoryStamp, _pastStoredStamp, _texel0Stamp, _texel1Stamp, _lodStamp, _blendedStamp, _shiftStamp, _pastShiftStamp) =
                (other._lastShadedStamp, other._memoryStamp, other._pastStoredStamp, other._texel0Stamp, other._texel1Stamp, other._lodStamp, other._blendedStamp, other._shiftStamp, other._pastShiftStamp);
            Array.Copy(other._coverageStamp, _coverageStamp, other._coverageStamp.Length);
        }

        public static uint Id(ulong word) => (uint)(word >> 56) & 0x3F;

        // What the interface's shadow takes after a state is read - see Mars_Rdp.md §2.6.
        public (int Taken, ulong First) Gathered => (_taken, _command[0]);

        public (uint Color, int Width, int Bytes, uint Depth, uint Texture, int TextureWidth, int TextureSize, int ScissorTop, int ScissorBottom, int ScissorRight) Images =>
            (_colorImage, _colorImageWidth, System.Math.Max(_colorImageBytes, 1), _depthImage, _textureImage, _textureImageWidth, _textureImageSize, _scissorTop >> 2, (_scissorBottom + 3) >> 2, (_scissorRight + 3) >> 2);

        // Every byte of RDRAM the processor reads, and every one it writes, passes here while the list runs on a thread - see Mars_Rdp.md §2.6.1.
        private void Touch(uint physical)
        {
            if (!_scaled && _bus.Dp.Verifying) _bus.Dp.Touched(physical, RunningWord);
        }

        private void Wrote(uint physical)
        {
            if (!_scaled && _bus.Dp.Verifying) _bus.Dp.Wrote(physical, RunningWord);
        }

        // In words: triangles grow by what they carry, texture rectangles take two, everything else one - see §3.
        public static int Length(uint id) => id switch
        {
            >= 0x08 and <= 0x0F => 4 + ((id & 4) != 0 ? 8 : 0) + ((id & 2) != 0 ? 8 : 0) + ((id & 1) != 0 ? 2 : 0),
            0x24 or 0x25 => 2,
            _ => 1,
        };

        // Commands that draw, load or set state act; the rest are taken whole and do nothing yet - see §4.
        private bool Execute(uint id, ulong word)
        {
            switch (id)
            {
                case >= 0x08 and <= 0x0F: Triangle(id); break;
                case TextureRectangle or TextureRectangleFlipped: TexturedRectangle(id == TextureRectangleFlipped); break;
                case SetTextureImage: TextureImage(word); break;
                case SetTile: Tile(word); break;
                case SetTileSize: TileSize(word); break;
                case LoadTile: Load(word, LoadKind.Tile); break;
                case LoadBlock: Load(word, LoadKind.Block); break;
                case LoadPalette: Load(word, LoadKind.Palette); break;
                case SyncFull: return true;
                case SetScissor: Scissor(word); break;
                case SetOtherModes: _otherModes = word; DecodeOtherModes(); break;
                case FillRectangle: Fill(word); break;
                case SetFillColor: _fillColor = (uint)word; break;
                case SetColorImage: ColorImage(word); break;
                default: SetRegister(id, word); break;
            }

            return false;
        }
    }
}
