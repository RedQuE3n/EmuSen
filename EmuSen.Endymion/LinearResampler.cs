using System;
using System.Collections.Generic;
using EmuSen.Endymion.Native;

namespace EmuSen.Endymion
{
    // Phase-continuous linear resampler for interleaved stereo shorts - see EmuSen_Audio_Sync.md §2.
    public sealed class LinearResampler
    {
        private readonly Managed? _managed;
        private readonly NativeHandle? _native;

        public LinearResampler() : this(EndymionNative.Active) { }

        // The library's rules or the C#'s, chosen for the instance; a parity test asks for each - see EmuSen_RustPlatform.md §12.2.
        internal unsafe LinearResampler(bool native)
        {
            if (native) _native = new NativeHandle(EndymionNative.ResamplerNew(), EndymionNative.ResamplerFree);
            else _managed = new Managed();
        }

        public void Reset()
        {
            if (_native is null) _managed!.Reset();
            else ResetNative(_native);
        }

        // ratio is output frames per input frame - see EmuSen_Audio_Sync.md §2.
        public short[] Resample(short[] input, double ratio) => _native is null ? _managed!.Resample(input, ratio) : ResampleNative(_native, input, ratio);

        private static unsafe void ResetNative(NativeHandle native)
        {
            EndymionNative.ResamplerReset(native.Value);
            GC.KeepAlive(native);
        }

        // Room for a ratio near one, as the rate control asks; output past it waits in the library to be taken.
        internal static unsafe short[] ResampleNative(NativeHandle native, short[] input, double ratio)
        {
            int length = input?.Length ?? 0;
            short[] room = new short[Room(length)];
            long made;
            short none = 0;
            fixed (short* i = input, o = room) made = EndymionNative.ResamplerRun(native.Value, Samples(input, i, &none), (nuint)length, ratio, o, (nuint)room.Length);
            if (made < 0)
            {
                GC.KeepAlive(native);
                throw new ArgumentOutOfRangeException(nameof(ratio), EndymionNative.Words());
            }
            if (input is null) throw new NullReferenceException();
            short[] output = Collect(native, room, made, EndymionNative.ResamplerTake);
            GC.KeepAlive(native);
            return output;
        }

        internal static int Room(int length) => length + length / 8 + 16;

        // A pinned array as the library takes it: null for no array, and a pointer that is not null for an empty one, which fixed makes null.
        internal static unsafe short* Samples(short[]? input, short* pinned, short* none) => input is null ? null : input.Length == 0 ? none : pinned;

        // The samples a call made: the room given when they fitted, else taken from where they wait.
        internal static unsafe short[] Collect(NativeHandle native, short[] room, long made, delegate* unmanaged[Cdecl]<nint, short*, nuint, long> takeFrom)
        {
            if (made == 0) return Array.Empty<short>();
            if (made <= room.Length) return made == room.Length ? room : room.AsSpan(0, (int)made).ToArray();
            short[] output = new short[made];
            fixed (short* o = output) takeFrom(native.Value, o, (nuint)output.Length);
            return output;
        }

        // The C# rules: the default, and what the library's are held to until Endymion's gate - see EmuSen_RustPlatform.md §3.9.
        internal sealed class Managed
        {
            // Position between _prev and the next unconsumed input frame.
            private double _frac;
            private short _prevL;
            private short _prevR;
            private bool _primed;

            public void Reset()
            {
                _frac = 0;
                _prevL = 0;
                _prevR = 0;
                _primed = false;
            }

            public short[] Resample(short[] input, double ratio)
            {
                if (ratio <= 0) throw new ArgumentOutOfRangeException(nameof(ratio), "Resample ratio must be positive.");
                if (input.Length < 2) return Array.Empty<short>();

                int inFrames = input.Length / 2;
                int i = 0;

                // First ever call has no previous frame to interpolate from.
                if (!_primed)
                {
                    _prevL = input[0];
                    _prevR = input[1];
                    _frac = 0;
                    _primed = true;
                    i = 1;
                }

                double step = 1.0 / ratio;
                var output = new List<short>((int)(inFrames * ratio) * 2 + 4);

                while (true)
                {
                    while (_frac >= 1.0)
                    {
                        if (i >= inFrames) return output.ToArray();
                        _prevL = input[i * 2];
                        _prevR = input[i * 2 + 1];
                        i++;
                        _frac -= 1.0;
                    }

                    if (i >= inFrames) return output.ToArray();

                    short nextL = input[i * 2];
                    short nextR = input[i * 2 + 1];
                    output.Add((short)Math.Round(_prevL + (nextL - _prevL) * _frac));
                    output.Add((short)Math.Round(_prevR + (nextR - _prevR) * _frac));
                    _frac += step;
                }
            }
        }
    }
}
