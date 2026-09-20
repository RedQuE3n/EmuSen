using System;
using System.Runtime.Intrinsics;

namespace EmuSen.Cores.Nintendo.Mars.Rsp
{
    // The vector unit's arithmetic eight elements at a time, the same functions as Rsp.VectorMath.cs lane for lane - see Mars_RspVector.md §14.
    public sealed partial class Rsp
    {
        // On by default where the hardware has 256-bit vectors; the element-by-element unit is the reference it is checked against.
        [EmuSen.Common.SkipInState] public bool UseSimd = Vector256.IsHardwareAccelerated && Vector128.IsHardwareAccelerated;

        private static readonly Vector128<byte>[] SelectorMasks = BuildSelectorMasks();
        private static readonly Vector128<ushort> LaneBits = Vector128.Create((ushort)1, 2, 4, 8, 16, 32, 64, 128);
        private static readonly Vector256<int> LaneBits32 = Vector256.Create(1, 2, 4, 8, 16, 32, 64, 128);

        private static Vector128<byte>[] BuildSelectorMasks()
        {
            var masks = new Vector128<byte>[16];
            for (int selector = 0; selector < 16; selector++)
            {
                var bytes = new byte[16];
                for (int i = 0; i < Elements; i++)
                {
                    int e = Selected(selector, i);
                    bytes[2 * i] = (byte)(2 * e);
                    bytes[2 * i + 1] = (byte)(2 * e + 1);
                }
                masks[selector] = Vector128.Create(bytes);
            }
            return masks;
        }

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private Vector128<ushort> Register(int index) => Vector128.LoadUnsafe(ref Vector[index * Elements]);

        // Inlined into a compiled block with its word a constant, the fields and both switches fold to the one operation - see Mars_Rsp.md §12.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private void ExecuteVectorSimd(uint instruction)
        {
            if (!_narrow) NarrowAccumulator();

            int vt = Rt(instruction);
            int vs = Rd(instruction);
            int vd = (int)((instruction >> 6) & 0x1F);
            int selector = (int)((instruction >> 21) & 0xF);

            // Both sources are read whole before the destination is stored, as §2 requires.
            Vector128<ushort> s = Register(vs);
            // Selections zero and one are the register as it stands; the rest are a byte shuffle whose indices are all in range, which the native form does in one instruction - see Mars_RspVector.md §15.
            Vector128<ushort> t = selector < 2 ? Register(vt) : Vector128.ShuffleNative(Register(vt).AsByte(), SelectorMasks[selector]).AsUInt16();
            Vector128<ushort> d = Register(vd);

            switch (instruction & 0x3F)
            {
                case 0x00: d = FractionSimd(s, t, unsigned: false, accumulate: false); break;
                case 0x01: d = FractionSimd(s, t, unsigned: true, accumulate: false); break;
                case 0x02: d = RoundSimd(t, shifted: (vs & 1) != 0, positive: true); break;
                case 0x03: d = QuarterSimd(s, t); break;
                case 0x04: d = LowSimd(s, t, accumulate: false); break;
                case 0x05: d = MiddleSimd(s, t, accumulate: false); break;
                case 0x06: d = NormalSimd(s, t, accumulate: false); break;
                case 0x07: d = HighSimd(s, t, accumulate: false); break;
                case 0x08: d = FractionSimd(s, t, unsigned: false, accumulate: true); break;
                case 0x09: d = FractionSimd(s, t, unsigned: true, accumulate: true); break;
                case 0x0A: d = RoundSimd(t, shifted: (vs & 1) != 0, positive: false); break;
                case 0x0B: d = AccumulatedQuarterSimd(); break;
                case 0x0C: d = LowSimd(s, t, accumulate: true); break;
                case 0x0D: d = MiddleSimd(s, t, accumulate: true); break;
                case 0x0E: d = NormalSimd(s, t, accumulate: true); break;
                case 0x0F: d = HighSimd(s, t, accumulate: true); break;

                case 0x10: d = AddSimd(s, t, subtract: false); break;
                case 0x11: d = AddSimd(s, t, subtract: true); break;
                case 0x13: d = AbsoluteSimd(s, t); break;
                case 0x14: d = AddCarryingSimd(s, t); break;
                case 0x15: d = SubtractCarryingSimd(s, t); break;
                case 0x1D: d = ReadAccumulatorSimd(selector); break;

                case 0x20: d = CompareSimd(s, t, VectorComparison.Less); break;
                case 0x21: d = CompareSimd(s, t, VectorComparison.Equal); break;
                case 0x22: d = CompareSimd(s, t, VectorComparison.NotEqual); break;
                case 0x23: d = CompareSimd(s, t, VectorComparison.GreaterOrEqual); break;
                case 0x24: d = ClipLowSimd(s, t); break;
                case 0x25: d = ClipHighSimd(s, t); break;
                case 0x26: d = ClipOnesComplementSimd(s, t); break;
                case 0x27: d = MergeSimd(s, t); break;

                case 0x28: d = LogicSimd(s & t); break;
                case 0x29: d = LogicSimd(~(s & t)); break;
                case 0x2A: d = LogicSimd(s | t); break;
                case 0x2B: d = LogicSimd(~(s | t)); break;
                case 0x2C: d = LogicSimd(s ^ t); break;
                case 0x2D: d = LogicSimd(~(s ^ t)); break;

                case 0x30: d = ReciprocateSimd(Vector[vt * Elements + (selector & 7)], vs & 7, t, d, VectorReciprocal.Single, root: false); break;
                case 0x31: d = ReciprocateSimd(Vector[vt * Elements + (selector & 7)], vs & 7, t, d, VectorReciprocal.Low, root: false); break;
                case 0x32: d = ReciprocateSimd(Vector[vt * Elements + (selector & 7)], vs & 7, t, d, VectorReciprocal.High, root: false); break;
                case 0x33: SetAccumulatorLow(t); d = d.WithElement(vs & 7, t.GetElement(vs & 7)); break;
                case 0x34: d = ReciprocateSimd(Vector[vt * Elements + (selector & 7)], vs & 7, t, d, VectorReciprocal.Single, root: true); break;
                case 0x35: d = ReciprocateSimd(Vector[vt * Elements + (selector & 7)], vs & 7, t, d, VectorReciprocal.Low, root: true); break;
                case 0x36: d = ReciprocateSimd(Vector[vt * Elements + (selector & 7)], vs & 7, t, d, VectorReciprocal.High, root: true); break;

                case 0x37:
                case 0x3F: return;

                default: SetAccumulatorLow(s + t); d = Vector128<ushort>.Zero; break;
            }

            d.StoreUnsafe(ref Vector[vd * Elements]);
        }

        // The accumulator as its three sixteen-bit thirds, which is what this unit computes in; the array a state carries is brought up to date only when something asks for it - see Mars_RspVector.md §15.
        [EmuSen.Common.SkipInState] private Vector128<ushort> _accHigh, _accMiddle, _accLow;
        [EmuSen.Common.SkipInState] private bool _narrow;

        private void NarrowAccumulator()
        {
            Span<ushort> high = stackalloc ushort[Elements], middle = stackalloc ushort[Elements], low = stackalloc ushort[Elements];
            for (int i = 0; i < Elements; i++)
            {
                high[i] = (ushort)(Accumulator[i] >> 32);
                middle[i] = (ushort)(Accumulator[i] >> 16);
                low[i] = (ushort)Accumulator[i];
            }

            (_accHigh, _accMiddle, _accLow) = (Vector128.Create<ushort>(high), Vector128.Create<ushort>(middle), Vector128.Create<ushort>(low));
            _narrow = true;
        }

        // Before anything reads the array: a state, the element-by-element unit, a test - see §15.
        public void WidenAccumulator()
        {
            if (!_narrow) return;
            for (int i = 0; i < Elements; i++) Accumulator[i] = ((ulong)_accHigh.GetElement(i) << 32) | ((ulong)_accMiddle.GetElement(i) << 16) | _accLow.GetElement(i);
            _narrow = false;
        }

        // After anything writes the array - see §15.
        public void AccumulatorWritten() => _narrow = false;

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector128<ushort> Narrow(Vector256<int> value) => Vector128.Narrow(value.GetLower(), value.GetUpper()).AsUInt16();

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private void SetAccumulatorLow(Vector128<ushort> value) => _accLow = value;

        // All ones where a lane's top bit is set.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector128<ushort> Sign(Vector128<ushort> value) => (value.AsInt16() >> 15).AsUInt16();

        // The lanes whose bit is set in the low eight bits of a flag word, as all-ones masks.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector128<ushort> LaneMask(int bits) => Vector128.Equals(Vector128.Create((ushort)bits) & LaneBits, LaneBits);

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector256<int> LaneMask32(int bits) => Vector256.Equals(Vector256.Create(bits) & LaneBits32, LaneBits32);

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static ushort Bits(Vector128<ushort> mask) => (ushort)mask.ExtractMostSignificantBits();

        // The products as 32-bit lanes; the mixed signedness fits a signed 32-bit lane exactly, so one multiply serves all four - see §14.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector256<int> Products(Vector128<ushort> s, Vector128<ushort> t, bool sSigned, bool tSigned)
        {
            Vector256<int> a = sSigned ? Vector256.WidenLower(s.AsInt16().ToVector256Unsafe()) : Vector256.WidenLower(s.ToVector256Unsafe()).AsInt32();
            Vector256<int> b = tSigned ? Vector256.WidenLower(t.AsInt16().ToVector256Unsafe()) : Vector256.WidenLower(t.ToVector256Unsafe()).AsInt32();
            return a * b;
        }

        // Forty-eight bits added in three lanes of sixteen, a lane that wrapped below its addend carrying one into the lane above; the accumulator wraps by having no fourth - see §15.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private void Add48(Vector128<ushort> high, Vector128<ushort> middle, Vector128<ushort> low, bool accumulate, ushort startLow = 0)
        {
            Vector128<ushort> baseHigh = accumulate ? _accHigh : Vector128<ushort>.Zero;
            Vector128<ushort> baseMiddle = accumulate ? _accMiddle : Vector128<ushort>.Zero;
            Vector128<ushort> baseLow = accumulate ? _accLow : Vector128.Create(startLow);

            Vector128<ushort> sumLow = baseLow + low;
            Vector128<ushort> carryLow = Vector128.LessThan(sumLow, low);
            Vector128<ushort> sumMiddle = baseMiddle + middle;
            Vector128<ushort> carryMiddle = Vector128.LessThan(sumMiddle, middle);
            Vector128<ushort> carried = sumMiddle - carryLow;
            Vector128<ushort> carryCarried = Vector128.LessThan(carried, sumMiddle);

            _accHigh = baseHigh + high - carryMiddle - carryCarried;
            _accMiddle = carried;
            _accLow = sumLow;
        }

        // The middle third, unless the upper two are not one signed sixteen-bit value, and then the nearer end - see §7.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private Vector128<ushort> ClampSignedMiddle()
        {
            Vector128<ushort> fits = Vector128.Equals(_accHigh, Sign(_accMiddle));
            return Vector128.ConditionalSelect(fits, _accMiddle, Sign(_accHigh) ^ Vector128.Create((ushort)0x7FFF));
        }

        // Nothing below zero, all ones above the largest positive upper word, the middle third between - see §7.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private Vector128<ushort> ClampUnsignedMiddle()
        {
            Vector128<ushort> large = ~Vector128.Equals(_accHigh, Vector128<ushort>.Zero) | Sign(_accMiddle);
            return (_accMiddle | large) & ~Sign(_accHigh);
        }

        // A low word is kept only while the word above it fits in sixteen signed bits - see §7.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private Vector128<ushort> ClampLow()
        {
            Vector128<ushort> fits = Vector128.Equals(_accHigh, Sign(_accMiddle));
            return Vector128.ConditionalSelect(fits, _accLow, ~Sign(_accHigh));
        }

        // The accumulator shifted down seventeen and clamped, less its low four bits: it fits only while the high third is all zeros or all ones - see §9.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private Vector128<ushort> ClampQuarter()
        {
            Vector128<ushort> fits = Vector128.Equals(_accHigh, Vector128<ushort>.Zero) | Vector128.Equals(_accHigh, Vector128<ushort>.AllBitsSet);
            Vector128<ushort> value = (_accMiddle >>> 1) | (_accHigh << 15);
            return Vector128.ConditionalSelect(fits, value, Sign(_accHigh) ^ Vector128.Create((ushort)0x7FFF)) & Vector128.Create((ushort)0xFFF0);
        }

        // Twice the signed product, whose one case past thirty-one bits, the most negative squared, is a plain positive here - see §15.
        private Vector128<ushort> FractionSimd(Vector128<ushort> s, Vector128<ushort> t, bool unsigned, bool accumulate)
        {
            Vector256<int> products = Products(s, t, sSigned: true, tSigned: true);
            Vector128<ushort> low = Narrow(products), high = Narrow(products >> 16);
            Add48(Sign(high), (high << 1) | (low >>> 15), low << 1, accumulate, startLow: 0x8000);
            return unsigned ? ClampUnsignedMiddle() : ClampSignedMiddle();
        }

        private Vector128<ushort> LowSimd(Vector128<ushort> s, Vector128<ushort> t, bool accumulate)
        {
            Vector128<ushort> upper = Narrow((Products(s, t, sSigned: false, tSigned: false).AsUInt32() >> 16).AsInt32());
            Add48(Vector128<ushort>.Zero, Vector128<ushort>.Zero, upper, accumulate);
            return ClampLow();
        }

        private Vector128<ushort> MiddleSimd(Vector128<ushort> s, Vector128<ushort> t, bool accumulate)
        {
            Vector256<int> products = Products(s, t, sSigned: true, tSigned: false);
            Vector128<ushort> high = Narrow(products >> 16);
            Add48(Sign(high), high, Narrow(products), accumulate);
            return ClampSignedMiddle();
        }

        private Vector128<ushort> NormalSimd(Vector128<ushort> s, Vector128<ushort> t, bool accumulate)
        {
            Vector256<int> products = Products(s, t, sSigned: false, tSigned: true);
            Vector128<ushort> high = Narrow(products >> 16);
            Add48(Sign(high), high, Narrow(products), accumulate);
            return ClampLow();
        }

        private Vector128<ushort> HighSimd(Vector128<ushort> s, Vector128<ushort> t, bool accumulate)
        {
            Vector256<int> products = Products(s, t, sSigned: true, tSigned: true);
            Add48(Narrow(products >> 16), Narrow(products), Vector128<ushort>.Zero, accumulate);
            return ClampSignedMiddle();
        }

        // A negative product is biased by thirty-one before it is kept, and the result loses its low four bits - see §7.
        private Vector128<ushort> QuarterSimd(Vector128<ushort> s, Vector128<ushort> t)
        {
            Vector256<int> products = Products(s, t, sSigned: true, tSigned: true);
            Vector128<ushort> high = Narrow(products >> 16);
            Add48(high, Narrow(products), Vector128<ushort>.Zero, accumulate: false);
            Add48(Vector128<ushort>.Zero, Sign(high) & Vector128.Create((ushort)0x1F), Vector128<ushort>.Zero, accumulate: true);
            return ClampQuarter();
        }

        // The accumulating quarter multiply ignores both sources and only nudges the accumulator toward zero - see §9.
        private Vector128<ushort> AccumulatedQuarterSimd()
        {
            Vector128<ushort> clear = Vector128.Equals(_accMiddle & Vector128.Create((ushort)0x20), Vector128<ushort>.Zero);
            Vector128<ushort> negative = Sign(_accHigh);
            Vector128<ushort> positive = ~negative & (~Vector128.Equals(_accHigh, Vector128<ushort>.Zero) | ~Vector128.Equals(_accMiddle >>> 6, Vector128<ushort>.Zero));

            Vector128<ushort> middle = clear & ((negative & Vector128.Create((ushort)0x0020)) | (positive & Vector128.Create((ushort)0xFFE0)));
            Add48(clear & positive, middle, Vector128<ushort>.Zero, accumulate: true);
            return ClampQuarter();
        }

        // The parity of vs's register number, not anything in it, decides whether vt is shifted - see §9.
        private Vector128<ushort> RoundSimd(Vector128<ushort> t, bool shifted, bool positive)
        {
            Vector128<ushort> take = positive ? ~Sign(_accHigh) : Sign(_accHigh);
            Vector128<ushort> sign = Sign(t);

            if (shifted) Add48(take & sign, take & t, Vector128<ushort>.Zero, accumulate: true);
            else Add48(take & sign, take & sign, take & t, accumulate: true);
            return ClampSignedMiddle();
        }

        // Carry in from the flags, clamp what is written, keep what was not clamped - see §8.
        private Vector128<ushort> AddSimd(Vector128<ushort> s, Vector128<ushort> t, bool subtract)
        {
            Vector256<int> a = Vector256.WidenLower(s.AsInt16().ToVector256Unsafe());
            Vector256<int> b = Vector256.WidenLower(t.AsInt16().ToVector256Unsafe());
            Vector256<int> carry = LaneMask32(Vco) & Vector256<int>.One;
            Vector256<int> sum = subtract ? a - b - carry : a + b + carry;

            SetAccumulatorLow(Narrow(sum));
            Vco = 0;
            return Narrow(Vector256.Min(Vector256.Max(sum, Vector256.Create((int)short.MinValue)), Vector256.Create((int)short.MaxValue)));
        }

        // The one negation that overflows keeps its overflow in the accumulator and clamps in the destination - see §8.
        private Vector128<ushort> AbsoluteSimd(Vector128<ushort> s, Vector128<ushort> t)
        {
            Vector128<ushort> negative = Vector128.LessThan(s.AsInt16(), Vector128<short>.Zero).AsUInt16();
            Vector128<ushort> zero = Vector128.Equals(s, Vector128<ushort>.Zero);
            Vector128<ushort> value = Vector128.ConditionalSelect(negative, Vector128<ushort>.Zero - t, Vector128.ConditionalSelect(zero, Vector128<ushort>.Zero, t));

            SetAccumulatorLow(value);
            return Vector128.ConditionalSelect(negative & Vector128.Equals(t, Vector128.Create((ushort)0x8000)), Vector128.Create((ushort)0x7FFF), value);
        }

        private Vector128<ushort> AddCarryingSimd(Vector128<ushort> s, Vector128<ushort> t)
        {
            Vector128<ushort> sum = s + t;
            SetAccumulatorLow(sum);
            Vco = Bits(Vector128.LessThan(sum, s));
            return sum;
        }

        // The borrow goes in the low byte and inequality in the high one - see §8.
        private Vector128<ushort> SubtractCarryingSimd(Vector128<ushort> s, Vector128<ushort> t)
        {
            Vector128<ushort> difference = s - t;
            SetAccumulatorLow(difference);
            Vco = (ushort)((Bits(~Vector128.Equals(s, t)) << 8) | Bits(Vector128.LessThan(s, t)));
            return difference;
        }

        // Selectors eight, nine and ten read the high, middle and low thirds, and every other one reads zero - see §6.
        private Vector128<ushort> ReadAccumulatorSimd(int selector) => selector switch
        {
            8 => _accHigh,
            9 => _accMiddle,
            10 => _accLow,
            _ => Vector128<ushort>.Zero,
        };

        // An equal pair is settled by whatever carry and not-equal flags VCO already holds - see §8.
        private Vector128<ushort> CompareSimd(Vector128<ushort> s, Vector128<ushort> t, VectorComparison comparison)
        {
            Vector128<ushort> equal = Vector128.Equals(s, t);
            Vector128<ushort> carry = LaneMask(Vco), notEqual = LaneMask(Vco >> 8);

            Vector128<ushort> chosen = comparison switch
            {
                VectorComparison.Less => Vector128.LessThan(s.AsInt16(), t.AsInt16()).AsUInt16() | (equal & carry & notEqual),
                VectorComparison.Equal => equal & ~notEqual,
                VectorComparison.NotEqual => ~equal | notEqual,
                _ => Vector128.GreaterThan(s.AsInt16(), t.AsInt16()).AsUInt16() | (equal & ~(carry & notEqual)),
            };

            Vector128<ushort> d = comparison switch
            {
                VectorComparison.Equal => t,
                VectorComparison.NotEqual => s,
                _ => Vector128.ConditionalSelect(chosen, s, t),
            };

            SetAccumulatorLow(d);
            Vcc = Bits(chosen);
            Vco = 0;
            return d;
        }

        // The low clip reads the flags a high clip left, and where not-equal is set the old comparison stands - see §8.
        private Vector128<ushort> ClipLowSimd(Vector128<ushort> s, Vector128<ushort> t)
        {
            Vector128<ushort> signsDiffered = LaneMask(Vco), notEqual = LaneMask(Vco >> 8), extension = LaneMask(Vce);
            Vector128<ushort> lessOrEqual = LaneMask(Vcc), greaterOrEqual = LaneMask(Vcc >> 8);

            Vector128<ushort> sum = s + t;
            Vector128<ushort> zero = Vector128.Equals(sum, Vector128<ushort>.Zero);
            Vector128<ushort> carried = Vector128.LessThan(sum, s);
            Vector128<ushort> lessNew = (zero & ~carried) | (extension & (zero | ~carried));
            Vector128<ushort> greaterNew = Vector128.GreaterThanOrEqual(s, t);

            Vector128<ushort> less = Vector128.ConditionalSelect(notEqual, lessOrEqual, lessNew);
            Vector128<ushort> greater = Vector128.ConditionalSelect(notEqual, greaterOrEqual, greaterNew);

            Vector128<ushort> d = Vector128.ConditionalSelect(signsDiffered,
                Vector128.ConditionalSelect(less, Vector128<ushort>.Zero - t, s),
                Vector128.ConditionalSelect(greater, t, s));

            SetAccumulatorLow(d);
            Vcc = (ushort)(Bits(Vector128.ConditionalSelect(signsDiffered, less, lessOrEqual)) | (Bits(Vector128.ConditionalSelect(signsDiffered, greaterOrEqual, greater)) << 8));
            Vco = 0;
            Vce = 0;
            return d;
        }

        private Vector128<ushort> ClipHighSimd(Vector128<ushort> s, Vector128<ushort> t)
        {
            Vector128<short> a = t.AsInt16(), b = s.AsInt16();
            Vector128<short> signsDiffer = Vector128.LessThan(a ^ b, Vector128<short>.Zero);
            Vector128<short> sum = a + b, difference = b - a;
            Vector128<short> aNegative = Vector128.LessThan(a, Vector128<short>.Zero);

            Vector128<short> greaterOrEqual = Vector128.ConditionalSelect(signsDiffer, aNegative, Vector128.GreaterThanOrEqual(difference, Vector128<short>.Zero));
            Vector128<short> lessOrEqual = Vector128.ConditionalSelect(signsDiffer, Vector128.LessThanOrEqual(sum, Vector128<short>.Zero), aNegative);
            Vector128<short> extension = signsDiffer & Vector128.Equals(sum, Vector128.Create((short)-1));
            Vector128<short> notEqual = Vector128.ConditionalSelect(signsDiffer,
                ~(Vector128.Equals(sum, Vector128<short>.Zero) | Vector128.Equals(sum, Vector128.Create((short)-1))),
                ~Vector128.Equals(difference, Vector128<short>.Zero));

            Vector128<short> d = Vector128.ConditionalSelect(signsDiffer,
                Vector128.ConditionalSelect(lessOrEqual, -a, b),
                Vector128.ConditionalSelect(greaterOrEqual, a, b));

            SetAccumulatorLow(d.AsUInt16());
            Vco = (ushort)(Bits(signsDiffer.AsUInt16()) | (Bits(notEqual.AsUInt16()) << 8));
            Vcc = (ushort)(Bits(lessOrEqual.AsUInt16()) | (Bits(greaterOrEqual.AsUInt16()) << 8));
            Vce = (byte)Bits(extension.AsUInt16());
            return d.AsUInt16();
        }

        // The one's-complement clip negates by inverting, and leaves no carry or extension flags behind - see §8.
        private Vector128<ushort> ClipOnesComplementSimd(Vector128<ushort> s, Vector128<ushort> t)
        {
            Vector128<short> a = t.AsInt16(), b = s.AsInt16();
            Vector128<short> signsDiffer = Vector128.LessThan(a ^ b, Vector128<short>.Zero);
            Vector128<short> aNegative = Vector128.LessThan(a, Vector128<short>.Zero);

            Vector128<short> greaterOrEqual = Vector128.ConditionalSelect(signsDiffer, aNegative, Vector128.GreaterThanOrEqual(b - a, Vector128<short>.Zero));
            Vector128<short> lessOrEqual = Vector128.ConditionalSelect(signsDiffer, Vector128.LessThan(a + b, Vector128<short>.Zero), aNegative);

            Vector128<short> d = Vector128.ConditionalSelect(signsDiffer,
                Vector128.ConditionalSelect(lessOrEqual, ~a, b),
                Vector128.ConditionalSelect(greaterOrEqual, a, b));

            SetAccumulatorLow(d.AsUInt16());
            Vco = 0;
            Vcc = (ushort)(Bits(lessOrEqual.AsUInt16()) | (Bits(greaterOrEqual.AsUInt16()) << 8));
            Vce = 0;
            return d.AsUInt16();
        }

        private Vector128<ushort> MergeSimd(Vector128<ushort> s, Vector128<ushort> t)
        {
            Vector128<ushort> d = Vector128.ConditionalSelect(LaneMask(Vcc), s, t);
            SetAccumulatorLow(d);
            Vco = 0;
            return d;
        }

        private Vector128<ushort> LogicSimd(Vector128<ushort> d)
        {
            SetAccumulatorLow(d);
            return d;
        }

        // One element is computed as before; the whole selected vt still reaches the accumulator - see §10.
        private Vector128<ushort> ReciprocateSimd(ushort input, int element, Vector128<ushort> t, Vector128<ushort> d, VectorReciprocal kind, bool root)
        {
            SetAccumulatorLow(t);

            if (kind == VectorReciprocal.High)
            {
                _divideInput = input;
                _divideInputLoaded = true;
                return d.WithElement(element, _divideOutput);
            }

            uint operand = kind == VectorReciprocal.Low && _divideInputLoaded
                ? ((uint)_divideInput << 16) | input
                : (uint)(int)(short)input;
            uint result = root ? Reciprocals.RootOf(operand) : Reciprocals.Of(operand);

            _divideOutput = (ushort)(result >> 16);
            _divideInputLoaded = false;
            return d.WithElement(element, (ushort)result);
        }
    }
}
