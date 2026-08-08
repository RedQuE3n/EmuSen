using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // Low-to-high on PPU A12, filtered by how long it stayed low. The threshold is in PPU dots - see Moon_Memory.md §4.6a.
    public sealed class A12Watcher
    {
        // Wiring, not data: the board picks it once and a save state cannot change it.
        [SkipInState] private readonly int _minimumDotsLow;

        // -1 rather than 0, because clock 0 is a real dot the very first fetch can land on.
        private long _lowSince = -1;

        public A12Watcher(int minimumDotsLow) => _minimumDotsLow = minimumDotsLow;

        public bool Rose(ushort address, long ppuClock)
        {
            if ((address & 0x1000) == 0)
            {
                if (_lowSince < 0) _lowSince = ppuClock;
                return false;
            }

            bool rising = _lowSince >= 0 && ppuClock - _lowSince >= _minimumDotsLow;
            _lowSince = -1;
            return rising;
        }
    }
}
