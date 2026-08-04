using System;
using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Apu
{
    // Delta modulation: the only channel that reads memory, and the only one that stalls the CPU - see Moon_APU.md §3.5.
    public sealed class DmcChannel
    {
        // Set by MoonCore once the bus exists, since the APU is built first.
        [SkipInState] public Func<ushort, byte>? ReadMemory;

        public bool IrqEnabled;
        public bool Loop;
        public int RateIndex;
        public int OutputLevel;
        public int SampleAddress;
        public int SampleLength;

        public bool IrqPending;

        // Cycles the fetch stole, drained by the bus the same way OAM DMA is.
        public int StallCycles;

        private int _timer;
        private int _currentAddress;
        private int _bytesRemaining;
        private int _shift;
        private int _bitsRemaining;
        private bool _silence = true;
        private bool _enabled;

        public bool Active => _bytesRemaining > 0;

        public int Output => OutputLevel;

        public bool Enabled
        {
            get => _enabled;
            set
            {
                _enabled = value;
                IrqPending = false;

                if (!value) _bytesRemaining = 0;
                else if (_bytesRemaining == 0) Restart();
            }
        }

        public void Restart()
        {
            _currentAddress = SampleAddress;
            _bytesRemaining = SampleLength;
        }

        // Clocked at the full CPU rate; the rate table is already in CPU cycles.
        public void StepTimer()
        {
            if (_timer > 0)
            {
                _timer--;
                return;
            }

            _timer = ApuTables.DmcRate[RateIndex];
            Clock();
        }

        private void Clock()
        {
            if (!_silence)
            {
                // The level moves by two and refuses to leave 0-127, which is what keeps DMC clicks bounded.
                if ((_shift & 0x01) != 0)
                {
                    if (OutputLevel <= 125) OutputLevel += 2;
                }
                else if (OutputLevel >= 2)
                {
                    OutputLevel -= 2;
                }
            }

            _shift >>= 1;
            _bitsRemaining--;

            if (_bitsRemaining > 0) return;

            _bitsRemaining = 8;
            FillSampleBuffer();
        }

        private void FillSampleBuffer()
        {
            if (_bytesRemaining == 0)
            {
                _silence = true;
                return;
            }

            _silence = false;
            _shift = ReadMemory?.Invoke((ushort)_currentAddress) ?? 0;

            // Four cycles per fetch is the usual figure; the exact count depends on alignment.
            StallCycles += 4;

            _currentAddress = _currentAddress == 0xFFFF ? 0x8000 : _currentAddress + 1;
            _bytesRemaining--;

            if (_bytesRemaining != 0) return;

            if (Loop) Restart();
            else if (IrqEnabled) IrqPending = true;
        }

        public void Reset()
        {
            IrqEnabled = false;
            Loop = false;
            RateIndex = 0;
            OutputLevel = 0;
            SampleAddress = 0xC000;
            SampleLength = 0;
            IrqPending = false;
            StallCycles = 0;
            _timer = 0;
            _currentAddress = 0;
            _bytesRemaining = 0;
            _shift = 0;
            _bitsRemaining = 8;
            _silence = true;
            _enabled = false;
        }
    }
}
