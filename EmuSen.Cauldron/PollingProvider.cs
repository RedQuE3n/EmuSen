using System;
using System.Threading;

namespace EmuSen.Cauldron
{
    // Wraps a "go read the live core" delegate; lock-free reference swap - see EmuSen_Cauldron.md §2.1.
    public sealed class PollingProvider<T> : IRealtimeProvider<T> where T : class
    {
        private readonly Func<T> _readLive;
        private T _current;

        // <initial> is read once here so Current is never "nothing" before the first Refresh - see §2.1.
        public PollingProvider(Func<T> readLive, T initial)
        {
            _readLive = readLive;
            _current = initial;
        }

        public T Current => Volatile.Read(ref _current);

        public void Refresh() => Volatile.Write(ref _current, _readLive());
    }
}
