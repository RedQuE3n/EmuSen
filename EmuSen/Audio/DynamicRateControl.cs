using System;

namespace EmuSen.Audio
{
    // Absorbs clock drift by resampling within a fraction of a percent
    // instead of discarding audio - see EmuSen_Audio_Sync.md §1/§3.
    public sealed class DynamicRateControl
    {
        private readonly LinearResampler _resampler = new();
        private bool _shedding;

        // Where the output queue is steered to sit, in frames.
        public int TargetQueuedFrames { get; set; }

        // Largest resample ratio departure from 1.0 - see EmuSen_Audio_Sync.md §3.
        public double MaxDeviation { get; set; } = AudioSettings.RateControlMaxDeviation;

        // Above TargetQueuedFrames * this, input is shed until it drains - see §3.1.
        public double SheddingEntryFactor { get; set; } = 3.0;
        public double SheddingExitFactor { get; set; } = 2.0;

        public double LastRatio { get; private set; } = 1.0;

        // How many times the shedding path has engaged - a stall counter, and
        // expected to stay at 0 in normal play.
        public int SheddingEvents { get; private set; }

        public bool IsShedding => _shedding;

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
            return _resampler.Resample(input, LastRatio);
        }

        // Call on any discontinuity - a ROM load, a state load, resuming from
        // fast-forward or rewind. See EmuSen_Audio_Sync.md §3.2.
        public void Reset()
        {
            _resampler.Reset();
            _shedding = false;
            LastRatio = 1.0;
        }
    }
}
