using System;
using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Apu
{
    // Delta modulation: the only channel that reads memory, and the only one that stalls the CPU - see Moon_APU.md §3.5.
    public sealed class DmcChannel
    {
        public bool IrqEnabled;
        public bool Loop;
        public int RateIndex;
        public int OutputLevel;
        public int SampleAddress;
        public int SampleLength;

        public bool IrqPending;

        // Always 0 since the fetch became a halt of the CPU; kept so version 3's layout is unchanged - see Moon_Native.md §3.9.
        public int StallCycles;

        // The byte the reader fetched and the output unit has not taken yet; written in version 4's tail - see Moon_Native.md §3.9.
        [SkipInState] public byte SampleBuffer;
        [SkipInState] public bool BufferFull;

        // CPU cycles until a $4015 write's load DMA is asked for, 0 when none is waiting.
        [SkipInState] public int LoadDelay;

        private int _timer;
        private int _currentAddress;
        private int _bytesRemaining;
        private int _shift;
        private int _bitsRemaining;
        private bool _silence = true;
        private bool _enabled;

        public bool Active => _bytesRemaining > 0;

        // The reader wants the bus: the buffer is empty, bytes remain, and no load delay is running.
        public bool DmaRequested => !BufferFull && _bytesRemaining > 0 && LoadDelay == 0;

        public ushort DmaAddress => (ushort)_currentAddress;

        public int Output => OutputLevel;

        public bool Enabled => _enabled;

        // A $4015 write; an empty buffer asks for its first byte two cycles later on a get, three on a put - see Moon_Native.md §3.9.
        public void SetEnabled(bool value, bool onGetCycle)
        {
            _enabled = value;
            IrqPending = false;

            if (!value)
            {
                _bytesRemaining = 0;
                LoadDelay = 0;
            }
            else if (_bytesRemaining == 0)
            {
                Restart();
                if (!BufferFull && _bytesRemaining > 0) LoadDelay = onGetCycle ? 2 : 3;
            }
        }

        public void Restart()
        {
            _currentAddress = SampleAddress;
            _bytesRemaining = SampleLength;
        }

        // Clocked at the full CPU rate; the table is the period in CPU cycles, so the count reloads one short (D5) - see Moon_Native.md §3.9.
        public void StepTimer()
        {
            if (LoadDelay > 0) LoadDelay--;

            if (_timer > 0)
            {
                _timer--;
                return;
            }

            _timer = ApuTables.DmcRate[RateIndex] - 1;
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
            if (!BufferFull)
            {
                _silence = true;
                return;
            }

            _silence = false;
            _shift = SampleBuffer;
            BufferFull = false;
        }

        // The reader's get cycle delivered <value>: the buffer fills and the sample moves on.
        public void CompleteDma(byte value)
        {
            SampleBuffer = value;
            BufferFull = true;

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
            SampleBuffer = 0;
            BufferFull = false;
            LoadDelay = 0;
        }
    }
}
