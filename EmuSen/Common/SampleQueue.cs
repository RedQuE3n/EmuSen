using System;
using System.Collections.Generic;
using EmuSen.Audio;

namespace EmuSen.Common
{
    // A core's undrained stereo samples, the oldest pair dropped past AudioSettings.AudioBufferMaxSamples - see EmuSen_Settings_Reference.md §4.85.2.
    public sealed class SampleQueue
    {
        public const int DefaultLimit = 128000;

        private readonly Queue<short> _samples = new();

        // Null follows AudioSettings.AudioBufferMaxSamples, read at every pair; a number is a test's or a core's own.
        public int? Limit { get; set; }

        public int MaxSamples => Limit ?? AudioSettings.AudioBufferMaxSamples;

        public int Count => _samples.Count;

        // Trimmed after the pair is queued, until within the limit, so a lowered limit takes hold at the next pair.
        public void Enqueue(short left, short right)
        {
            _samples.Enqueue(left);
            _samples.Enqueue(right);
            int max = MaxSamples;
            while (_samples.Count > max && _samples.Count >= 2)
            {
                _samples.Dequeue();
                _samples.Dequeue();
            }
        }

        // Destructive, interleaved left then right, whole pairs only, at most maxFrames of them.
        public short[] Drain(int maxFrames)
        {
            long wanted = Math.Min((long)maxFrames * 2, _samples.Count);
            wanted -= wanted & 1;
            if (wanted <= 0) return Array.Empty<short>();

            var samples = new short[wanted];
            for (int i = 0; i < wanted; i++) samples[i] = _samples.Dequeue();
            return samples;
        }

        public short[] Peek() => _samples.ToArray();

        public void Clear() => _samples.Clear();
    }
}
