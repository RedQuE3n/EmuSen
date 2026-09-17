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
                case SetCombine: _combine = word; return true;
                case SetPrimitiveColor:
                    _primitiveColor = Color.FromWord(word);
                    _primitiveLodFraction = (int)(word >> 32) & 0xFF;
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
                    _keyCenter.G = (int)(word >> 24) & 0xFF;
                    _keyScale.G = (int)(word >> 16) & 0xFF;
                    _keyCenter.B = (int)(word >> 8) & 0xFF;
                    _keyScale.B = (int)word & 0xFF;
                    return true;
                case SetKeyRed:
                    _keyCenter.R = (int)(word >> 8) & 0xFF;
                    _keyScale.R = (int)word & 0xFF;
                    return true;
                case SetPrimitiveDepth:
                    _primitiveDeltaZ = (int)word & 0xFFFF;
                    _primitiveZ = (int)((uint)word & (0x7FFFu << 16));
                    return true;
                case SetMaskImage: _depthImage = (uint)word & 0x00FF_FFFF; return true;
                default: return false;
            }
        }

        private int CycleType => (int)(_otherModes >> 52) & 3;
        private bool Perspective => ((_otherModes >> 51) & 1) != 0;
        private bool PaletteEnabled => ((_otherModes >> 47) & 1) != 0;
        private bool PaletteIntensityAlpha => ((_otherModes >> 46) & 1) != 0;
        private bool SampleFour => ((_otherModes >> 45) & 1) != 0;
        private bool MidTexel => ((_otherModes >> 44) & 1) != 0;
        private bool BilinearFirstCycle => ((_otherModes >> 43) & 1) != 0;
        private int RgbDither => (int)(_otherModes >> 38) & 3;
        private int AlphaDither => (int)(_otherModes >> 36) & 3;
        private bool KeyEnabled => ((_otherModes >> 40) & 1) != 0;

        // The one-cycle mode's blender reads the first cycle's selectors.
        private int BlendFirstColor => (int)(_otherModes >> 30) & 3;
        private int BlendFirstAlpha => (int)(_otherModes >> 26) & 3;
        private int BlendSecondColor => (int)(_otherModes >> 22) & 3;
        private int BlendSecondAlpha => (int)(_otherModes >> 18) & 3;

        private bool ForceBlend => ((_otherModes >> 14) & 1) != 0;
        private bool AlphaFromCoverage => ((_otherModes >> 13) & 1) != 0;
        private bool CoverageTimesAlpha => ((_otherModes >> 12) & 1) != 0;
        private int DepthMode => (int)(_otherModes >> 10) & 3;
        private int CoverageDestination => (int)(_otherModes >> 8) & 3;
        private bool ColorOnCoverage => ((_otherModes >> 7) & 1) != 0;
        private bool ImageRead => ((_otherModes >> 6) & 1) != 0;
        private bool DepthUpdate => ((_otherModes >> 5) & 1) != 0;
        private bool DepthCompare => ((_otherModes >> 4) & 1) != 0;
        private bool Antialias => ((_otherModes >> 3) & 1) != 0;
        private bool PrimitiveDepth => ((_otherModes >> 2) & 1) != 0;
        private bool DitherAlpha => ((_otherModes >> 1) & 1) != 0;
        private bool AlphaCompare => (_otherModes & 1) != 0;

        // (A - B) × C + D per channel; the one-cycle mode's combiner reads the second cycle's selectors.
        private int CombineColorA => (int)(_combine >> 37) & 0xF;
        private int CombineColorC => (int)(_combine >> 32) & 0x1F;
        private int CombineColorB => (int)(_combine >> 24) & 0xF;
        private int CombineColorD => (int)(_combine >> 6) & 7;
        private int CombineAlphaA => (int)(_combine >> 21) & 7;
        private int CombineAlphaC => (int)(_combine >> 18) & 7;
        private int CombineAlphaB => (int)(_combine >> 3) & 7;
        private int CombineAlphaD => (int)_combine & 7;
    }
}
