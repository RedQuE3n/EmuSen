using System;

namespace EmuSen.LunaP.Settings
{
    // The one place a host points LunaP at its own settings and its own log - see EmuSen_LunaP.md §19.
    public static class LunaSettings
    {
        private static ISettingsStore? _store;

        // Resolved on first use rather than at type load, so a host that assigns one at startup is never a moment too late.
        public static ISettingsStore Store
        {
            get => _store ??= JsonSettingsStore.ForApplication();
            set => _store = value;
        }

        // Set once by the host; null discards, which is what tests and any caller with nowhere to print want.
        public static Action<string>? Diagnostics { get; set; }

        // "This file would not load, and why" - loading stays best-effort, this only stops it happening in silence.
        public static void Report(string message) => Diagnostics?.Invoke(message);
    }
}
