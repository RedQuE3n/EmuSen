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

        // The request's lag behind a $4015 write, in CPU cycles: positive while a load's rises, negative while a disable's falls - see Moon_Native.md §3.11.
        [SkipInState] public int LoadDelay;

        private int _timer;
        private int _currentAddress;
        private int _bytesRemaining;
        private int _shift;
        private int _bitsRemaining;
        private bool _silence = true;
        private bool _enabled;

        public bool Active => _bytesRemaining > 0;

        // The reader wants the bus: an empty buffer, and bytes left with no load delay running or a disable's lag not yet run out.
        public bool DmaRequested => !BufferFull && ((_bytesRemaining > 0 && LoadDelay == 0) || LoadDelay < 0);

        public ushort DmaAddress => (ushort)_currentAddress;

        public int Output => OutputLevel;

        public bool Enabled => _enabled;

        // A $4015 write; the request waits three cycles on a get, two on a put, full buffer or not - see Moon_Native.md §3.12.
        public void SetEnabled(bool value, bool onGetCycle)
        {
            _enabled = value;
            IrqPending = false;

            if (!value)
            {
                // The request follows the enable through the same pipeline, so a disable lets it fall only after the lag.
                LoadDelay = _bytesRemaining > 0 && LoadDelay == 0 ? -(onGetCycle ? 3 : 2) : 0;
                _bytesRemaining = 0;
            }
            else if (_bytesRemaining == 0)
            {
                Restart();
                if (_bytesRemaining > 0) LoadDelay = onGetCycle ? 3 : 2;
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
            else if (LoadDelay < 0) LoadDelay++;

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
            LoadDelay = 0;

            // A fetch that got its get inside a disable's lag fills the buffer and ends there.
            if (_bytesRemaining == 0) return;

            _currentAddress = _currentAddress == 0xFFFF ? 0x8000 : _currentAddress + 1;
            _bytesRemaining--;

            if (_bytesRemaining != 0) return;

            if (Loop)
            {
                Restart();
                return;
            }
            if (IrqEnabled) IrqPending = true;

            // The last fetch drops the enable, and the request trails it by the get's lag: an empty buffer inside it asks once more - see Moon_Native.md §3.11.
            LoadDelay = -3;
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
