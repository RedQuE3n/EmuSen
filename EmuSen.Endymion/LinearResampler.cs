using System;
using System.Collections.Generic;

namespace EmuSen.Endymion
{
    // Phase-continuous linear resampler for interleaved stereo shorts - see EmuSen_Audio_Sync.md §2.
    public sealed class LinearResampler
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

        // ratio is output frames per input frame - see EmuSen_Audio_Sync.md §2.
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
