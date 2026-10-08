using System;
using EmuSen.Endymion.Native;

namespace EmuSen.Endymion
{
    // Absorbs clock drift by resampling, never by discarding - see EmuSen_Audio_Sync.md §1/§3.
    public sealed class DynamicRateControl
    {
        private readonly Managed? _managed;
        private readonly NativeHandle? _native;

        public DynamicRateControl(int targetQueuedFrames) : this(targetQueuedFrames, EndymionNative.Active) { }

        // The library's rules or the C#'s, chosen for the instance; a parity test asks for each - see EmuSen_RustPlatform.md §12.2.
        internal unsafe DynamicRateControl(int targetQueuedFrames, bool native)
        {
            if (native) _native = new NativeHandle(EndymionNative.RateNew(targetQueuedFrames), EndymionNative.RateFree);
            else _managed = new Managed(targetQueuedFrames);
        }

        // Where the output queue is steered to sit, in frames.
        public int TargetQueuedFrames
        {
            get => _native is null ? _managed!.TargetQueuedFrames : State().TargetQueuedFrames;
            set { if (_native is null) _managed!.TargetQueuedFrames = value; else Set(EndymionNative.RateTarget, value); }
        }

        // Largest resample ratio departure from 1.0; the caller overrides it - see EmuSen_Audio_Sync.md §3.
        public double MaxDeviation
        {
            get => _native is null ? _managed!.MaxDeviation : State().MaxDeviation;
            set { if (_native is null) _managed!.MaxDeviation = value; else Set(EndymionNative.RateMaxDeviation, value); }
        }

        // Above TargetQueuedFrames * this, input is shed until it drains - see §3.1.
        public double SheddingEntryFactor
        {
            get => _native is null ? _managed!.SheddingEntryFactor : State().SheddingEntryFactor;
            set { if (_native is null) _managed!.SheddingEntryFactor = value; else Set(EndymionNative.RateEntry, value); }
        }

        public double SheddingExitFactor
        {
            get => _native is null ? _managed!.SheddingExitFactor : State().SheddingExitFactor;
            set { if (_native is null) _managed!.SheddingExitFactor = value; else Set(EndymionNative.RateExit, value); }
        }

        // The ratio the drift is centred on, set by a pacer that knowingly runs the content off its own rate - see EmuSen_Audio_Sync.md §3.4.
        public double NominalRatio
        {
            get => _native is null ? _managed!.NominalRatio : State().NominalRatio;
            set { if (_native is null) _managed!.NominalRatio = value; else Set(EndymionNative.RateNominal, value); }
        }

        public double LastRatio => _native is null ? _managed!.LastRatio : State().LastRatio;

        // A stall counter; expected to stay at 0 in normal play - see EmuSen_Audio_Sync.md §3.1.
        public int SheddingEvents => _native is null ? _managed!.SheddingEvents : State().SheddingEvents;

        public bool IsShedding => _native is null ? _managed!.IsShedding : State().IsShedding != 0;

        // Their difference is what this class did to the queue - see EmuSen_Audio_Sync.md §3.3.
        public long TotalInputFrames => _native is null ? _managed!.TotalInputFrames : State().TotalInputFrames;
        public long TotalOutputFrames => _native is null ? _managed!.TotalOutputFrames : State().TotalOutputFrames;

        // Ratio the queue's current fill calls for - see EmuSen_Audio_Sync.md §3.
        public unsafe double ComputeRatio(int queuedFrames)
        {
            if (_native is null) return _managed!.ComputeRatio(queuedFrames);
            double ratio;
            EndymionNative.RateCompute(_native.Value, queuedFrames, &ratio);
            GC.KeepAlive(_native);
            return ratio;
        }

        // Returns what to hand the output device, or empty while shedding.
        public unsafe short[] Process(short[] input, int queuedFrames)
        {
            if (_native is null) return _managed!.Process(input, queuedFrames);
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
            if (_native is null) _managed!.Reset();
            else
            {
                EndymionNative.RateReset(_native.Value);
                GC.KeepAlive(_native);
            }
        }

        private unsafe RateState State()
        {
            RateState state = new() { Size = (uint)sizeof(RateState) };
            EndymionNative.RateRead(_native!.Value, &state);
            GC.KeepAlive(_native);
            return state;
        }

        private unsafe void Set(uint which, double value)
        {
            EndymionNative.RateSet(_native!.Value, which, value);
            GC.KeepAlive(_native);
        }

        // The C# rules: the default, and what the library's are held to until Endymion's gate - see EmuSen_RustPlatform.md §3.9.
        internal sealed class Managed
        {
            private readonly LinearResampler.Managed _resampler = new();
            private bool _shedding;

            public int TargetQueuedFrames { get; set; }

            public double MaxDeviation { get; set; } = 0.005;

            public double SheddingEntryFactor { get; set; } = 3.0;
            public double SheddingExitFactor { get; set; } = 2.0;

            public double NominalRatio { get; set; } = 1.0;

            public double LastRatio { get; private set; } = 1.0;

            public int SheddingEvents { get; private set; }

            public bool IsShedding => _shedding;

            public long TotalInputFrames { get; private set; }
            public long TotalOutputFrames { get; private set; }

            public Managed(int targetQueuedFrames)
            {
                TargetQueuedFrames = targetQueuedFrames;
            }

            public double ComputeRatio(int queuedFrames)
            {
                if (TargetQueuedFrames <= 0) return NominalRatio;
                double delta = (queuedFrames - (double)TargetQueuedFrames) / TargetQueuedFrames;
                return NominalRatio * (1.0 - Math.Clamp(delta, -1.0, 1.0) * MaxDeviation);
            }

            public short[] Process(short[] input, int queuedFrames)
            {
                LastRatio = ComputeRatio(queuedFrames);
                TotalInputFrames += input.Length / 2;

                if (_shedding)
                {
                    if (queuedFrames > TargetQueuedFrames * SheddingExitFactor) return Array.Empty<short>();
                    _shedding = false;
                    _resampler.Reset();
                }
                else if (queuedFrames > TargetQueuedFrames * SheddingEntryFactor)
                {
                    _shedding = true;
                    SheddingEvents++;
                    _resampler.Reset();
                    return Array.Empty<short>();
                }

                if (input.Length < 2) return Array.Empty<short>();

                short[] output = _resampler.Resample(input, LastRatio);
                TotalOutputFrames += output.Length / 2;
                return output;
            }

            public void Reset()
            {
                _resampler.Reset();
                _shedding = false;
                LastRatio = NominalRatio;
            }
        }
    }
}
