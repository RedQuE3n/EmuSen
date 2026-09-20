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
        private static readonly Vector256<ulong> Mask48 = Vector256.Create(AccumulatorMask);
        private static readonly Vector256<ulong> LowMask = Vector256.Create(0xFFFFUL);

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

        private void ExecuteVectorSimd(uint instruction)
        {
            int vt = Rt(instruction);
            int vs = Rd(instruction);
            int vd = (int)((instruction >> 6) & 0x1F);
            int selector = (int)((instruction >> 21) & 0xF);

            // Both sources are read whole before the destination is stored, as §2 requires.
            Vector128<ushort> s = Register(vs);
            Vector128<ushort> t = Vector128.Shuffle(Register(vt).AsByte(), SelectorMasks[selector]).AsUInt16();
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

        // The accumulator as two vectors of four 64-bit lanes, the array itself being the state a save carries - see §14.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private (Vector256<ulong> Low, Vector256<ulong> High) LoadAccumulator() =>
            (Vector256.LoadUnsafe(ref Accumulator[0]), Vector256.LoadUnsafe(ref Accumulator[4]));

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private void StoreAccumulator(Vector256<ulong> low, Vector256<ulong> high)
        {
            low.StoreUnsafe(ref Accumulator[0]);
            high.StoreUnsafe(ref Accumulator[4]);
        }

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector256<long> Signed48(Vector256<ulong> value) => (value << 16).AsInt64() >> 16;

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector256<ulong> Wrap48(Vector256<long> value) => value.AsUInt64() & Mask48;

        // Sixteen-bit lanes widened to the two halves' 64-bit lanes, zero-filled or sign-filled.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static (Vector256<ulong> Low, Vector256<ulong> High) WidenUnsigned(Vector128<ushort> value)
        {
            Vector256<uint> wide = Vector256.WidenLower(value.ToVector256Unsafe());
            return (Vector256.WidenLower(wide), Vector256.WidenUpper(wide));
        }

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static (Vector256<long> Low, Vector256<long> High) WidenSigned(Vector128<ushort> value)
        {
            Vector256<int> wide = Vector256.WidenLower(value.AsInt16().ToVector256Unsafe());
            return (Vector256.WidenLower(wide), Vector256.WidenUpper(wide));
        }

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static (Vector256<long> Low, Vector256<long> High) WidenSigned(Vector256<int> value) =>
            (Vector256.WidenLower(value), Vector256.WidenUpper(value));

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector128<ushort> Narrow(Vector256<long> low, Vector256<long> high)
        {
            Vector256<int> wide = Vector256.Narrow(low, high);
            return Vector128.Narrow(wide.GetLower(), wide.GetUpper()).AsUInt16();
        }

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector128<ushort> Narrow(Vector256<int> value) => Vector128.Narrow(value.GetLower(), value.GetUpper()).AsUInt16();

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private void SetAccumulatorLow(Vector128<ushort> value)
        {
            (Vector256<ulong> low, Vector256<ulong> high) = LoadAccumulator();
            (Vector256<ulong> valueLow, Vector256<ulong> valueHigh) = WidenUnsigned(value);
            StoreAccumulator((low & ~LowMask) | valueLow, (high & ~LowMask) | valueHigh);
        }

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

        // The accumulator wraps first and every clamp reads what it wrapped to, as Accumulate does - see §6.1.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private (Vector256<long> Low, Vector256<long> High) Accumulate(Vector256<long> addendLow, Vector256<long> addendHigh, bool accumulate, long start)
        {
            (Vector256<ulong> low, Vector256<ulong> high) = LoadAccumulator();
            Vector256<long> baseLow = accumulate ? Signed48(low) : Vector256.Create(start);
            Vector256<long> baseHigh = accumulate ? Signed48(high) : Vector256.Create(start);

            low = Wrap48(baseLow + addendLow);
            high = Wrap48(baseHigh + addendHigh);
            StoreAccumulator(low, high);
            return (Signed48(low), Signed48(high));
        }

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector256<long> ClampSigned(Vector256<long> value) =>
            Vector256.Min(Vector256.Max(value, Vector256.Create((long)short.MinValue)), Vector256.Create((long)short.MaxValue));

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector128<ushort> ClampSignedHalf(Vector256<long> low, Vector256<long> high) => Narrow(ClampSigned(low >> 16), ClampSigned(high >> 16));

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector256<long> ClampUnsigned(Vector256<long> value)
        {
            Vector256<long> negative = Vector256.LessThan(value, Vector256<long>.Zero);
            Vector256<long> large = Vector256.GreaterThan(value, Vector256.Create(0x7FFF_FFFFL));
            return Vector256.ConditionalSelect(negative, Vector256<long>.Zero, Vector256.ConditionalSelect(large, Vector256.Create(0xFFFFL), (value >> 16) & Vector256.Create(0xFFFFL)));
        }

        // A low word is kept only while the word above it fits in sixteen signed bits - see §7.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private static Vector256<long> ClampLow(Vector256<long> value)
        {
            Vector256<long> upper = value >> 16;
            Vector256<long> under = Vector256.LessThan(upper, Vector256.Create((long)short.MinValue));
            Vector256<long> over = Vector256.GreaterThan(upper, Vector256.Create((long)short.MaxValue));
            return Vector256.ConditionalSelect(under, Vector256<long>.Zero, Vector256.ConditionalSelect(over, Vector256.Create(0xFFFFL), value & Vector256.Create(0xFFFFL)));
        }

        private Vector128<ushort> FractionSimd(Vector128<ushort> s, Vector128<ushort> t, bool unsigned, bool accumulate)
        {
            (Vector256<long> low, Vector256<long> high) = WidenSigned(Products(s, t, sSigned: true, tSigned: true));
            (low, high) = Accumulate(low << 1, high << 1, accumulate, accumulate ? 0 : 0x8000);
            return unsigned ? Narrow(ClampUnsigned(low), ClampUnsigned(high)) : ClampSignedHalf(low, high);
        }

        private Vector128<ushort> LowSimd(Vector128<ushort> s, Vector128<ushort> t, bool accumulate)
        {
            Vector256<uint> products = Products(s, t, sSigned: false, tSigned: false).AsUInt32() >> 16;
            (Vector256<long> low, Vector256<long> high) = Accumulate(Vector256.WidenLower(products).AsInt64(), Vector256.WidenUpper(products).AsInt64(), accumulate, 0);
            return Narrow(ClampLow(low), ClampLow(high));
        }

        private Vector128<ushort> MiddleSimd(Vector128<ushort> s, Vector128<ushort> t, bool accumulate)
        {
            (Vector256<long> low, Vector256<long> high) = WidenSigned(Products(s, t, sSigned: true, tSigned: false));
            (low, high) = Accumulate(low, high, accumulate, 0);
            return ClampSignedHalf(low, high);
        }

        private Vector128<ushort> NormalSimd(Vector128<ushort> s, Vector128<ushort> t, bool accumulate)
        {
            (Vector256<long> low, Vector256<long> high) = WidenSigned(Products(s, t, sSigned: false, tSigned: true));
            (low, high) = Accumulate(low, high, accumulate, 0);
            return Narrow(ClampLow(low), ClampLow(high));
        }

        private Vector128<ushort> HighSimd(Vector128<ushort> s, Vector128<ushort> t, bool accumulate)
        {
            (Vector256<long> low, Vector256<long> high) = WidenSigned(Products(s, t, sSigned: true, tSigned: true));
            (low, high) = Accumulate(low << 16, high << 16, accumulate, 0);
            return ClampSignedHalf(low, high);
        }

        // A negative product is biased by thirty-one before it is kept, and the result loses its low four bits - see §7.
        private Vector128<ushort> QuarterSimd(Vector128<ushort> s, Vector128<ushort> t)
        {
            (Vector256<long> low, Vector256<long> high) = WidenSigned(Products(s, t, sSigned: true, tSigned: true));
            Vector256<long> bias = Vector256.Create(0x1F_0000L);
            low = (low << 16) + (Vector256.LessThan(low, Vector256<long>.Zero) & bias);
            high = (high << 16) + (Vector256.LessThan(high, Vector256<long>.Zero) & bias);
            (low, high) = Accumulate(low, high, accumulate: false, 0);
            return Narrow(ClampSigned(low >> 17), ClampSigned(high >> 17)) & Vector128.Create((ushort)0xFFF0);
        }

        // The accumulating quarter multiply ignores both sources and only nudges the accumulator toward zero - see §9.
        private Vector128<ushort> AccumulatedQuarterSimd()
        {
            (Vector256<ulong> low, Vector256<ulong> high) = LoadAccumulator();
            low = Wrap48(NudgeToZero(Signed48(low)));
            high = Wrap48(NudgeToZero(Signed48(high)));
            StoreAccumulator(low, high);

            return Narrow(ClampSigned(Signed48(low) >> 17), ClampSigned(Signed48(high) >> 17)) & Vector128.Create((ushort)0xFFF0);

            static Vector256<long> NudgeToZero(Vector256<long> value)
            {
                Vector256<long> step = Vector256.Create(0x20_0000L);
                Vector256<long> clear = Vector256.Equals(value & step, Vector256<long>.Zero);
                Vector256<long> upper = value >> 22;
                Vector256<long> nudge = Vector256.ConditionalSelect(Vector256.LessThan(upper, Vector256<long>.Zero), step,
                    Vector256.ConditionalSelect(Vector256.GreaterThan(upper, Vector256<long>.Zero), -step, Vector256<long>.Zero));

                return value + (clear & nudge);
            }
        }

        // The parity of vs's register number, not anything in it, decides whether vt is shifted - see §9.
        private Vector128<ushort> RoundSimd(Vector128<ushort> t, bool shifted, bool positive)
        {
            (Vector256<ulong> low, Vector256<ulong> high) = LoadAccumulator();
            (Vector256<long> addLow, Vector256<long> addHigh) = WidenSigned(t);
            if (shifted) (addLow, addHigh) = (addLow << 16, addHigh << 16);

            Vector256<long> valueLow = Signed48(low), valueHigh = Signed48(high);
            Vector256<long> takeLow = Vector256.GreaterThanOrEqual(valueLow, Vector256<long>.Zero), takeHigh = Vector256.GreaterThanOrEqual(valueHigh, Vector256<long>.Zero);
            if (!positive) (takeLow, takeHigh) = (~takeLow, ~takeHigh);

            low = Wrap48(valueLow + (takeLow & addLow));
            high = Wrap48(valueHigh + (takeHigh & addHigh));
            StoreAccumulator(low, high);
            return ClampSignedHalf(Signed48(low), Signed48(high));
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
        private Vector128<ushort> ReadAccumulatorSimd(int selector)
        {
            int shift = selector switch { 8 => 32, 9 => 16, 10 => 0, _ => -1 };
            if (shift < 0) return Vector128<ushort>.Zero;

            (Vector256<ulong> low, Vector256<ulong> high) = LoadAccumulator();
            return Narrow((low >> shift).AsInt64(), (high >> shift).AsInt64());
        }

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
