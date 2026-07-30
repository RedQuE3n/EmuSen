using System;
using System.Threading;

namespace EmuSen.Cauldron
{
    // The one IRealtimeProvider<T> implementation this project needs so
    // far: wraps a plain "go read the live core" delegate, so a core
    // supplies a Func<T> instead of writing its own IRealtimeProvider
    // boilerplate. Constrained to T : class (every current use is an
    // IReadOnlyList<...> snapshot) so Current can be published via
    // Volatile.Write/read via Volatile.Read - a plain reference swap,
    // not a lock - matching this project's own "Current never blocks"
    // contract even under concurrent Refresh()/Current access.
    public sealed class PollingProvider<T> : IRealtimeProvider<T> where T : class
    {
        private readonly Func<T> _readLive;
        private T _current;

        // <initial> is read once, here, rather than left null/default -
        // a provider should never hand back "nothing" before its first
        // real Refresh() just because that hasn't happened yet on
        // whatever thread owns the core.
        public PollingProvider(Func<T> readLive, T initial)
        {
            _readLive = readLive;
            _current = initial;
        }

        public T Current => Volatile.Read(ref _current);

        public void Refresh() => Volatile.Write(ref _current, _readLive());
    }
}
