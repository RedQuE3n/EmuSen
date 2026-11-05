using System;
using EmuSen.Endymion.Native;

namespace EmuSen.Endymion
{
    // Absorbs clock drift by resampling, never by discarding, by the platform library's rules - see EmuSen_Audio_Sync.md §1/§3.
    public sealed class DynamicRateControl
    {
        private readonly NativeHandle _native;

        public unsafe DynamicRateControl(int targetQueuedFrames)
        {
            EndymionNative.Up();
            _native = new NativeHandle(EndymionNative.RateNew(targetQueuedFrames), EndymionNative.RateFree);
        }

        // The library's rate control, which its audio player is lent for each submit.
        internal NativeHandle NativeValue => _native;

        // Where the output queue is steered to sit, in frames.
        public int TargetQueuedFrames
        {
            get => State().TargetQueuedFrames;
            set => Set(EndymionNative.RateTarget, value);
        }

        // Largest resample ratio departure from 1.0; the caller overrides it - see EmuSen_Audio_Sync.md §3.
        public double MaxDeviation
        {
            get => State().MaxDeviation;
            set => Set(EndymionNative.RateMaxDeviation, value);
        }

        // Above TargetQueuedFrames * this, input is shed until it drains - see §3.1.
        public double SheddingEntryFactor
        {
            get => State().SheddingEntryFactor;
            set => Set(EndymionNative.RateEntry, value);
        }

        public double SheddingExitFactor
        {
            get => State().SheddingExitFactor;
            set => Set(EndymionNative.RateExit, value);
        }

        // The ratio the drift is centred on, set by a pacer that knowingly runs the content off its own rate - see EmuSen_Audio_Sync.md §3.4.
        public double NominalRatio
        {
            get => State().NominalRatio;
            set => Set(EndymionNative.RateNominal, value);
        }

        public double LastRatio => State().LastRatio;

        // A stall counter; expected to stay at 0 in normal play - see EmuSen_Audio_Sync.md §3.1.
        public int SheddingEvents => State().SheddingEvents;

        public bool IsShedding => State().IsShedding != 0;

        // Their difference is what this class did to the queue - see EmuSen_Audio_Sync.md §3.3.
        public long TotalInputFrames => State().TotalInputFrames;
        public long TotalOutputFrames => State().TotalOutputFrames;

        // Ratio the queue's current fill calls for - see EmuSen_Audio_Sync.md §3.
        public unsafe double ComputeRatio(int queuedFrames)
        {
            double ratio;
            EndymionNative.RateCompute(_native.Value, queuedFrames, &ratio);
            GC.KeepAlive(_native);
            return ratio;
        }

        // Returns what to hand the output device, or empty while shedding.
        public unsafe short[] Process(short[] input, int queuedFrames)
        {
            int length = input?.Length ?? 0;
            short[] room = new short[LinearResampler.Room(length)];
            long made;
            short none = 0;
            fixed (short* i = input, o = room) made = EndymionNative.RateProcess(_native.Value, LinearResampler.Samples(input, i, &none), (nuint)length, queuedFrames, o, (nuint)room.Length);
            if (made == EndymionNative.Null) throw new NullReferenceException();
            if (made < 0) throw new ArgumentOutOfRangeException("ratio", EndymionNative.Words());
            short[] output = LinearResampler.Collect(_native, room, made, EndymionNative.RateTake);
            GC.KeepAlive(_native);
            return output;
        }

        // Call on any discontinuity - see EmuSen_Audio_Sync.md §3.2.
        public unsafe void Reset()
        {
            EndymionNative.RateReset(_native.Value);
            GC.KeepAlive(_native);
        }

        private unsafe RateState State()
        {
            RateState state = new() { Size = (uint)sizeof(RateState) };
            EndymionNative.RateRead(_native.Value, &state);
            GC.KeepAlive(_native);
            return state;
        }

        private unsafe void Set(uint which, double value)
        {
            EndymionNative.RateSet(_native.Value, which, value);
            GC.KeepAlive(_native);
        }
    }
}
