namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.Sa1
{
    // The SA-1's own programmable timer, independent of the S-CPU's H/V IRQ - see Venus_SA1.md §5.
    public sealed partial class Sa1
    {
        // $2210 TMC: bit 7 picks linear mode, bits 1/0 enable the V/H compare.
        private byte _tmc;

        private ushort _hCompare, _vCompare;
        private ushort _hCounter, _vCounter;
        private uint _linearCounter;

        // $2302-$2305 read the counters through a latch taken on the $2302 read.
        private ushort _hcrLatch, _vcrLatch;

        // Master clocks not yet worth a whole dot.
        private int _dotRemainder;

        // Set from the cartridge region so V wraps at the right line count.
        public int TotalScanlines = 262;

        private void StepTimer(int masterClocks)
        {
            if ((_tmc & 0x80) != 0)
            {
                StepLinearTimer(masterClocks);
                return;
            }

            // HV mode counts the video dot clock, master/4 - see Venus_SA1.md §5.1.
            _dotRemainder += masterClocks;
            int dots = _dotRemainder / 4;
            _dotRemainder -= dots * 4;

            for (int i = 0; i < dots; i++)
            {
                _hCounter++;
                if (_hCounter >= 341)
                {
                    _hCounter = 0;
                    _vCounter++;
                    if (_vCounter >= TotalScanlines) _vCounter = 0;
                }
                CheckHvCompare();
            }
        }

        private void CheckHvCompare()
        {
            bool hEnabled = (_tmc & 0x01) != 0;
            bool vEnabled = (_tmc & 0x02) != 0;
            if (!hEnabled && !vEnabled) return;

            bool hit = (hEnabled, vEnabled) switch
            {
                (true, false) => _hCounter == _hCompare,
                (false, true) => _hCounter == 0 && _vCounter == _vCompare,
                _ => _hCounter == _hCompare && _vCounter == _vCompare,
            };

            if (hit) _timerIrqToSa1 = true;
        }

        private void StepLinearTimer(int masterClocks)
        {
            uint target = ((uint)_vCompare << 16) | _hCompare;
            int cycles = (_dotRemainder + masterClocks) / MasterClocksPerCycle;
            _dotRemainder = (_dotRemainder + masterClocks) - cycles * MasterClocksPerCycle;

            for (int i = 0; i < cycles; i++)
            {
                _linearCounter = (_linearCounter + 1) & 0x01FFFFFF;
                if (_linearCounter == target) _timerIrqToSa1 = true;
            }
        }

        private void LatchTimerRead()
        {
            if ((_tmc & 0x80) != 0)
            {
                _hcrLatch = (ushort)_linearCounter;
                _vcrLatch = (ushort)(_linearCounter >> 16);
                return;
            }
            _hcrLatch = _hCounter;
            _vcrLatch = _vCounter;
        }
    }
}
