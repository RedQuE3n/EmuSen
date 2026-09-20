namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // Other modes, the combiner's selectors, and the colour registers they read - see Mars_RdpCoverage.md §1.
    public sealed partial class Rdp
    {
        public const uint SetKeyGreenBlue = 0x2A;
        public const uint SetKeyRed = 0x2B;
        public const uint SetConvert = 0x2C;
        public const uint SetPrimitiveDepth = 0x2E;
        public const uint SetFogColor = 0x38;
        public const uint SetBlendColor = 0x39;
        public const uint SetPrimitiveColor = 0x3A;
        public const uint SetEnvironmentColor = 0x3B;
        public const uint SetCombine = 0x3C;
        public const uint SetMaskImage = 0x3E;

        private struct Color
        {
            public int R, G, B, A;

            public static Color FromWord(ulong word) =>
                new() { R = (int)(word >> 24) & 0xFF, G = (int)(word >> 16) & 0xFF, B = (int)(word >> 8) & 0xFF, A = (int)word & 0xFF };
        }

        private ulong _combine;

        private Color _primitiveColor;
        private Color _environmentColor;
        private Color _blendColor;
        private Color _fogColor;
        private Color _keyCenter;
        private Color _keyScale;

        private int _primitiveLodFraction;
        private int _minLevel;
        private int _lodFraction;
        private int _k4;
        private int _k5;

        // The YUV conversion's four constants, each nine-bit signed and taken as twice itself plus one - see Mars_RdpTextures.md §5.3.
        private int _k0;
        private int _k1;
        private int _k2;
        private int _k3;
        private int _primitiveDeltaZ;
        private int _primitiveZ;

        private bool SetRegister(uint id, ulong word)
        {
            switch (id)
            {
                case SetCombine: _combine = word; DecodeCombine(); return true;
                case SetPrimitiveColor:
                    _primitiveColor = Color.FromWord(word);
                    _primitiveLodFraction = (int)(word >> 32) & 0xFF;
                    _minLevel = (int)(word >> 40) & 0x1F;
                    return true;
                case SetEnvironmentColor: _environmentColor = Color.FromWord(word); return true;
                case SetBlendColor: _blendColor = Color.FromWord(word); return true;
                case SetFogColor: _fogColor = Color.FromWord(word); return true;
                case SetConvert:
                    _k0 = (SignExtend(word >> 45, 9) << 1) + 1;
                    _k1 = (SignExtend(word >> 36, 9) << 1) + 1;
                    _k2 = (SignExtend(word >> 27, 9) << 1) + 1;
                    _k3 = (SignExtend(word >> 18, 9) << 1) + 1;
                    _k4 = (int)(word >> 9) & 0x1FF;
                    _k5 = (int)word & 0x1FF;
                    return true;
                case SetKeyGreenBlue:
                    _keyWidth.G = (int)(word >> 44) & 0xFFF;
                    _keyWidth.B = (int)(word >> 32) & 0xFFF;
                    _keyCenter.G = (int)(word >> 24) & 0xFF;
                    _keyScale.G = (int)(word >> 16) & 0xFF;
                    _keyCenter.B = (int)(word >> 8) & 0xFF;
                    _keyScale.B = (int)word & 0xFF;
                    return true;
                case SetKeyRed:
                    _keyWidth.R = (int)(word >> 16) & 0xFFF;
                    _keyCenter.R = (int)(word >> 8) & 0xFF;
                    _keyScale.R = (int)word & 0xFF;
                    return true;
                case SetPrimitiveDepth:
                    _primitiveDeltaZ = (int)word & 0xFFFF;
                    _primitiveZ = (int)((uint)word & (0x7FFFu << 16));
                    return true;
                case SetMaskImage: _depthImage = ((uint)word & 0x00FF_FFFF) * (uint)(_scale * _scale); _depthDrawnTo = 0; return true;
                default: return false;
            }
        }

        // Every mode bit is read once a pixel and changes only with its word, so each is a field decoded when the word lands - see Mars_Performance.md §23.
        [EmuSen.Common.SkipInState] private int _cycleType;
        [EmuSen.Common.SkipInState] private bool _perspective;
        [EmuSen.Common.SkipInState] private bool _detailEnabled;
        [EmuSen.Common.SkipInState] private bool _sharpenEnabled;
        [EmuSen.Common.SkipInState] private bool _lodEnabled;
        [EmuSen.Common.SkipInState] private bool _paletteEnabled;
        [EmuSen.Common.SkipInState] private bool _paletteIntensityAlpha;
        [EmuSen.Common.SkipInState] private bool _sampleFour;
        [EmuSen.Common.SkipInState] private bool _midTexel;
        [EmuSen.Common.SkipInState] private bool _bilinearFirstCycle;
        [EmuSen.Common.SkipInState] private int _rgbDither;
        [EmuSen.Common.SkipInState] private int _alphaDither;
        [EmuSen.Common.SkipInState] private bool _keyEnabled;
        [EmuSen.Common.SkipInState] private bool _bilinearSecondCycle;
        [EmuSen.Common.SkipInState] private bool _convertOne;
        [EmuSen.Common.SkipInState] private BlendSelectors _firstBlendCycle;
        [EmuSen.Common.SkipInState] private BlendSelectors _secondBlendCycle;
        [EmuSen.Common.SkipInState] private int _blendFirstColor;
        [EmuSen.Common.SkipInState] private int _blendFirstAlpha;
        [EmuSen.Common.SkipInState] private int _blendSecondColor;
        [EmuSen.Common.SkipInState] private int _blendSecondAlpha;
        [EmuSen.Common.SkipInState] private bool _forceBlend;
        [EmuSen.Common.SkipInState] private bool _alphaFromCoverage;
        [EmuSen.Common.SkipInState] private bool _coverageTimesAlpha;
        [EmuSen.Common.SkipInState] private int _depthMode;
        [EmuSen.Common.SkipInState] private int _coverageDestination;
        [EmuSen.Common.SkipInState] private bool _colorOnCoverage;
        [EmuSen.Common.SkipInState] private bool _imageRead;
        [EmuSen.Common.SkipInState] private bool _depthUpdate;
        [EmuSen.Common.SkipInState] private bool _depthCompare;
        [EmuSen.Common.SkipInState] private bool _antialias;
        [EmuSen.Common.SkipInState] private bool _primitiveDepth;
        [EmuSen.Common.SkipInState] private bool _ditherAlpha;
        [EmuSen.Common.SkipInState] private bool _alphaCompare;
        [EmuSen.Common.SkipInState] private CombinerSelectors _secondCombineCycle;
        [EmuSen.Common.SkipInState] private int _combineColorA;
        [EmuSen.Common.SkipInState] private int _combineColorC;
        [EmuSen.Common.SkipInState] private int _combineColorB;
        [EmuSen.Common.SkipInState] private int _combineColorD;
        [EmuSen.Common.SkipInState] private int _combineAlphaA;
        [EmuSen.Common.SkipInState] private int _combineAlphaC;
        [EmuSen.Common.SkipInState] private int _combineAlphaB;
        [EmuSen.Common.SkipInState] private int _combineAlphaD;

        // After every write of either word, and after a loaded state; Debug builds check at every draw that neither was missed - see Mars_Performance.md §23.
        public void Refresh()
        {
            DecodeOtherModes();
            DecodeCombine();
        }

        private void DecodeOtherModes()
        {
            _cycleType = (int)(_otherModes >> 52) & 3;
            _perspective = ((_otherModes >> 51) & 1) != 0;
            _detailEnabled = ((_otherModes >> 50) & 1) != 0;
            _sharpenEnabled = ((_otherModes >> 49) & 1) != 0;
            _lodEnabled = ((_otherModes >> 48) & 1) != 0;
            _paletteEnabled = ((_otherModes >> 47) & 1) != 0;
            _paletteIntensityAlpha = ((_otherModes >> 46) & 1) != 0;
            _sampleFour = ((_otherModes >> 45) & 1) != 0;
            _midTexel = ((_otherModes >> 44) & 1) != 0;
            _bilinearFirstCycle = ((_otherModes >> 43) & 1) != 0;
            _rgbDither = (int)(_otherModes >> 38) & 3;
            _alphaDither = (int)(_otherModes >> 36) & 3;
            _ditherTable = DitherTables[(_rgbDither << 2) | _alphaDither];
            _keyEnabled = ((_otherModes >> 40) & 1) != 0;
            _bilinearSecondCycle = ((_otherModes >> 42) & 1) != 0;
            _convertOne = ((_otherModes >> 41) & 1) != 0;
            _blendFirstColor = (int)(_otherModes >> 30) & 3;
            _blendFirstAlpha = (int)(_otherModes >> 26) & 3;
            _blendSecondColor = (int)(_otherModes >> 22) & 3;
            _blendSecondAlpha = (int)(_otherModes >> 18) & 3;
            _forceBlend = ((_otherModes >> 14) & 1) != 0;
            _alphaFromCoverage = ((_otherModes >> 13) & 1) != 0;
            _coverageTimesAlpha = ((_otherModes >> 12) & 1) != 0;
            _depthMode = (int)(_otherModes >> 10) & 3;
            _coverageDestination = (int)(_otherModes >> 8) & 3;
            _colorOnCoverage = ((_otherModes >> 7) & 1) != 0;
            _imageRead = ((_otherModes >> 6) & 1) != 0;
            _depthUpdate = ((_otherModes >> 5) & 1) != 0;
            _depthCompare = ((_otherModes >> 4) & 1) != 0;
            _antialias = ((_otherModes >> 3) & 1) != 0;
            _primitiveDepth = ((_otherModes >> 2) & 1) != 0;
            _ditherAlpha = ((_otherModes >> 1) & 1) != 0;
            _alphaCompare = (_otherModes & 1) != 0;
            _firstBlendCycle = new(BlendFirstColor, BlendFirstAlpha, BlendSecondColor, BlendSecondAlpha);
            _secondBlendCycle = new((int)(_otherModes >> 28) & 3, (int)(_otherModes >> 24) & 3, (int)(_otherModes >> 20) & 3, (int)(_otherModes >> 16) & 3);
        }

        private void DecodeCombine()
        {
            _combineColorA = (int)(_combine >> 37) & 0xF;
            _combineColorC = (int)(_combine >> 32) & 0x1F;
            _combineColorB = (int)(_combine >> 24) & 0xF;
            _combineColorD = (int)(_combine >> 6) & 7;
            _combineAlphaA = (int)(_combine >> 21) & 7;
            _combineAlphaC = (int)(_combine >> 18) & 7;
            _combineAlphaB = (int)(_combine >> 3) & 7;
            _combineAlphaD = (int)_combine & 7;
            _secondCombineCycle = new(CombineColorA, CombineColorB, CombineColorC, CombineColorD, CombineAlphaA, CombineAlphaB, CombineAlphaC, CombineAlphaD);
            _firstCombineCycle = DecodedFirstCombineCycle;
        }

        [System.Diagnostics.Conditional("DEBUG")]
        private void VerifyModes()
        {

            if (CycleType != ((int)(_otherModes >> 52) & 3)) Stale(nameof(CycleType));
            if (Perspective != (((_otherModes >> 51) & 1) != 0)) Stale(nameof(Perspective));
            if (DetailEnabled != (((_otherModes >> 50) & 1) != 0)) Stale(nameof(DetailEnabled));
            if (SharpenEnabled != (((_otherModes >> 49) & 1) != 0)) Stale(nameof(SharpenEnabled));
            if (LodEnabled != (((_otherModes >> 48) & 1) != 0)) Stale(nameof(LodEnabled));
            if (PaletteEnabled != (((_otherModes >> 47) & 1) != 0)) Stale(nameof(PaletteEnabled));
            if (PaletteIntensityAlpha != (((_otherModes >> 46) & 1) != 0)) Stale(nameof(PaletteIntensityAlpha));
            if (SampleFour != (((_otherModes >> 45) & 1) != 0)) Stale(nameof(SampleFour));
            if (MidTexel != (((_otherModes >> 44) & 1) != 0)) Stale(nameof(MidTexel));
            if (BilinearFirstCycle != (((_otherModes >> 43) & 1) != 0)) Stale(nameof(BilinearFirstCycle));
            if (RgbDither != ((int)(_otherModes >> 38) & 3)) Stale(nameof(RgbDither));
            if (!ReferenceEquals(_ditherTable, DitherTables[(RgbDither << 2) | AlphaDither])) Stale("the dither table");
            if (AlphaDither != ((int)(_otherModes >> 36) & 3)) Stale(nameof(AlphaDither));
            if (KeyEnabled != (((_otherModes >> 40) & 1) != 0)) Stale(nameof(KeyEnabled));
            if (BilinearSecondCycle != (((_otherModes >> 42) & 1) != 0)) Stale(nameof(BilinearSecondCycle));
            if (ConvertOne != (((_otherModes >> 41) & 1) != 0)) Stale(nameof(ConvertOne));
            if (FirstCombineCycle != DecodedFirstCombineCycle) Stale(nameof(FirstCombineCycle));
            if (SecondBlendCycle != (new BlendSelectors((int)(_otherModes >> 28) & 3, (int)(_otherModes >> 24) & 3, (int)(_otherModes >> 20) & 3, (int)(_otherModes >> 16) & 3))) Stale(nameof(SecondBlendCycle));
            if (BlendFirstColor != ((int)(_otherModes >> 30) & 3)) Stale(nameof(BlendFirstColor));
            if (BlendFirstAlpha != ((int)(_otherModes >> 26) & 3)) Stale(nameof(BlendFirstAlpha));
            if (BlendSecondColor != ((int)(_otherModes >> 22) & 3)) Stale(nameof(BlendSecondColor));
            if (BlendSecondAlpha != ((int)(_otherModes >> 18) & 3)) Stale(nameof(BlendSecondAlpha));
            if (ForceBlend != (((_otherModes >> 14) & 1) != 0)) Stale(nameof(ForceBlend));
            if (AlphaFromCoverage != (((_otherModes >> 13) & 1) != 0)) Stale(nameof(AlphaFromCoverage));
            if (CoverageTimesAlpha != (((_otherModes >> 12) & 1) != 0)) Stale(nameof(CoverageTimesAlpha));
            if (DepthMode != ((int)(_otherModes >> 10) & 3)) Stale(nameof(DepthMode));
            if (CoverageDestination != ((int)(_otherModes >> 8) & 3)) Stale(nameof(CoverageDestination));
            if (ColorOnCoverage != (((_otherModes >> 7) & 1) != 0)) Stale(nameof(ColorOnCoverage));
            if (ImageRead != (((_otherModes >> 6) & 1) != 0)) Stale(nameof(ImageRead));
            if (DepthUpdate != (((_otherModes >> 5) & 1) != 0)) Stale(nameof(DepthUpdate));
            if (DepthCompare != (((_otherModes >> 4) & 1) != 0)) Stale(nameof(DepthCompare));
            if (Antialias != (((_otherModes >> 3) & 1) != 0)) Stale(nameof(Antialias));
            if (PrimitiveDepth != (((_otherModes >> 2) & 1) != 0)) Stale(nameof(PrimitiveDepth));
            if (DitherAlpha != (((_otherModes >> 1) & 1) != 0)) Stale(nameof(DitherAlpha));
            if (AlphaCompare != ((_otherModes & 1) != 0)) Stale(nameof(AlphaCompare));
            if (CombineColorA != ((int)(_combine >> 37) & 0xF)) Stale(nameof(CombineColorA));
            if (CombineColorC != ((int)(_combine >> 32) & 0x1F)) Stale(nameof(CombineColorC));
            if (CombineColorB != ((int)(_combine >> 24) & 0xF)) Stale(nameof(CombineColorB));
            if (CombineColorD != ((int)(_combine >> 6) & 7)) Stale(nameof(CombineColorD));
            if (CombineAlphaA != ((int)(_combine >> 21) & 7)) Stale(nameof(CombineAlphaA));
            if (CombineAlphaC != ((int)(_combine >> 18) & 7)) Stale(nameof(CombineAlphaC));
            if (CombineAlphaB != ((int)(_combine >> 3) & 7)) Stale(nameof(CombineAlphaB));
            if (CombineAlphaD != ((int)_combine & 7)) Stale(nameof(CombineAlphaD));
        }

        private static void Stale(string mode) =>
            throw new System.InvalidOperationException($"{mode} was read from a mode word that changed without a decode; the display processor's modes are stale.");

        private int CycleType => _cycleType;
        private bool Perspective => _perspective;
        private bool DetailEnabled => _detailEnabled;
        private bool SharpenEnabled => _sharpenEnabled;
        private bool LodEnabled => _lodEnabled;
        private bool PaletteEnabled => _paletteEnabled;
        private bool PaletteIntensityAlpha => _paletteIntensityAlpha;
        private bool SampleFour => _sampleFour;
        private bool MidTexel => _midTexel;
        private bool BilinearFirstCycle => _bilinearFirstCycle;
        private int RgbDither => _rgbDither;
        private int AlphaDither => _alphaDither;
        private bool KeyEnabled => _keyEnabled;

        private bool BilinearSecondCycle => _bilinearSecondCycle;
        private bool ConvertOne => _convertOne;

        // Which colour and alpha each blender cycle weighs; the one-cycle mode reads the first cycle's - see Mars_RdpTwoCycle.md §4.
        private readonly record struct BlendSelectors(int FirstColor, int FirstAlpha, int SecondColor, int SecondAlpha);

        private BlendSelectors FirstBlendCycle => _firstBlendCycle;
        private BlendSelectors SecondBlendCycle => _secondBlendCycle;

        // The one-cycle mode's blender reads the first cycle's selectors.
        private int BlendFirstColor => _blendFirstColor;
        private int BlendFirstAlpha => _blendFirstAlpha;
        private int BlendSecondColor => _blendSecondColor;
        private int BlendSecondAlpha => _blendSecondAlpha;

        private bool ForceBlend => _forceBlend;
        private bool AlphaFromCoverage => _alphaFromCoverage;
        private bool CoverageTimesAlpha => _coverageTimesAlpha;
        private int DepthMode => _depthMode;
        private int CoverageDestination => _coverageDestination;
        private bool ColorOnCoverage => _colorOnCoverage;
        private bool ImageRead => _imageRead;
        private bool DepthUpdate => _depthUpdate;
        private bool DepthCompare => _depthCompare;
        private bool Antialias => _antialias;
        private bool PrimitiveDepth => _primitiveDepth;
        private bool DitherAlpha => _ditherAlpha;
        private bool AlphaCompare => _alphaCompare;

        // Each combiner cycle's eight inputs; the one-cycle mode's combiner reads the second cycle's - see Mars_RdpTwoCycle.md §4.
        private readonly record struct CombinerSelectors(int ColorA, int ColorB, int ColorC, int ColorD, int AlphaA, int AlphaB, int AlphaC, int AlphaD);

        private CombinerSelectors FirstCombineCycle => _firstCombineCycle;

        [EmuSen.Common.SkipInState] private CombinerSelectors _firstCombineCycle;

        private CombinerSelectors DecodedFirstCombineCycle => new((int)(_combine >> 52) & 0xF, (int)(_combine >> 28) & 0xF, (int)(_combine >> 47) & 0x1F, (int)(_combine >> 15) & 7,
            (int)(_combine >> 44) & 7, (int)(_combine >> 12) & 7, (int)(_combine >> 41) & 7, (int)(_combine >> 9) & 7);

        private CombinerSelectors SecondCombineCycle => _secondCombineCycle;

        // (A - B) × C + D per channel; the one-cycle mode's combiner reads the second cycle's selectors.
        private int CombineColorA => _combineColorA;
        private int CombineColorC => _combineColorC;
        private int CombineColorB => _combineColorB;
        private int CombineColorD => _combineColorD;
        private int CombineAlphaA => _combineAlphaA;
        private int CombineAlphaC => _combineAlphaC;
        private int CombineAlphaB => _combineAlphaB;
        private int CombineAlphaD => _combineAlphaD;
    }
}
