using System;
using EmuSen.Endymion.Native;

namespace EmuSen.Endymion
{
    // Phase-continuous linear resampler for interleaved stereo shorts, the platform library's - see EmuSen_Audio_Sync.md §2.
    public sealed class LinearResampler
    {
        private readonly NativeHandle _native;

        public unsafe LinearResampler()
        {
            EndymionNative.Up();
            _native = new NativeHandle(EndymionNative.ResamplerNew(), EndymionNative.ResamplerFree);
        }

        public unsafe void Reset()
        {
            EndymionNative.ResamplerReset(_native.Value);
            GC.KeepAlive(_native);
        }

        // ratio is output frames per input frame - see EmuSen_Audio_Sync.md §2.
        public short[] Resample(short[] input, double ratio) => ResampleNative(_native, input, ratio);

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
    }
}
