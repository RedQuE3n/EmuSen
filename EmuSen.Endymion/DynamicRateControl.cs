using System;

namespace EmuSen.Endymion
{
    // Absorbs clock drift by resampling, never by discarding - see EmuSen_Audio_Sync.md §1/§3.
    public sealed class DynamicRateControl
    {
        private readonly LinearResampler _resampler = new();
        private bool _shedding;

        // Where the output queue is steered to sit, in frames.
        public int TargetQueuedFrames { get; set; }

        // Largest resample ratio departure from 1.0; the caller overrides it - see EmuSen_Audio_Sync.md §3.
        public double MaxDeviation { get; set; } = 0.005;

        // Above TargetQueuedFrames * this, input is shed until it drains - see §3.1.
        public double SheddingEntryFactor { get; set; } = 3.0;
        public double SheddingExitFactor { get; set; } = 2.0;

        public double LastRatio { get; private set; } = 1.0;

        // A stall counter; expected to stay at 0 in normal play - see EmuSen_Audio_Sync.md §3.1.
        public int SheddingEvents { get; private set; }

        public bool IsShedding => _shedding;

        // Their difference is what this class did to the queue - see EmuSen_Audio_Sync.md §3.3.
        public long TotalInputFrames { get; private set; }
        public long TotalOutputFrames { get; private set; }

        public DynamicRateControl(int targetQueuedFrames)
        {
            TargetQueuedFrames = targetQueuedFrames;
        }

        // Ratio the queue's current fill calls for - see EmuSen_Audio_Sync.md §3.
        public double ComputeRatio(int queuedFrames)
        {
            if (TargetQueuedFrames <= 0) return 1.0;
            double delta = (queuedFrames - (double)TargetQueuedFrames) / TargetQueuedFrames;
            return 1.0 - Math.Clamp(delta, -1.0, 1.0) * MaxDeviation;
        }

        // Returns what to hand the output device, or empty while shedding.
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

        // Call on any discontinuity - see EmuSen_Audio_Sync.md §3.2.
        public void Reset()
        {
            _resampler.Reset();
            _shedding = false;
            LastRatio = 1.0;
        }
    }
}
