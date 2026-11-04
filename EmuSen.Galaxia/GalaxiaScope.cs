using System;
using System.Threading;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia
{
    // A test's redirects for its own flow of execution, followed onto every thread it reaches - see EmuSen_Galaxia.md §3.14.
    public sealed class GalaxiaScope : IDisposable
    {
        private static readonly AsyncLocal<GalaxiaScope?> Ambient = new(Moved);

        private readonly GalaxiaScope? _outer;
        private bool _left;

        private GalaxiaScope(string? data, string? config, string? legacy, GalaxiaScope? outer)
        {
            Data = data;
            Config = config;
            Legacy = legacy;
            _outer = outer;
        }

        public string? Data { get; }
        public string? Config { get; }
        public string? Legacy { get; }

        // The scope of the calling flow, or null.
        public static GalaxiaScope? Current => Ambient.Value;

        // Tests only, from EmuSen.WiseMan; the library has to be in the process first, so every thread the flow reaches is told.
        internal static GalaxiaScope Enter(string? data = null, string? config = null, string? legacy = null)
        {
            GalaxiaNative.Up();
            var scope = new GalaxiaScope(data, config, legacy, Ambient.Value);
            Ambient.Value = scope;
            return scope;
        }

        // Work the library hands to its own threads cannot see a scope, so it refuses to start inside one.
        public static void Refuse(string what)
        {
            if (Ambient.Value is not null)
                throw new InvalidOperationException($"{what} runs on the platform library's own threads, which a Galaxia scope does not reach; a test class that starts it sets the overrides and stays in the serial collection - see EmuSen_Galaxia.md §3.14.");
        }

        public void Dispose()
        {
            if (_left) return;
            _left = true;
            Ambient.Value = _outer;
        }

        // Called whenever the scope a thread runs under changes, a continuation moving to another thread included.
        private static void Moved(AsyncLocalValueChangedArgs<GalaxiaScope?> change)
        {
            GalaxiaScope? now = change.CurrentValue;
            GalaxiaNative.ThreadScope(now?.Config, now?.Data, now?.Legacy);
        }
    }
}
