using System;

namespace EmuSen.Cores
{
    // The arrays a core lends through GetFrameBufferRgba and takes back through IFrameBufferPool; small, bounded, any thread - see EmuSen_Multicore.md §16.
    public sealed class FrameBufferLending
    {
        public const int FreeLimit = 4, LentLimit = 8;

        private readonly object _lock = new();
        private readonly byte[]?[] _lent = new byte[LentLimit][];
        private readonly byte[]?[] _free = new byte[FreeLimit][];
        private int _nextLent, _freeCount, _length = -1;
        private bool _closed;
        private long _made, _reused, _dropped;

        public int Free { get { lock (_lock) return _freeCount; } }
        public long Made { get { lock (_lock) return _made; } }
        public long Reused { get { lock (_lock) return _reused; } }
        public long Dropped { get { lock (_lock) return _dropped; } }

        // An array of the length asked for that no caller holds; the lent record forgets the oldest past LentLimit, which is then the caller's for good.
        public byte[] Lend(int length)
        {
            byte[]? buffer = null;
            lock (_lock)
            {
                if (length != _length)
                {
                    Array.Clear(_free);
                    _freeCount = 0;
                    _length = length;
                }
                if (_freeCount > 0)
                {
                    buffer = _free[--_freeCount];
                    _free[_freeCount] = null;
                    _reused++;
                }
                else _made++;
            }

            buffer ??= GC.AllocateUninitializedArray<byte>(length);
            lock (_lock)
            {
                int slot = _nextLent;
                for (int i = 0; i < LentLimit; i++)
                {
                    if (_lent[(_nextLent + i) % LentLimit] is null) { slot = (_nextLent + i) % LentLimit; break; }
                }
                _lent[slot] = buffer;
                _nextLent = (slot + 1) % LentLimit;
            }
            return buffer;
        }

        // Taken back only if lent, still at the size lent and there is room; a second return of the same lend finds nothing and is dropped.
        public void Return(byte[] buffer)
        {
            if (buffer is null) return;
            lock (_lock)
            {
                int slot = -1;
                for (int i = 0; i < LentLimit && slot < 0; i++) if (ReferenceEquals(_lent[i], buffer)) slot = i;
                if (slot < 0) { _dropped++; return; }
                _lent[slot] = null;
                if (_closed || buffer.Length != _length || _freeCount == FreeLimit) { _dropped++; return; }
                _free[_freeCount++] = buffer;
            }
        }

        // The core is gone: nothing is lent again, and anything returned is dropped.
        public void Close()
        {
            lock (_lock)
            {
                _closed = true;
                Array.Clear(_free);
                Array.Clear(_lent);
                _freeCount = 0;
            }
        }
    }
}
